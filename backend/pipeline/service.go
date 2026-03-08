package pipeline

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"soloveyko/backend/api"
	"soloveyko/backend/utils"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// GlobalWhisperMutex ensures that only one Whisper (local, amd, or whisperx) process runs at a time globally.
var GlobalWhisperMutex sync.Mutex

// ControlAction represents an action from the UI control editor
type ControlAction struct {
	Action   string                 `json:"action"`   // "confirm", "regenerate", "cancel_queue"
	Text     string                 `json:"text"`     // Updated text
	Settings map[string]interface{} `json:"settings"` // Updated settings for regeneration
}

// PipelineService handles the execution of multi-stage tasks
type PipelineService struct {
	ctx             context.Context
	settings        *utils.SettingsService
	openRouter      *api.OpenRouterService
	elevenLabs      *api.ElevenLabsBotService
	elevenLabsUnlim *api.ElevenLabsUnlimService
	elevenLabsUA    *api.ElevenLabsUAService
	voiceMaker      *api.VoiceMakerService
	pollinations    *api.PollinationsService
	googler         *api.GooglerService
	elevenLabsImage *api.ElevenLabsImageService
	localWhisper    *LocalWhisperService
	amdWhisper      *AmdWhisperService
	edgeTTS         *api.EdgeTTSService
	assemblyAI      *api.AssemblyAIService

	// Callbacks for UI updates
	OnLog                       func(level string, message string, details ...string)
	OnStageStatus               func(id string, stage string, status string, message string)
	OnTextResult                func(id string, resultText string)
	OnRequestControl            func(id string, text string)
	OnRequestImageControl       func(id string)
	OnTaskStatus                func(id string, status string, progress int)
	OnImageGenerated            func(taskName string, templateName string, imageName string, path string, prompt string)
	OnImageDeleted              func(imgPath string)
	OnRequestExistingFilesCheck func(data ExistingFilesData)
	OnPipelineSuccess           func(id string, taskName string, taskType string, subName string, original string, processed string, settings map[string]interface{}, duration float64)

	pendingControl sync.Map // Map taskID -> chan string
	pendingSkip    sync.Map // Map taskID -> chan []string

	elevenLabsSem      chan struct{}
	elevenLabsUnlimSem chan struct{}
	elevenLabsUASem    chan struct{}
	subtitleSem        chan struct{}
	subtitleSemSize    int
	subtitleSemMu      sync.Mutex

	montageSem     chan struct{}
	montageSemSize int
	montageSemMu   sync.Mutex

	edgeTTSSem chan struct{}
	cancelled  atomic.Bool
}

// NewPipelineService creates a new PipelineService
func NewPipelineService(
	settings *utils.SettingsService,
	openRouter *api.OpenRouterService,
	elevenLabs *api.ElevenLabsBotService,
	elevenLabsUnlim *api.ElevenLabsUnlimService,
	elevenLabsUA *api.ElevenLabsUAService,
	voiceMaker *api.VoiceMakerService,
	pollinations *api.PollinationsService,
	googler *api.GooglerService,
	elevenLabsImage *api.ElevenLabsImageService,
	localWhisper *LocalWhisperService,
	amdWhisper *AmdWhisperService,
	edgeTTS *api.EdgeTTSService,
	assemblyAI *api.AssemblyAIService,
) *PipelineService {
	return &PipelineService{
		settings:           settings,
		openRouter:         openRouter,
		elevenLabs:         elevenLabs,
		elevenLabsUnlim:    elevenLabsUnlim,
		elevenLabsUA:       elevenLabsUA,
		voiceMaker:         voiceMaker,
		pollinations:       pollinations,
		googler:            googler,
		elevenLabsImage:    elevenLabsImage,
		localWhisper:       localWhisper,
		amdWhisper:         amdWhisper,
		edgeTTS:            edgeTTS,
		assemblyAI:         assemblyAI,
		elevenLabsSem:      make(chan struct{}, 5),
		elevenLabsUnlimSem: make(chan struct{}, 5),
		elevenLabsUASem:    make(chan struct{}, 5),
		subtitleSemSize:    settings.GetSubtitleMaxConnections(),
		subtitleSem:        make(chan struct{}, settings.GetSubtitleMaxConnections()),
		montageSemSize:     settings.GetMontageMaxConnections(),
		montageSem:         make(chan struct{}, settings.GetMontageMaxConnections()),
		edgeTTSSem:         make(chan struct{}, 5),
	}
}

