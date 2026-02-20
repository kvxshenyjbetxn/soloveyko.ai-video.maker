package pipeline

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"soloveyko/backend/api"
	"soloveyko/backend/utils"
	"sync"
	"time"
)

// ProcessVoiceover handles voice synthesis using ElevenLabs
func (s *PipelineService) ProcessVoiceover(id string, taskLabel string, processedText string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	var vEnabled bool
	if val, ok := settings["voiceoverEnabled"].(bool); ok {
		vEnabled = val
	} else {
		vEnabled = pSettings.VoiceoverEnabled
	}

	if !vEnabled {
		s.log("INFO", "[Pipeline] Voiceover stage is disabled, skipping.", id, taskLabel)
		return nil
	}

	vService, _ := settings["voiceoverService"].(string)
	if vService == "" {
		vService = pSettings.VoiceoverService
	}
	vTemplate, _ := settings["voiceoverTemplate"].(string)
	if vTemplate == "" {
		vTemplate = pSettings.VoiceoverTemplate
	}
	vKeyID, _ := settings["voiceoverElevenLabsBotKeyID"].(string)
	if vKeyID == "" {
		vKeyID = pSettings.VoiceoverElevenLabsBotKeyID
	}

	// Conditional logging based on service type
	if vService == "elevenlabsunlim" {
		vID, _ := settings["elevenLabsUnlimVoiceID"].(string)
		if vID == "" {
			vID = pSettings.ElevenLabsUnlimVoiceID
		}
		if vID == "" {
			vID = "AB9XsbSA4eLG12t2myjN" // Default voice from docs
		}
		s.log("INFO", fmt.Sprintf("[Pipeline] Voiceover stage started. Service: %s, Voice ID: %s", vService, vID), id, taskLabel)
	} else {
		s.log("INFO", fmt.Sprintf("[Pipeline] Voiceover stage started. Service: %s, Template: %s", vService, vTemplate), id, taskLabel)
	}

	if vService == "elevenlabsbot" {
		if vTemplate == "" {
			s.log("ERROR", "[ElevenLabsBot] Voice template is not selected!", id, taskLabel)
			return fmt.Errorf("voice template not selected")
		}

		vApiKey := ""
		vKeys := s.settings.GetElevenLabsBotKeys()
		for _, k := range vKeys {
			if k.ID == vKeyID {
				vApiKey = k.Key
				break
			}
		}
		if vApiKey == "" && len(vKeys) > 0 {
			vApiKey = vKeys[0].Key
		}

		if vApiKey == "" {
			s.log("ERROR", "[ElevenLabsBot] API key not found for voiceover", id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return fmt.Errorf("API key not found")
		}

		s.emitStageStatus(id, "voice", "running")
		voiceFilePath := filepath.Join(finalDir, "voice.mp3")

		var err error
		backoffs := []int{5, 10, 15}
		maxRetries := 3

		for attempt := 0; attempt <= maxRetries; attempt++ {
			if attempt > 0 {
				s.log("WARN", fmt.Sprintf("[ElevenLabsBot] Retry attempt %d/%d after %ds...", attempt, maxRetries, backoffs[attempt-1]), id, taskLabel)
				time.Sleep(time.Duration(backoffs[attempt-1]) * time.Second)
			}

			err = s.elevenLabs.Synthesize(vApiKey, processedText, vTemplate, voiceFilePath, id, taskLabel)
			if err == nil {
				break
			}
			s.log("ERROR", fmt.Sprintf("[ElevenLabsBot] Attempt %d failed: %v", attempt+1, err), id, taskLabel)
		}

		if err != nil {
			s.log("ERROR", fmt.Sprintf("[ElevenLabsBot] All 3 retry attempts failed. Final Error: %v", err), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return err
		}

		duration, _ := utils.GetAudioDuration(voiceFilePath)
		s.log("SUCCESS", fmt.Sprintf("[ElevenLabsBot] Success: Voice saved to voice.mp3 (%s)", duration), id, taskLabel)
		s.emitStageStatus(id, "voice", "completed", duration)
	} else if vService == "elevenlabsunlim" {
		vKeyID, _ := settings["voiceoverElevenLabsUnlimKeyID"].(string)
		if vKeyID == "" {
			vKeyID = pSettings.VoiceoverElevenLabsUnlimKeyID
		}

		vApiKey := ""
		vKeys := s.settings.GetElevenLabsUnlimKeys()
		for _, k := range vKeys {
			if k.ID == vKeyID {
				vApiKey = k.Key
				break
			}
		}
		if vApiKey == "" && len(vKeys) > 0 {
			vApiKey = vKeys[0].Key
		}

		if vApiKey == "" {
			s.log("ERROR", "[ElevenLabsUnlim] API key not found", id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return fmt.Errorf("API key not found")
		}

		vID, _ := settings["elevenLabsUnlimVoiceID"].(string)
		if vID == "" {
			vID = pSettings.ElevenLabsUnlimVoiceID
		}
		if vID == "" {
			vID = "AB9XsbSA4eLG12t2myjN" // Default voice from docs
		}

		// Extract sliders
		stability, _ := settings["elevenLabsUnlimStability"].(float64)
		if stability == 0 {
			stability = pSettings.ElevenLabsUnlimStability
			if stability == 0 {
				stability = 0.5
			}
		}

		similarity, _ := settings["elevenLabsUnlimSimilarity"].(float64)
		if similarity == 0 {
			similarity = pSettings.ElevenLabsUnlimSimilarity
			if similarity == 0 {
				similarity = 0.75
			}
		}

		style, _ := settings["elevenLabsUnlimStyle"].(float64)
		if style == 0 {
			style = pSettings.ElevenLabsUnlimStyle
		}

		boost, ok := settings["elevenLabsUnlimSpeakerBoost"].(bool)
		if !ok {
			boost = pSettings.ElevenLabsUnlimSpeakerBoost
		}

		vSettings := map[string]interface{}{
			"stability":         stability,
			"similarity_boost":  similarity,
			"style":             style,
			"use_speaker_boost": boost,
		}

		s.emitStageStatus(id, "voice", "running")
		voiceFilePath := filepath.Join(finalDir, "voice.mp3")

		var err error
		backoffs := []int{5, 10, 15}
		maxRetries := 3

		for attempt := 0; attempt <= maxRetries; attempt++ {
			if attempt > 0 {
				s.log("WARN", fmt.Sprintf("[ElevenLabsUnlim] Retry attempt %d/%d after %ds...", attempt, maxRetries, backoffs[attempt-1]), id, taskLabel)
				time.Sleep(time.Duration(backoffs[attempt-1]) * time.Second)
			}

			err = s.elevenLabsUnlim.Synthesize(vApiKey, processedText, vID, vSettings, voiceFilePath, id, taskLabel)
			if err == nil {
				break
			}
			s.log("ERROR", fmt.Sprintf("[ElevenLabsUnlim] Attempt %d failed: %v", attempt+1, err), id, taskLabel)
		}

		if err != nil {
			s.log("ERROR", fmt.Sprintf("[ElevenLabsUnlim] All 3 retry attempts failed. Final Error: %v", err), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return err
		}

		duration, _ := utils.GetAudioDuration(voiceFilePath)
		s.log("SUCCESS", fmt.Sprintf("[ElevenLabsUnlim] Success: Voice saved to voice.mp3 (%s)", duration), id, taskLabel)
		s.emitStageStatus(id, "voice", "completed", duration)
	} else if vService == "elevenlabsua" {
		vKeyID, _ := settings["voiceoverElevenLabsUAKeyID"].(string)
		if vKeyID == "" {
			vKeyID = pSettings.VoiceoverElevenLabsUAKeyID
		}

		vApiKey := ""
		vKeys := s.settings.GetElevenLabsUAKeys()
		for _, k := range vKeys {
			if k.ID == vKeyID {
				vApiKey = k.Key
				break
			}
		}
		if vApiKey == "" && len(vKeys) > 0 {
			vApiKey = vKeys[0].Key
		}

		if vApiKey == "" {
			s.log("ERROR", "[ElevenLabsUA] API key not found", id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return fmt.Errorf("API key not found")
		}

		vID, _ := settings["elevenLabsUAVoiceID"].(string)
		if vID == "" {
			vID = pSettings.ElevenLabsUAVoiceID
		}
		if vID == "" {
			vID = "eBthAb30UYbt2nojGXeA" // Default voice from docs
		}

		modelID, _ := settings["elevenLabsUAModel"].(string)
		if modelID == "" {
			modelID = pSettings.ElevenLabsUAModel
		}

		// Extract sliders
		stability, _ := settings["elevenLabsUAStability"].(float64)
		if stability == 0 {
			stability = pSettings.ElevenLabsUAStability
			if stability == 0 {
				stability = 0.5
			}
		}

		similarity, _ := settings["elevenLabsUASimilarity"].(float64)
		if similarity == 0 {
			similarity = pSettings.ElevenLabsUASimilarity
			if similarity == 0 {
				similarity = 0.75
			}
		}

		style, _ := settings["elevenLabsUAStyle"].(float64)
		if style == 0 {
			style = pSettings.ElevenLabsUAStyle
		}

		boost, ok := settings["elevenLabsUASpeakerBoost"].(bool)
		if !ok {
			boost = pSettings.ElevenLabsUASpeakerBoost
		}

		vSettings := &api.ElevenLabsUAVoiceSettings{
			Stability:       stability,
			SimilarityBoost: similarity,
			Style:           style,
			UseSpeakerBoost: boost,
		}

		s.emitStageStatus(id, "voice", "running")
		voiceFilePath := filepath.Join(finalDir, "voice.mp3")

		var err error
		backoffs := []int{5, 10, 15}
		maxRetries := 3

		for attempt := 0; attempt <= maxRetries; attempt++ {
			if attempt > 0 {
				s.log("WARN", fmt.Sprintf("[ElevenLabsUA] Retry attempt %d/%d after %ds...", attempt, maxRetries, backoffs[attempt-1]), id, taskLabel)
				time.Sleep(time.Duration(backoffs[attempt-1]) * time.Second)
			}

			err = s.elevenLabsUA.Synthesize(vApiKey, processedText, vID, modelID, vSettings, voiceFilePath, id, taskLabel)
			if err == nil {
				break
			}
			s.log("ERROR", fmt.Sprintf("[ElevenLabsUA] Attempt %d failed: %v", attempt+1, err), id, taskLabel)
		}

		if err != nil {
			s.log("ERROR", fmt.Sprintf("[ElevenLabsUA] All 3 retry attempts failed. Final Error: %v", err), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return err
		}

		duration, _ := utils.GetAudioDuration(voiceFilePath)
		s.log("SUCCESS", fmt.Sprintf("[ElevenLabsUA] Success: Voice saved to voice.mp3 (%s)", duration), id, taskLabel)
		s.emitStageStatus(id, "voice", "completed", duration)
	} else if vService == "voicemaker" {
		vKeyID, _ := settings["voiceoverVoiceMakerKeyID"].(string)
		vID, _ := settings["voiceMakerVoiceID"].(string)
		vLang, _ := settings["voiceMakerLanguageCode"].(string)
		if vLang == "" {
			vLang = pSettings.VoiceMakerLanguageCode
		}
		if vLang == "" {
			vLang = "multi-lang"
		}

		charLimit, _ := settings["voiceMakerCharLimit"].(int)
		if charLimit <= 0 {
			charLimit = pSettings.VoiceMakerCharLimit
		}
		if charLimit <= 0 {
			charLimit = 3000
		}

		vApiKey := ""
		vKeys := s.settings.GetVoiceMakerKeys()
		for _, k := range vKeys {
			if k.ID == vKeyID {
				vApiKey = k.Key
				break
			}
		}
		if vApiKey == "" && len(vKeys) > 0 {
			vApiKey = vKeys[0].Key
		}

		if vApiKey == "" {
			s.log("ERROR", "[VoiceMaker] API key not found", id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return fmt.Errorf("API key not found")
		}

		if vID == "" {
			s.log("ERROR", "[VoiceMaker] Voice ID is not selected!", id, taskLabel)
			return fmt.Errorf("voice ID not selected")
		}

		s.emitStageStatus(id, "voice", "running")

		// 1. Splitting text
		chunks := utils.SplitTextByChunks(processedText, charLimit)
		s.log("INFO", fmt.Sprintf("[VoiceMaker] Split text into %d chunks (limit: %d chars)", len(chunks), charLimit), id, taskLabel)

		tempDir := filepath.Join(finalDir, "temp_audio")
		os.MkdirAll(tempDir, 0755)
		// Removed defer os.RemoveAll(tempDir) as requested by user to keep temporary files

		chunkFiles := make([]string, len(chunks))
		var wg sync.WaitGroup
		semaphore := make(chan struct{}, 10) // Limit to 10 concurrent connections
		var firstErr error
		var errOnce sync.Once

		for i, chunk := range chunks {
			wg.Add(1)
			go func(idx int, text string) {
				defer wg.Done()
				semaphore <- struct{}{}
				defer func() { <-semaphore }()

				if firstErr != nil {
					return
				}

				chunkPath := filepath.Join(tempDir, fmt.Sprintf("chunk_%03d.mp3", idx))
				// Retry logic: 10 attempts, 5s delay
				var err error
				maxAttempts := 10
				for attempt := 0; attempt < maxAttempts; attempt++ {
					err = s.voiceMaker.Synthesize(vApiKey, text, vID, vLang, chunkPath, id, taskLabel)
					if err == nil {
						chunkFiles[idx] = chunkPath
						s.log("INFO", fmt.Sprintf("[VoiceMaker] Chunk %d/%d generated", idx+1, len(chunks)), id, taskLabel)
						return
					}
					s.log("WARN", fmt.Sprintf("[VoiceMaker] Chunk %d failed (attempt %d/%d): %v. Retrying in 5s...", idx+1, attempt+1, maxAttempts, err), id, taskLabel)
					time.Sleep(5 * time.Second)
				}

				errOnce.Do(func() {
					firstErr = err
				})
			}(i, chunk)
		}

		wg.Wait()

		if firstErr != nil {
			s.log("ERROR", fmt.Sprintf("[VoiceMaker] Synthesis failed: %v", firstErr), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return firstErr
		}

		// 2. Merging files
		voiceFilePath := filepath.Join(finalDir, "voice.mp3")
		err := s.mergeAudioFiles(chunkFiles, voiceFilePath)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[VoiceMaker] Failed to merge audio files: %v", err), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return err
		}

		duration, _ := utils.GetAudioDuration(voiceFilePath)
		s.log("SUCCESS", fmt.Sprintf("[VoiceMaker] Success: Voice saved to voice.mp3 (%s)", duration), id, taskLabel)
		s.emitStageStatus(id, "voice", "completed", duration)
	} else if vService != "" {
		s.log("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for auto-synthesis", vService), id, taskLabel)
	} else {
		s.log("ERROR", "[Pipeline] Voiceover service is not selected!", id, taskLabel)
	}

	return nil
}

// mergeAudioFiles concatenates multiple audio files into one
func (s *PipelineService) mergeAudioFiles(files []string, outputPath string) error {
	out, err := os.Create(outputPath)
	if err != nil {
		return err
	}
	defer out.Close()

	for _, f := range files {
		if f == "" {
			continue
		}
		in, err := os.Open(f)
		if err != nil {
			return err
		}
		_, err = io.Copy(out, in)
		in.Close()
		if err != nil {
			return err
		}
	}
	return nil
}
