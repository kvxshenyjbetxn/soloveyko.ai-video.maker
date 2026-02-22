package pipeline

import (
	"bufio"
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
	entries, err := os.ReadDir(imagesDir)
	if err != nil {
		return fmt.Errorf("failed to read images directory: %v", err)
	}

	videoExts := map[string]bool{".mp4": true, ".mkv": true, ".mov": true, ".avi": true, ".webm": true}
	imageExts := map[string]bool{".jpg": true, ".jpeg": true, ".png": true, ".webp": true}

	var visualFiles []string
	for _, entry := range entries {
		ext := strings.ToLower(filepath.Ext(entry.Name()))
		if videoExts[ext] || imageExts[ext] {
			visualFiles = append(visualFiles, filepath.Join("images", entry.Name()))
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
	totalTransLoss := float64(numFiles-1) * transDur
	clipDur := (audioDur + totalTransLoss) / float64(numFiles)

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
	upW := int(math.Round(float64(baseW) * upFactor))
	upH := int(math.Round(float64(baseH) * upFactor))

	swayFactor := pSettings.MontageSwayFactor
	zoomFactor := pSettings.MontageZoomFactor
	transEffect := pSettings.MontageTransitionEffect
	threadsPerProcess := pSettings.MontageThreadsPerProcess
	videoCodec := resolveCodec(ffmpegPath, pSettings.MontageVideoCodec)
	procPriority := pSettings.MontageProcessPriority
	cpuCores := pSettings.MontageCPUCores

	s.log("INFO", fmt.Sprintf("[Montage] Codec: %s | Priority: %s | CPUCores: %d | Clips: %d | clipDur: %.2fs",
		videoCodec, procPriority, cpuCores, numFiles, clipDur), id, taskLabel)

	// 5. Build filter graph — single-pass
	type inputSpec struct {
		loop bool
		path string
	}
	var inputSpecs []inputSpec
	var filterParts []string
	effectiveDurs := make([]float64, numFiles)

	for i, relPath := range visualFiles {
		ext := strings.ToLower(filepath.Ext(relPath))
		isVideo := videoExts[ext]
		inputSpecs = append(inputSpecs, inputSpec{loop: !isVideo, path: relPath})

		vIn := fmt.Sprintf("[%d:v]", i)
		vOut := fmt.Sprintf("v%d_final", i)

		if isVideo {
			actualDur, _ := s.getDuration(ffprobePath, filepath.Join(finalDir, relPath))
			if actualDur <= 0 {
				actualDur = clipDur
			}
			effDur := math.Min(actualDur, clipDur)
			effectiveDurs[i] = effDur
			filterParts = append(filterParts, fmt.Sprintf(
				"%strim=duration=%.6f,scale=%d:%d:force_original_aspect_ratio=increase,crop=%d:%d,format=yuv420p,setsar=1,fps=%d,setpts=PTS-STARTPTS[%s]",
				vIn, effDur, baseW, baseH, baseW, baseH, fps, vOut,
			))
		} else {
			effectiveDurs[i] = clipDur
			vUp := fmt.Sprintf("v%d_up", i)

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
					baseZoomVal, zAmp, fps, clipDur)
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

			dFrames := int(clipDur*float64(fps)) + 5
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
	finalV := lastV
	assPath := filepath.Join(finalDir, "subtitle.ass")
	if _, err := os.Stat(assPath); err == nil {
		filterParts = append(filterParts, fmt.Sprintf("[%s]subtitles='subtitle.ass'[v_sub]", finalV))
		finalV = "v_sub"
	}

	// Final trim to exact audio duration
	filterParts = append(filterParts, fmt.Sprintf(
		"[%s]trim=duration=%.6f,setpts=PTS-STARTPTS[v_final]", finalV, audioDur,
	))

	// Write filter script
	fullGraph := strings.Join(filterParts, ";")
	if err := os.WriteFile(filepath.Join(finalDir, "montage_script.txt"), []byte(fullGraph), 0644); err != nil {
		return fmt.Errorf("failed to write filter script: %v", err)
	}

	// 6. Build FFmpeg command
	bitrateStr := fmt.Sprintf("%dM", pSettings.MontageBitrate)
	bufSize := fmt.Sprintf("%dM", pSettings.MontageBitrate*2)
	audioIdx := numFiles

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
		cmdArgs = append(cmdArgs, "-i", spec.path)
	}
	cmdArgs = append(cmdArgs, "-thread_queue_size", "4096", "-i", "voice.mp3")

	// Filter + map
	cmdArgs = append(cmdArgs,
		"-filter_complex_script", "montage_script.txt",
		"-map", "[v_final]",
		"-map", fmt.Sprintf("%d:a", audioIdx),
		"-c:v", videoCodec,
	)

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
				percent := math.Min((currentTime/audioDur)*100, 100)
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