func (s *PipelineService) SetContext(ctx context.Context) {
	s.ctx = ctx
	if s.localWhisper != nil {
		s.localWhisper.SetContext(ctx)
	}
	if s.amdWhisper != nil {
		s.amdWhisper.SetContext(ctx)
	}
}

// ProcessTask handles the execution of a single pipeline task
func (s *PipelineService) ProcessTask(id string, taskNumber int, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	settings = s.flattenSettings(settings)
	displayTaskName := taskName
	if len([]rune(displayTaskName)) > 10 {
		displayTaskName = string([]rune(displayTaskName)[:10]) + "..."
	}

	label := displayTaskName
	if subName != "" {
		label = fmt.Sprintf("%s - %s", displayTaskName, subName)
	}
	taskLabel := fmt.Sprintf("%s (#%d)", label, taskNumber)

	return s.runPipeline(id, taskLabel, taskType, content, settings, taskName, subName)
}

func (s *PipelineService) log(level string, message string, details ...string) {
	if s.OnLog != nil {
		s.OnLog(level, message, details...)
	}
}

func (s *PipelineService) emitStageStatus(id string, stage string, status string, message ...string) {
	msg := ""
	if len(message) > 0 {
		msg = message[0]
	}
	if s.OnStageStatus != nil {
		s.OnStageStatus(id, stage, status, msg)
	}
}

func (s *PipelineService) CancelProcessing() {
	s.cancelled.Store(true)
}

func (s *PipelineService) ResetCancellation() {
	s.cancelled.Store(false)
}

// LoadChatHistory loads chat history from a file in the task directory
func (s *PipelineService) LoadChatHistory(finalDir string) ([]api.ChatMessage, error) {
	historyPath := filepath.Join(finalDir, "chat_history.json")
	if _, err := os.Stat(historyPath); os.IsNotExist(err) {
		return []api.ChatMessage{}, nil
	}

	data, err := os.ReadFile(historyPath)
	if err != nil {
		return nil, err
	}

	var history []api.ChatMessage
	if err := json.Unmarshal(data, &history); err != nil {
		return nil, err
	}

	return history, nil
}

