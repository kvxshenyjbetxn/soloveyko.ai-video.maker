package pipeline

import (
	"fmt"
	"path/filepath"
	"soloveyko/backend/utils"
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

	s.log("INFO", fmt.Sprintf("[Pipeline] Voiceover stage started. Service: %s, Template: %s", vService, vTemplate), id, taskLabel)

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
		err := s.elevenLabs.Synthesize(vApiKey, processedText, vTemplate, voiceFilePath, id, taskLabel)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[ElevenLabsBot] Synthesis Error: %v", err), id, taskLabel)
			s.emitStageStatus(id, "voice", "failed")
			return err
		}

		s.log("SUCCESS", "[ElevenLabsBot] Success: Voice saved to voice.mp3", id, taskLabel)
		s.emitStageStatus(id, "voice", "completed")
	} else if vService != "" {
		s.log("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for auto-synthesis", vService), id, taskLabel)
	} else {
		s.log("ERROR", "[Pipeline] Voiceover service is not selected!", id, taskLabel)
	}

	return nil
}
