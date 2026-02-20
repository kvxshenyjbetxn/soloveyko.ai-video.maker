package pipeline

import (
	"fmt"
	"path/filepath"
	"soloveyko/backend/utils"
)

// ProcessImage handles image generation
func (s *PipelineService) ProcessImage(id string, taskLabel string, processedText string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) error {
	var iEnabled bool
	if val, ok := settings["imageEnabled"].(bool); ok {
		iEnabled = val
	} else {
		iEnabled = pSettings.ImageEnabled
	}

	if !iEnabled {
		s.log("INFO", "[Pipeline] Image stage is disabled, skipping.", id, taskLabel)
		return nil
	}

	iService, _ := settings["imageService"].(string)
	if iService == "" {
		iService = pSettings.ImageService
	}

	if iService == "pollinations" {
		iKeyID, _ := settings["imagePollinationsKeyID"].(string)
		if iKeyID == "" {
			iKeyID = pSettings.ImagePollinationsKeyID
		}

		iApiKey := ""
		iKeys := s.settings.GetPollinationsKeys()
		for _, k := range iKeys {
			if k.ID == iKeyID {
				iApiKey = k.Key
				break
			}
		}
		if iApiKey == "" && len(iKeys) > 0 {
			iApiKey = iKeys[0].Key
		}

		// If still empty, it might be that the user is using the free tier without a key
		// PollinationsService.GenerateImage handles empty apiKey for free tier rate limiting

		iModel, _ := settings["imageModel"].(string)
		if iModel == "" {
			iModel = pSettings.ImageModel
		}

		iWidth, _ := settings["imageWidth"].(float64) // Settings often come as float64 from JSON map
		if iWidth == 0 {
			iWidth = float64(pSettings.ImageWidth)
		}
		if iWidth == 0 {
			iWidth = 1920
		}

		iHeight, _ := settings["imageHeight"].(float64)
		if iHeight == 0 {
			iHeight = float64(pSettings.ImageHeight)
		}
		if iHeight == 0 {
			iHeight = 1080
		}

		iNoLogo, ok := settings["imageNoLogo"].(bool)
		if !ok {
			iNoLogo = pSettings.ImageNoLogo
		}

		iEnhance, ok := settings["imageEnhance"].(bool)
		if !ok {
			iEnhance = pSettings.ImageEnhance
		}

		s.log("INFO", fmt.Sprintf("[Pipeline] Image stage started. Service: %s, Model: %s, Size: %dx%d", iService, iModel, int(iWidth), int(iHeight)), id, taskLabel)
		s.emitStageStatus(id, "image", "running")

		imagePath := filepath.Join(finalDir, "image.jpg")

		// Pollinations doesn't need much logic here, it's a single call
		err := s.pollinations.GenerateImage(iApiKey, processedText, iModel, int(iWidth), int(iHeight), iNoLogo, iEnhance, imagePath)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Pollinations] Image generation failed: %v", err), id, taskLabel)
			s.emitStageStatus(id, "image", "failed")
			return err
		}

		s.log("SUCCESS", "[Pollinations] Success: Image saved to image.jpg", id, taskLabel)
		s.emitStageStatus(id, "image", "completed")
	} else if iService != "" {
		s.log("WARN", fmt.Sprintf("[Pipeline] Image service %s is not yet implemented", iService), id, taskLabel)
	} else {
		s.log("ERROR", "[Pipeline] Image service is not selected!", id, taskLabel)
	}

	return nil
}