// SaveChatHistory saves chat history to a file in the task directory
func (s *PipelineService) SaveChatHistory(finalDir string, history []api.ChatMessage) error {
	historyPath := filepath.Join(finalDir, "chat_history.json")
	data, err := json.MarshalIndent(history, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(historyPath, data, 0644)
}

func (s *PipelineService) runPipeline(id string, taskLabel string, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	if s.cancelled.Load() {
		return "", fmt.Errorf("queue execution cancelled")
	}
	startTime := time.Now()
	s.log("INFO", fmt.Sprintf("[Pipeline] runPipeline started. Type: %s, ID: %s", taskType, id), id, taskLabel)
	s.log("INFO", "[Pipeline] Task started and pre-processing...", id, taskLabel)
	pSettings := s.settings.GetPipelineSettings()

	if taskType != "translate" && taskType != "rewrite" && taskType != "voiceover" {
		return "", fmt.Errorf("task type %s not implemented", taskType)
	}

	finalDir := s.ResolveFinalDir(taskName, taskType, subName, settings)
	s.log("INFO", fmt.Sprintf("[Pipeline] Final directory resolved: %s", finalDir), id, taskLabel)
	templateDir := subName
	if templateDir == "" {
		pipelineName, _ := settings[taskType+"PipelineName"].(string)
		templateDir = pipelineName
		if templateDir == "" {
			templateDir = "Default"
		}
	}
	var skippedStages []string
	hasSkippedInfo := false

	// Sync template-specific settings to our local pSettings copy
	// We MUST be strict here: if a template is used (subName != ""), we prioritize its settings.
	// If the template doesn't have these keys, we should probably default to disabled/empty
	// for THIS task to avoid leakage from global settings.
	if val, ok := settings["customStagesEnabled"].(bool); ok {
		pSettings.CustomStagesEnabled = val
	} else if subName != "" {
		// If template is used but flag is missing, default to false for safety
		pSettings.CustomStagesEnabled = false
	}

	if val, ok := settings["customStages"]; ok {
		if slice, ok := val.([]interface{}); ok {
			var stages []utils.CustomStage
			for _, v := range slice {
				if m, ok := v.(map[string]interface{}); ok {
					var stage utils.CustomStage
					jsonData, _ := json.Marshal(m)
					json.Unmarshal(jsonData, &stage)
					if stage.ID != "" {
						stages = append(stages, stage)
					}
				}
			}
			pSettings.CustomStages = stages
		}
	} else if subName != "" {
		// If template is used but stages are missing, clear them to avoid global inheritance
		pSettings.CustomStages = []utils.CustomStage{}
	}

	if val, ok := settings["skippedStages"]; ok {
		hasSkippedInfo = true
		if slice, ok := val.([]interface{}); ok {
			for _, v := range slice {
				if str, ok := v.(string); ok {
					skippedStages = append(skippedStages, str)
				}
			}
		}
	}

	isFromModal := hasSkippedInfo
	if !hasSkippedInfo {
		if _, err := os.Stat(finalDir); err == nil {
			data := s.CheckExistingFiles(id, finalDir, taskType, settings)
			if len(data.FoundStages) > 0 {
				resChan := make(chan []string)
				s.pendingSkip.Store(id, resChan)

				if s.OnRequestExistingFilesCheck != nil {
					s.log("INFO", "[Pipeline] Requesting user confirmation for existing files...", id, taskLabel)
					s.OnRequestExistingFilesCheck(data)
					isFromModal = true
				}

				// Block until result received
				select {
				case skipped := <-resChan:
					skippedStages = skipped
				case <-s.ctx.Done():
					return "", fmt.Errorf("task cancelled")
				}
				s.pendingSkip.Delete(id)
			}
		}
	} else {
		s.log("INFO", fmt.Sprintf("[Pipeline] Using pre-defined skipped stages: %v", skippedStages), id, taskLabel)
	}

	if isFromModal {
		// If user was prompted, we respect their choice.
		// If a stage is NOT in skippedStages, we should FORCE regeneration if it's already there
		// to avoid the "auto-skip if file exists" behavior in sub-stages.
		stagesToProcess := []string{"voice", "subtitle", "image"}
		for _, stage := range stagesToProcess {
			isSkipped := false
			for _, st := range skippedStages {
				if st == stage {
					isSkipped = true
					break
				}
			}
			if !isSkipped {
				switch stage {
				case "voice":
					settings["voiceoverRegenerate"] = true
				case "subtitle":
					settings["subtitleRegenerate"] = true
				case "image":
					settings["imageRegeneratePrompts"] = true
					settings["imageGooglerRegenerateImages"] = true
				}
			}
		}
	}

	settings["skippedStages"] = skippedStages

	if s.OnTaskStatus != nil {
		s.OnTaskStatus(id, "running", 5)
	}

	shouldSkipText := false
	for _, st := range skippedStages {
		if st == "text" {
			shouldSkipText = true
			break
		}
	}

	// 1. Text Stage Loop
	processedText := ""
	orSuccess := false
	var err error

	// 2. FS Stage - Prepare Directory (Already determined as finalDir)
	if _, err := os.Stat(finalDir); os.IsNotExist(err) {
		err = os.MkdirAll(finalDir, 0755)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[FileSystem] Failed to create directory: %v", err), id, taskLabel)
			return "", err
		}
	}

	for {
		if s.cancelled.Load() {
			return "", fmt.Errorf("queue execution cancelled")
		}

		if shouldSkipText {
			s.log("INFO", "[Pipeline] Skipping text stage, loading existing result...", id, taskLabel)
			processedText, err = s.LoadTextResult(finalDir, taskType)
			if err != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Failed to load existing text: %v. Running generation anyway.", err), id, taskLabel)
				processedText, orSuccess, err = s.ProcessText(id, taskLabel, taskType, content, finalDir, settings, &pSettings)
			} else {
				s.emitStageStatus(id, "text", "completed")
			}
		} else {
			processedText, orSuccess, err = s.ProcessText(id, taskLabel, taskType, content, finalDir, settings, &pSettings)
		}

		if err != nil {
			s.emitStageStatus(id, "text", "failed")
			if s.OnTaskStatus != nil {
				s.OnTaskStatus(id, "failed", 0)
			}
			return "", err
		}

		// Emit text result immediately after processing
		if s.OnTextResult != nil {
			s.OnTextResult(id, processedText)
		}

		// 1.5 Control Stage (Only if not skipped)
		tControlEnabled := pSettings.TranslateControlEnabled
		if val, ok := settings["translateControlEnabled"].(bool); ok {
			tControlEnabled = val
		}

		if val, ok := settings["imageSyncEnabled"].(bool); ok {
			pSettings.ImageSyncEnabled = val
		}

		if !shouldSkipText && tControlEnabled && (taskType == "translate" || taskType == "rewrite") && orSuccess {
			s.emitStageStatus(id, "text", "waiting")
			s.log("INFO", "[Control] Waiting for user translation review...", id, taskLabel)

			resChan := make(chan *ControlAction)
			s.pendingControl.Store(id, resChan)

			if s.OnRequestControl != nil {
				s.OnRequestControl(id, processedText)
			}

			// Block until result received
			var action *ControlAction
			select {
			case action = <-resChan:
				// received action
			case <-s.ctx.Done():
				s.pendingControl.Delete(id)
				return "", fmt.Errorf("task cancelled")
			}
			s.pendingControl.Delete(id)

			if action.Action == "cancel_queue" {
				s.log("WARN", "[Control] Queue cancellation requested by user.", id, taskLabel)
				s.CancelProcessing()
				return "", fmt.Errorf("queue execution cancelled")
			}

			if action.Action == "cancel" {
				s.log("WARN", "[Control] Task cancellation requested by user.", id, taskLabel)
				return "", fmt.Errorf("task execution cancelled")
			}

			if action.Action == "regenerate" {
				s.log("INFO", "[Control] Regeneration requested by user.", id, taskLabel)
				if action.Settings != nil {
					// Update local settings for the next iteration
					for k, v := range action.Settings {
						settings[k] = v
					}
					// Also update pSettings if necessary (some fields are mirrored)
					if val, ok := action.Settings["translateControlEnabled"].(bool); ok {
						pSettings.TranslateControlEnabled = val
					}
				}
				// Loop back to regenerate
				continue
			}

			// Default: "confirm"
			processedText = action.Text
			s.log("SUCCESS", "[Control] Text approved.", id, taskLabel)
			s.emitStageStatus(id, "text", "completed")
			// Re-emit result length if it changed
			if s.OnTextResult != nil {
				s.OnTextResult(id, processedText)
			}
		}

		// If we reached here, it means we don't need to regenerate
		break
	}

	// Save Text Result
	shouldProcessText := false // Simple check for saving
	if taskType == "translate" || taskType == "rewrite" {
		shouldProcessText = true
	}

	if (orSuccess || shouldProcessText || taskType != "voiceover") && !shouldSkipText {
		s.SaveTextResult(finalDir, taskType, processedText)
	}

	// 1.6 Custom Stages (Background)
	go s.ProcessCustomStages(id, taskLabel, taskType, taskName, content, processedText, finalDir, settings, &pSettings)

	shouldSkipVoice := false
	shouldSkipSubtitle := false
	shouldSkipImage := false
	for _, st := range skippedStages {
		switch st {
		case "voice":
			shouldSkipVoice = true
		case "subtitle":
			shouldSkipSubtitle = true
		case "image":
			shouldSkipImage = true
		}
	}

	// 3 & 4. Voiceover and Image Generation Stages logic in parallel
	var stagesWg sync.WaitGroup
	var voiceErr error
	var imageErr error
	var subtitleErr error

	stagesWg.Add(1)
	go func() {
		defer stagesWg.Done()
		if shouldSkipVoice {
			s.log("INFO", "[Pipeline] Skipping voiceover stage, using existing file.", id, taskLabel)
			dur, _ := utils.GetAudioDuration(filepath.Join(finalDir, "voice.mp3"))
			s.emitStageStatus(id, "voice", "completed", dur)
		} else {
			voiceErr = s.ProcessVoiceover(id, taskLabel, processedText, finalDir, settings, &pSettings)
		}

		if voiceErr != nil {
			s.log("ERROR", fmt.Sprintf("[Pipeline] Voiceover stage failed: %v", voiceErr), id, taskLabel)
		} else if taskType == "voiceover" || taskType == "translate" || taskType == "rewrite" {
			// Після успішного створення озвучки запускаємо створення субтитрів
			if shouldSkipSubtitle {
				s.log("INFO", "[Pipeline] Skipping subtitle stage, using existing files.", id, taskLabel)
				s.emitStageStatus(id, "subtitle", "completed")
			} else {
				subtitleErr = s.ProcessSubtitle(id, taskLabel, finalDir, settings, &pSettings)
			}

			if subtitleErr != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Subtitle stage failed: %v", subtitleErr), id, taskLabel)
			}
		}
	}()

	stagesWg.Add(1)
	go func() {
		defer stagesWg.Done()
		imageErr = s.ProcessImage(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings, taskName, templateDir)
		if imageErr != nil {
			s.log("ERROR", fmt.Sprintf("[Pipeline] Image stage failed: %v", imageErr), id, taskLabel)
			return
		}

		// Image Control
		iControlEnabled := pSettings.ImageControlEnabled
		if val, ok := settings["imageControlEnabled"].(bool); ok {
			iControlEnabled = val
		}

		if iControlEnabled && !shouldSkipImage {
			s.emitStageStatus(id, "image", "waiting")
			s.log("INFO", "[Control] Waiting for user image/video review...", id, taskLabel)

			resChan := make(chan string)
			s.pendingControl.Store(id+"_image", resChan)

			if s.OnRequestImageControl != nil {
				s.OnRequestImageControl(id)
			}

			// Block goroutine until result received or timeout/context cancel
			select {
			case <-resChan:
				s.log("SUCCESS", "[Control] Media approved.", id, taskLabel)
				s.emitStageStatus(id, "image", "completed")
			case <-s.ctx.Done():
				s.log("INFO", "[Control] Task cancelled while waiting for image review", id, taskLabel)
				return
			}
			s.pendingControl.Delete(id + "_image")
		} else {
			s.emitStageStatus(id, "image", "completed")
		}
	}()

	stagesWg.Wait()

	if voiceErr != nil || imageErr != nil || subtitleErr != nil {
		if s.OnTaskStatus != nil {
			s.OnTaskStatus(id, "failed", 0)
		}
		if voiceErr != nil {
			return processedText, voiceErr
		}
		if imageErr != nil {
			return processedText, imageErr
		}
		return processedText, subtitleErr
	}

	// 5. Montage Stage
	montageErr := s.ProcessMontage(id, taskLabel, finalDir, settings, &pSettings, taskName, subName)
	if montageErr != nil {
		s.log("ERROR", fmt.Sprintf("[Pipeline] Montage stage failed: %v", montageErr), id, taskLabel)
		s.emitStageStatus(id, "montage", "failed")
		if s.OnTaskStatus != nil {
			s.OnTaskStatus(id, "failed", 0)
		}
		return processedText, montageErr
	}

	if s.OnPipelineSuccess != nil {
		duration := time.Since(startTime).Seconds()
		s.OnPipelineSuccess(id, taskName, taskType, subName, content, processedText, settings, duration)
	}

	if s.OnTaskStatus != nil {
		s.OnTaskStatus(id, "completed", 100)
	}

	return processedText, nil
}

