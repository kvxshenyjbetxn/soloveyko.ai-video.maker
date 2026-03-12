package pipeline

import (
	"bufio"
	"encoding/json"
	"fmt"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"soloveyko/backend/utils"
	"sort"
	"strconv"
	"strings"
	"time"
)

// resolveCodec probes the preferred GPU codec, falls back to libx264.
func (s *PipelineService) resolveCodec(ffmpegPath string, preferred string, id string, taskLabel string) string {
	codecMap := map[string]string{
		"nvidia": "h264_nvenc",
		"amd":    "h264_amf",
		"apple":  "h264_videotoolbox",
	}
	codec, ok := codecMap[preferred]
	if !ok || preferred == "cpu" || preferred == "" {
		s.log("INFO", fmt.Sprintf("[Montage] Using CPU (libx264) codec (preferred: %s)", preferred), id, taskLabel)
		return "libx264"
	}

	// Probe command: small test encode
	// 128x128 is safer than 16x16 for some hardware encoders
	cmd := exec.Command(ffmpegPath,
		"-y", "-hide_banner", "-loglevel", "error",
		"-f", "lavfi", "-i", "color=black:s=128x128:d=0.1",
		"-pix_fmt", "yuv420p",
		"-c:v", codec, "-f", "null", "-",
	)
	utils.PrepareHiddenCmd(cmd)

	output, err := cmd.CombinedOutput()
	if err != nil {
		s.log("WARN", fmt.Sprintf("[Montage] Preferred GPU codec %s (%s) failed probe: %v\nOutput: %s\nFalling back to libx264.", preferred, codec, err, string(output)), id, taskLabel)
		return "libx264"
	}

	s.log("SUCCESS", fmt.Sprintf("[Montage] Verified GPU codec: %s (%s)", preferred, codec), id, taskLabel)
	return codec
}

