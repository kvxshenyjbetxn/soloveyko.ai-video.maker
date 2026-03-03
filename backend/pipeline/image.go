package pipeline

import (
	"encoding/json"
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

	// Robust detection of skipped stages (Restore choice from modal)
	var shouldSkipImage bool
	if val, ok := settings["skippedStages"]; ok {
		switch s := val.(type) {
		case []string:
			for _, st := range s {
				if st == "image" {
					shouldSkipImage = true
					break
				}
			}
		case []interface{}:
			for _, st := range s {
				if str, ok := st.(string); ok && str == "image" {
					shouldSkipImage = true
					break
				}
			}
		}
	}

	shouldRegeneratePrompts, _ := settings["imageRegeneratePrompts"].(bool)
	shouldRegenerateImages, _ := settings["imageGooglerRegenerateImages"].(bool)
	if !shouldRegeneratePrompts {
		shouldRegeneratePrompts = shouldRegenerateImages
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

	iInitialCount := 0
	if val, ok := settings["imageInitialSentenceCount"].(float64); ok {
		iInitialCount = int(val)
	} else if pSettings.ImageInitialSentenceCount > 0 {
		iInitialCount = pSettings.ImageInitialSentenceCount
	}

	iMode, _ := settings["imageMode"].(string)
	if iMode == "" {
		iMode = pSettings.ImageMode
	}
	if iMode == "" {
		iMode = "normal"
	}

	iMemType, _ := settings["imageMemoryType"].(string)
	if iMemType == "" {
		iMemType = pSettings.ImageMemoryType
	}
	if iMemType == "" {
		iMemType = "primitive"
	}

	iMemChars := 1000
	if val, ok := settings["imageMemoryChars"].(float64); ok && val > 0 {
		iMemChars = int(val)
	} else if pSettings.ImageMemoryChars > 0 {
		iMemChars = pSettings.ImageMemoryChars
	}

	// For lines mode, if initial count is set, we treat it as starting individual lines,
	// and we force grouping for the rest to make the dynamic start meaningful.
	if iGenMethod == "lines" && iInitialCount > 0 {
		iGroup = true
	}

	// Determine directories early
	imagesDir := filepath.Join(finalDir, "images")
	promptsFilePath := filepath.Join(finalDir, "prompts.txt")

	// [AGGRESSIVE RESTORE CHECK]
	// If images already exist, we skip the heavy prep (Character Detection, API prompt generation)
	// and either skip the stage entirely or only generate missing pieces.
	hasExistingImages := false
	if info, err := os.Stat(imagesDir); err == nil && info.IsDir() {
		files, _ := os.ReadDir(imagesDir)
		for _, f := range files {
			if !f.IsDir() {
				ext := strings.ToLower(filepath.Ext(f.Name()))
				if ext == ".png" || ext == ".jpg" || ext == ".jpeg" || ext == ".webp" || ext == ".mp4" {
					hasExistingImages = true
					break
				}
			}
		}
	}

	if hasExistingImages && !shouldRegeneratePrompts && !shouldSkipImage {
		s.log("INFO", "[Pipeline] Existing files found in 'images' folder. Activating Restore Mode: Skipping heavy prep (prompts).", id, taskLabel)
	}

	s.log("INFO", fmt.Sprintf("[Pipeline] Image chunking method: %s", iGenMethod), id, taskLabel)

	var baseSegments []string
	if iGenMethod == "lines" {
		baseSegments = splitIntoLines(processedText)
	} else {
		baseSegments = splitIntoSentences(processedText)
	}

	var chunks []string
	if iInitialCount > 0 && len(baseSegments) > 0 {
		// First N dynamic segments, but each must be at least 50 chars
		currentBaseIdx := 0
		for dynamicChunkIdx := 0; dynamicChunkIdx < iInitialCount && currentBaseIdx < len(baseSegments); dynamicChunkIdx++ {
			currentChunk := baseSegments[currentBaseIdx]
			currentBaseIdx++

			// Merge next segments until we hit 50 chars (hook/dynamic start constraint)
			for len([]rune(currentChunk)) < 50 && currentBaseIdx < len(baseSegments) {
				currentChunk += " " + baseSegments[currentBaseIdx]
				currentBaseIdx++
			}
			chunks = append(chunks, currentChunk)
		}

		remaining := baseSegments[currentBaseIdx:]
		// Remaining segments grouped by limit
		if len(remaining) > 0 {
			if iGroup {
				chunks = append(chunks, groupSentences(remaining, int(iLimit))...)
			} else {
				chunks = append(chunks, remaining...)
			}
		}
	} else {
		if iGroup {
			chunks = groupSentences(baseSegments, int(iLimit))
		} else {
			chunks = baseSegments
		}
	}

	if len(chunks) == 0 {
		s.log("WARN", "[Pipeline] No text segments found for image generation.", id, taskLabel)
		return nil
	}

	// [SYNC] Save chunks to segments.json
	chunksData, _ := json.MarshalIndent(chunks, "", "  ")
	_ = os.WriteFile(filepath.Join(finalDir, "segments.json"), chunksData, 0644)
	s.log("INFO", fmt.Sprintf("[Pipeline] Updated %d segments for synchronization", len(chunks)), id, taskLabel)

	// [USER RESTORE SKIP]
	if shouldSkipImage {
		s.log("INFO", "[Pipeline] Restore Mode: Using existing images/videos as-is. Sync updated.", id, taskLabel)
		// Get actual counts of what was restored to show the user
		data := s.CheckExistingFiles(id, finalDir, taskType, settings)
		s.emitStageStatus(id, "image", "completed", fmt.Sprintf("p:%d i:%d v:%d", data.PromptCount, data.ImageCount, data.VideoCount))
		return nil
	}

	// [FULL STAGE SKIP]
	// If we have images for every chunk and a prompts file, we skip everything.
	// BUT: if we are coming from the restoration modal (skippedStages exists in settings)
	// AND shouldSkipImage is false, we MUST re-process (don't auto-skip).
	isFromModal := false
	if _, ok := settings["skippedStages"]; ok {
		isFromModal = true
	}

	if (!isFromModal) && hasExistingImages && !shouldRegeneratePrompts {
		allFound := true
		countImg := 0
		countVid := 0
		for i := 1; i <= len(chunks); i++ {
			found := false
			paths := []string{
				filepath.Join(imagesDir, fmt.Sprintf("%d.png", i)),
				filepath.Join(imagesDir, fmt.Sprintf("%d.mp4", i)),
			}
			for _, p := range paths {
				if st, err := os.Stat(p); err == nil && !st.IsDir() {
					found = true
					if strings.HasSuffix(p, ".mp4") {
						countVid++
					} else {
						countImg++
					}
					break
				}
			}
			if !found {
				allFound = false
				break
			}
		}

		if allFound {
			s.log("SUCCESS", fmt.Sprintf("[Pipeline] All %d assets found (i:%d, v:%d). Auto-skipping.", len(chunks), countImg, countVid), id, taskLabel)
			s.emitStageStatus(id, "image", "completed", fmt.Sprintf("i:%d v:%d", countImg, countVid))
			return nil
		}
	}

	s.log("INFO", fmt.Sprintf("[Pipeline] Image instructions: %d chunks", len(chunks)), id, taskLabel)

	// Fetch OpenRouter API Key for prompt generation
	orKeyID, _ := settings[taskType+"OpenRouterKeyID"].(string)
	orKeys := s.settings.GetOpenRouterKeys()
	var orApiKey, orKeyName string
	for _, k := range orKeys {
		if k.ID == orKeyID {
			orApiKey = k.Key
			orKeyName = k.Name
			break
		}
	}
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
		orModel = "google/gemini-2.0-flash"
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

	// [CHARACTER DETECTION] - Skip if we have images and are not regenerating
	if !hasExistingImages || shouldRegeneratePrompts {
		detChars, ok := settings["imageDetermineCharacters"].(bool)
		if !ok {
			detChars = pSettings.ImageDetermineCharacters
		}
		if detChars && strings.Contains(promptTemplate, "{{characters}}") {
			detPrompt, _ := settings["imageDetermineCharactersPrompt"].(string)
			if detPrompt == "" {
				detPrompt = pSettings.ImageDetermineCharactersPrompt
			}
			if detPrompt != "" {
				s.log("INFO", "[Pipeline] Determining characters from text...", id, taskLabel)
				s.emitStageStatus(id, "image", "running", "characters...")
				charRes, err := s.openRouter.Chat(id, taskLabel, "image_characters", orKeyName, orApiKey, orModel, detPrompt+"\n\n"+processedText, temp, tokens)
				if err != nil {
					s.log("ERROR", fmt.Sprintf("[Pipeline] Failed to determine characters: %v", err), id, taskLabel)
				} else {
					charDesc := strings.TrimSpace(charRes)
					s.log("SUCCESS", "[Pipeline] Characters determined and added to instruction template", id, taskLabel)
					promptTemplate = strings.ReplaceAll(promptTemplate, "{{characters}}", charDesc)
				}
			}
		}
	} else {
		// Restore Mode: Remove placeholder even if we don't detect
		promptTemplate = strings.ReplaceAll(promptTemplate, "{{characters}}", "")
	}

	var loadedExisting bool
	var prompts []string

	if shouldRegeneratePrompts {
		s.log("INFO", "[Pipeline] User requested to regenerate all files, will not load existing prompts.", id, taskLabel)
		prompts = make([]string, len(chunks))
	} else {
		if content, err := os.ReadFile(promptsFilePath); err == nil {
			pStrs := strings.Split(string(content), "\n\n--------------------\n\n")
			prompts = make([]string, len(chunks))
			count := 0
			for i := 0; i < len(chunks) && i < len(pStrs); i++ {
				prompts[i] = strings.TrimSpace(pStrs[i])
				if prompts[i] != "" {
					count++
				}
			}
			if count >= len(chunks) {
				loadedExisting = true
				s.log("INFO", "[Pipeline] Loaded all existing image prompts from prompts.txt", id, taskLabel)
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d", count))
			} else if count > 0 {
				s.log("INFO", fmt.Sprintf("[Pipeline] Loaded %d/%d existing image prompts, will generate missing ones.", count, len(chunks)), id, taskLabel)
			}
		}
		if !loadedExisting && len(prompts) == 0 {
			prompts = make([]string, len(chunks))
		}
	}

	if !loadedExisting {
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:0/%d", len(chunks)))

		memoryContexts := make([]string, len(chunks))
		if iMode == "memory" && iMemType == "primitive" {
			currentPos := 0
			for idx, chunk := range chunks {
				pos := strings.Index(processedText[currentPos:], chunk)
				if pos >= 0 {
					absolutePos := currentPos + pos
					textBefore := processedText[:absolutePos]

					runesBefore := []rune(textBefore)
					if len(runesBefore) > iMemChars {
						cutoffIndex := len(runesBefore) - iMemChars
						for cutoffIndex < len(runesBefore) {
							r := runesBefore[cutoffIndex]
							if r == '.' || r == '!' || r == '?' || r == '\n' {
								if r == '\n' {
									cutoffIndex++
								}
								break
							}
							cutoffIndex++
						}
						if cutoffIndex < len(runesBefore) {
							memoryContexts[idx] = strings.TrimSpace(string(runesBefore[cutoffIndex:]))
						}
					} else {
						memoryContexts[idx] = strings.TrimSpace(textBefore)
					}

					currentPos = absolutePos + len(chunk)
				}
			}
		}

		var wg sync.WaitGroup
		var mu sync.Mutex
		var genError error
		var generatedPromptsCount int

		s.log("INFO", fmt.Sprintf("[Pipeline] Generating %d prompts via OpenRouter (%s)...", len(chunks), orModel), id, taskLabel)
		// Update initial count for already loaded prompts
		loadedCount := 0
		for _, p := range prompts {
			if p != "" {
				loadedCount++
			}
		}
		generatedPromptsCount = loadedCount

		for i, chunk := range chunks {
			// Skip if already loaded from file
			if prompts[i] != "" {
				continue
			}

			wg.Add(1)
			go func(index int, textChunk string) {
				defer wg.Done()
				var fullPrompt string
				if strings.Contains(promptTemplate, "{{content}}") {
					fullPrompt = strings.ReplaceAll(promptTemplate, "{{content}}", textChunk)
				} else {
					fullPrompt = promptTemplate + "\n\n" + textChunk
				}

				if iMode == "memory" && iMemType == "primitive" {
					contextText := memoryContexts[index]
					if contextText != "" {
						contextStr := contextText + "\n\n"
						if strings.Contains(fullPrompt, "{{memory}}") {
							fullPrompt = strings.ReplaceAll(fullPrompt, "{{memory}}", contextStr)
						} else {
							fullPrompt = contextStr + fullPrompt
						}
					} else {
						fullPrompt = strings.ReplaceAll(fullPrompt, "{{memory}}", "")
					}
				}

				// Final placeholder cleanup
				fullPrompt = strings.ReplaceAll(fullPrompt, "{{memory}}", "")
				fullPrompt = strings.ReplaceAll(fullPrompt, "{{characters}}", "")

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
					s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d/%d", generatedPromptsCount, len(chunks)))
				}
				mu.Unlock()
			}(i, chunk)
		}
		wg.Wait()

		if generatedPromptsCount == 0 && len(chunks) > 0 {
			if genError != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Failed to generate ANY image prompts: %v", genError), id, taskLabel)
				s.emitStageStatus(id, "image", "failed")
				return genError
			}
			return fmt.Errorf("no prompts were generated")
		}

		if genError != nil {
			s.log("WARN", fmt.Sprintf("[Pipeline] Some image prompts failed to generate, but continuing with %d successful ones. Last error: %v", generatedPromptsCount, genError), id, taskLabel)
		}

		// Save prompts to file
		promptsContent := strings.Join(prompts, "\n\n--------------------\n\n")
		err := os.WriteFile(promptsFilePath, []byte(promptsContent), 0644)
		if err != nil {
			s.log("WARN", fmt.Sprintf("[Pipeline] Failed to save prompts.txt: %v", err), id, taskLabel)
		} else {
			s.log("SUCCESS", fmt.Sprintf("[Pipeline] Saved generated prompts to %s", promptsFilePath), id, taskLabel)
		}
	}

	// Create images dir
	imagesDir = filepath.Join(finalDir, "images")
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
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:0/%d v:0", validPrompts, validPrompts))

		successCount := 0
		for i, prompt := range prompts {
			// Skip empty prompts
			if len(prompt) == 0 {
				continue
			}

			// save simply as 1.png, 2.png, etc
			imgName := fmt.Sprintf("%d.png", i+1)
			imgPath := filepath.Join(imagesDir, imgName)

			// Check if file already exists
			if _, err := os.Stat(imgPath); err == nil {
				s.log("INFO", fmt.Sprintf("[Pollinations] Image %s already exists, skipping generation.", imgName), id, taskLabel)
				successCount++
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:0", validPrompts, successCount, validPrompts))
				continue
			}

			s.log("INFO", fmt.Sprintf("[Pollinations] Sending request for Image %s...", imgName), id, taskLabel)
			err := s.pollinations.GenerateImage(iApiKey, prompt, iModel, int(iWidth), int(iHeight), iNoLogo, iEnhance, imgPath)
			if err != nil {
				s.log("ERROR", fmt.Sprintf("[Pollinations] Image %s failed: %v", imgName, err), id, taskLabel)
				// continue generating others instead of failing completely, but we can fail completely if desired
			} else {
				successCount++
				if s.OnImageGenerated != nil {
					s.OnImageGenerated(taskName, subName, imgName, imgPath, prompt)
				}
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:0", validPrompts, successCount, validPrompts))
				s.log("SUCCESS", fmt.Sprintf("[Pollinations] Success: Generated %s", imgName), id, taskLabel)
			}
		}

		if validPrompts > 0 && successCount == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("p:%d i:0 v:0", validPrompts))
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
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:0/%d v:0/%d", validPrompts, totalImages, totalVideos))

		successCount := 0
		imagesCount := 0
		videosCount := 0
		var imageWg sync.WaitGroup
		var imgMu sync.Mutex

		// PHASE 1: Generate all media (Images and Videos) in parallel
		validIdx := 0
		for absoluteIdx, prompt := range prompts {
			if len(prompt) == 0 {
				continue
			}
			currentValidIdx := validIdx
			validIdx++

			imageWg.Add(1)
			go func(aIdx int, vIdx int, p string) {
				defer imageWg.Done()

				isVideo := iVideoEnabled && vIdx < iVideoCount
				imgName := fmt.Sprintf("%d.png", aIdx+1)
				vidName := fmt.Sprintf("%d.mp4", aIdx+1)
				imgPath := filepath.Join(imagesDir, imgName)
				vidPath := filepath.Join(imagesDir, vidName)

				// Skip if already exists
				if isVideo {
					if _, err := os.Stat(vidPath); err == nil {
						imgMu.Lock()
						s.log("INFO", fmt.Sprintf("[Googler] [%d] Video %s already exists, skipping.", aIdx, vidName), id, taskLabel)
						successCount++
						videosCount++
						if s.OnImageGenerated != nil {
							s.OnImageGenerated(taskName, subName, vidName, vidPath, p)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:%d/%d", validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						imgMu.Unlock()
						return
					}
				} else {
					if _, err := os.Stat(imgPath); err == nil {
						imgMu.Lock()
						s.log("INFO", fmt.Sprintf("[Googler] [%d] Image %s already exists, skipping.", aIdx, imgName), id, taskLabel)
						successCount++
						imagesCount++
						if s.OnImageGenerated != nil {
							s.OnImageGenerated(taskName, subName, imgName, imgPath, p)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:%d/%d", validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						imgMu.Unlock()
						return
					}
				}

				// Case A: Text-to-Video
				if isVideo && iVideoMode == "text" {
					s.log("INFO", fmt.Sprintf("[Googler] [%d] START Text-to-Video: %s...", aIdx, vidName), id, taskLabel)
					err := s.googler.GenerateVideo(iApiKey, iVideoModel, p, "", iRatio, iVideoUpscale, vidPath)

					imgMu.Lock()
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Video %s failed: %v", aIdx, vidName, err), id, taskLabel)
					} else {
						successCount++
						videosCount++
						if s.OnImageGenerated != nil {
							s.OnImageGenerated(taskName, subName, vidName, vidPath, p)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:%d/%d", validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Video generation: %s", aIdx, vidName), id, taskLabel)
					}
					imgMu.Unlock()
					return
				}

				// Case B: Generate Image (either as final OR as base for animation)
				s.log("INFO", fmt.Sprintf("[Googler] [%d] START Image generation: %s...", aIdx, imgName), id, taskLabel)
				var err error
				if len(refImages) > 0 {
					err = s.googler.RemixImage(iApiKey, p, refImages, iRatio, iStrictMode, imgPath)
				} else {
					err = s.googler.GenerateImage(iApiKey, iModel, p, iRatio, imgPath)
				}

				if err != nil {
					s.log("ERROR", fmt.Sprintf("[Googler] [%d] Image %s failed: %v", aIdx, imgName, err), id, taskLabel)
					return // Task failed for this segment
				}

				// Success for Image
				imgMu.Lock()
				imagesCount++
				if s.OnImageGenerated != nil {
					s.OnImageGenerated(taskName, subName, imgName, imgPath, p)
				}
				// If we DON'T plan to animate it, it's a final success now
				if !(isVideo && iVideoMode == "image") {
					successCount++
				}
				s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:%d/%d", validPrompts, imagesCount, totalImages, videosCount, totalVideos))
				s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Image generation: %s", aIdx, imgName), id, taskLabel)
				imgMu.Unlock()

				// Case C: If Image-to-Video, animate IMMEDIATELY
				if isVideo && iVideoMode == "image" {
					s.log("INFO", fmt.Sprintf("[Googler] [%d] START Video animation: %s from %s...", aIdx, vidName, imgName), id, taskLabel)
					b64, err := utils.GetImageAsBase64(imgPath)
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Failed to read image for animation %s: %v", aIdx, imgName, err), id, taskLabel)
						return
					}

					err = s.googler.GenerateVideo(iApiKey, iVideoModel, p, b64, iRatio, iVideoUpscale, vidPath)

					imgMu.Lock()
					if err != nil {
						s.log("ERROR", fmt.Sprintf("[Googler] [%d] Video animation failed: %v", aIdx, err), id, taskLabel)
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
							s.OnImageGenerated(taskName, subName, vidName, vidPath, p)
						}
						s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:%d/%d", validPrompts, imagesCount, totalImages, videosCount, totalVideos))
						s.log("SUCCESS", fmt.Sprintf("[Googler] [%d] END Video animation: %s", aIdx, vidName), id, taskLabel)
					}
					imgMu.Unlock()
				}
			}(absoluteIdx, currentValidIdx, prompt)
		}
		imageWg.Wait()

		imgMu.Lock()
		finalSuccess := successCount
		finalImages := imagesCount
		finalVideos := videosCount
		imgMu.Unlock()

		if validPrompts > 0 && finalSuccess == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("p:%d i:%d v:%d", validPrompts, finalImages, finalVideos))
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
		s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:0/%d v:0", validPrompts, validPrompts))

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

				// Check if file already exists
				if _, err := os.Stat(imgPath); err == nil {
					imgMu.Lock()
					s.log("INFO", fmt.Sprintf("[ElevenLabs Image] Image %s already exists, skipping generation.", imgName), id, taskLabel)
					successCount++
					imagesCount++ // Increment imagesCount as well for existing images
					if s.OnImageGenerated != nil {
						s.OnImageGenerated(taskName, subName, imgName, imgPath, p)
					}
					s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:0", validPrompts, imagesCount, validPrompts))
					imgMu.Unlock()
					return // Use return to exit the goroutine
				}

				s.log("INFO", fmt.Sprintf("[ElevenLabs Image] Sending request for Image %s | Ratio: %s", imgName, iRatio), id, taskLabel)
				err := s.elevenLabsImage.GenerateImage(iApiKey, p, iRatio, imgPath)

				imgMu.Lock()
				if err != nil {
					s.log("ERROR", fmt.Sprintf("[ElevenLabs Image] Image %s failed: %v", imgName, err), id, taskLabel)
				} else {
					successCount++
					imagesCount++
					if s.OnImageGenerated != nil {
						s.OnImageGenerated(taskName, subName, imgName, imgPath, p)
					}
					s.emitStageStatus(id, "image", "running", fmt.Sprintf("p:%d i:%d/%d v:0", validPrompts, imagesCount, validPrompts))
					s.log("SUCCESS", fmt.Sprintf("[ElevenLabs Image] Success: Generated and saved %s", imgName), id, taskLabel)
				}
				imgMu.Unlock()
			}(i, prompt)
		}
		imageWg.Wait()

		if validPrompts > 0 && successCount == 0 {
			s.emitStageStatus(id, "image", "failed", fmt.Sprintf("p:%d i:0 v:0", validPrompts))
			return fmt.Errorf("failed to generate any images")
		}
	} else if iService != "" {
		s.log("WARN", fmt.Sprintf("[Pipeline] Image service %s is not yet implemented", iService), id, taskLabel)
	} else {
		s.log("ERROR", "[Pipeline] Image service is not selected!", id, taskLabel)
	}

	return nil
}

