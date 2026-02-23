package pipeline

import (
	"fmt"
	"soloveyko/backend/utils"
	"strings"
)

// ProcessText handles translation or rewriting using OpenRouter
func (s *PipelineService) ProcessText(id string, taskLabel string, taskType string, content string, settings map[string]interface{}, pSettings *utils.PipelineSettings) (string, bool, error) {
	var apiKey string
	keyID, _ := settings[taskType+"OpenRouterKeyID"].(string)

	shouldProcessText := false
	switch taskType {
	case "translate":
		enabled, ok := settings["translateEnabled"].(bool)
		if (ok && enabled) || (!ok && pSettings.TranslateEnabled) {
			shouldProcessText = true
		}
	case "rewrite":
		enabled, ok := settings["rewriteEnabled"].(bool)
		if (ok && enabled) || (!ok && pSettings.RewriteEnabled) {
			shouldProcessText = true
		}
	}

	if !shouldProcessText {
		s.log("INFO", "[OpenRouter] Text processing disabled, using original content", id, taskLabel)
		s.emitStageStatus(id, "text", "completed")
		return content, false, nil
	}

	// Handle OpenRouter Keys
	keys := s.settings.GetOpenRouterKeys()
	for _, k := range keys {
		if k.ID == keyID {
			apiKey = k.Key
			break
		}
	}
	if apiKey == "" && len(keys) > 0 {
		apiKey = keys[0].Key
	}

	if apiKey == "" {
		s.log("WARN", fmt.Sprintf("[OpenRouter] [%s] API key not found, skipping text processing", strings.Title(taskType)), id, taskLabel)
		s.emitStageStatus(id, "text", "completed")
		return content, false, nil
	}

	s.emitStageStatus(id, "text", "running")
	model, _ := settings[taskType+"Model"].(string)
	prompt, _ := settings[taskType+"Prompt"].(string)
	temp, _ := settings[taskType+"Temperature"].(float64)
	tokens, _ := settings[taskType+"MaxTokens"].(float64)

	keyName := "Default/First"
	for _, k := range keys {
		if k.ID == keyID {
			keyName = k.Name
			break
		}
	}

	var fullPrompt string
	if strings.Contains(prompt, "{{content}}") {
		fullPrompt = strings.ReplaceAll(prompt, "{{content}}", content)
	} else {
		fullPrompt = prompt + "\n\n" + content
	}

	result, err := s.openRouter.Chat(id, taskLabel, taskType, keyName, apiKey, model, fullPrompt, temp, int(tokens))
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[OpenRouter] [%s] Error: %v", strings.Title(taskType), err), id, taskLabel)
		s.emitStageStatus(id, "text", "failed")
		return "", false, err
	}

	s.log("SUCCESS", fmt.Sprintf("[OpenRouter] [%s] Success: Result received", strings.Title(taskType)), id, taskLabel)
	s.emitStageStatus(id, "text", "completed")
	return result, true, nil
}
