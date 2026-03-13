package pipeline

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"soloveyko/backend/utils"
)


// ProcessWhisperX executes the WhisperX CLI executable for transcription and preserves JSON, SRT, and ASS files.
func (s *PipelineService) ProcessWhisperX(id string, taskLabel string, finalDir string, voiceFilePath string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	s.log("INFO", "[WhisperX] Starting WhisperX transcription process...", id, taskLabel)

	// 1. Resolve paths
	configDir := s.settings.GetConfigDir()
	
	// Check multiple possible locations for the executable
	possibleExes := []string{
		filepath.Join(configDir, "bin", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx_aligner_win", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx-win", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx-mac", "whisperx_cli"),
	}


	var whisperxExe string
	for _, p := range possibleExes {
		if _, err := os.Stat(p); err == nil {
			whisperxExe = p
			break
		}
	}

	if whisperxExe == "" {
		s.log("ERROR", "[WhisperX] WhisperX executable not found. Looked in: "+strings.Join(possibleExes, ", "), id, taskLabel)
		return fmt.Errorf("whisperx executable not found. Please ensure whisperx_cli.exe is in your user/bin folder")
	}

	ffmpegName := "ffmpeg"
	if runtime.GOOS == "windows" {
		ffmpegName = "ffmpeg.exe"
	}
	ffmpegExe := filepath.Join(configDir, "bin", ffmpegName)

	// Check if karaoke effect is enabled
	karaokeEffect := pSettings.SubtitleKaraokeEffect
	if val, ok := settings["subtitleKaraokeEffect"].(bool); ok {
		karaokeEffect = val
	}

	// Output base path (WhisperX CLI adds .json and .srt)
	// We use "subtitle" so it creates subtitle.srt and subtitle.json directly
	outputBase := filepath.Join(finalDir, "subtitle")
	outputJSONPath := outputBase + ".json"
	outputSRTPath := outputBase + ".srt"

	// 2. Prepare command
	cmdArgs := []string{
		"--audio", voiceFilePath,
		"--output", outputBase,
	}

	// Model selection
	sModel, _ := settings["subtitleModel"].(string)
	if sModel == "" {
		sModel = pSettings.SubtitleModel
	}
	if sModel == "" {
		sModel = "base"
	}
	cmdArgs = append(cmdArgs, "--model", sModel)

	// Language selection
	language, _ := settings["subtitleWhisperxLanguage"].(string)
	if language == "" {
		language = pSettings.SubtitleWhisperxLanguage
	}
	
	if language != "" {
		cmdArgs = append(cmdArgs, "--language", language)
	}

	// FFmpeg path
	if ffmpegExe != "" {
		if _, err := os.Stat(ffmpegExe); err == nil {
			cmdArgs = append(cmdArgs, "--ffmpeg-path", ffmpegExe)
		}
	}

	// Device selection (auto by default)
	cmdArgs = append(cmdArgs, "--device", "auto")

	s.log("INFO", fmt.Sprintf("[WhisperX] Running command: %s %s", whisperxExe, strings.Join(cmdArgs, " ")), id, taskLabel)

	cmd := exec.CommandContext(s.ctx, whisperxExe, cmdArgs...)
	utils.PrepareHiddenCmd(cmd)
	cmd.Dir = filepath.Dir(whisperxExe)
	cmd.Env = append(os.Environ(), "PYTHONIOENCODING=utf-8", "PYTHONUTF8=1", "HF_HUB_DISABLE_SYMLINKS=1")

	// 3. Execute command
	output, err := cmd.CombinedOutput()
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Execution failed: %v", err), id, taskLabel)
		s.log("ERROR", fmt.Sprintf("[WhisperX] Output: %s", string(output)), id, taskLabel)
		return fmt.Errorf("whisperx execution failed: %v", err)
	}

	s.log("INFO", "[WhisperX] Execution completed successfully.", id, taskLabel)

	// 4. Handle output files
	
	// WhisperX might sometimes use the input filename (voice.mp3 -> voice.srt) 
	// even if --output is specified as a path. Let's be robust.
	voiceSrt := filepath.Join(finalDir, "voice.srt")
	voiceJson := filepath.Join(finalDir, "voice.json")

	if _, err := os.Stat(voiceSrt); err == nil && outputSRTPath != voiceSrt {
		_ = os.Rename(voiceSrt, outputSRTPath)
	}
	if _, err := os.Stat(voiceJson); err == nil && outputJSONPath != voiceJson {
		_ = os.Rename(voiceJson, outputJSONPath)
	}

	// Parse output JSON and generate ASS
	if _, err := os.Stat(outputJSONPath); os.IsNotExist(err) {
		s.log("ERROR", "[WhisperX] Output JSON not found at "+outputJSONPath, id, taskLabel)
		return fmt.Errorf("whisperx output JSON not found")
	}

	jsonBytes, err := os.ReadFile(outputJSONPath)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to read JSON: %v", err), id, taskLabel)
		return fmt.Errorf("failed to read whisperx json: %v", err)
	}

	// Convert to ASS
	assData, err := utils.JsonToAss(string(jsonBytes), pSettings, karaokeEffect)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to convert JSON to ASS: %v", err), id, taskLabel)
		return fmt.Errorf("failed to convert json to ass: %v", err)
	}

	subtitleAssPath := filepath.Join(finalDir, "subtitle.ass")
	err = os.WriteFile(subtitleAssPath, []byte(assData), 0644)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to save ASS file: %v", err), id, taskLabel)
		return fmt.Errorf("failed to save ass file: %v", err)
	}

	s.log("SUCCESS", "[WhisperX] Subtitles created successfully (SRT, JSON, ASS preserved).", id, taskLabel)
	return nil
}