// RegenerateImage regenerates a single image/video for a given gallery path
func (s *PipelineService) RegenerateImage(imgPath string, prompt string, service string, settings map[string]interface{}) (string, error) {
	s.log("INFO", fmt.Sprintf("[Regenerate] Starting for: %s", filepath.Base(imgPath)), "", "Regeneration")

	// Determine directories
	imagesDir := filepath.Dir(imgPath)
	// TaskName and SubName are harder to guess, so we use placeholders or try to deduce
	// Path is usually .../Outputs/{TaskName}/{SubName}/images/{Name}.png
	parentDir := filepath.Dir(imagesDir)
	subName := filepath.Base(parentDir)
	taskDir := filepath.Dir(parentDir)
	taskName := filepath.Base(taskDir)

	// Determine target extension
	targetExt := ".png"
	if service == "googler" {
		isVid, _ := settings["imageGooglerVideoEnabled"].(bool)
		if isVid {
			targetExt = ".mp4"
		}
	}

	// Determine new path
	currentExt := filepath.Ext(imgPath)
	imgBase := strings.TrimSuffix(filepath.Base(imgPath), currentExt)
	newPath := filepath.Join(imagesDir, imgBase+targetExt)

	// If path changed (e.g. png -> mp4), remove old one
	if filepath.Clean(newPath) != filepath.Clean(imgPath) {
		os.Remove(imgPath)
		if s.OnImageDeleted != nil {
			s.OnImageDeleted(imgPath)
		}
	}

	var err error
	switch service {
	case "pollinations":
		err = s.regeneratePollinations(prompt, settings, newPath)
	case "googler":
		err = s.regenerateGoogler(imgPath, prompt, settings, newPath)
	case "elevenlabs":
		err = s.regenerateElevenLabs(prompt, settings, newPath)
	default:
		return "", fmt.Errorf("unknown service: %s", service)
	}

	if err != nil {
		s.log("ERROR", fmt.Sprintf("[Regenerate] Failed: %v", err), "", "Regeneration")
		return "", err
	}

	s.log("SUCCESS", fmt.Sprintf("[Regenerate] Completed: %s", filepath.Base(newPath)), "", "Regeneration")

	// Trigger UI update
	if s.OnImageGenerated != nil {
		s.OnImageGenerated(taskName, subName, filepath.Base(newPath), newPath, prompt)
	}

	return newPath, nil
}

