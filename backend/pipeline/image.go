package pipeline

import (
	"fmt"
	"os"
	"path/filepath"
	bapi "soloveyko/backend/api"
	"soloveyko/backend/utils"
	"strings"
	"sync"
)

// splitIntoLines splits text by line breaks
func splitIntoLines(text string) []string {
	var lines []string
	for _, line := range strings.Split(text, "\n") {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) > 0 {
			lines = append(lines, trimmed)
		}
	}
	return lines
}

// splitIntoSentences splits text by standard sentence delimiters
func splitIntoSentences(text string) []string {
	var sentences []string
	var current []rune
	runes := []rune(text)
	for _, r := range runes {
		current = append(current, r)
		if r == '.' || r == '!' || r == '?' {
			s := strings.TrimSpace(string(current))
			if len(s) > 0 {
				sentences = append(sentences, s)
			}
			current = nil
		}
	}
	if len(current) > 0 {
		s := strings.TrimSpace(string(current))
		if len(s) > 0 {
			sentences = append(sentences, s)
		}
	}
	return sentences
}

// groupSentences groups sentences up to a character limit
func groupSentences(sentences []string, limit int) []string {
	var groups []string
	var currentGroup string
	for _, s := range sentences {
		if len(currentGroup) == 0 {
			currentGroup = s
		} else if len([]rune(currentGroup))+1+len([]rune(s)) <= limit {
			currentGroup += " " + s
		} else {
			groups = append(groups, currentGroup)
			currentGroup = s
		}
	}
	if len(currentGroup) > 0 {
		groups = append(groups, currentGroup)
	}
	return groups
}