func (s *PipelineService) ProcessCustomStages(id string, taskLabel string, taskType string, taskName string, originalText string, processedText string, finalDir string, settings map[string]interface{}, pSettings *utils.PipelineSettings) {
	if !pSettings.CustomStagesEnabled || len(pSettings.CustomStages) == 0 {
		return
	}

	stages := pSettings.CustomStages

	s.log("INFO", fmt.Sprintf("[Custom] Processing %d custom stages...", len(stages)), id, taskLabel)

	// Get API Key for Custom Stages (using task-specific key)
	var apiKey string
	keyField := "translateOpenRouterKeyID"
	if taskType == "rewrite" {
		keyField = "rewriteOpenRouterKeyID"
	}
	keyID, _ := settings[keyField].(string)

	keys := s.settings.GetOpenRouterKeys()
	keyName := "Default"
	for _, k := range keys {
		if k.ID == keyID {
			apiKey = k.Key
			keyName = k.Name
			break
		}
	}
	if apiKey == "" && len(keys) > 0 {
		apiKey = keys[0].Key
		keyName = keys[0].Name
	}

	if apiKey == "" {
		s.log("WARN", "[Custom] API key not found, skipping custom stages", id, taskLabel)
		return
	}

	defaultModel, _ := settings["translateModel"].(string)
	if taskType == "rewrite" {
		defaultModel, _ = settings["rewriteModel"].(string)
	}
	defaultTemp, _ := settings["translateTemperature"].(float64)
	if taskType == "rewrite" {
		defaultTemp, _ = settings["rewriteTemperature"].(float64)
	}

	for _, stage := range stages {
		if !stage.Enabled {
			continue
		}

		// Calculate safe name early for file existence check
		safeName := utils.SanitizeFilename(stage.Name)
		if safeName == "" {
			safeName = "custom_stage_" + stage.ID
		}

		// Check if file already exists
		savePath := filepath.Join(finalDir, safeName+".txt")
		if _, err := os.Stat(savePath); err == nil {
			s.log("INFO", fmt.Sprintf("[Custom] Stage %s already exists (%s), skipping generation.", stage.Name, safeName+".txt"), id, taskLabel)
			continue
		}

		s.log("INFO", fmt.Sprintf("[Custom] Running stage: %s", stage.Name), id, taskLabel)

		var sourceContent string
		switch stage.DataSource {
		case "taskName":
			sourceContent = taskName
		default:
			sourceContent = processedText
		}

		var fullPrompt string
		if strings.Contains(stage.Prompt, "{{content}}") {
			fullPrompt = strings.ReplaceAll(stage.Prompt, "{{content}}", sourceContent)
		} else {
			fullPrompt = stage.Prompt + "\n\n" + sourceContent
		}

		// Use per-stage settings if available
		useModel := stage.Model
		if useModel == "" {
			useModel = defaultModel
		}
		useTemp := stage.Temperature
		if useTemp == 0 {
			useTemp = defaultTemp
		}
		useMaxTokens := stage.MaxTokens

		result, err := s.openRouter.Chat(id, taskLabel, "custom", keyName, apiKey, useModel, fullPrompt, useTemp, useMaxTokens)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Custom] Stage %s failed: %v", stage.Name, err), id, taskLabel)
			continue
		}

		// Save result to file
		err = os.WriteFile(savePath, []byte(result), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Custom] Failed to save result for %s: %v", stage.Name, err), id, taskLabel)
		} else {
			s.log("SUCCESS", fmt.Sprintf("[Custom] Stage %s completed, saved to %s", stage.Name, safeName+".txt"), id, taskLabel)
		}
	}
}