// ProcessWhisperXAlign performs only word-alignment using an existing transcription JSON file.
func (s *PipelineService) ProcessWhisperXAlign(id string, taskLabel string, finalDir string, voiceFilePath string, transcriptionJsonPath string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	s.log("INFO", "[WhisperX] Starting WhisperX alignment process...", id, taskLabel)

	// 1. Resolve paths
	configDir := s.settings.GetConfigDir()
	possibleExes := []string{
		filepath.Join(configDir, "bin", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx_aligner_win", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx-win", "whisperx_cli.exe"),
		filepath.Join(configDir, "bin", "whisperx-mac", "whisperx_cli"),
	}

	var whisperxExe string
	for _, p := range possibleExes {
		if _, err := os.Stat(p); err == nil {
			whisperxExe = p
			break
		}
	}

	if whisperxExe == "" {
		s.log("ERROR", "[WhisperX] WhisperX executable not found.", id, taskLabel)
		return fmt.Errorf("whisperx executable not found")
	}

	ffmpegName := "ffmpeg"
	if runtime.GOOS == "windows" {
		ffmpegName = "ffmpeg.exe"
	}
	ffmpegExe := filepath.Join(configDir, "bin", ffmpegName)

	karaokeEffect := pSettings.SubtitleKaraokeEffect
	if val, ok := settings["subtitleKaraokeEffect"].(bool); ok {
		karaokeEffect = val
	}

	// Check if input JSON exists
	if _, err := os.Stat(transcriptionJsonPath); os.IsNotExist(err) {
		s.log("ERROR", "[WhisperX] Transcription JSON not found: "+transcriptionJsonPath, id, taskLabel)
		return fmt.Errorf("transcription json not found for alignment")
	}

	outputBase := filepath.Join(finalDir, "subtitle")
	outputJSONPath := outputBase + ".json"

	// 2. Prepare command for alignment
	// Reverted to hyphens and --output based on binary usage output
	cmdArgs := []string{
		"--audio", voiceFilePath,
		"--align-json", transcriptionJsonPath,
		"--output", outputBase,
	}

	// Language selection (Required for alignment if not in JSON)
	language, _ := settings["subtitleAmdLanguage"].(string)
	if language == "" {
		language = pSettings.SubtitleAmdLanguage
	}
	if language == "" {
		language = "uk"
	}
	cmdArgs = append(cmdArgs, "--language", language)

	if ffmpegExe != "" {
		if _, err := os.Stat(ffmpegExe); err == nil {
			cmdArgs = append(cmdArgs, "--ffmpeg-path", ffmpegExe)
		}
	}

	cmdArgs = append(cmdArgs, "--device", "auto")

	s.log("INFO", fmt.Sprintf("[WhisperX] Running alignment: %s %s", whisperxExe, strings.Join(cmdArgs, " ")), id, taskLabel)

	cmd := exec.CommandContext(s.ctx, whisperxExe, cmdArgs...)
	utils.PrepareHiddenCmd(cmd)
	cmd.Dir = filepath.Dir(whisperxExe)
	cmd.Env = append(os.Environ(), "PYTHONIOENCODING=utf-8", "PYTHONUTF8=1", "HF_HUB_DISABLE_SYMLINKS=1")

	// 3. Execute command
	s.log("INFO", "[WhisperX] Command execution started...", id, taskLabel)
	output, err := cmd.CombinedOutput()
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Alignment failed: %v", err), id, taskLabel)
		s.log("ERROR", fmt.Sprintf("[WhisperX] Command Output:\n%s", string(output)), id, taskLabel)
		return fmt.Errorf("whisperx alignment failed: %v", err)
	}

	s.log("INFO", "[WhisperX] Alignment completed successfully.", id, taskLabel)

	// 4. Handle output files
	// WhisperX usually uses the audio filename for output (voice.mp3 -> voice.json)
	voiceJson := filepath.Join(finalDir, "voice.json")
	if _, err := os.Stat(voiceJson); err == nil {
		_ = os.Rename(voiceJson, outputJSONPath)
	}

	jsonBytes, err := os.ReadFile(outputJSONPath)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to read aligned JSON at %s: %v", outputJSONPath, err), id, taskLabel)
		return fmt.Errorf("failed to read aligned json: %v", err)
	}

	// Convert aligned JSON to ASS (with Karaoke effect if enabled)
	assData, err := utils.JsonToAss(string(jsonBytes), pSettings, karaokeEffect)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to convert aligned JSON to ASS: %v", err), id, taskLabel)
		return fmt.Errorf("failed to convert aligned json to ass: %v", err)
	}

	subtitleAssPath := filepath.Join(finalDir, "subtitle.ass")
	err = os.WriteFile(subtitleAssPath, []byte(assData), 0644)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[WhisperX] Failed to save aligned ASS file: %v", err), id, taskLabel)
		return fmt.Errorf("failed to save aligned ass file: %v", err)
	}

	s.log("SUCCESS", "[WhisperX] Aligned subtitles created successfully.", id, taskLabel)
	return nil
}
