package pipeline

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"soloveyko/backend/utils"
)

// ProcessSubtitle handles subtitle generation using chosen transcriber
func (s *PipelineService) ProcessSubtitle(id string, taskLabel string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	var sEnabled bool
	var jsonRes string
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

	// Apply template overrides to pSettings so they are used in saveSubtitles -> SrtToAss
	if val, ok := settings["subtitleMaxWords"].(float64); ok {
		pSettings.SubtitleMaxWords = int(val)
	} else if val, ok := settings["subtitleMaxWords"].(int); ok {
		pSettings.SubtitleMaxWords = val
	}

	if val, ok := settings["subtitleAmdLanguage"].(string); ok && val != "" {
		pSettings.SubtitleAmdLanguage = val
	}

	// Also Font/Color/Size if needed
	if val, ok := settings["subtitleFont"].(string); ok && val != "" {
		pSettings.SubtitleFont = val
	}
	if val, ok := settings["subtitleSize"].(float64); ok && val > 0 {
		pSettings.SubtitleSize = int(val)
	} else if val, ok := settings["subtitleSize"].(int); ok && val > 0 {
		pSettings.SubtitleSize = val
	}
	if val, ok := settings["subtitleColor"].(string); ok && val != "" {
		pSettings.SubtitleColor = val
	}
	if val, ok := settings["subtitleFadeEnabled"].(bool); ok {
		pSettings.SubtitleFadeEnabled = val
	}
	if val, ok := settings["subtitleMaxLen"].(float64); ok && val > 0 {
		pSettings.SubtitleMaxLen = int(val)
	} else if val, ok := settings["subtitleMaxLen"].(int); ok && val > 0 {
		pSettings.SubtitleMaxLen = val
	} else if _, hasLen := settings["subtitleMaxLen"]; !hasLen {
		// Fallback for old templates that only have subtitleMaxWords
		if val, ok := settings["subtitleMaxWords"].(float64); ok && val > 0 {
			pSettings.SubtitleMaxLen = int(val)
		} else if val, ok := settings["subtitleMaxWords"].(int); ok && val > 0 {
			pSettings.SubtitleMaxLen = val
		}
	}

	voiceFilePath := filepath.Join(finalDir, "voice.mp3")
	if _, err := os.Stat(voiceFilePath); os.IsNotExist(err) {
		s.log("WARN", "[Pipeline] No voice.mp3 found for subtitle generation.", id, taskLabel)
		return nil
	}

	regenerate, _ := settings["subtitleRegenerate"].(bool)
	subtitleSrtPath := filepath.Join(finalDir, "subtitle.srt")
	subtitleAssPath := filepath.Join(finalDir, "subtitle.ass")
	if _, errSrt := os.Stat(subtitleSrtPath); errSrt == nil && !regenerate {
		if _, errAss := os.Stat(subtitleAssPath); errAss == nil {
			s.log("INFO", "[Pipeline] Subtitles already exist (SRT and ASS), skipping generation (Restore Mode).", id, taskLabel)
			s.emitStageStatus(id, "subtitle", "completed")
			return nil
		}
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

	// Ensure only one whisper process (local, amd) runs at a time globally
	// WhisperX is excluded to allow parallelism based on the subtitle semaphore settings
	if sService == "standard" || sService == "amd" {
		GlobalWhisperMutex.Lock()
		defer GlobalWhisperMutex.Unlock()
	}

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

		result, err = s.amdWhisper.Transcribe(voiceFilePath, sModel, amdLang, pSettings.SubtitleMaxLen)
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

		if pSettings.SubtitleKaraokeEffect {
			var srtRes string
			srtRes, jsonRes, err = s.assemblyAI.TranscribeFull(s.ctx, voiceFilePath)
			result = srtRes
		} else {
			result, err = s.assemblyAI.Transcribe(s.ctx, voiceFilePath)
		}

		if err != nil {
			s.log("ERROR", fmt.Sprintf("[AssemblyAI] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}
	case "whisperx":
		err = s.ProcessWhisperX(id, taskLabel, finalDir, voiceFilePath, settings, pSettings)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[WhisperX] Failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "subtitle", "failed")
			return err
		}
		// WhisperX directly saves the ASS and JSON files, so we don't need to call saveSubtitles
		s.emitStageStatus(id, "subtitle", "completed")
		return nil
	default:
		s.log("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for subtitle generation", sService), id, taskLabel)
		s.emitStageStatus(id, "subtitle", "completed")
		return nil
	}

	// Save results (SRT and convert to ASS)
	if sService == "assemblyai" && pSettings.SubtitleKaraokeEffect && jsonRes != "" {
		// Save JSON (pretty-printed and unescaped for readability)
		jsonPath := filepath.Join(finalDir, "subtitle.json")
		var apiData interface{}
		if err := json.Unmarshal([]byte(jsonRes), &apiData); err == nil {
			if f, createErr := os.Create(jsonPath); createErr == nil {
				enc := json.NewEncoder(f)
				enc.SetEscapeHTML(false)
				enc.SetIndent("", "  ")
				_ = enc.Encode(apiData)
				f.Close()
			} else {
				_ = os.WriteFile(jsonPath, []byte(jsonRes), 0644)
			}
		} else {
			_ = os.WriteFile(jsonPath, []byte(jsonRes), 0644)
		}

		// Convert to ASS
		assData, err := utils.JsonToAss(jsonRes, pSettings, true)
		if err != nil {
			s.log("WARN", fmt.Sprintf("[Subtitle] Failed to convert JSON to ASS: %v", err), id, taskLabel)
		} else {
			subtitleAssPath := filepath.Join(finalDir, "subtitle.ass")
			_ = os.WriteFile(subtitleAssPath, []byte(assData), 0644)
		}

		// Still save SRT
		return s.saveSubtitles(finalDir, result, id, taskLabel, pSettings)
	}

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

	// 2. Convert to ASS and save (only if not already exists - e.g. from high-quality JSON)
	if _, statErr := os.Stat(subtitleAssPath); os.IsNotExist(statErr) {
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
			s.log("SUCCESS", "[Subtitle] Success: Subtitles saved in SRT and standard ASS formats", id, taskLabel)
		}
	} else {
		s.log("INFO", "[Subtitle] Skipping standard ASS generation (high-quality ASS already exists)", id, taskLabel)
	}

	return nil
}