func (s *PipelineService) SubmitImageControlResult(id string) {
	if val, ok := s.pendingControl.Load(id + "_image"); ok {
		ch := val.(chan string)
		ch <- "done"
	}
}

func (s *PipelineService) flattenSettings(m map[string]interface{}) map[string]interface{} {
	res := make(map[string]interface{})

	// Спеціальний мапінг для блоків, де імена в JSON не збігаються з іменами в PipelineSettings
	// або де є конфлікти імен (наприклад 'image')
	if stages, ok := m["stages"].(map[string]interface{}); ok {
		if val, ok := stages["image"].(bool); ok {
			res["imageEnabled"] = val
		}
		if val, ok := stages["voiceover"].(bool); ok {
			res["voiceoverEnabled"] = val
		}
		if val, ok := stages["subtitle"].(bool); ok {
			res["subtitleEnabled"] = val
		}
		if val, ok := stages["translate"].(bool); ok {
			res["translateEnabled"] = val
		}
		if val, ok := stages["montage"].(bool); ok {
			res["montageEnabled"] = val
		}
		if val, ok := stages["rewrite"].(bool); ok {
			res["rewriteEnabled"] = val
		}
	}

	if control, ok := m["control"].(map[string]interface{}); ok {
		if val, ok := control["image"].(bool); ok {
			res["imageControlEnabled"] = val
		}
		if val, ok := control["translate"].(bool); ok {
			res["translateControlEnabled"] = val
		}
	}

	// Рекурсивна функція для розгортання всіх інших налаштувань
	var flatten func(map[string]interface{})
	flatten = func(current map[string]interface{}) {
		for k, v := range current {
			// Пропускаємо блоки, які ми вже опрацювали спеціальним чином
			if k == "stages" || k == "control" {
				continue
			}

			if sub, ok := v.(map[string]interface{}); ok {
				// Якщо це блок 'services', ми заходимо в нього глибше
				// Якщо це будь-який інший блок (наприклад 'image', 'voiceover'),
				// ми розгортаємо його вміст у корінь, але також продовжуємо рекурсію
				flatten(sub)
			} else {
				// Звичайне значення - копіюємо в результуючу мапу
				// (Тут пізніші значення можуть перезаписувати ранні, якщо імена однакові)
				res[k] = v
			}
		}
	}

	flatten(m)
	return res
}