// ProcessMontage handles the final video rendering stage (single-pass FFmpeg).
func (s *PipelineService) ProcessMontage(id string, taskLabel string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings, taskName string, subName string) error {
	montageEnabled := pSettings.MontageEnabled
	if val, ok := settings["montageEnabled"].(bool); ok {
		montageEnabled = val
	}

	if !montageEnabled {
		return nil
	}

	s.emitStageStatus(id, "montage", "waiting")

	// 1. Get FFmpeg and FFprobe paths
	ffmpegPath, err := utils.EnsureEngine("ffmpeg")
	if err != nil {
		return fmt.Errorf("failed to ensure ffmpeg: %v", err)
	}

	// Try to find ffprobe in user's home bin first, then fallback to EnsureEngine
	ffprobePath := ""
	if homeDir, err := os.UserHomeDir(); err == nil {
		binaryName := "ffprobe"
		if runtime.GOOS == "windows" {
			binaryName += ".exe"
		}
		customPath := filepath.Join(homeDir, "bin", binaryName)
		if _, err := os.Stat(customPath); err == nil {
			ffprobePath = customPath
		}
	}

	if ffprobePath == "" {
		ffprobePath, err = utils.EnsureEngine("ffprobe")
		if err != nil {
			return fmt.Errorf("failed to ensure ffprobe: %v", err)
		}
	}

	// 2. Identify inputs
	audioPath := filepath.Join(finalDir, "voice.mp3")
	if _, err := os.Stat(audioPath); err != nil {
		return fmt.Errorf("audio file not found: %s", audioPath)
	}

	imagesDir := filepath.Join(finalDir, "images")

	videoExts := map[string]bool{".mp4": true, ".mkv": true, ".mov": true, ".avi": true, ".webm": true}
	imageExts := map[string]bool{".jpg": true, ".jpeg": true, ".png": true, ".webp": true}

	// [SYNC] Determine total expected visual files
	var segments []string
	segmentsData, _ := os.ReadFile(filepath.Join(finalDir, "segments.json"))
	_ = json.Unmarshal(segmentsData, &segments)
	numSegments := len(segments)
	if numSegments == 0 {
		numSegments = 1 // Fallback
	}

	// Try to determine image_count (multiplication factor)
	imageCount := 0
	files, _ := os.ReadDir(imagesDir)
	for _, f := range files {
		if !f.IsDir() {
			imageCount++
		}
	}
	totalExpected := imageCount
	if totalExpected < numSegments {
		totalExpected = numSegments
	}

	var visualFiles []string
	var lastValidFile string
	for i := 1; i <= totalExpected; i++ {
		found := false
		// Order of preference: video, then images
		prefixes := []string{fmt.Sprintf("%d", i)}
		exts := []string{".mp4", ".png", ".jpg", ".jpeg", ".webp", ".webm", ".mov", ".avi", ".mkv"}

		for _, ext := range exts {
			file := prefixes[0] + ext
			path := filepath.Join(imagesDir, file)
			if _, err := os.Stat(path); err == nil {
				visualFiles = append(visualFiles, filepath.Join("images", file))
				lastValidFile = filepath.Join("images", file)
				found = true
				break
			}
		}

		if !found {
			if lastValidFile != "" {
				visualFiles = append(visualFiles, lastValidFile)
				s.log("WARN", fmt.Sprintf("[Montage] File %d not found, reusing %s", i, lastValidFile), id, taskLabel)
			} else {
				// Search for ANY valid file in the folder to use as fallback
				for _, f := range files {
					ext := strings.ToLower(filepath.Ext(f.Name()))
					if videoExts[ext] || imageExts[ext] {
						visualFiles = append(visualFiles, filepath.Join("images", f.Name()))
						lastValidFile = filepath.Join("images", f.Name())
						found = true
						break
					}
				}
			}
		}
	}
	numFiles := len(visualFiles)
	s.log("INFO", fmt.Sprintf("[Montage] Total visual files to process: %d", numFiles), id, taskLabel)
	if numFiles == 0 {
		return fmt.Errorf("no visual files found for montage")
	}

	// 3. Get Audio Duration
	audioDur, err := s.getDuration(ffprobePath, filepath.Join(finalDir, "voice.mp3"))
	if err != nil {
		return fmt.Errorf("failed to get audio duration: %v", err)
	}
	if audioDur <= 0 {
		return fmt.Errorf("audio duration is zero")
	}

	type MontageSegment struct {
		Start float64 `json:"start"`
		End   float64 `json:"end"`
	}
	var audioSegments []MontageSegment

	// 4. Settings
	// [OVERRIDES] Apply template/task settings before calculating derived variables
	if val, ok := settings["imageSyncEnabled"].(bool); ok {
		pSettings.ImageSyncEnabled = val
	}
	if val, ok := settings["montageSwayFactor"].(float64); ok {
		pSettings.MontageSwayFactor = val
	}
	if val, ok := settings["montageTransitionDuration"].(float64); ok {
		pSettings.MontageTransitionDuration = val
	}
	if val, ok := settings["montageTransitionEffect"].(string); ok {
		pSettings.MontageTransitionEffect = val
	}
	if val, ok := settings["montageZoomFactor"].(float64); ok {
		pSettings.MontageZoomFactor = val
	}
	if val, ok := settings["montageEncodingPreset"].(string); ok {
		pSettings.MontageEncodingPreset = val
	}
	if val, ok := settings["bitrate"]; ok {
		switch v := val.(type) {
		case float64:
			pSettings.MontageBitrate = int(v)
		case int:
			pSettings.MontageBitrate = v
		}
	}
	if val, ok := settings["montageResolution"].(string); ok {
		pSettings.MontageResolution = val
	}
	if val, ok := settings["montageFPS"]; ok {
		switch v := val.(type) {
		case float64:
			pSettings.MontageFPS = int(v)
		case int:
			pSettings.MontageFPS = v
		}
	}
	if val, ok := settings["montageUpscaleFactor"].(float64); ok {
		pSettings.MontageUpscaleFactor = val
	}
	if val, ok := settings["montageVideoCodec"].(string); ok {
		pSettings.MontageVideoCodec = val
	}
	if val, ok := settings["montageThreadsPerProcess"]; ok {
		switch v := val.(type) {
		case float64:
			pSettings.MontageThreadsPerProcess = int(v)
		case int:
			pSettings.MontageThreadsPerProcess = v
		}
	}
	if val, ok := settings["montageProcessPriority"].(string); ok {
		pSettings.MontageProcessPriority = val
	}
	if val, ok := settings["montageCPUCores"]; ok {
		switch v := val.(type) {
		case float64:
			pSettings.MontageCPUCores = int(v)
		case int:
			pSettings.MontageCPUCores = v
		}
	}
	if val, ok := settings["montageIntroVideoEnabled"].(bool); ok {
		pSettings.MontageIntroVideoEnabled = val
	}
	if val, ok := settings["montageIntroVideoPath"].(string); ok {
		pSettings.MontageIntroVideoPath = val
	}
	if val, ok := settings["imageShortVideoFillMode"].(string); ok {
		pSettings.ImageShortVideoFillMode = val
	}
	if val, ok := settings["montageWatermarkEnabled"].(bool); ok {
		pSettings.MontageWatermarkEnabled = val
	}
	if val, ok := settings["montageWatermarkPath"].(string); ok {
		pSettings.MontageWatermarkPath = val
	}
	if val, ok := settings["montageWatermarkPosition"].(string); ok {
		pSettings.MontageWatermarkPosition = val
	}
	if val, ok := settings["montageWatermarkOpacity"].(float64); ok {
		pSettings.MontageWatermarkOpacity = val
	}
	if val, ok := settings["montageWatermarkSize"]; ok {
		switch v := val.(type) {
		case float64:
			pSettings.MontageWatermarkSize = int(v)
		case int:
			pSettings.MontageWatermarkSize = v
		}
	}
	if val, ok := settings["montageWatermarkOnIntro"].(bool); ok {
		pSettings.MontageWatermarkOnIntro = val
	}
	if val, ok := settings["montageOverlayEnabled"].(bool); ok {
		pSettings.MontageOverlayEnabled = val
	}
	if val, ok := settings["montageOverlayPath"].(string); ok {
		pSettings.MontageOverlayPath = val
	}
	if val, ok := settings["montageOverlayOnIntro"].(bool); ok {
		pSettings.MontageOverlayOnIntro = val
	}
	if val, ok := settings["montageOverlayTriggersEnabled"].(bool); ok {
		pSettings.MontageOverlayTriggersEnabled = val
	}
	if val, ok := settings["montageOverlayTriggers"].([]interface{}); ok {
		var triggers []utils.OverlayTrigger
		for _, v := range val {
			if m, ok := v.(map[string]interface{}); ok {
				var tr utils.OverlayTrigger
				if phrase, ok := m["phrase"].(string); ok {
					tr.Phrase = phrase
				}
				if path, ok := m["path"].(string); ok {
					tr.Path = path
				}
				if x, ok := m["x"].(float64); ok {
					tr.X = int(x)
				}
				if y, ok := m["y"].(float64); ok {
					tr.Y = int(y)
				}
				if tr.Phrase != "" && tr.Path != "" {
					triggers = append(triggers, tr)
				}
			}
		}
		if len(triggers) > 0 {
			pSettings.MontageOverlayTriggers = triggers
		}
	}

	// Derived variables from finalized pSettings
	transDur := pSettings.MontageTransitionDuration
	if numFiles <= 1 {
		transDur = 0
	}

	isFadeFast := pSettings.MontageTransitionEffect == "fade_fast"

	baseW, baseH := 1920, 1080
	if pSettings.ImageWidth < pSettings.ImageHeight && pSettings.ImageWidth > 0 {
		baseW, baseH = 1080, 1920
	}
	switch pSettings.MontageResolution {
	case "720p":
		if baseW > baseH {
			baseW, baseH = 1280, 720
		} else {
			baseW, baseH = 720, 1280
		}
	case "2k":
		if baseW > baseH {
			baseW, baseH = 2560, 1440
		} else {
			baseW, baseH = 1440, 2560
		}
	}

	fps := pSettings.MontageFPS
	if fps <= 0 {
		fps = 30
	}

	upFactor := pSettings.MontageUpscaleFactor
	if upFactor < 1.0 {
		upFactor = 1.0
	}

	upW := int(math.Round(float64(baseW) * upFactor))
	upH := int(math.Round(float64(baseH) * upFactor))

	effectiveDurs := make([]float64, numFiles)
	if pSettings.ImageSyncEnabled {
		timings, err := utils.GetImageTimings(finalDir, audioDur, numFiles, visualFiles, taskLabel)
		if err != nil {
			clipDur := audioDur / float64(numFiles)
			if !isFadeFast {
				clipDur = (audioDur + float64(numFiles-1)*transDur) / float64(numFiles)
			}
			for i := range effectiveDurs {
				effectiveDurs[i] = clipDur
			}
		} else {
			for i, t := range timings {
				if i < numFiles {
					if isFadeFast {
						if i < len(timings)-1 {
							effectiveDurs[i] = timings[i+1].Start - t.Start
						} else {
							effectiveDurs[i] = audioDur - t.Start
						}
					} else {
						effectiveDurs[i] = t.Duration
						effectiveDurs[i] += transDur
					}
				}
			}
		}
	} else {
		if isFadeFast {
			clipDur := audioDur / float64(numFiles)
			for i := range effectiveDurs {
				effectiveDurs[i] = clipDur
			}
		} else {
			totalTransLoss := float64(numFiles-1) * transDur
			clipDur := (audioDur + totalTransLoss) / float64(numFiles)
			for i := range effectiveDurs {
				effectiveDurs[i] = clipDur
			}
		}
	}

	// [SYNC FIX] Snap clip durations to exact frame boundaries for concat mode.
	if isFadeFast && numFiles > 1 {
		totalFrames := int(math.Round(audioDur * float64(fps)))
		idealFrames := make([]float64, numFiles)
		baseFrames := make([]int, numFiles)
		baseSum := 0
		for i := 0; i < numFiles; i++ {
			idealFrames[i] = effectiveDurs[i] * float64(fps)
			baseFrames[i] = int(math.Floor(idealFrames[i]))
			if baseFrames[i] < 1 {
				baseFrames[i] = 1
			}
			baseSum += baseFrames[i]
		}
		remaining := totalFrames - baseSum
		if remaining > 0 {
			type indexRemainder struct {
				idx       int
				remainder float64
			}
			remainders := make([]indexRemainder, numFiles)
			for i := 0; i < numFiles; i++ {
				remainders[i] = indexRemainder{idx: i, remainder: idealFrames[i] - float64(baseFrames[i])}
			}
			sort.Slice(remainders, func(a, b int) bool {
				return remainders[a].remainder > remainders[b].remainder
			})
			for j := 0; j < remaining && j < numFiles; j++ {
				baseFrames[remainders[j].idx]++
			}
		}
		for i := 0; i < numFiles; i++ {
			effectiveDurs[i] = float64(baseFrames[i]) / float64(fps)
		}
	}

	swayFactor := pSettings.MontageSwayFactor
	zoomFactor := pSettings.MontageZoomFactor
	transEffect := pSettings.MontageTransitionEffect
	if transEffect == "" {
		transEffect = "fade"
	}
	threadsPerProcess := pSettings.MontageThreadsPerProcess
	videoCodec := s.resolveCodec(ffmpegPath, pSettings.MontageVideoCodec, id, taskLabel)
	procPriority := pSettings.MontageProcessPriority
	cpuCores := pSettings.MontageCPUCores

	// [CONTROL] -------------------------------------------------------------
	// Wait for user confirmation if Montage Control is enabled
	mControlEnabled := pSettings.MontageControlEnabled
	if val, ok := settings["montageControlEnabled"].(bool); ok {
		mControlEnabled = val
	}

	if mControlEnabled {
		s.emitStageStatus(id, "montage", "waiting")
		s.log("INFO", "[Control] Waiting for user montage review...", id, taskLabel)

		type MontageClip struct {
			Path           string  `json:"path"`
			Duration       float64 `json:"duration"`
			IsVideo        bool    `json:"isVideo"`
			ActualDuration float64 `json:"actualDuration"`
		}

		type MontageTrigger struct {
			Phrase    string  `json:"phrase"`
			Path      string  `json:"path"`
			StartTime float64 `json:"startTime"`
			Duration  float64 `json:"duration"`
			IsVideo   bool    `json:"isVideo"`
			X         int     `json:"x"`
			Y         int     `json:"y"`
			W         int     `json:"w"`
			H         int     `json:"h"`
		}

		type MontageWatermark struct {
			ID        string  `json:"id"`
			Path      string  `json:"path"`
			StartTime float64 `json:"startTime"`
			Duration  float64 `json:"duration"`
			X         int     `json:"x"`
			Y         int     `json:"y"`
			W         int     `json:"w"`
			H         int     `json:"h"`
			Opacity   float64 `json:"opacity"`
		}

		type MontagePlan struct {
			AudioDuration float64          `json:"audioDuration"`
			AudioPath     string           `json:"audioPath"`
			SubtitlePath  string           `json:"subtitlePath"`
			TransDuration float64          `json:"transDuration"`
			IsFadeFast    bool             `json:"isFadeFast"`
			Clips         []MontageClip    `json:"clips"`
			AudioSegments []MontageSegment `json:"audioSegments"`
			Triggers      []MontageTrigger `json:"triggers"`
			Watermarks    []MontageWatermark `json:"watermarks"`
			BaseW         int              `json:"baseW"`
			BaseH         int              `json:"baseH"`
			IntroPath     string           `json:"introPath,omitempty"`
			IntroDuration float64          `json:"introDuration,omitempty"`
			IntroIsVideo  bool             `json:"introIsVideo,omitempty"`
		}

		plan := MontagePlan{
			AudioDuration: audioDur,
			AudioPath:     audioPath,
			SubtitlePath:  filepath.Join(finalDir, "subtitle.srt"),
			TransDuration: transDur,
			IsFadeFast:    isFadeFast,
			BaseW:         baseW,
			BaseH:         baseH,
			Clips:         make([]MontageClip, numFiles),
			Triggers:      []MontageTrigger{},
			Watermarks:    []MontageWatermark{},
		}

		if pSettings.MontageIntroVideoEnabled && pSettings.MontageIntroVideoPath != "" {
			if _, err := os.Stat(pSettings.MontageIntroVideoPath); err == nil {
				dur, _ := s.getDuration(ffprobePath, pSettings.MontageIntroVideoPath)
				plan.IntroPath = pSettings.MontageIntroVideoPath
				plan.IntroDuration = dur
				plan.IntroIsVideo = true // Assuming it's video for now
			}
		}

		// Calculate Triggers for UI
		if pSettings.MontageOverlayTriggersEnabled && len(pSettings.MontageOverlayTriggers) > 0 {
			assPath := filepath.Join(finalDir, "subtitle.ass")
			srtPath := filepath.Join(finalDir, "subtitle.srt")
			
			// Determine which subtitle file to use for finding timings
			activeSubPath := assPath
			if _, err := os.Stat(assPath); err != nil {
				if _, errSrt := os.Stat(srtPath); errSrt == nil {
					activeSubPath = srtPath
				}
			}

			for _, tr := range pSettings.MontageOverlayTriggers {
				if tr.Phrase == "" || tr.Path == "" {
					continue
				}
				startT := s.findTextTiming(activeSubPath, tr.Phrase, taskLabel)
				if startT != nil {
					trDur := 3.0 // Default for images
					ext := strings.ToLower(filepath.Ext(tr.Path))
					isTrVideo := videoExts[ext]
					if isTrVideo {
						if d, err := s.getDuration(ffprobePath, tr.Path); err == nil && d > 0 {
							trDur = d
						}
					}
					plan.Triggers = append(plan.Triggers, MontageTrigger{
						Phrase:    tr.Phrase,
						Path:      tr.Path,
						StartTime: *startT,
						Duration:  trDur,
						IsVideo:   isTrVideo,
						X:         tr.X,
						Y:         tr.Y,
						W:         baseW, // Full screen by default
						H:         baseH,
					})
				}
			}
		}

		// Calculate Watermarks for UI
		if len(pSettings.MontageWatermarks) > 0 {
			for _, wm := range pSettings.MontageWatermarks {
				if wm.Path == "" {
					continue
				}
				startTime := 0.0
				if wm.StartTime != nil {
					startTime = *wm.StartTime
				}
				duration := 5.0
				if wm.Duration != nil {
					duration = *wm.Duration
				}
				plan.Watermarks = append(plan.Watermarks, MontageWatermark{
					ID:        wm.ID,
					Path:      wm.Path,
					StartTime: startTime,
					Duration:  duration,
					X:         wm.X,
					Y:         wm.Y,
					W:         wm.W,
					H:         wm.H,
					Opacity:   wm.Opacity,
				})
			}
		}

		for i, f := range visualFiles {
			ext := strings.ToLower(filepath.Ext(f))
			isVid := videoExts[ext]
			actualDur := 0.0
			if isVid {
				actualDur, _ = s.getDuration(ffprobePath, filepath.Join(finalDir, f))
			}
			plan.Clips[i] = MontageClip{
				Path:           filepath.Join(finalDir, f),
				Duration:       effectiveDurs[i],
				IsVideo:        isVid,
				ActualDuration: actualDur,
			}
		}

		planJSON, _ := json.Marshal(plan)

		resChan := make(chan string)
		s.pendingControl.Store(id+"_montage", resChan)

		if s.OnRequestMontageControl != nil {
			s.OnRequestMontageControl(id, string(planJSON))
		}

		// Block until result received or timeout/context cancel
		select {
		case actionData := <-resChan:
			// Parse modified plan if we get new durations/files from UI
			if strings.HasPrefix(actionData, "confirm_v2:") {
				parts := strings.Split(actionData, ";")
				for _, p := range parts {
					if strings.HasPrefix(p, "confirm_v2:") {
						clipStr := strings.TrimPrefix(p, "confirm_v2:")
						clipParts := strings.Split(clipStr, "::")
						var newVisualFiles []string
						var newEffectiveDurs []float64
						for _, cp := range clipParts {
							subParts := strings.Split(cp, "|")
							if len(subParts) == 3 {
								path := subParts[0]
								dur, _ := strconv.ParseFloat(subParts[1], 64)
								newVisualFiles = append(newVisualFiles, path)
								newEffectiveDurs = append(newEffectiveDurs, dur)
							}
						}
						if len(newVisualFiles) > 0 {
							visualFiles = newVisualFiles
							effectiveDurs = newEffectiveDurs
							numFiles = len(visualFiles)
						}
					} else if strings.HasPrefix(p, "segments:") {
						segStr := strings.TrimPrefix(p, "segments:")
						segStrs := strings.Split(segStr, "|")
						var newSegments []MontageSegment
						var totalAudio float64
						for _, s := range segStrs {
							coords := strings.Split(s, ",")
							if len(coords) == 2 {
								st, _ := strconv.ParseFloat(coords[0], 64)
								en, _ := strconv.ParseFloat(coords[1], 64)
								if en > st {
									newSegments = append(newSegments, MontageSegment{Start: st, End: en})
									totalAudio += (en - st)
								}
							}
						}
						if len(newSegments) > 0 {
							audioSegments = newSegments
							audioDur = totalAudio
						}
					} else if strings.HasPrefix(p, "triggers:") {
						trStr := strings.TrimPrefix(p, "triggers:")
						trItems := strings.Split(trStr, "::")
						var newTrs []utils.OverlayTrigger
						for _, item := range trItems {
							bits := strings.Split(item, "|")
							if len(bits) >= 9 {
								// phrase|path|startTime|duration|x|y|w|h|type
								start, _ := strconv.ParseFloat(bits[2], 64)
								dur, _ := strconv.ParseFloat(bits[3], 64)
								x, _ := strconv.Atoi(bits[4])
								y, _ := strconv.Atoi(bits[5])
								w, _ := strconv.Atoi(bits[6])
								h, _ := strconv.Atoi(bits[7])
								newTrs = append(newTrs, utils.OverlayTrigger{
									Phrase:    bits[0],
									Path:      bits[1],
									X:         x,
									Y:         y,
									W:         w,
									H:         h,
									StartTime: &start,
									Duration:  &dur,
								})
							}
						}
						pSettings.MontageOverlayTriggers = newTrs
					} else if strings.HasPrefix(p, "watermarks:") {
						wmStr := strings.TrimPrefix(p, "watermarks:")
						wmItems := strings.Split(wmStr, "::")
						var newWms []utils.OverlayWatermark
						for _, item := range wmItems {
							bits := strings.Split(item, "|")
							if len(bits) >= 9 {
								// id|path|startTime|duration|x|y|w|h|opacity
								start, _ := strconv.ParseFloat(bits[2], 64)
								dur, _ := strconv.ParseFloat(bits[3], 64)
								x, _ := strconv.Atoi(bits[4])
								y, _ := strconv.Atoi(bits[5])
								w, _ := strconv.Atoi(bits[6])
								h, _ := strconv.Atoi(bits[7])
								opacity, _ := strconv.ParseFloat(bits[8], 64)
								newWms = append(newWms, utils.OverlayWatermark{
									ID:        bits[0],
									Path:      bits[1],
									X:         x,
									Y:         y,
									W:         w,
									H:         h,
									StartTime: &start,
									Duration:  &dur,
									Opacity:   opacity,
								})
							}
						}
						pSettings.MontageWatermarks = newWms
					} else if strings.HasPrefix(p, "intro:") {
						introData := strings.TrimPrefix(p, "intro:")
						if introData == "" || introData == "none" {
							pSettings.MontageIntroVideoEnabled = false
						} else {
							pSettings.MontageIntroVideoEnabled = true
							pSettings.MontageIntroVideoPath = introData
						}
					}
				}
				s.log("SUCCESS", fmt.Sprintf("[Control] Montage updated (V2). Audio length: %.2fs, Clips: %d, Triggers: %d", audioDur, numFiles, len(pSettings.MontageOverlayTriggers)), id, taskLabel)
			} else if strings.HasPrefix(actionData, "confirm:") {
				mainParts := strings.Split(actionData, ";segments:")
				parts := strings.Split(strings.TrimPrefix(mainParts[0], "confirm:"), ",")
				for i, p := range parts {
					if i < numFiles {
						if parsedDur, err := strconv.ParseFloat(p, 64); err == nil && parsedDur > 0 {
							effectiveDurs[i] = parsedDur
						}
					}
				}
				// (rest of confirm: parsing remains same, segments handling follows)
				if len(mainParts) > 1 {
					segStrs := strings.Split(mainParts[1], "|")
					var newSegments []MontageSegment
					var totalAudio float64
					for _, s := range segStrs {
						coords := strings.Split(s, ",")
						if len(coords) == 2 {
							st, _ := strconv.ParseFloat(coords[0], 64)
							en, _ := strconv.ParseFloat(coords[1], 64)
							if en > st {
								newSegments = append(newSegments, MontageSegment{Start: st, End: en})
								totalAudio += (en - st)
							}
						}
					}
					if len(newSegments) > 0 {
						audioSegments = newSegments
						audioDur = totalAudio
					}
				}
				s.log("SUCCESS", fmt.Sprintf("[Control] Montage updated. Audio length: %.2fs, Clips: %d", audioDur, numFiles), id, taskLabel)
			} else if actionData == "cancel" {
				s.log("INFO", "[Control] Task cancelled by user", id, taskLabel)
				return fmt.Errorf("task cancelled")
			} else {
				s.log("SUCCESS", "[Control] Montage approved (default timings).", id, taskLabel)
			}
			s.MarkMontageConfirmed(id)
		case <-s.ctx.Done():
			s.log("INFO", "[Control] Task cancelled while waiting for montage review", id, taskLabel)
			return fmt.Errorf("task cancelled")
		}
		s.pendingControl.Delete(id + "_montage")
	} else {
		// If control not enabled, still mark as confirmed to not block others
		s.MarkMontageConfirmed(id)
	}

	// [BARRIER] Wait for all controlled tasks in the batch
	s.log("INFO", "[Pipeline] Waiting for batch montage synchronization...", id, taskLabel)
	s.WaitForMontageBatch(id)

	// [SEMAPHORE] Acquire slot AFTER batch confirmation
	s.log("INFO", "[Pipeline] Waiting for montage slot...", id, taskLabel)
	sem := s.getMontageSem()
	sem <- struct{}{}
	defer func() { <-sem }()

	s.log("INFO", "[Pipeline] Montage slot acquired, starting...", id, taskLabel)
	s.emitStageStatus(id, "montage", "running")

	// Helper for relative paths
	getRel := func(p string) string {
		if p == "" {
			return p
		}
		if rel, err := filepath.Rel(finalDir, p); err == nil {
			if !strings.HasPrefix(rel, "..") {
				return rel
			}
		}
		return p
	}

	// Build final filename: TaskName + TemplateName
	limit := 180
	tplName := subName
	if tplName == "" || tplName == "Default" {
		tplName = ""
	}

	safeTask := utils.SanitizeFilename(taskName)
	safeTpl := utils.SanitizeFilename(tplName)

	if safeTask == "" {
		safeTask = "Task"
	}

	var finalBaseName string
	if safeTpl != "" {
		tplRunes := []rune(safeTpl)
		availableForTask := limit - len(tplRunes) - 3
		if availableForTask < 20 {
			availableForTask = 20
		}
		taskRunes := []rune(safeTask)
		if len(taskRunes) > availableForTask {
			safeTask = string(taskRunes[:availableForTask])
		}
		finalBaseName = strings.TrimRight(safeTask, ". ") + " - " + safeTpl
	} else {
		taskRunes := []rune(safeTask)
		if len(taskRunes) > limit {
			safeTask = string(taskRunes[:limit])
		}
		finalBaseName = strings.TrimRight(safeTask, ". ")
	}

	outputFile := strings.TrimSpace(finalBaseName) + ".mp4"
	s.log("INFO", fmt.Sprintf("[Montage] Output file will be: %s", outputFile), id, taskLabel)

	// 5. Build filter graph — single-pass
	type inputSpec struct {
		loop       bool
		path       string
		streamLoop bool
		framerate  int // Explicit -framerate for looped images (0 = not set)
	}
	var inputSpecs []inputSpec
	var filterParts []string

	introIdx := -1
	watermarkIdx := -1
	introDur := 0.0
	if pSettings.MontageIntroVideoEnabled && pSettings.MontageIntroVideoPath != "" {
		if _, err := os.Stat(pSettings.MontageIntroVideoPath); err == nil {
			introIdx = 0
			introDur, _ = s.getDuration(ffprobePath, pSettings.MontageIntroVideoPath)
			hasA := s.hasAudio(ffprobePath, pSettings.MontageIntroVideoPath)
			inputSpecs = append(inputSpecs, inputSpec{loop: false, path: getRel(pSettings.MontageIntroVideoPath)})

			// Process intro video to match output format (Premium Blurred Background Fit)
			vFilter := fmt.Sprintf(
				"[0:v]scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,boxblur=20:10[bg]; "+
					"[0:v]scale=%d:%d:force_original_aspect_ratio=decrease[fg]; "+
					"[bg][fg]overlay=(W-w)/2:(H-h)/2,format=yuv420p,setsar=1,fps=%d,settb=AVTB[v_intro_base]",
				baseW, baseH, baseW, baseH, baseW, baseH, fps)

			aFilter := ""
			if hasA {
				aFilter = "[0:a]aresample=44100,aformat=sample_fmts=fltp:sample_rates=44100:channel_layouts=stereo[a_intro]"
			} else {
				filterParts = append(filterParts, fmt.Sprintf("anullsrc=r=44100:cl=stereo:d=%.6f,aformat=sample_fmts=fltp[a_intro_silence]", introDur))
				aFilter = "[a_intro_silence]copy[a_intro]"
			}
			filterParts = append(filterParts, vFilter, aFilter)
		}
	}

	visualOffset := 0
	if introIdx != -1 {
		visualOffset = 1
	}

	padAmount := 0.0
	if !isFadeFast {
		padAmount = 1.0 // Pad each clip to prevent xfade frame underrun
	}

	for idx, vFile := range visualFiles {
		ext := strings.ToLower(filepath.Ext(vFile))
		isVideo := videoExts[ext]
		vIn := fmt.Sprintf("[%d:v]", idx+visualOffset)
		vOut := fmt.Sprintf("v%d_final", idx)
		relVFile := getRel(vFile)
		// For images: don't use -loop 1. zoompan with d=<frames> generates all
		// needed frames from a single input image (its canonical usage).
		// Using -loop 1 would feed at 25fps by default, causing rate mismatches.
		inputSpecs = append(inputSpecs, inputSpec{loop: false, path: relVFile})

		if isVideo {
			actualDur, _ := s.getDuration(ffprobePath, filepath.Join(finalDir, vFile))
			requiredDur := effectiveDurs[idx]
			paddedDur := requiredDur + padAmount

			if actualDur > 0 && actualDur < requiredDur {
				if pSettings.ImageShortVideoFillMode == "mirror" {
					// Mirror Transition: Forward -> (fade) -> Forward Mirrored
					tDur := 0.5
					if actualDur < 1.0 {
						tDur = actualDur * 0.3
					}
					offset := actualDur - tDur
					s.log("INFO", fmt.Sprintf("[Montage] [%d] Applying mirror loop with transition (actual: %.2fs, req: %.2fs, trans: %.2fs)", idx, actualDur, requiredDur, tDur), id, taskLabel)
					filterParts = append(filterParts, fmt.Sprintf(
						"[%d:v]trim=duration=%.6f,setpts=PTS-STARTPTS[f%d_1];"+
							"[f%d_1]split=2[pts%d_a][pts%d_b];"+
							"[pts%d_b]hflip,setpts=PTS-STARTPTS[b%d_wd];"+
							"[pts%d_a][b%d_wd]xfade=transition=fade:duration=%.3f:offset=%.3f[v%d_boom];"+
							"[v%d_boom]loop=loop=-1:size=0:start=0,scale=%d:%d,scale=1.07*iw:-1,crop=%d:%d:0:0,format=yuv420p,setsar=1,fps=%d,settb=AVTB,trim=duration=%.6f,setpts=PTS-STARTPTS[%s]",
						idx+visualOffset, actualDur, idx, idx, idx, idx, idx, idx, idx, idx, tDur, offset, idx, idx, baseW, baseH, baseW, baseH, fps, paddedDur, vOut,
					))
				} else {
					// Boomerang: Forward -> Backward
					s.log("INFO", fmt.Sprintf("[Montage] [%d] Applying boomerang loop (actual: %.2fs, req: %.2fs)", idx, actualDur, requiredDur), id, taskLabel)
					filterParts = append(filterParts, fmt.Sprintf(
						"[%d:v]trim=duration=%.6f,setpts=PTS-STARTPTS[f%d_1];"+
							"[f%d_1]split=2[pts%d_a][pts%d_b];"+
							"[pts%d_b]reverse,setpts=PTS-STARTPTS[b%d_wd];"+
							"[pts%d_a][b%d_wd]concat=n=2:v=1[v%d_boom];"+
							"[v%d_boom]loop=loop=-1:size=0:start=0,scale=%d:%d,scale=1.07*iw:-1,crop=%d:%d:0:0,format=yuv420p,setsar=1,fps=%d,settb=AVTB,trim=duration=%.6f,setpts=PTS-STARTPTS[%s]",
						idx+visualOffset, actualDur, idx, idx, idx, idx, idx, idx, idx, idx, idx, idx, baseW, baseH, baseW, baseH, fps, paddedDur, vOut,
					))
				}
			} else {
				if actualDur <= 0 {
					actualDur = requiredDur
				}
				effDur := math.Min(actualDur, requiredDur)
				filterParts = append(filterParts, fmt.Sprintf(
					"%strim=duration=%.6f,scale=%d:%d,scale=1.07*iw:-1,crop=%d:%d:0:0,format=yuv420p,setsar=1,fps=%d,settb=AVTB,tpad=stop_mode=clone:stop=-1,trim=duration=%.6f,setpts=PTS-STARTPTS[%s]",
					vIn, effDur, baseW, baseH, baseW, baseH, fps, paddedDur, vOut,
				))
			}
		} else { // Image
			requiredDur := effectiveDurs[idx]
			paddedDur := requiredDur + padAmount
			vUp := fmt.Sprintf("v%d_up", idx)

			// Scale to upscaled resolution
			filterParts = append(filterParts, fmt.Sprintf(
				"%sscale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,format=yuv420p,setsar=1[%s]",
				vIn, upW, upH, upW, upH, vUp,
			))

			// Zoom expression (Python exact port)
			baseZoomVal := 1.0
			if swayFactor > 0 {
				baseZoomVal = 1.1
			}
			var zExpr string
			if zoomFactor > 0 {
				zAmp := 0.15 * zoomFactor
				zExpr = fmt.Sprintf("%.3f+%.3f*(1-cos(6.283*((on/%d)/%.6f)))/2",
					baseZoomVal, zAmp, fps, requiredDur)
			} else {
				zExpr = fmt.Sprintf("%.3f", baseZoomVal)
			}

			// Sway expression
			var xExpr, yExpr string
			if swayFactor > 0 {
				baseAmpX := 50.0 * upFactor * swayFactor
				baseAmpY := 25.0 * upFactor * swayFactor
				valX := fmt.Sprintf("sin(on*0.0200)*%.2f + cos(on*0.0500)*%.2f", baseAmpX, baseAmpX/2)
				valY := fmt.Sprintf("cos(on*0.0250)*%.2f + sin(on*0.0600)*%.2f", baseAmpY, baseAmpY/2)
				xExpr = fmt.Sprintf("iw/2-(iw/zoom/2)+%s", valX)
				yExpr = fmt.Sprintf("ih/2-(ih/zoom/2)+%s", valY)
			} else {
				xExpr = "iw/2-(iw/zoom/2)"
				yExpr = "ih/2-(ih/zoom/2)"
			}

			// Calculate exact frame count for zoompan to produce precise duration.
			// Using d=<frames> instead of d=1 ensures zoompan generates exactly the
			// right number of frames, preventing rate mismatches with the input stream.
			exactFrames := int(math.Round(paddedDur * float64(fps)))
			if exactFrames < 1 {
				exactFrames = 1
			}
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s]zoompan=z='%s':x='%s':y='%s':d=%d:s=%dx%d:fps=%d,format=yuv420p,setsar=1,settb=AVTB,trim=duration=%.6f,setpts=PTS-STARTPTS[%s]",
				vUp, zExpr, xExpr, yExpr, exactFrames, baseW, baseH, fps, paddedDur, vOut,
			))
		}
	}

	// Transitions — per-clip effectiveDurs for correct offsets
	lastV := ""
	if isFadeFast {
		s.log("INFO", "[Montage] Using Fast Fade (fade in/out + concat) transition", id, taskLabel)
		var concatParts []string
		for i := 0; i < numFiles; i++ {
			vIn := fmt.Sprintf("v%d_final", i)
			vOut := fmt.Sprintf("v%d_faded", i)
			dur := effectiveDurs[i]

			// Limit fade duration to 40% of clip duration to prevent disappearing images
			safeFade := transDur / 2
			if safeFade > dur*0.4 {
				safeFade = dur * 0.4
			}

			fadeInDur := safeFade
			fadeOutSt := dur - safeFade
			if fadeOutSt < 0 {
				fadeOutSt = 0
			}
			fadeOutDur := safeFade

			if i == numFiles-1 {
				// The last clip doesn't fade out locally. We fade out the final combined video globally!
				filterParts = append(filterParts, fmt.Sprintf(
					"[%s]fade=t=in:st=0:d=%.3f[%s]",
					vIn, fadeInDur, vOut,
				))
			} else {
				filterParts = append(filterParts, fmt.Sprintf(
					"[%s]fade=t=in:st=0:d=%.3f,fade=t=out:st=%.3f:d=%.3f[%s]",
					vIn, fadeInDur, fadeOutSt, fadeOutDur, vOut,
				))
			}
			concatParts = append(concatParts, fmt.Sprintf("[%s]", vOut))
			lastV = vOut
		}
		if numFiles > 1 {
			filterParts = append(filterParts, fmt.Sprintf("%sconcat=n=%d:v=1:a=0[v_montage_raw]", strings.Join(concatParts, ""), numFiles))
			lastV = "v_montage_raw"
		}
	} else {
		lastV = "v0_final"
		if numFiles > 1 {
			currentOffset := effectiveDurs[0] - transDur
			for i := 1; i < numFiles; i++ {
				nextV := fmt.Sprintf("v%d_final", i)
				targetV := fmt.Sprintf("v_m%d", i)
				filterParts = append(filterParts, fmt.Sprintf(
					"[%s][%s]xfade=transition=%s:duration=%.3f:offset=%.3f[%s]",
					lastV, nextV, transEffect, transDur, currentOffset, targetV,
				))
				lastV = targetV
				currentOffset += effectiveDurs[i] - transDur
			}
		}
	}

	// Pad the montage stream to infinity before subtitles, so subtitles have infinite frames to draw on.
	// This prevents subtitles from freezing if the combined video naturally ends early due to rounding deficits.
	filterParts = append(filterParts, fmt.Sprintf("[%s]tpad=stop_mode=clone:stop=-1[v_padded_montage]", lastV))

	// Subtitles
	montageV := "v_padded_montage"
	assName := "subtitle.ass"
	if len(audioSegments) > 0 {
		srtPath := filepath.Join(finalDir, "subtitle.srt")
		if srtData, err := os.ReadFile(srtPath); err == nil {
			var utilsSegments []utils.AudioSegment
			for _, seg := range audioSegments {
				utilsSegments = append(utilsSegments, utils.AudioSegment{Start: seg.Start, End: seg.End})
			}
			trimmedSrt := utils.TrimSrt(string(srtData), utilsSegments)
			_ = os.WriteFile(filepath.Join(finalDir, "subtitle_trimmed.srt"), []byte(trimmedSrt), 0644)

			trimmedAss, err := utils.SrtToAss(trimmedSrt, pSettings)
			if err == nil {
				_ = os.WriteFile(filepath.Join(finalDir, "subtitle_trimmed.ass"), []byte(trimmedAss), 0644)
				assName = "subtitle_trimmed.ass"
			}
		}
	}

	assPath := filepath.Join(finalDir, assName)
	if _, err := os.Stat(assPath); err == nil {
		filterParts = append(filterParts, fmt.Sprintf("[%s]subtitles='%s'[v_sub]", montageV, assName))
		montageV = "v_sub"
	}

	// Watermark preparation (once for all uses)
	wmAvailable := false
	overlayX := "W-w-20"
	overlayY := "H-h-20"
	if pSettings.MontageWatermarkEnabled && pSettings.MontageWatermarkPath != "" {
		if _, err := os.Stat(pSettings.MontageWatermarkPath); err == nil {
			wmAvailable = true
			watermarkIdx = len(inputSpecs)
			inputSpecs = append(inputSpecs, inputSpec{loop: true, path: getRel(pSettings.MontageWatermarkPath)})

			wmScale := float64(pSettings.MontageWatermarkSize) / 100.0
			wmOpacity := pSettings.MontageWatermarkOpacity
			if wmOpacity <= 0 {
				wmOpacity = 0.8
			}

			// Pre-process watermark: scale and opacity
			filterParts = append(filterParts, fmt.Sprintf(
				"[%d:v]scale=%d*%f:-1,format=rgba,colorchannelmixer=aa=%.3f[wm]",
				watermarkIdx, baseW, wmScale, wmOpacity,
			))

			// Coordinates
			switch pSettings.MontageWatermarkPosition {
			case "top-left":
				overlayX = "20"
				overlayY = "20"
			case "top-center":
				overlayX = "(W-w)/2"
				overlayY = "20"
			case "top-right":
				overlayX = "W-w-20"
				overlayY = "20"
			case "bottom-left":
				overlayX = "20"
				overlayY = "H-h-20"
			case "bottom-center":
				overlayX = "(W-w)/2"
				overlayY = "H-h-20"
			case "bottom-right":
				overlayX = "W-w-20"
				overlayY = "H-h-20"
			case "center":
				overlayX = "(W-w)/2"
				overlayY = "(H-h)/2"
			}
		}
	}

	// Overlay preparation
	overlayAvailable := false
	overlayIdx := -1
	if pSettings.MontageOverlayEnabled && pSettings.MontageOverlayPath != "" {
		if _, err := os.Stat(pSettings.MontageOverlayPath); err == nil {
			overlayAvailable = true
			overlayIdx = len(inputSpecs)

			ext := strings.ToLower(filepath.Ext(pSettings.MontageOverlayPath))
			isOverlayVideo := videoExts[ext]
			inputSpecs = append(inputSpecs, inputSpec{
				loop:       !isOverlayVideo,
				path:       getRel(pSettings.MontageOverlayPath),
				streamLoop: isOverlayVideo,
			})

			// Pre-process overlay: scale to fit video
			filterParts = append(filterParts, fmt.Sprintf(
				"[%d:v]scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,format=rgba[ovl]",
				overlayIdx, baseW, baseH, baseW, baseH,
			))
		}
	}

	// Trigger-based overlays
	type triggerInfo struct {
		phrase    string
		path      string
		startTime float64
		duration  float64
		idx       int
		x         int
		y         int
		w         int
		h         int
	}
	var activeTriggers []triggerInfo

	// Custom Watermarks
	type watermarkInfo struct {
		id        string
		path      string
		startTime float64
		duration  float64
		idx       int
		x         int
		y         int
		w         int
		h         int
		opacity   float64
	}
	var activeCustomWatermarks []watermarkInfo
	if len(pSettings.MontageWatermarks) > 0 {
		for _, wm := range pSettings.MontageWatermarks {
			if wm.Path == "" {
				continue
			}
			if _, err := os.Stat(wm.Path); err != nil {
				s.log("WARN", fmt.Sprintf("[Montage] Custom watermark path not found: %s", wm.Path), id, taskLabel)
				continue
			}

			startT := 0.0
			if wm.StartTime != nil {
				startT = *wm.StartTime
			}
			durT := 5.0
			if wm.Duration != nil {
				durT = *wm.Duration
			}

			if startT > audioDur {
				continue
			}

			customWmIdx := len(inputSpecs)
			inputSpecs = append(inputSpecs, inputSpec{loop: true, path: getRel(wm.Path)})

			activeCustomWatermarks = append(activeCustomWatermarks, watermarkInfo{
				id:        wm.ID,
				path:      wm.Path,
				startTime: startT,
				duration:  durT,
				idx:       customWmIdx,
				x:         wm.X,
				y:         wm.Y,
				w:         wm.W,
				h:         wm.H,
				opacity:   wm.Opacity,
			})
		}
	}
	if len(pSettings.MontageOverlayTriggers) > 0 {
		for _, tr := range pSettings.MontageOverlayTriggers {
			if tr.Phrase == "" || tr.Path == "" {
				continue
			}
			if _, err := os.Stat(tr.Path); err != nil {
				s.log("WARN", fmt.Sprintf("[Montage] Trigger path not found: %s", tr.Path), id, taskLabel)
				continue
			}

			var startT *float64
			if tr.StartTime != nil {
				startT = tr.StartTime
			} else {
				startT = s.findTextTiming(assPath, tr.Phrase, taskLabel)
			}
			
			if startT != nil {
				s.log("INFO", fmt.Sprintf("[Montage] Active trigger '%s' at %.2fs", tr.Phrase, *startT), id, taskLabel)
				tIdx := len(inputSpecs)
				ext := strings.ToLower(filepath.Ext(tr.Path))
				isTrVideo := videoExts[ext]
				inputSpecs = append(inputSpecs, inputSpec{
					loop:       !isTrVideo,
					path:       getRel(tr.Path),
					streamLoop: false, 
				})

				trDur := 3.0
				if tr.Duration != nil {
					trDur = *tr.Duration
				} else if isTrVideo {
					if d, err := s.getDuration(ffprobePath, tr.Path); err == nil && d > 0 {
						trDur = d
					}
				}

				activeTriggers = append(activeTriggers, triggerInfo{
					phrase:    tr.Phrase,
					path:      tr.Path,
					startTime: *startT,
					duration:  trDur,
					idx:       tIdx,
					x:         tr.X,
					y:         tr.Y,
					w:         tr.W,
					h:         tr.H,
				})
			} else {
				s.log("INFO", fmt.Sprintf("[Montage] Trigger phrase '%s' not found", tr.Phrase), id, taskLabel)
			}
		}
	}
	s.log("INFO", fmt.Sprintf("[Montage] Active triggers found: %d", len(activeTriggers)), id, taskLabel)

	// Split streams for double use (intro + montage) if needed
	wmMontageTag := "wm"
	wmIntroTag := ""
	if wmAvailable {
		if introIdx != -1 && pSettings.MontageWatermarkOnIntro {
			filterParts = append(filterParts, "[wm]split[wm_montage][wm_intro]")
			wmMontageTag = "wm_montage"
			wmIntroTag = "wm_intro"
		}
	}

	ovlMontageTag := "ovl"
	ovlIntroTag := ""
	if overlayAvailable {
		if introIdx != -1 && pSettings.MontageOverlayOnIntro {
			filterParts = append(filterParts, "[ovl]split[ovl_montage][ovl_intro]")
			ovlMontageTag = "ovl_montage"
			ovlIntroTag = "ovl_intro"
		}
	}

	// Apply Watermark & Overlay to Intro
	finalIntroV := "v_intro_base"
	if introIdx != -1 {
		currentV := "v_intro_base"
		// Watermark
		if wmAvailable && pSettings.MontageWatermarkOnIntro && wmIntroTag != "" {
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][%s]overlay=x=%s:y=%s:format=yuv420:shortest=1[v_intro_wm]",
				currentV, wmIntroTag, overlayX, overlayY,
			))
			currentV = "v_intro_wm"
		}
		// Overlay
		if overlayAvailable && pSettings.MontageOverlayOnIntro && ovlIntroTag != "" {
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][%s]overlay=x=0:y=0:format=yuv420:shortest=1[v_intro_ovl]",
				currentV, ovlIntroTag,
			))
			currentV = "v_intro_ovl"
		}

		if currentV != "v_intro_base" {
			finalIntroV = currentV
		} else {
			finalIntroV = "v_intro_base"
		}
	}

	// Apply Watermark & Overlay to Montage
	currentMontageV := montageV
	if wmAvailable {
		filterParts = append(filterParts, fmt.Sprintf(
			"[%s][%s]overlay=x=%s:y=%s:format=yuv420:shortest=1[v_wm_montage]",
			currentMontageV, wmMontageTag, overlayX, overlayY,
		))
		currentMontageV = "v_wm_montage"
	}
	if overlayAvailable {
		filterParts = append(filterParts, fmt.Sprintf(
			"[%s][%s]overlay=x=0:y=0:format=yuv420:shortest=1[v_ovl_montage]",
			currentMontageV, ovlMontageTag,
		))
		currentMontageV = "v_ovl_montage"
	}

	for i, tr := range activeTriggers {
		trDur := tr.duration
		ext := strings.ToLower(filepath.Ext(tr.path))
		isTrVideo := videoExts[ext]

		// 1. Pre-process trigger: scale, crop, format, and setpts (delay)
		trigProcessedLabel := fmt.Sprintf("v_trig_ready_%d", i)
		filterParts = append(filterParts, fmt.Sprintf(
			"[%d:v]format=yuva420p,scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,setpts=PTS-STARTPTS+%.3f/TB[%s]",
			tr.idx, tr.w, tr.h, tr.w, tr.h, tr.startTime, trigProcessedLabel,
		))

		// 2. Apply overlay
		outLabel := fmt.Sprintf("v_tr_%d", i)
		enableExpr := fmt.Sprintf("between(t,%.3f,%.3f)", tr.startTime, tr.startTime+trDur)
		if isTrVideo {
			// For videos, we can also use eof_action=pass
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][%s]overlay=x=%d:y=%d:eof_action=pass:enable='%s'[%s]",
				currentMontageV, trigProcessedLabel, tr.x, tr.y, enableExpr, outLabel,
			))
		} else {
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][%s]overlay=x=%d:y=%d:enable='%s'[%s]",
				currentMontageV, trigProcessedLabel, tr.x, tr.y, enableExpr, outLabel,
			))
		}
		currentMontageV = outLabel
	}

	for i, wm := range activeCustomWatermarks {
		wmDur := wm.duration
		wmProcessedLabel := fmt.Sprintf("v_pwm_ready_%d", i)
		
		opacity := wm.opacity
		if opacity <= 0 {
			opacity = 1.0
		}

		filterParts = append(filterParts, fmt.Sprintf(
			"[%d:v]format=yuva420p,scale=%d:%d,colorchannelmixer=aa=%.3f,setpts=PTS-STARTPTS+%.3f/TB[%s]",
			wm.idx, wm.w, wm.h, opacity, wm.startTime, wmProcessedLabel,
		))

		outLabel := fmt.Sprintf("v_pwm_out_%d", i)
		enableExpr := fmt.Sprintf("between(t,%.3f,%.3f)", wm.startTime, wm.startTime+wmDur)
		filterParts = append(filterParts, fmt.Sprintf(
			"[%s][%s]overlay=x=%d:y=%d:enable='%s'[%s]",
			currentMontageV, wmProcessedLabel, wm.x, wm.y, enableExpr, outLabel,
		))
		currentMontageV = outLabel
	}

	montageV = currentMontageV

	// Global Video Fade Out (At the exact end of actual audio)
	finalFadeOut := ""
	if transDur > 0 {
		finalFadeSt := audioDur - (transDur / 2)
		if finalFadeSt < 0 {
			finalFadeSt = 0
		}
		finalFadeOut = fmt.Sprintf(",fade=t=out:st=%.3f:d=%.3f", finalFadeSt, transDur/2)
	}

	// Montage trim
	// We trim exactly to audioDur to match the voice length.
	// Because we added an infinite tpad before subtitles, there will NEVER be an underrun.
	filterParts = append(filterParts, fmt.Sprintf(
		"[%s]trim=duration=%.6f%s,setpts=PTS-STARTPTS[v_montage_final]", montageV, audioDur, finalFadeOut,
	))

	finalV := "v_montage_final"
	finalA := ""
	audioIdx := len(inputSpecs) // voice.mp3 index
	actualTransDur := 0.0

	// Handle Audio Cuts/Trimming
	voiceASource := fmt.Sprintf("[%d:a]", audioIdx)
	if len(audioSegments) > 0 {
		var segLabels []string
		for i, seg := range audioSegments {
			label := fmt.Sprintf("aseg%d", i)
			filterParts = append(filterParts, fmt.Sprintf(
				"[%d:a]atrim=start=%.3f:end=%.3f,asetpts=PTS-STARTPTS[%s]",
				audioIdx, seg.Start, seg.End, label,
			))
			segLabels = append(segLabels, "["+label+"]")
		}
		filterParts = append(filterParts, fmt.Sprintf(
			"%sconcat=n=%d:v=0:a=1[a_cut]",
			strings.Join(segLabels, ""), len(audioSegments),
		))
		voiceASource = "[a_cut]"
		finalA = "a_cut"
	}

	if introIdx != -1 {
		// Prepare voice.mp3 audio to match intro audio
		filterParts = append(filterParts, fmt.Sprintf(
			"%saresample=44100,aformat=sample_fmts=fltp:sample_rates=44100:channel_layouts=stereo[a_voice_res]",
			voiceASource,
		))

		// Use transition if both parts are long enough
		if transDur > 0 && introDur > transDur && audioDur > transDur {
			actualTransDur = transDur
			introTransEffect := transEffect
			if introTransEffect == "fade_fast" {
				introTransEffect = "fade"
			}
			// Video transition (xfade)
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][v_montage_final]xfade=transition=%s:duration=%.3f:offset=%.3f[v_total]",
				finalIntroV, introTransEffect, transDur, introDur-transDur,
			))
			// Audio transition (acrossfade)
			filterParts = append(filterParts, fmt.Sprintf(
				"[a_intro][a_voice_res]acrossfade=d=%.3f[a_total]",
				transDur,
			))
			finalV = "v_total"
			finalA = "a_total"
		} else {
			// Fallback to simple concat
			filterParts = append(filterParts, fmt.Sprintf("[%s][v_montage_final]concat=n=2:v=1:a=0[v_total]", finalIntroV))
			filterParts = append(filterParts, "[a_intro][a_voice_res]concat=n=2:v=0:a=1[a_total]")
			finalV = "v_total"
			finalA = "a_total"
		}
	}

	// Write filter script
	fullGraph := strings.Join(filterParts, ";")
	s.log("INFO", fmt.Sprintf("[Montage] Filter Graph Length: %d characters, Labels: %d", len(fullGraph), len(filterParts)), id, taskLabel)
	if err := os.WriteFile(filepath.Join(finalDir, "montage_script.txt"), []byte(fullGraph), 0644); err != nil {
		return fmt.Errorf("failed to write filter script: %v", err)
	}

	// 6. Build FFmpeg command
	bitrateStr := fmt.Sprintf("%dM", pSettings.MontageBitrate)
	bufSize := fmt.Sprintf("%dM", pSettings.MontageBitrate*2)

	var cmdArgs []string
	cmdArgs = append(cmdArgs, "-y", "-hide_banner", "-loglevel", "info", "-stats")

	// threads flag
	if threadsPerProcess > 0 {
		cmdArgs = append(cmdArgs, "-threads", strconv.Itoa(threadsPerProcess))
	}

	// Inputs
	for _, spec := range inputSpecs {
		// Removed -thread_queue_size 4096 here to save 24 chars per file.
		// For 600 files, this saves ~15,000 characters, bypassing Windows 32KB command limit!
		if spec.framerate > 0 {
			cmdArgs = append(cmdArgs, "-framerate", strconv.Itoa(spec.framerate))
		}
		if spec.loop {
			cmdArgs = append(cmdArgs, "-loop", "1")
		}
		if spec.streamLoop {
			cmdArgs = append(cmdArgs, "-stream_loop", "-1")
		}
		cmdArgs = append(cmdArgs, "-i", spec.path)
	}
	cmdArgs = append(cmdArgs, "-i", "voice.mp3")

	// Map
	cmdArgs = append(cmdArgs,
		"-filter_complex_script", "montage_script.txt",
		"-map", "["+finalV+"]",
	)
	if finalA != "" {
		cmdArgs = append(cmdArgs, "-map", "["+finalA+"]")
	} else {
		cmdArgs = append(cmdArgs, "-map", fmt.Sprintf("%d:a", audioIdx))
	}
	cmdArgs = append(cmdArgs, "-c:v", videoCodec)

	// Codec-specific quality args
	switch videoCodec {
	case "libx264":
		cmdArgs = append(cmdArgs, "-preset", pSettings.MontageEncodingPreset, "-b:v", bitrateStr, "-maxrate", bitrateStr, "-bufsize", bufSize)
	case "h264_nvenc":
		cmdArgs = append(cmdArgs, "-preset", "p4", "-rc", "vbr", "-b:v", bitrateStr, "-maxrate", bitrateStr, "-bufsize", bufSize)
	case "h264_amf":
		cmdArgs = append(cmdArgs, "-quality", "balanced", "-b:v", bitrateStr, "-maxrate", bitrateStr, "-bufsize", bufSize)
	case "h264_videotoolbox":
		cmdArgs = append(cmdArgs, "-b:v", bitrateStr)
	}

	cmdArgs = append(cmdArgs,
		"-pix_fmt", "yuv420p",
		"-r", strconv.Itoa(fps),
	)

	// DaVinci Resolve metadata simulation
	if pSettings.MontageMetadataSimulation == "DaVinci Resolve Studio" {
		currentTime := time.Now().UTC().Format("2006-01-02T15:04:05.000") + "Z"
		cmdArgs = append(cmdArgs,
			"-metadata", "creation_time="+currentTime,
			"-metadata:s:v:0", "creation_time="+currentTime,
			"-metadata:s:v:0", "handler_name=VideoHandler",
			"-metadata:s:v:0", "timecode=01:00:00:00",
			"-metadata:s:v:0", "encoder=H.264",
			"-metadata:s:v:0", "vendor_id=[0][0][0][0]",
			"-metadata:s:v:0", "language=und",
			"-metadata:s:a:0", "creation_time="+currentTime,
			"-metadata:s:a:0", "handler_name=SoundHandler",
			"-metadata:s:a:0", "vendor_id=[0][0][0][0]",
			"-metadata:s:a:0", "language=und",
			"-write_tmcd", "1",
			"-metadata:s:d:0", "creation_time="+currentTime,
			"-metadata:s:d:0", "handler_name=TimeCodeHandler",
			"-metadata:s:d:0", "language=eng",
			"-color_primaries", "bt709",
			"-color_trc", "bt709",
			"-colorspace", "bt709",
			"-color_range", "pc",
			"-movflags", "+write_colr+faststart",
		)
		if videoCodec == "libx264" || videoCodec == "h264_nvenc" || videoCodec == "h264_amf" {
			// Force BT.709 and Full Range at bitstream level for H.264
			cmdArgs = append(cmdArgs, "-bsf:v", "h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1:video_full_range_flag=1")
		}
	}

	cmdArgs = append(cmdArgs, outputFile)

	cmd := exec.Command(ffmpegPath, cmdArgs...)
	utils.PrepareHiddenCmd(cmd)

	cmd.Dir = finalDir

	// Apply process priority BEFORE start (Windows: CreationFlags; macOS: ignored here)
	setProcPriority(cmd, procPriority)

	stderr, _ := cmd.StderrPipe()
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start ffmpeg: %v", err)
	}

	// Apply CPU affinity AFTER start (Windows: SetProcessAffinityMask; macOS: skip)
	if cpuCores > 0 {
		setProcAffinity(cmd.Process.Pid, cpuCores)
	}
	// On macOS/Linux: apply nice value post-start
	applyNicePriority(cmd.Process.Pid, procPriority)

	// Progress
	timeRegex := regexp.MustCompile(`time=(\d+):(\d+):(\d+\.\d+)`)
	fpsRegex := regexp.MustCompile(`fps=\s*(\d+)`)
	speedRegex := regexp.MustCompile(`speed=\s*(\d+\.\d+)x`)

	scanner := bufio.NewScanner(stderr)
	scanner.Split(func(data []byte, atEOF bool) (advance int, token []byte, err error) {
		if atEOF && len(data) == 0 {
			return 0, nil, nil
		}
		for i, b := range data {
			if b == '\r' || b == '\n' {
				return i + 1, data[0:i], nil
			}
		}
		if atEOF {
			return len(data), data, nil
		}
		return 0, nil, nil
	})

	totalDur := introDur + audioDur - actualTransDur
	var lastPercent float64 = -1
	lastLogTime := time.Now()

	go func() {
		for scanner.Scan() {
			line := scanner.Text()
			if line == "" {
				continue
			}
			timeMatch := timeRegex.FindStringSubmatch(line)
			if len(timeMatch) > 1 {
				h, _ := strconv.Atoi(timeMatch[1])
				m, _ := strconv.Atoi(timeMatch[2])
				sVal, _ := strconv.ParseFloat(timeMatch[3], 64)
				currentTime := float64(h*3600+m*60) + sVal
				percent := math.Min((currentTime/totalDur)*100, 100)
				s.emitStageStatus(id, "montage", "running", fmt.Sprintf("%.1f%%", percent))
				if s.OnTaskStatus != nil {
					s.OnTaskStatus(id, "running", 80+int(percent*0.2))
				}
				if (percent-lastPercent >= 0.5) || time.Since(lastLogTime) > 5*time.Second {
					fpsMatch := fpsRegex.FindStringSubmatch(line)
					speedMatch := speedRegex.FindStringSubmatch(line)
					msg := fmt.Sprintf("[Montage] %.1f%%", percent)
					if len(fpsMatch) > 1 && len(speedMatch) > 1 {
						msg = fmt.Sprintf("[Montage] %.1f%% | FPS: %s | Speed: %s", percent, fpsMatch[1], speedMatch[1])
					}
					s.log("INFO", msg, id, taskLabel)
					lastPercent = percent
					lastLogTime = time.Now()
				}
			}
		}
	}()

	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("ffmpeg failed: %v", err)
	}

	// Apply DaVinci metadata simulation if enabled (post-processing)
	s.applyMetadata(filepath.Join(finalDir, outputFile), pSettings, id, taskLabel)

	s.log("SUCCESS", fmt.Sprintf("[Pipeline] Montage complete! Video saved: %s", outputFile), id, taskLabel)

	// Get video size in GB
	videoWeight := s.getVideoSizeGB(ffprobePath, filepath.Join(finalDir, outputFile))

	s.emitStageStatus(id, "montage", "completed", videoWeight)
	if s.OnTaskStatus != nil {
		s.OnTaskStatus(id, "completed", 100)
	}

	return nil
}

