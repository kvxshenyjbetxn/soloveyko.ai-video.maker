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

	s.emitStageStatus(id, "subtitle", "waiting")
	s.log("INFO", "[Pipeline] Waiting for subtitle engine slot...", id, taskLabel)

	sem := s.getSubtitleSem()
	sem <- struct{}{}
	defer func() { <-sem }()

	s.log("INFO", "[Pipeline] Subtitle engine slot acquired, starting transcription...", id, taskLabel)
	s.emitStageStatus(id, "subtitle", "running")

	var result string
	var err error

	switch sService {
	case "standard":
		result, err = s.localWhisper.TranscribeBase(voiceFilePath, sModel, pSettings.SubtitleMaxLen)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[LocalWhisper] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}
	case "amd":
		amdLang, _ := settings["subtitleAmdLanguage"].(string)
		if amdLang == "" {
			amdLang = pSettings.SubtitleAmdLanguage
		}
		if amdLang == "" {
			amdLang = "uk"
		}

		result, err = s.amdWhisper.Transcribe(voiceFilePath, sModel, amdLang)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AmdWhisper] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}
	case "assemblyai":
		if s.assemblyAI == nil {
			s.log("ERROR", "[AssemblyAI] Service is not initialized", id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return fmt.Errorf("AssemblyAI service not initialized")
		}

		result, err = s.assemblyAI.Transcribe(s.ctx, voiceFilePath)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AssemblyAI] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}
	default:
		s.log("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for subtitle generation", sService), id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
		return nil
	}

	// Save results (SRT and convert to ASS)
	err = s.saveSubtitles(finalDir, result, id, taskLabel, pSettings)
	if err != nil {
		s.emitStageStatus(id, "subtitle", "failed")
		return err
	}

	s.emitStageStatus(id, "subtitle", "completed")
	return nil
}

func (s *PipelineService) saveSubtitles(finalDir string, srtData string, id string, taskLabel string, pSettings *utils.PipelineSettings) error {
	subtitleSrtPath := filepath.Join(finalDir, "subtitle.srt")
	subtitleAssPath := filepath.Join(finalDir, "subtitle.ass")

	// 1. Save SRT
	err := os.WriteFile(subtitleSrtPath, []byte(srtData), 0644)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[Subtitle] Failed to save SRT: %v", err), id, taskLabel)
		return err
	}

	// 2. Convert to ASS and save
	assData, err := utils.SrtToAss(srtData, pSettings)
	if err != nil {
		s.log("WARN", fmt.Sprintf("[Subtitle] Failed to convert to ASS: %v", err), id, taskLabel)
		// We still have SRT, so we can continue
	} else {
		err = os.WriteFile(subtitleAssPath, []byte(assData), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Subtitle] Failed to save ASS: %v", err), id, taskLabel)
			return err
		}
		s.log("SUCCESS", "[Subtitle] Success: Subtitles saved in SRT and high-quality ASS formats", id, taskLabel)
	}

	return nil
}