// SubmitControlResult resumes a paused pipeline with updated text (legacy/simple confirm)
func (s *PipelineService) SubmitControlResult(id string, text string) {
	s.SubmitControlAction(id, &ControlAction{
		Action: "confirm",
		Text:   text,
	})
}

// SubmitControlAction sends a complex control response to the pipeline
func (s *PipelineService) SubmitControlAction(id string, action *ControlAction) {
	if val, ok := s.pendingControl.Load(id); ok {
		ch := val.(chan *ControlAction)
		ch <- action
	}
}

func (s *PipelineService) UpdateSubtitleSemaphore(newSize int) {
	s.subtitleSemMu.Lock()
	defer s.subtitleSemMu.Unlock()

	if newSize == s.subtitleSemSize {
		return
	}

	// Just re-create the channel. Old tasks might still be using the old one,
	// but the new tasks will respect the new limit.
	// This is a simple but effective way to handle it for this app.
	s.subtitleSem = make(chan struct{}, newSize)
	s.subtitleSemSize = newSize
	s.log("INFO", fmt.Sprintf("[Pipeline] Subtitle semaphore updated to %d slots", newSize))
}

func (s *PipelineService) getSubtitleSem() chan struct{} {
	s.subtitleSemMu.Lock()
	defer s.subtitleSemMu.Unlock()
	return s.subtitleSem
}