// ProcessImage handles image generation
func (s *PipelineService) ProcessImage(id string, taskLabel string, taskType string, processedText string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings, taskName string, subName string) error {
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

	iGenMethod, _ := settings["imageGenerationMethod"].(string)
	if iGenMethod == "" {
		iGenMethod = pSettings.ImageGenerationMethod
	}
	if iGenMethod == "" {
		iGenMethod = "sentences"
	}

	iGroup, ok := settings["imageGroupSentences"].(bool)
	if !ok {
		iGroup = pSettings.ImageGroupSentences
	}

	iLimit := 1000.0
	if val, ok := settings["imageSentenceLimit"].(float64); ok && val > 0 {
		iLimit = val
	} else if pSettings.ImageSentenceLimit > 0 {
		iLimit = float64(pSettings.ImageSentenceLimit)
	}

	s.log("INFO", fmt.Sprintf("[Pipeline] Image chunking method: %s", iGenMethod), id, taskLabel)

	var chunks []string
	if iGenMethod == "lines" {
		chunks = splitIntoLines(processedText)
	} else {
		sentences := splitIntoSentences(processedText)
		if iGroup {
			chunks = groupSentences(sentences, int(iLimit))
		} else {
			chunks = sentences
		}
	}

	if len(chunks) == 0 {
		s.log("WARN", "[Pipeline] No text segments found for image generation.", id, taskLabel)
		return nil
	}

	s.log("INFO", fmt.Sprintf("[Pipeline] Created %d segments for image instructions", len(chunks)), id, taskLabel)

	// Fetch OpenRouter API Key for prompt generation
	orKeyID, _ := settings[taskType+"OpenRouterKeyID"].(string)

	orKeys := s.settings.GetOpenRouterKeys()
	var orApiKey, orKeyName string

	// First try to match the selected key
	for _, k := range orKeys {
		if k.ID == orKeyID {
			orApiKey = k.Key
			orKeyName = k.Name
			break
		}
	}

	// Fallback to first available key
	if orApiKey == "" && len(orKeys) > 0 {
		orApiKey = orKeys[0].Key
		orKeyName = orKeys[0].Name
	}

	if orApiKey == "" {
		s.log("ERROR", "[Pipeline] OpenRouter API Key missing! Required for interpreting prompts.", id, taskLabel)
		return fmt.Errorf("OpenRouter API Key required")
	}

	orModel, _ := settings["imagePromptModel"].(string)
	if orModel == "" {
		orModel = pSettings.ImagePromptModel
	}
	if orModel == "" {
		orModels := s.settings.GetOpenRouterModels()
		if len(orModels) > 0 {
			orModel = orModels[0]
		} else {
			orModel = "google/gemini-2.5-flash"
		}
	}

	temp := 0.7
	if val, ok := settings["imagePromptTemperature"].(float64); ok {
		temp = val
	} else if pSettings.ImagePromptTemperature > 0 {
		temp = pSettings.ImagePromptTemperature
	}

	tokens := 0
	if val, ok := settings["imagePromptMaxTokens"].(float64); ok {
		tokens = int(val)
	} else if pSettings.ImagePromptMaxTokens > 0 {
		tokens = pSettings.ImagePromptMaxTokens
	}

	promptTemplate, _ := settings["imagePrompt"].(string)
	if promptTemplate == "" {
		promptTemplate = pSettings.ImagePrompt
	}

	s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: 0/%d", len(chunks)))
	prompts := make([]string, len(chunks))
	var wg sync.WaitGroup
	var mu sync.Mutex
	var genError error
	var generatedPromptsCount int

	s.log("INFO", fmt.Sprintf("[Pipeline] Generating %d prompts via OpenRouter (%s)...", len(chunks), orModel), id, taskLabel)
	for i, chunk := range chunks {
		wg.Add(1)
		go func(index int, textChunk string) {
			defer wg.Done()
			var fullPrompt string
			if strings.Contains(promptTemplate, "{{content}}") {
				fullPrompt = strings.ReplaceAll(promptTemplate, "{{content}}", textChunk)
			} else {
				fullPrompt = promptTemplate + "\n\n" + textChunk
			}

			// We use OpenRouter's internal Chat which handles rate limiting/semaphore automatically
			res, err := s.openRouter.Chat(id, taskLabel, "image_prompt", orKeyName, orApiKey, orModel, fullPrompt, temp, tokens)

			mu.Lock()
			if err != nil {
				if genError == nil {
					genError = err
				}
			} else {
				prompts[index] = strings.TrimSpace(res)
				generatedPromptsCount++
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d", generatedPromptsCount, len(chunks)))
			}
			mu.Unlock()
		}(i, chunk)
	}
	wg.Wait()

	if genError != nil {
		s.log("ERROR", fmt.Sprintf("[Pipeline] Failed to generate some image prompts: %v", genError), id, taskLabel)
		s.emitStageStatus(id, "image", "failed")
		return genError
	}

	// Save prompts to file
	promptsFilePath := filepath.Join(finalDir, "prompts.txt")
	promptsContent := strings.Join(prompts, "\n\n--------------------\n\n")
	err := os.WriteFile(promptsFilePath, []byte(promptsContent), 0644)
	if err != nil {
		s.log("WARN", fmt.Sprintf("[Pipeline] Failed to save prompts.txt: %v", err), id, taskLabel)
	} else {
		s.log("SUCCESS", fmt.Sprintf("[Pipeline] Saved generated prompts to %s", promptsFilePath), id, taskLabel)
	}

	// Create images dir
	imagesDir := filepath.Join(finalDir, "images")
	if err := os.MkdirAll(imagesDir, 0755); err != nil {
		return fmt.Errorf("failed to create images dir: %v", err)
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

		iModel, _ := settings["imageModel"].(string)
		if iModel == "" {
			iModel = pSettings.ImageModel
		}

		iWidth, _ := settings["imageWidth"].(float64)
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

		var validPrompts int
		for _, p := range prompts {
			if len(p) > 0 {
				validPrompts++
			}
		}

		s.log("INFO", fmt.Sprintf("[Pipeline] Image Generation started. Service: %s, Model: %s", iService, iModel), id, taskLabel)
		s.log("INFO", fmt.Sprintf("[Pollinations] Model: %s, Size: %dx%d, NoLogo: %t, Enhance: %t", iModel, int(iWidth), int(iHeight), iNoLogo, iEnhance), id, taskLabel)
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: 0/%d\nvideos: 0/0", validPrompts, validPrompts, validPrompts))

		successCount := 0
		for i, prompt := range prompts {
			// Skip empty prompts
			if len(prompt) == 0 {
				continue
			}

			// save simply as 1.png, 2.png, etc
			imgName := fmt.Sprintf("%d.png", i+1)
			imgPath := filepath.Join(imagesDir, imgName)

			s.log("INFO", fmt.Sprintf("[Pollinations] Sending request for Image %s...", imgName), id, taskLabel)
			err := s.pollinations.GenerateImage(iApiKey, prompt, iModel, int(iWidth), int(iHeight), iNoLogo, iEnhance, imgPath)
			if err != nil {
				s.log("ERROR", fmt.Sprintf("[Pollinations] Image %s failed: %v", imgName, err), id, taskLabel)
				// continue generating others instead of failing completely, but we can fail completely if desired
			} else {
				successCount++
				if s.OnImageGenerated != nil {
					s.OnImageGenerated(taskName, subName, imgName, imgPath)
				}
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: 0/0", validPrompts, validPrompts, successCount, validPrompts))
				s.log("SUCCESS", fmt.Sprintf("[Pollinations] Success: Generated %s", imgName), id, taskLabel)
			}
		}

		if validPrompts > 0 && successCount == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("prompts: %d/%d\nimages: 0/%d\nvideos: 0/0", validPrompts, validPrompts, validPrompts))
			return fmt.Errorf("failed to generate any images")
		}
	} else if iService == "googler" {
		iApiKey := s.googler.GetAPIKey()

		iModel, _ := settings["imageGooglerModel"].(string)
		if iModel == "" {
			iModel = pSettings.ImageGooglerModel
		}
		if iModel == "" {
			iModel = "whisk"
		}

		iRatio, _ := settings["imageGooglerAspectRatio"].(string)
		if iRatio == "" {
			iRatio = pSettings.ImageGooglerAspectRatio
		}
		if iRatio == "" {
			iRatio = "IMAGE_ASPECT_RATIO_LANDSCAPE"
		}

		iRemixEnabled, ok := settings["imageGooglerRemixEnabled"].(bool)
		if !ok {
			iRemixEnabled = pSettings.ImageGooglerRemixEnabled
		}

		iRefImage, ok := settings["imageGooglerReferenceImage"].(string)
		if !ok {
			iRefImage = pSettings.ImageGooglerReferenceImage
		}

		iStrictMode, ok := settings["imageGooglerRemixStrictMode"].(bool)
		if !ok {
			iStrictMode = pSettings.ImageGooglerRemixStrictMode
		}

		iVideoEnabled, ok := settings["imageGooglerVideoEnabled"].(bool)
		if !ok {
			iVideoEnabled = pSettings.ImageGooglerVideoEnabled
		}

		iVideoModel, _ := settings["imageGooglerVideoModel"].(string)
		if iVideoModel == "" {
			iVideoModel = pSettings.ImageGooglerVideoModel
		}
		if iVideoModel == "" {
			iVideoModel = "whisk"
		}

		iVideoMode, _ := settings["imageGooglerVideoMode"].(string)
		if iVideoMode == "" {
			iVideoMode = pSettings.ImageGooglerVideoMode
		}
		if iVideoMode == "" {
			iVideoMode = "text"
		}

		iVideoCount := 0
		if val, ok := settings["imageGooglerVideoCount"]; ok {
			switch v := val.(type) {
			case float64:
				iVideoCount = int(v)
			case int:
				iVideoCount = v
			}
		}
		if iVideoCount <= 0 {
			iVideoCount = pSettings.ImageGooglerVideoCount
		}
		if iVideoCount <= 0 {
			iVideoCount = 1
		}

		iVideoUpscale, ok := settings["imageGooglerVideoUpscale"].(bool)
		if !ok {
			iVideoUpscale = pSettings.ImageGooglerVideoUpscale
		}

		var refImages []bapi.ReferenceImage
		if iRemixEnabled && iRefImage != "" && iModel == "whisk" {
			b64, err := utils.GetImageAsBase64(iRefImage)
			if err != nil {
				s.log("WARN", fmt.Sprintf("[Googler] Reference image not found: %s. Falling back to standard generation without remix.", iRefImage), id, taskLabel)
			} else {
				refImages = append(refImages, bapi.ReferenceImage{
					Category: "MEDIA_CATEGORY_STYLE",
					Image:    b64,
				})
				s.log("INFO", fmt.Sprintf("[Googler] Reference image loaded (%s)", iRefImage), id, taskLabel)
			}
		}

		var validPrompts int
		for _, p := range prompts {
			if len(p) > 0 {
				validPrompts++
			}
		}

		totalVideos := 0
		if iVideoEnabled {
			totalVideos = iVideoCount
			if totalVideos > validPrompts {
				totalVideos = validPrompts
			}
		}

		totalImages := validPrompts
		if iVideoEnabled && iVideoMode == "text" {
			totalImages = validPrompts - totalVideos
		}

		s.log("INFO", fmt.Sprintf("[Pipeline] Image/Video Generation started. Service: %s", iService), id, taskLabel)
		if len(refImages) > 0 {
			s.log("INFO", fmt.Sprintf("[Googler] Mode: REMIX (Style), Aspect Ratio: %s, Strict: %v", iRatio, iStrictMode), id, taskLabel)
		} else {
			s.log("INFO", fmt.Sprintf("[Googler] Model: %s, Aspect Ratio: %s", iModel, iRatio), id, taskLabel)
		}
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: 0/%d\nvideos: 0/%d", validPrompts, validPrompts, totalImages, totalVideos))

		successCount := 0
		imagesCount := 0
		videosCount := 0
		var imageWg sync.WaitGroup
		var imgMu sync.Mutex

		// PHASE 1: Generate all media (Images and Videos) in parallel
		validIdx := 0
		for _, prompt := range prompts {
			if len(prompt) == 0 {
				continue
			}
			currentIdx := validIdx
			validIdx++

			imageWg.Add(1)
			go func(idx int, p string) {
				defer imageWg.Done()

				isVideo := iVideoEnabled && idx < iVideoCount
				imgName := fmt.Sprintf("%d.png", idx+1)
				vidName := fmt.Sprintf("%d.mp4", idx+1)
				imgPath := filepath.Join(imagesDir, imgName)
				vidPath := filepath.Join(imagesDir, vidName)

				// Case A: Text-to-Video
				if isVideo && iVideoMode == "text" {
					s.log("INFO", fmt.Sprintf("[Googler] [%d] START Text-to-Video: %s...", idx, vidName), id, taskLabel)
					err := s.googler.GenerateVideo(iApiKey, iVideoModel, p, "", iRatio, iVideoUpscale, vidPath)

					imgMu.Lock()
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Video %s failed: %v", idx, vidName, err), id, taskLabel)
					} else {
						successCount++
						videosCount++
						if s.OnImageGenerated != nil {
							s.OnImageGenerated(taskName, subName, vidName, vidPath)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: %d/%d", validPrompts, validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Video generation: %s", idx, vidName), id, taskLabel)
					}
					imgMu.Unlock()
					return
				}

				// Case B: Generate Image (either as final OR as base for animation)
				s.log("INFO", fmt.Sprintf("[Googler] [%d] START Image generation: %s...", idx, imgName), id, taskLabel)
				var err error
				if len(refImages) > 0 {
					err = s.googler.RemixImage(iApiKey, p, refImages, iRatio, iStrictMode, imgPath)
				} else {
					err = s.googler.GenerateImage(iApiKey, iModel, p, iRatio, imgPath)
				}

				if err != nil {
					s.log("ERROR", fmt.Sprintf("[Googler] [%d] Image %s failed: %v", idx, imgName, err), id, taskLabel)
					return // Task failed for this segment
				}

				// Success for Image
				imgMu.Lock()
				imagesCount++
				if s.OnImageGenerated != nil {
					s.OnImageGenerated(taskName, subName, imgName, imgPath)
				}
				// If we DON'T plan to animate it, it's a final success now
				if !(isVideo && iVideoMode == "image") {
					successCount++
				}
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: %d/%d", validPrompts, validPrompts, imagesCount, totalImages, videosCount, totalVideos))
				s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Image generation: %s", idx, imgName), id, taskLabel)
				imgMu.Unlock()

				// Case C: If Image-to-Video, animate IMMEDIATELY
				if isVideo && iVideoMode == "image" {
					s.log("INFO", fmt.Sprintf("[Googler] [%d] START Video animation: %s from %s...", idx, vidName, imgName), id, taskLabel)
					b64, err := utils.GetImageAsBase64(imgPath)
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Failed to read image for animation %s: %v", idx, imgName, err), id, taskLabel)
						return
					}

					err = s.googler.GenerateVideo(iApiKey, iVideoModel, p, b64, iRatio, iVideoUpscale, vidPath)

					imgMu.Lock()
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Video animation failed: %v", idx, err), id, taskLabel)
					} else {
						// Video success!
						// Note: we Keep imagesCount as is (reflecting total images generated)
						videosCount++
						successCount++ // Now it's a final success

						if s.OnImageDeleted != nil {
							s.OnImageDeleted(imgPath)
						}
						_ = os.Remove(imgPath)

						if s.OnImageGenerated != nil {
							s.OnImageGenerated(taskName, subName, vidName, vidPath)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: %d/%d", validPrompts, validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Video animation: %s", idx, vidName), id, taskLabel)
					}
					imgMu.Unlock()
				}
			}(currentIdx, prompt)
		}
		imageWg.Wait()

		imgMu.Lock()
		finalSuccess := successCount
		finalImages := imagesCount
		finalVideos := videosCount
		imgMu.Unlock()

		if validPrompts > 0 && finalSuccess == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: %d/%d", validPrompts, validPrompts, finalImages, totalImages, finalVideos, totalVideos))
			return fmt.Errorf("failed to generate any media")
		}

		s.log("SUCCESS", fmt.Sprintf("[Pipeline] Image/Video stage DONE for %s. Success: %d/%d, Images: %d, Videos: %d", id, finalSuccess, validPrompts, finalImages, finalVideos), id, taskLabel)
	} else if iService == "elevenlabsimage" {
		iKeyID, _ := settings["elevenLabsImageKeyID"].(string)
		if iKeyID == "" {
			iKeyID = pSettings.ElevenLabsImageKeyID
		}

		iApiKey := ""
		iKeys := s.settings.GetElevenLabsImageKeys()
		for _, k := range iKeys {
			if k.ID == iKeyID {
				iApiKey = k.Key
				break
			}
		}
		if iApiKey == "" && len(iKeys) > 0 {
			iApiKey = iKeys[0].Key
		}

		iRatio, _ := settings["elevenLabsImageAspectRatio"].(string)
		if iRatio == "" {
			iRatio = pSettings.ElevenLabsImageAspectRatio
		}
		if iRatio == "" {
			iRatio = "16:9"
		}

		var validPrompts int
		for _, p := range prompts {
			if len(p) > 0 {
				validPrompts++
			}
		}

		s.log("INFO", fmt.Sprintf("[Pipeline] Image Generation started. Service: %s", iService), id, taskLabel)
		s.log("INFO", fmt.Sprintf("[ElevenLabs Image] Aspect Ratio: %s", iRatio), id, taskLabel)
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: 0/%d\nvideos: 0/0", validPrompts, validPrompts, validPrompts))

		successCount := 0
		imagesCount := 0
		var imageWg sync.WaitGroup
		var imgMu sync.Mutex

		for i, prompt := range prompts {
			if len(prompt) == 0 {
				continue
			}

			imageWg.Add(1)
			go func(idx int, p string) {
				defer imageWg.Done()
				imgName := fmt.Sprintf("%d.png", idx+1)
				imgPath := filepath.Join(imagesDir, imgName)

				s.log("INFO", fmt.Sprintf("[ElevenLabs Image] Sending request for Image %s | Ratio: %s", imgName, iRatio), id, taskLabel)
				err := s.elevenLabsImage.GenerateImage(iApiKey, p, iRatio, imgPath)

				imgMu.Lock()
				if err != nil {
					s.log("ERROR", fmt.Sprintf("[ElevenLabs Image] Image %s failed: %v", imgName, err), id, taskLabel)
				} else {
					successCount++
					imagesCount++
					if s.OnImageGenerated != nil {
						s.OnImageGenerated(taskName, subName, imgName, imgPath)
					}
					s.emitStageStatus(id, "image", "running", fmt.Sprintf("prompts: %d/%d\nimages: %d/%d\nvideos: 0/0", validPrompts, validPrompts, imagesCount, validPrompts))
					s.log("SUCCESS", fmt.Sprintf("[ElevenLabs Image] Success: Generated and saved %s", imgName), id, taskLabel)
				}
				imgMu.Unlock()
			}(i, prompt)
		}
		imageWg.Wait()

		if validPrompts > 0 && successCount == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("prompts: %d/%d\nimages: 0/%d\nvideos: 0/0", validPrompts, validPrompts, validPrompts))
			return fmt.Errorf("failed to generate any images")
		}
	} else if iService != "" {
		s.log("WARN", fmt.Sprintf("[Pipeline] Image service %s is not yet implemented", iService), id, taskLabel)
	} else {
		s.log("ERROR", "[Pipeline] Image service is not selected!", id, taskLabel)
	}

	return nil
}
