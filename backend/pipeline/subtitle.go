package pipeline

import (
	"fmt"
	"os"
	"path/filepath"
	"soloveyko/backend/utils"
)

// ProcessSubtitle handles subtitle generation using chosen transcriber
func (s *PipelineService) ProcessSubtitle(id string, taskLabel string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	var sEnabled bool
	if val, ok := settings["subtitleEnabled"].(bool); ok {
		sEnabled = val
	} else {
		sEnabled = pSettings.SubtitleEnabled
	}

	if !sEnabled {
		s.log("INFO", "[Pipeline] Subtitle stage is disabled, skipping.", id, taskLabel)
		return nil
	}

	sService, _ := settings["subtitleService"].(string)
	if sService == "" {
		sService = pSettings.SubtitleService
	}
	sModel, _ := settings["subtitleModel"].(string)
	if sModel == "" {
		sModel = pSettings.SubtitleModel
	}
	if sModel == "" {
		sModel = "base"
	}

	voiceFilePath := filepath.Join(finalDir, "voice.mp3")
	if _, err := os.Stat(voiceFilePath); os.IsNotExist(err) {
		s.log("WARN", "[Pipeline] No voice.mp3 found for subtitle generation.", id, taskLabel)
		return nil
	}

	if sService == "assemblyai" {
		s.log("INFO", fmt.Sprintf("[Pipeline] Subtitle stage started. Service: %s", sService), id, taskLabel)
	} else {
		s.log("INFO", fmt.Sprintf("[Pipeline] Subtitle stage started. Service: %s, Model: %s", sService, sModel), id, taskLabel)
	}

	s.emitStageStatus(id, "subtitle", "running")

	subtitleFilePath := filepath.Join(finalDir, "subtitle.srt")

	if sService == "standard" {
		// Mock transcription for now
		result, err := s.localWhisper.TranscribeBase(voiceFilePath, sModel, pSettings.SubtitleMaxLen)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[LocalWhisper] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		// Write result to subtitle.srt
		err = os.WriteFile(subtitleFilePath, []byte(result), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[LocalWhisper] Failed to save subtitle: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		s.log("SUCCESS", "[LocalWhisper] Success: Subtitles saved to subtitle.srt", id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
	} else if sService == "amd" {
		amdLang, _ := settings["subtitleAmdLanguage"].(string)
		if amdLang == "" {
			amdLang = pSettings.SubtitleAmdLanguage
		}
		if amdLang == "" {
			amdLang = "uk"
		}

		result, err := s.amdWhisper.Transcribe(voiceFilePath, sModel, amdLang, pSettings.SubtitleMaxLen)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AmdWhisper] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		// Write result to subtitle.srt
		err = os.WriteFile(subtitleFilePath, []byte(result), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AmdWhisper] Failed to save subtitle: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		s.log("SUCCESS", "[AmdWhisper] Success: Subtitles saved to subtitle.srt", id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
	} else if sService == "assemblyai" {
		if s.assemblyAI == nil {
			s.log("ERROR", "[AssemblyAI] Service is not initialized", id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return fmt.Errorf("AssemblyAI service not initialized")
		}

		result, err := s.assemblyAI.Transcribe(s.ctx, voiceFilePath)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AssemblyAI] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		// Write result to subtitle.srt
		err = os.WriteFile(subtitleFilePath, []byte(result), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AssemblyAI] Failed to save subtitle: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}

		s.log("SUCCESS", "[AssemblyAI] Success: Subtitles saved to subtitle.srt", id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
	} else {
		s.log("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for subtitle generation", sService), id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
	}

	return nil
}