func (s *PipelineService) getDuration(ffprobePath, path string) (float64, error) {
	if ffprobePath == "" {
		return 0, fmt.Errorf("ffprobe not found")
	}
	cmd := exec.Command(ffprobePath, "-v", "error", "-show_entries", "format=duration",
		"-of", "default=noprint_wrappers=1:nokey=1", path)
	utils.PrepareHiddenCmd(cmd)

	out, err := cmd.Output()
	if err != nil {
		return 0, err
	}
	var dur float64
	_, err = fmt.Sscanf(strings.TrimSpace(string(out)), "%f", &dur)
	return dur, err
}

func (s *PipelineService) hasAudio(ffprobePath, path string) bool {
	if ffprobePath == "" {
		return false
	}
	cmd := exec.Command(ffprobePath, "-v", "error", "-select_streams", "a", "-show_entries", "stream=codec_type",
		"-of", "csv=p=0", path)
	utils.PrepareHiddenCmd(cmd)

	out, _ := cmd.Output()
	return strings.TrimSpace(string(out)) == "audio"
}

// getVideoSizeGB returns the size of the video file in GB using ffprobe.
func (s *PipelineService) getVideoSizeGB(ffprobePath, path string) string {
	sizeBytes := int64(0)

	// Try ffprobe first
	if ffprobePath != "" {
		cmd := exec.Command(ffprobePath, "-v", "error", "-show_entries", "format=size",
			"-of", "default=noprint_wrappers=1:nokey=1", path)
		utils.PrepareHiddenCmd(cmd)

		out, err := cmd.Output()
		if err == nil {
			sizeStr := strings.TrimSpace(string(out))
			if val, err := strconv.ParseInt(sizeStr, 10, 64); err == nil {
				sizeBytes = val
			}
		}
	}

	// Fallback to os.Stat if ffprobe failed or returned 0
	if sizeBytes <= 0 {
		info, err := os.Stat(path)
		if err != nil {
			return ""
		}
		sizeBytes = info.Size()
	}

	if sizeBytes <= 0 {
		return ""
	}

	gb := float64(sizeBytes) / (1024 * 1024 * 1024)
	return fmt.Sprintf("%.2f GB", gb)
}