func (s *PipelineService) regeneratePollinations(prompt string, settings map[string]interface{}, outPath string) error {
	iKeyID, _ := settings["imagePollinationsKeyID"].(string)
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
	iWidth, _ := settings["imageWidth"].(float64)
	if iWidth == 0 {
		iWidth = 1920
	}
	iHeight, _ := settings["imageHeight"].(float64)
	if iHeight == 0 {
		iHeight = 1080
	}
	iNoLogo, _ := settings["imageNoLogo"].(bool)
	iEnhance, _ := settings["imageEnhance"].(bool)

	s.log("INFO", fmt.Sprintf("[Pollinations] Regenerating with model: %s, size: %dx%d", iModel, int(iWidth), int(iHeight)), "", "Regeneration")
	return s.pollinations.GenerateImage(iApiKey, prompt, iModel, int(iWidth), int(iHeight), iNoLogo, iEnhance, outPath)
}

func (s *PipelineService) regenerateGoogler(imgPath string, prompt string, settings map[string]interface{}, outPath string) error {
	iApiKey := s.googler.GetAPIKey()
	iModel, _ := settings["imageGooglerModel"].(string)
	iRatio, _ := settings["imageGooglerAspectRatio"].(string)
	if iRatio == "" {
		iRatio = "IMAGE_ASPECT_RATIO_LANDSCAPE"
	}

	iVideoEnabled, _ := settings["imageGooglerVideoEnabled"].(bool)
	iVideoModel, _ := settings["imageGooglerVideoModel"].(string)
	iVideoUpscale, _ := settings["imageGooglerVideoUpscale"].(bool)
	iRefImage, _ := settings["imageGooglerReferenceImage"].(string)
	iStrictMode, _ := settings["imageGooglerRemixStrictMode"].(bool)

	// Build reference images for remix or animation
	var refImages []bapi.ReferenceImage
	if iRefImage != "" && iModel == "whisk" {
		b64, err := utils.GetImageAsBase64(iRefImage)
		if err == nil {
			refImages = append(refImages, bapi.ReferenceImage{
				Category: "MEDIA_CATEGORY_STYLE",
				Image:    b64,
			})
		}
	}

	if iVideoEnabled {
		// If video enabled, we use current image as SOURCE if it's there
		sourceB64 := ""
		if imgPath != "" && !strings.Contains(strings.ToLower(filepath.Ext(imgPath)), ".mp4") {
			b64, err := utils.GetImageAsBase64(imgPath)
			if err == nil {
				sourceB64 = b64
			}
		}

		s.log("INFO", fmt.Sprintf("[Googler] Regenerating Video. Model: %s, Ratio: %s, FromImage: %v", iVideoModel, iRatio, sourceB64 != ""), "", "Regeneration")
		return s.googler.GenerateVideo(iApiKey, iVideoModel, prompt, sourceB64, iRatio, iVideoUpscale, outPath)
	} else {
		if len(refImages) > 0 {
			s.log("INFO", fmt.Sprintf("[Googler] Regenerating Image (Remix). Model: %s, Ratio: %s", iModel, iRatio), "", "Regeneration")
			return s.googler.RemixImage(iApiKey, prompt, refImages, iRatio, iStrictMode, outPath)
		} else {
			s.log("INFO", fmt.Sprintf("[Googler] Regenerating Image. Model: %s, Ratio: %s", iModel, iRatio), "", "Regeneration")
			return s.googler.GenerateImage(iApiKey, iModel, prompt, iRatio, outPath)
		}
	}
}

func (s *PipelineService) regenerateElevenLabs(prompt string, settings map[string]interface{}, outPath string) error {
	iApiKey := s.elevenLabsImage.GetAPIKey() // Use default key if not provided in settings
	iRatio, _ := settings["elevenLabsImageAspectRatio"].(string)
	if iRatio == "" {
		iRatio = "landscape"
	}

	s.log("INFO", fmt.Sprintf("[ElevenLabs] Regenerating Image. Ratio: %s", iRatio), "", "Regeneration")
	return s.elevenLabsImage.GenerateImage(iApiKey, prompt, iRatio, outPath)
}