func (s *PipelineService) UpdateMontageSemaphore(newSize int) {
	s.montageSemMu.Lock()
	defer s.montageSemMu.Unlock()

	if newSize == s.montageSemSize {
		return
	}

	s.montageSem = make(chan struct{}, newSize)
	s.montageSemSize = newSize
	s.log("INFO", fmt.Sprintf("[Pipeline] Montage semaphore updated to %d slots", newSize))
}

func (s *PipelineService) getMontageSem() chan struct{} {
	s.montageSemMu.Lock()
	defer s.montageSemMu.Unlock()
	return s.montageSem
}

type ExistingFilesData struct {
	ID            string   `json:"id"`
	FoundStages   []string `json:"foundStages"`
	ImageCount    int      `json:"imageCount"`
	VideoCount    int      `json:"videoCount"`
	PromptCount   int      `json:"promptCount"`
	TextChars     int      `json:"textChars"`
	VoiceDuration string   `json:"voiceDuration"`
	CustomCount   int      `json:"customCount"`
}

func (s *PipelineService) CheckExistingFiles(id string, finalDir string, taskType string, settings map[string]interface{}) ExistingFilesData {
	data := ExistingFilesData{
		ID:          id,
		FoundStages: []string{},
	}

	// 1. Check Text (lenient check for any result file)
	textFiles := []string{"result.txt", "translation.txt", "rewrite.txt"}
	var textPath string
	for _, f := range textFiles {
		p := filepath.Join(finalDir, f)
		if info, err := os.Stat(p); err == nil && !info.IsDir() {
			textPath = p
			break
		}
	}

	if textPath != "" {
		data.FoundStages = append(data.FoundStages, "text")
		content, _ := os.ReadFile(textPath)
		data.TextChars = len([]rune(string(content)))
	}

	// 2. Check Voice
	voicePath := filepath.Join(finalDir, "voice.mp3")
	if info, err := os.Stat(voicePath); err == nil && !info.IsDir() {
		data.FoundStages = append(data.FoundStages, "voice")
		dur, err := utils.GetAudioDuration(voicePath)
		if err == nil {
			data.VoiceDuration = dur
		}
	}

	// 3. Check Subtitles
	subtitleSrt := filepath.Join(finalDir, "subtitle.srt")
	subtitleAss := filepath.Join(finalDir, "subtitle.ass")
	if _, errSrt := os.Stat(subtitleSrt); errSrt == nil {
		data.FoundStages = append(data.FoundStages, "subtitle")
	} else if _, errAss := os.Stat(subtitleAss); errAss == nil {
		data.FoundStages = append(data.FoundStages, "subtitle")
	}

	// 4. Check Images (both in images/ subfolder and directly in finalDir as fallback)
	promptsPath := filepath.Join(finalDir, "prompts.txt")
	if info, err := os.Stat(promptsPath); err == nil && !info.IsDir() {
		content, _ := os.ReadFile(promptsPath)
		pStrs := strings.Split(string(content), "\n\n--------------------\n\n")
		data.PromptCount = len(pStrs)
	}

	scanDir := func(dir string) {
		if info, err := os.Stat(dir); err == nil && info.IsDir() {
			files, _ := os.ReadDir(dir)
			for _, f := range files {
				if f.IsDir() {
					continue
				}
				ext := strings.ToLower(filepath.Ext(f.Name()))
				switch ext {
				case ".png", ".jpg", ".jpeg", ".webp":
					data.ImageCount++
				case ".mp4", ".mov", ".avi", ".mkv", ".webm":
					data.VideoCount++
				}
			}
		}
	}

	scanDir(filepath.Join(finalDir, "images"))
	if data.ImageCount == 0 && data.VideoCount == 0 {
		scanDir(finalDir)
	}

	if data.ImageCount > 0 || data.VideoCount > 0 {
		data.FoundStages = append(data.FoundStages, "image")
	}

	// 5. Check Custom Stages
	if settings != nil {
		if val, ok := settings["customStages"]; ok {
			var stages []utils.CustomStage
			if slice, ok := val.([]interface{}); ok {
				for _, v := range slice {
					if m, ok := v.(map[string]interface{}); ok {
						var cs utils.CustomStage
						jsonData, _ := json.Marshal(m)
						json.Unmarshal(jsonData, &cs)
						stages = append(stages, cs)
					}
				}
			}

			foundCustom := 0
			for _, cs := range stages {
				if !cs.Enabled {
					continue
				}
				safeName := utils.SanitizeFilename(cs.Name)
				if safeName == "" {
					safeName = "custom_stage_" + cs.ID
				}
				p := filepath.Join(finalDir, safeName+".txt")
				if info, err := os.Stat(p); err == nil && !info.IsDir() {
					foundCustom++
				}
			}
			if foundCustom > 0 {
				data.CustomCount = foundCustom
				data.FoundStages = append(data.FoundStages, "custom")
			}
		}
	}

	return data
}