func (s *PipelineService) findTextTiming(subPath string, phrase string, taskLabel string) *float64 {
	data, err := os.ReadFile(subPath)
	if err != nil {
		return nil
	}

	isAss := strings.HasSuffix(strings.ToLower(subPath), ".ass")

	normalize := func(t string) string {
		t = strings.ToLower(t)
		t = strings.ReplaceAll(t, "ё", "е")
		reg := regexp.MustCompile(`[^\p{L}\p{N}]+`)
		t = reg.ReplaceAllString(t, " ")
		return strings.Join(strings.Fields(t), " ")
	}

	phraseNormalized := normalize(phrase)
	targetWords := strings.Fields(phraseNormalized)
	if len(targetWords) == 0 {
		return nil
	}

	lines := strings.Split(string(data), "\n")

	type subWord struct {
		text  string
		start float64
		end   float64
	}
	var subWords []subWord

	tagRe := regexp.MustCompile(`\{.*?\}`)
	
	if isAss {
		reAss := regexp.MustCompile(`Dialogue: \d+,(\d+:\d+:\d+\.\d+),(\d+:\d+:\d+\.\d+),.*,,(.*)`)
		for _, line := range lines {
			matches := reAss.FindStringSubmatch(line)
			if len(matches) > 3 {
				startTimeStr := matches[1]
				endTimeStr := matches[2]
				text := matches[3]
				text = tagRe.ReplaceAllString(text, "")
				text = strings.ReplaceAll(text, "\\N", " ")
				text = strings.ReplaceAll(text, "\\n", " ")
				text = strings.ReplaceAll(text, "\\h", " ")

				startT := s.assTimeToSeconds(startTimeStr)
				endT := s.assTimeToSeconds(endTimeStr)

				cleaned := normalize(text)
				words := strings.Fields(cleaned)
				if len(words) > 0 {
					wordDur := (endT - startT) / float64(len(words))
					for i, w := range words {
						subWords = append(subWords, subWord{
							text:  w,
							start: startT + float64(i)*wordDur,
							end:   startT + float64(i+1)*wordDur,
						})
					}
				}
			}
		}
	} else {
		// SRT Parsing
		reSrtTime := regexp.MustCompile(`(\d{2}:\d{2}:\d{2}[,\.]\d{3}) --> (\d{2}:\d{2}:\d{2}[,\.]\d{3})`)
		var currentStart, currentEnd float64
		for _, line := range lines {
			line = strings.TrimSpace(line)
			if line == "" {
				continue
			}
			timeMatches := reSrtTime.FindStringSubmatch(line)
			if len(timeMatches) > 2 {
				currentStart = s.srtTimeToSeconds(timeMatches[1])
				currentEnd = s.srtTimeToSeconds(timeMatches[2])
			} else if (currentStart != 0 || currentEnd != 0) && !regexp.MustCompile(`^\d+$`).MatchString(line) {
				// It's text (and not just the index number)
				cleaned := normalize(line)
				words := strings.Fields(cleaned)
				if len(words) > 0 {
					wordDur := (currentEnd - currentStart) / float64(len(words))
					for i, w := range words {
						subWords = append(subWords, subWord{
							text:  w,
							start: currentStart + float64(i)*wordDur,
							end:   currentStart + float64(i+1)*wordDur,
						})
					}
				}
			}
		}
	}

	if len(subWords) < len(targetWords) {
		return nil
	}

	threshold := 0.60
	if len(targetWords) <= 2 {
		threshold = 1.0
	}

	for i := 0; i <= len(subWords)-len(targetWords); i++ {
		matchCount := 0
		currentSubIdx := i
		firstMatchIdx := -1

		for _, tw := range targetWords {
			lookahead := 6
			limit := currentSubIdx + lookahead
			if limit > len(subWords) {
				limit = len(subWords)
			}

			for j := currentSubIdx; j < limit; j++ {
				if utils.IsWordSimilar(subWords[j].text, tw, 0.4) {
					matchCount++
					currentSubIdx = j + 1
					if firstMatchIdx == -1 {
						firstMatchIdx = j
					}
					break
				}
			}
		}

		similarity := float64(matchCount) / float64(len(targetWords))
		if similarity >= threshold && firstMatchIdx != -1 {
			s.log("INFO", fmt.Sprintf("[Montage] Trigger match found: '%s' (similarity: %.0f%%) at %.3fs", phrase, similarity*100, subWords[firstMatchIdx].start), "", taskLabel)
			return &subWords[firstMatchIdx].start
		}
	}

	s.log("WARN", fmt.Sprintf("[Montage] Trigger phrase not found after full scan: '%s'", phrase), "", taskLabel)
	return nil
}

