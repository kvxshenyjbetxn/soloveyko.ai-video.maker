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
	"strconv"
	"strings"
	"time"
)

// resolveCodec probes the preferred GPU codec, falls back to libx264.
func resolveCodec(ffmpegPath string, preferred string) string {
	codecMap := map[string]string{
		"nvidia": "h264_nvenc",
		"amd":    "h264_amf",
		"apple":  "h264_videotoolbox",
	}
	codec, ok := codecMap[preferred]
	if !ok || preferred == "cpu" || preferred == "" {
		return "libx264"
	}
	cmd := exec.Command(ffmpegPath,
		"-y", "-hide_banner", "-loglevel", "error",
		"-f", "lavfi", "-i", "color=black:s=16x16:d=0.1",
		"-c:v", codec, "-f", "null", "-",
	)
	if err := cmd.Run(); err != nil {
		return "libx264"
	}
	return codec
}

// ProcessMontage handles the final video rendering stage (single-pass FFmpeg).
func (s *PipelineService) ProcessMontage(id string, taskLabel string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	if !pSettings.MontageEnabled {
		return nil
	}

	s.emitStageStatus(id, "montage", "waiting")
	s.log("INFO", "[Pipeline] Waiting for montage slot...", id, taskLabel)

	sem := s.getMontageSem()
	sem <- struct{}{}
	defer func() { <-sem }()

	s.log("INFO", "[Pipeline] Montage slot acquired, starting...", id, taskLabel)
	s.emitStageStatus(id, "montage", "running")

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

	if len(visualFiles) == 0 {
		return fmt.Errorf("no visual files found in %s", imagesDir)
	}

	// 3. Get Audio Duration
	audioDur, err := s.getDuration(ffprobePath, filepath.Join(finalDir, "voice.mp3"))
	if err != nil {
		return fmt.Errorf("failed to get audio duration: %v", err)
	}
	if audioDur <= 0 {
		return fmt.Errorf("audio duration is zero")
	}

	// 4. Settings
	numFiles := len(visualFiles)
	transDur := pSettings.MontageTransitionDuration
	if numFiles <= 1 {
		transDur = 0
	}

	effectiveDurs := make([]float64, numFiles)
	if pSettings.ImageSyncEnabled {
		s.log("INFO", "[Montage] Synchronous Mode enabled, calculating timings...", id, taskLabel)
		timings, err := utils.GetImageTimings(finalDir, audioDur, numFiles, visualFiles, taskLabel)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Montage] Sync failed: %v. Falling back to equal distribution.", err), id, taskLabel)
			clipDur := (audioDur + float64(numFiles-1)*transDur) / float64(numFiles)
			for i := range effectiveDurs {
				effectiveDurs[i] = clipDur
			}
		} else {
			for i, t := range timings {
				if i < numFiles {
					// We add transDur to each effective duration because xfade consumes it
					effectiveDurs[i] = t.Duration + transDur
				}
			}
			// Special handling for first and last to avoid over-calculating total duration
			if len(effectiveDurs) > 0 {
				effectiveDurs[0] -= transDur / 2
				effectiveDurs[len(effectiveDurs)-1] -= transDur / 2
			}
		}
	} else {
		totalTransLoss := float64(numFiles-1) * transDur
		clipDur := (audioDur + totalTransLoss) / float64(numFiles)
		for i := range effectiveDurs {
			effectiveDurs[i] = clipDur
		}
	}

	baseW, baseH := 1920, 1080
	switch pSettings.MontageResolution {
	case "720p":
		baseW, baseH = 1280, 720
	case "2k":
		baseW, baseH = 2560, 1440
	}

	fps := pSettings.MontageFPS
	if fps <= 0 {
		fps = 30
	}
	upFactor := pSettings.MontageUpscaleFactor
	if upFactor < 1.0 {
		upFactor = 1.0
	}

	// Overrides from settings map (e.g. from templates)
	if val, ok := settings["montageIntroVideoEnabled"].(bool); ok {
		pSettings.MontageIntroVideoEnabled = val
	}
	if val, ok := settings["montageIntroVideoPath"].(string); ok {
		pSettings.MontageIntroVideoPath = val
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
	if val, ok := settings["montageWatermarkSize"].(float64); ok {
		pSettings.MontageWatermarkSize = int(val)
	} else if val, ok := settings["montageWatermarkSize"].(int); ok {
		pSettings.MontageWatermarkSize = val
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
	// We don't usually override MontageOverlayTriggers from template as it's a slice/complex object,
	// but we could if needed.

	upW := int(math.Round(float64(baseW) * upFactor))
	upH := int(math.Round(float64(baseH) * upFactor))

	swayFactor := pSettings.MontageSwayFactor
	zoomFactor := pSettings.MontageZoomFactor
	transEffect := pSettings.MontageTransitionEffect
	threadsPerProcess := pSettings.MontageThreadsPerProcess
	videoCodec := resolveCodec(ffmpegPath, pSettings.MontageVideoCodec)
	procPriority := pSettings.MontageProcessPriority
	cpuCores := pSettings.MontageCPUCores

	s.log("INFO", fmt.Sprintf("[Montage] Codec: %s | Priority: %s | CPUCores: %d | Clips: %d",
		videoCodec, procPriority, cpuCores, numFiles), id, taskLabel)

	// 5. Build filter graph — single-pass
	type inputSpec struct {
		loop       bool
		path       string
		streamLoop bool
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
			inputSpecs = append(inputSpecs, inputSpec{loop: false, path: pSettings.MontageIntroVideoPath})

			// Process intro video to match output format (Premium Blurred Background Fit)
			vFilter := fmt.Sprintf(
				"[0:v]scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,boxblur=20:10[bg]; "+
					"[0:v]scale=%d:%d:force_original_aspect_ratio=decrease[fg]; "+
					"[bg][fg]overlay=(W-w)/2:(H-h)/2,format=yuv420p,setsar=1,fps=%d[v_intro_base]",
				baseW, baseH, baseW, baseH, baseW, baseH, fps)

			aFilter := ""
			if hasA {
				aFilter = "[0:a]aresample=44100,aformat=sample_fmts=fltp:sample_rates=44100:channel_layouts=stereo[a_intro]"
			} else {
				// No audio in intro? Generate silence
				aFilter = fmt.Sprintf("anullsrc=r=44100:cl=stereo:d=%.6f,aformat=sample_fmts=fltp[a_intro]", introDur)
			}
			filterParts = append(filterParts, vFilter, aFilter)
		}
	}

	visualOffset := 0
	if introIdx != -1 {
		visualOffset = 1
	}

	for idx, vFile := range visualFiles {
		ext := strings.ToLower(filepath.Ext(vFile))
		isVideo := videoExts[ext]
		inputSpecs = append(inputSpecs, inputSpec{loop: !isVideo, path: vFile})

		vIn := fmt.Sprintf("[%d:v]", idx+visualOffset)
		vOut := fmt.Sprintf("v%d_final", idx)

		if isVideo {
			actualDur, _ := s.getDuration(ffprobePath, filepath.Join(finalDir, vFile))
			requiredDur := effectiveDurs[idx]
			if actualDur > 0 && actualDur < requiredDur {
				// Apply Boomerang Effect with infinite looping
				s.log("INFO", fmt.Sprintf("[Montage] [%d] Applying boomerang loop (actual: %.2fs, req: %.2fs)", idx, actualDur, requiredDur), id, taskLabel)
				loopFrames := int(actualDur * 2 * float64(fps))
				filterParts = append(filterParts, fmt.Sprintf(
					"[%d:v]trim=duration=%.6f,setpts=PTS-STARTPTS[f%d_1];"+
						"[f%d_1]split=2[pts%d_a][pts%d_b];"+
						"[pts%d_b]reverse,setpts=PTS-STARTPTS[b%d_wd];"+
						"[pts%d_a][b%d_wd]concat=n=2:v=1[v%d_boom];"+
						"[v%d_boom]loop=loop=-1:size=%d:start=0,scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,format=yuv420p,setsar=1,fps=%d,trim=duration=%.6f,setpts=PTS-STARTPTS[%s]",
					idx+visualOffset, actualDur, idx, idx, idx, idx, idx, idx, idx, idx, idx, idx, loopFrames, baseW, baseH, baseW, baseH, fps, requiredDur, vOut,
				))
			} else {
				if actualDur <= 0 {
					actualDur = requiredDur
				}
				effDur := math.Min(actualDur, requiredDur)
				filterParts = append(filterParts, fmt.Sprintf(
					"%strim=duration=%.6f,scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,format=yuv420p,setsar=1,fps=%d,setpts=PTS-STARTPTS[%s]",
					vIn, effDur, baseW, baseH, baseW, baseH, fps, vOut,
				))
			}
		} else { // Image
			requiredDur := effectiveDurs[idx]
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

			dFrames := int(requiredDur*float64(fps)) + 5
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s]zoompan=z='%s':x='%s':y='%s':d=%d:s=%dx%d:fps=%d,setpts=PTS-STARTPTS[%s]",
				vUp, zExpr, xExpr, yExpr, dFrames, baseW, baseH, fps, vOut,
			))
		}
	}

	// Transitions (xfade) — per-clip effectiveDurs for correct offsets
	lastV := "v0_final"
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

	// Subtitles
	montageV := lastV
	assPath := filepath.Join(finalDir, "subtitle.ass")
	if _, err := os.Stat(assPath); err == nil {
		filterParts = append(filterParts, fmt.Sprintf("[%s]subtitles='subtitle.ass'[v_sub]", montageV))
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
			inputSpecs = append(inputSpecs, inputSpec{loop: true, path: pSettings.MontageWatermarkPath})

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
				path:       pSettings.MontageOverlayPath,
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
		idx       int
		x         int
		y         int
	}
	var activeTriggers []triggerInfo
	if pSettings.MontageOverlayTriggersEnabled && len(pSettings.MontageOverlayTriggers) > 0 {
		for _, tr := range pSettings.MontageOverlayTriggers {
			if tr.Phrase == "" || tr.Path == "" {
				continue
			}
			if _, err := os.Stat(tr.Path); err != nil {
				s.log("WARN", fmt.Sprintf("[Montage] Trigger path not found: %s", tr.Path), id, taskLabel)
				continue
			}

			startT := s.findTextTiming(assPath, tr.Phrase)
			if startT != nil {
				s.log("INFO", fmt.Sprintf("[Montage] Found trigger '%s' at %.2fs", tr.Phrase, *startT), id, taskLabel)
				tIdx := len(inputSpecs)
				ext := strings.ToLower(filepath.Ext(tr.Path))
				isTrVideo := videoExts[ext]
				inputSpecs = append(inputSpecs, inputSpec{
					loop:       !isTrVideo,
					path:       tr.Path,
					streamLoop: false, // Triggers play once? Or loop for duration?
					// In python it's overlay=...:enable='between(t,start,end)'.
					// We'll play once or loop for a fixed duration if image.
				})

				activeTriggers = append(activeTriggers, triggerInfo{
					phrase:    tr.Phrase,
					path:      tr.Path,
					startTime: *startT,
					idx:       tIdx,
					x:         tr.X,
					y:         tr.Y,
				})
			} else {
				s.log("INFO", fmt.Sprintf("[Montage] Trigger phrase '%s' not found in subtitles", tr.Phrase), id, taskLabel)
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
			filterParts = append(filterParts, "[v_intro_base]copy[v_intro_final_processed]")
			finalIntroV = "v_intro_final_processed"
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
		trDur := 3.0 // Default 3s for images
		ext := strings.ToLower(filepath.Ext(tr.path))
		isTrVideo := videoExts[ext]
		if isTrVideo {
			d, _ := s.getDuration(ffprobePath, tr.path)
			if d > 0 {
				trDur = d
			}
		}

		// 1. Pre-process trigger: scale, crop, format, and setpts (delay)
		trigProcessedLabel := fmt.Sprintf("v_trig_ready_%d", i)
		filterParts = append(filterParts, fmt.Sprintf(
			"[%d:v]format=yuva420p,scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,setpts=PTS-STARTPTS+%.3f/TB[%s]",
			tr.idx, baseW, baseH, baseW, baseH, tr.startTime, trigProcessedLabel,
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

	montageV = currentMontageV

	// Montage trim
	filterParts = append(filterParts, fmt.Sprintf(
		"[%s]trim=duration=%.6f,setpts=PTS-STARTPTS[v_montage_final]", montageV, audioDur,
	))

	finalV := "v_montage_final"
	finalA := ""
	audioIdx := len(inputSpecs) // voice.mp3 index
	actualTransDur := 0.0

	if introIdx != -1 {
		// Prepare voice.mp3 audio to match intro audio
		filterParts = append(filterParts, fmt.Sprintf(
			"[%d:a]aresample=44100,aformat=sample_fmts=fltp:sample_rates=44100:channel_layouts=stereo[a_voice_res]",
			audioIdx,
		))

		// Use transition if both parts are long enough
		if transDur > 0 && introDur > transDur && audioDur > transDur {
			actualTransDur = transDur
			// Video transition (xfade)
			filterParts = append(filterParts, fmt.Sprintf(
				"[%s][v_montage_final]xfade=transition=%s:duration=%.3f:offset=%.3f[v_total]",
				finalIntroV, transEffect, transDur, introDur-transDur,
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
			filterParts = append(filterParts, fmt.Sprintf("[%s][v_montage_final]concat=n=2:v=1:a=0[v_total]; [a_intro][a_voice_res]concat=n=2:v=0:a=1[a_total]", finalIntroV))
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
		cmdArgs = append(cmdArgs, "-thread_queue_size", "4096")
		if spec.loop {
			cmdArgs = append(cmdArgs, "-loop", "1")
		}
		if spec.streamLoop {
			cmdArgs = append(cmdArgs, "-stream_loop", "-1")
		}
		cmdArgs = append(cmdArgs, "-i", spec.path)
	}
	cmdArgs = append(cmdArgs, "-thread_queue_size", "4096", "-i", "voice.mp3")

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
		"output.mp4",
	)

	cmd := exec.Command(ffmpegPath, cmdArgs...)
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

	s.log("SUCCESS", "[Pipeline] Montage complete! Video saved: output.mp4", id, taskLabel)

	// Get video size in GB
	videoWeight := s.getVideoSizeGB(ffprobePath, filepath.Join(finalDir, "output.mp4"))

	s.emitStageStatus(id, "montage", "completed", videoWeight)
	if s.OnTaskStatus != nil {
		s.OnTaskStatus(id, "completed", 100)
	}

	// Clean up temporary files as requested by user
	tempFiles := []string{"subtitle.srt", "segments.json", "montage_script.txt"}
	for _, f := range tempFiles {
		_ = os.Remove(filepath.Join(finalDir, f))
	}

	return nil
}

func (s *PipelineService) getDuration(ffprobePath, path string) (float64, error) {
	if ffprobePath == "" {
		return 0, fmt.Errorf("ffprobe not found")
	}
	cmd := exec.Command(ffprobePath, "-v", "error", "-show_entries", "format=duration",
		"-of", "default=noprint_wrappers=1:nokey=1", path)
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

func (s *PipelineService) findTextTiming(assPath string, phrase string) *float64 {
	data, err := os.ReadFile(assPath)
	if err != nil {
		return nil
	}

	normalize := func(t string) string {
		t = strings.ToLower(t)
		t = strings.ReplaceAll(t, "ё", "е")
		// Replace all non-word characters (punctuation, special chars) with a space
		reg := regexp.MustCompile(`[^\p{L}\p{N}]+`)
		t = reg.ReplaceAllString(t, " ")
		// Normalize whitespace
		return strings.Join(strings.Fields(t), " ")
	}

	phrase = normalize(phrase)
	if phrase == "" {
		return nil
	}

	lines := strings.Split(string(data), "\n")

	type segment struct {
		start float64
		text  string
	}
	var segments []segment
	var fullBuffer strings.Builder
	var charToSeg []int

	// Dialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Text
	re := regexp.MustCompile(`Dialogue: \d+,(\d+:\d+:\d+\.\d+),(\d+:\d+:\d+\.\d+),.*,,(.*)`)
	tagRe := regexp.MustCompile(`\{.*?\}`)

	for _, line := range lines {
		matches := re.FindStringSubmatch(line)
		if len(matches) > 3 {
			startTimeStr := matches[1]
			text := matches[3]
			text = tagRe.ReplaceAllString(text, "")

			// Parse time 0:00:01.00
			parts := strings.Split(startTimeStr, ":")
			if len(parts) == 3 {
				h, _ := strconv.ParseFloat(parts[0], 64)
				m, _ := strconv.ParseFloat(parts[1], 64)
				sec, _ := strconv.ParseFloat(parts[2], 64)
				startT := h*3600 + m*60 + sec

				cleaned := normalize(text)
				if cleaned == "" {
					continue
				}

				segIdx := len(segments)
				segments = append(segments, segment{start: startT, text: cleaned})

				// Map each byte of this cleaned segment to its start time
				for i := 0; i < len(cleaned); i++ {
					fullBuffer.WriteByte(cleaned[i])
					charToSeg = append(charToSeg, segIdx)
				}
				// Add space between segments to avoid merging words
				fullBuffer.WriteByte(' ')
				charToSeg = append(charToSeg, segIdx)
			}
		}
	}

	fullText := fullBuffer.String()
	idx := strings.Index(fullText, phrase)
	if idx != -1 && idx < len(charToSeg) {
		// Found it! Return the start time of the segment where the phrase starts
		segIdx := charToSeg[idx]
		return &segments[segIdx].start
	}

	return nil
}