func (s *PipelineService) SubmitExistingFilesResult(id string, skipStages []string) {
	if val, ok := s.pendingSkip.Load(id); ok {
		ch := val.(chan []string)
		ch <- skipStages
	}
}

func (s *PipelineService) ResolveFinalDir(taskName string, taskType string, subName string, settings map[string]interface{}) string {
	s.log("INFO", fmt.Sprintf("[Resolve] Resolving directory for %s, Type: %s, Sub: %s", taskName, taskType, subName))
	pSettings := s.settings.GetPipelineSettings()
	outPath, _ := settings[taskType+"OutputPath"].(string)
	if outPath == "" {
		if taskType == "rewrite" {
			outPath = pSettings.RewriteOutputPath
		} else {
			outPath = pSettings.TranslateOutputPath
		}
	}
	if outPath == "" {
		outPath = pSettings.OutputPath
	}

	if outPath == "" {
		home, _ := os.UserHomeDir()
		outPath = filepath.Join(home, "Videos")
	}

	templateDir := subName
	if templateDir == "" {
		pipelineName, _ := settings[taskType+"PipelineName"].(string)
		templateDir = pipelineName
		if templateDir == "" {
			templateDir = "Default"
		}
	}

	safeTaskName := utils.SanitizeFilename(taskName)
	safeTemplateDir := utils.SanitizeFilename(templateDir)
	finalDir := filepath.Join(outPath, safeTaskName, safeTemplateDir)

	// Backward compatibility check: if Default dir doesn't exist OR is empty, check parent
	if templateDir == "Default" {
		dataPrimary := s.CheckExistingFiles("tmp", finalDir, taskType, settings)
		if len(dataPrimary.FoundStages) == 0 {
			parentDir := filepath.Join(outPath, safeTaskName)
			dataParent := s.CheckExistingFiles("tmp", parentDir, taskType, settings)
			if len(dataParent.FoundStages) > 0 {
				return parentDir
			}
		}
	}

	return finalDir
}