func (s *PipelineService) assTimeToSeconds(t string) float64 {
	parts := strings.Split(t, ":")
	if len(parts) != 3 {
		return 0
	}
	h, _ := strconv.ParseFloat(parts[0], 64)
	m, _ := strconv.ParseFloat(parts[1], 64)
	sec, _ := strconv.ParseFloat(parts[2], 64)
	return h*3600 + m*60 + sec
}

func (s *PipelineService) srtTimeToSeconds(t string) float64 {
	t = strings.ReplaceAll(t, ",", ".")
	parts := strings.Split(t, ":")
	if len(parts) != 3 {
		return 0
	}
	h, _ := strconv.ParseFloat(parts[0], 64)
	m, _ := strconv.ParseFloat(parts[1], 64)
	sec, _ := strconv.ParseFloat(parts[2], 64)
	return h*3600 + m*60 + sec
}

// applyMetadata applies specialized metadata simulation (e.g. for DaVinci Resolve) using exiftool.
func (s *PipelineService) applyMetadata(filePath string, pSettings *utils.PipelineSettings, id string, taskLabel string) {
	if pSettings.MontageMetadataSimulation != "DaVinci Resolve Studio" {
		return
	}

	s.log("INFO", "[Metadata] Applying DaVinci Resolve metadata simulation...", id, taskLabel)

	exifPath := s.getExifToolPath()
	if exifPath == "" {
		s.log("WARN", "[Metadata] exiftool not found, skipping metadata simulation", id, taskLabel)
		return
	}

	// Ensure we have an absolute path to the file
	absFilePath, err := filepath.Abs(filePath)
	if err != nil {
		absFilePath = filePath
	}

	// Double check file accessibility
	if _, err := os.Stat(absFilePath); err != nil {
		s.log("WARN", fmt.Sprintf("[Metadata] Target file not found: %s", absFilePath), id, taskLabel)
		return
	}

	// Create a temporary argument file to pass parameters and path literally.
	// This is the ONLY 100% reliable way to pass "dirty" Windows paths to ExifTool/Perl.
	tmpArgs, err := os.CreateTemp("", "exif_args_*.txt")
	if err != nil {
		s.log("WARN", fmt.Sprintf("[Metadata] Failed to create temp arg file: %v", err), id, taskLabel)
		return
	}
	argFileName := tmpArgs.Name()
	defer os.Remove(argFileName)

	// Build arguments list
	argsList := []string{
		"-overwrite_original",
		"-api", "NoWildcards=1",
		// We remove MajorBrand/CompatibleBrands as they were causing redundant Apple tags
		"-MinorVersion=512",
		"-Encoder=Blackmagic Design DaVinci Resolve Studio",
		"-Apple:Encoder=Blackmagic Design DaVinci Resolve Studio",
		"-ColorPrimaries=BT.709",
		"-TransferCharacteristics=BT.709",
		"-MatrixCoefficients=BT.709",
		"-VideoFullRange=Full",
		// Explicitly set handler names for all streams to avoid defaults/mismatches
		"-HandlerName:Video=VideoHandler",
		"-HandlerName:Audio=SoundHandler",
		"-HandlerName:Data=TimeCodeHandler",
		absFilePath, // This is now read literally from the file
	}

	// Write one argument per line
	for _, arg := range argsList {
		if _, err := tmpArgs.WriteString(arg + "\n"); err != nil {
			tmpArgs.Close()
			return
		}
	}
	tmpArgs.Close()

	s.log("INFO", fmt.Sprintf("[Metadata] Using arg-file for: %s", filepath.Base(absFilePath)), id, taskLabel)

	// Execute exiftool with the arg-file
	cmd := exec.Command(exifPath, "-@", argFileName)
	utils.PrepareHiddenCmd(cmd)

	if output, err := cmd.CombinedOutput(); err != nil {
		s.log("WARN", fmt.Sprintf("[Metadata] exiftool failed: %v\nOutput: %s", err, string(output)), id, taskLabel)
	} else {
		s.log("SUCCESS", "[Metadata] DaVinci Resolve metadata simulation applied!", id, taskLabel)
	}
}

// getExifToolPath attempts to locate the exiftool executable.
func (s *PipelineService) getExifToolPath() string {
	// 1. Спробуємо стандартний розгорнутий шлях (автоматичне розпакування)
	if p, err := utils.EnsureExifTool(); err == nil && p != "" {
		return p
	}

	// 2. Check system PATH (якщо вбудований не знайшовся або не розпакувався)
	if p, err := exec.LookPath("exiftool"); err == nil {
		return p
	}

	return ""
}
