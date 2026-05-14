package pipeline

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"soloveyko/backend/api"
	"soloveyko/backend/utils"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

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
	OnRequestImageControl       func(id string, files []string)
	OnRequestMontageControl     func(id string, planData string)
	OnTaskStatus                func(id string, status string, progress int)
	OnImageGenerated            func(taskName string, templateName string, imageName string, path string, prompt string, duration float64)
	OnImageDeleted              func(imgPath string)
	OnImageReplaced             func(oldPath string, taskName string, templateName string, newName string, newPath string, prompt string)
	OnRequestExistingFilesCheck func(data ExistingFilesData)
	OnPipelineSuccess           func(id string, taskName string, taskType string, subName string, original string, processed string, settings map[string]interface{}, duration float64)

	pendingControl sync.Map // Map taskID -> chan string
	pendingSkip    sync.Map // Map taskID -> chan []string

	elevenLabsSem           chan struct{}
	elevenLabsUnlimSem      chan struct{}
	elevenLabsUASem         chan struct{}
	subtitleSem             chan struct{}
	subtitleSemSize         int
	subtitleAmdSem          chan struct{}
	subtitleAmdSemSize      int
	subtitleWhisperXSem     chan struct{}
	subtitleWhisperXSemSize int
	subtitleSemMu           sync.Mutex

	montageSem     chan struct{}
	montageSemSize int
	montageSemMu   sync.Mutex

	edgeTTSSem chan struct{}
	cancelled  atomic.Bool

	montageSync struct {
		sync.Mutex
		cond          *sync.Cond
		pendingTasks  map[string]bool // taskID -> confirmed
		totalExpected int
		activeBatchID string
	}

	subtitleBarrier struct {
		sync.Mutex
		cond          *sync.Cond
		inFlightCount int32
	}
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
	s := &PipelineService{
		settings:                settings,
		openRouter:              openRouter,
		elevenLabs:              elevenLabs,
		elevenLabsUnlim:         elevenLabsUnlim,
		elevenLabsUA:            elevenLabsUA,
		voiceMaker:              voiceMaker,
		pollinations:            pollinations,
		googler:                 googler,
		elevenLabsImage:         elevenLabsImage,
		localWhisper:            localWhisper,
		amdWhisper:              amdWhisper,
		edgeTTS:                 edgeTTS,
		assemblyAI:              assemblyAI,
		elevenLabsSem:           make(chan struct{}, 5),
		elevenLabsUnlimSem:      make(chan struct{}, 5),
		elevenLabsUASem:         make(chan struct{}, 5),
		subtitleSemSize:         settings.GetSubtitleMaxConnections(),
		subtitleSem:             make(chan struct{}, settings.GetSubtitleMaxConnections()),
		subtitleAmdSemSize:      settings.GetSubtitleAmdMaxConnections(),
		subtitleAmdSem:          make(chan struct{}, settings.GetSubtitleAmdMaxConnections()),
		subtitleWhisperXSemSize: settings.GetSubtitleWhisperXMaxConnections(),
		subtitleWhisperXSem:     make(chan struct{}, settings.GetSubtitleWhisperXMaxConnections()),
		montageSemSize:          settings.GetMontageMaxConnections(),
		montageSem:              make(chan struct{}, settings.GetMontageMaxConnections()),
		edgeTTSSem:              make(chan struct{}, 5),
	}
	s.montageSync.cond = sync.NewCond(&s.montageSync.Mutex)
	s.montageSync.pendingTasks = make(map[string]bool)
	s.subtitleBarrier.cond = sync.NewCond(&s.subtitleBarrier.Mutex)
	return s
}

// PrepareMontageBatch initializes the synchronization for a batch of tasks with montage control enabled
func (s *PipelineService) PrepareMontageBatch(taskIDs []string) {
	s.montageSync.Lock()
	defer s.montageSync.Unlock()

	s.montageSync.pendingTasks = make(map[string]bool)
	for _, id := range taskIDs {
		s.montageSync.pendingTasks[id] = false
	}
	s.montageSync.totalExpected = len(taskIDs)
	s.montageSync.activeBatchID = fmt.Sprintf("batch_%d", time.Now().Unix())
	s.log("INFO", fmt.Sprintf("[Pipeline] Prepared montage batch with %d controlled tasks", len(taskIDs)))
}

// MarkMontageConfirmed marks a task as confirmed by the user and broadcasts if all are ready
func (s *PipelineService) MarkMontageConfirmed(id string) {
	s.montageSync.Lock()
	defer s.montageSync.Unlock()

	if _, ok := s.montageSync.pendingTasks[id]; ok {
		s.montageSync.pendingTasks[id] = true
		s.log("INFO", fmt.Sprintf("[Pipeline] Task %s confirmed for montage batch", id))
	}

	// Check if all are confirmed
	allConfirmed := true
	count := 0
	for _, confirmed := range s.montageSync.pendingTasks {
		if !confirmed {
			allConfirmed = false
			break
		}
		count++
	}

	if allConfirmed && count > 0 {
		s.log("INFO", "[Pipeline] All tasks in batch confirmed! Starting montages...")
		s.montageSync.cond.Broadcast()
	}
}

// WaitForMontageBatch blocks until all tasks in the current batch are confirmed
func (s *PipelineService) WaitForMontageBatch(id string) {
	s.montageSync.Lock()
	defer s.montageSync.Unlock()

	// If this task is not part of the batch, just continue
	if _, ok := s.montageSync.pendingTasks[id]; !ok {
		return
	}

	// Wait while NOT all are confirmed
	for {
		allConfirmed := true
		count := 0
		for _, confirmed := range s.montageSync.pendingTasks {
			if !confirmed {
				allConfirmed = false
				break
			}
			count++
		}

		if allConfirmed || count == 0 {
			break
		}
		s.montageSync.cond.Wait()
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

// RestartStage re-runs a single failed stage without restarting the whole pipeline.
func (s *PipelineService) RestartStage(id string, stage string, taskName string, taskType string, subName string, settings map[string]interface{}) error {
	settings = s.flattenSettings(settings)
	finalDir := s.ResolveFinalDir(taskName, taskType, subName, settings)
	taskLabel := taskName

	var pSettings utils.PipelineSettings
	pSettings = s.settings.GetPipelineSettings()
	pSettings.SyncFromMap(settings)

	templateDir := subName
	if templateDir == "" {
		pipelineName, _ := settings[taskType+"PipelineName"].(string)
		templateDir = pipelineName
		if templateDir == "" {
			templateDir = "Default"
		}
	}

	go func() {
		s.log("INFO", fmt.Sprintf("[Pipeline] Restarting stage '%s'", stage), id, taskLabel)
		switch stage {
		case "voice":
			processedText, _ := s.LoadTextResult(finalDir, taskType)
			if err := s.ProcessVoiceover(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings); err != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Voiceover restart failed: %v", err), id, taskLabel)
				return
			}
			// Subtitle depends on voice.mp3, so re-run it too (mirrors normal pipeline)
			subtitleEnabled := pSettings.SubtitleEnabled
			if val, ok := settings["subtitleEnabled"].(bool); ok {
				subtitleEnabled = val
			}
			if subtitleEnabled {
				if err := s.ProcessSubtitle(id, taskLabel, finalDir, settings, &pSettings); err != nil {
					s.log("ERROR", fmt.Sprintf("[Pipeline] Subtitle restart failed: %v", err), id, taskLabel)
				}
			}
		case "subtitle":
			if err := s.ProcessSubtitle(id, taskLabel, finalDir, settings, &pSettings); err != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Subtitle restart failed: %v", err), id, taskLabel)
			}
		case "image":
			processedText, _ := s.LoadTextResult(finalDir, taskType)
			if err := s.ProcessImage(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings, taskName, templateDir); err != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Image restart failed: %v", err), id, taskLabel)
			}
		case "montage":
			if err := s.ProcessMontage(id, taskLabel, finalDir, settings, &pSettings, taskName, subName); err != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Montage restart failed: %v", err), id, taskLabel)
				s.emitStageStatus(id, "montage", "failed")
			}
		default:
			s.log("ERROR", fmt.Sprintf("[Pipeline] Unknown stage for restart: %s", stage), id, taskLabel)
		}
	}()
	return nil
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
	s.log("INFO", fmt.Sprintf("[Pipeline] Task execution started. Type: %s, Name: %s, ID: %s", taskType, taskName, id), id, taskLabel)

	// 0. Subtitle Barrier: Mark this task as "pre-montage"
	s.subtitleBarrier.Lock()
	s.subtitleBarrier.inFlightCount++
	s.subtitleBarrier.Unlock()
	subtitleStageFinished := false
	defer func() {
		if !subtitleStageFinished {
			s.subtitleBarrier.Lock()
			s.subtitleBarrier.inFlightCount--
			s.subtitleBarrier.cond.Broadcast()
			s.subtitleBarrier.Unlock()
		}
	}()

	var pSettings utils.PipelineSettings
	s.log("INFO", "[Pipeline] Task started and pre-processing...", id, taskLabel)

	// !!! КРИТИЧНО ПРІОРИТЕТ НАЛАШТУВАНЬ / CRITICAL SETTINGS PRIORITY !!!
	pSettings = s.settings.GetPipelineSettings()

	// ВІДТЕПЕР ПРИ ОБРАННІ ШАБЛОНУ (subName != ""), ВСІ НАЛАШТУВАННЯ ПАЙПЛАЙНУ МАЮТЬ БУТИ ПЕРЕЗАПИСАНІ
	// НАЛАШТУВАННЯМИ З ШАБЛОНУ. ЦЕ ЗАПОБІГАЄ "ПРОТІКАННЮ" ПАРАМЕТРІВ З ПАНЕЛІ ПАЙПЛАЙНУ.
	// !!! ATTENTION FUTURE AGENTS: DO NOT REMOVE THIS SyncFromMap CALL !!!
	if subName != "" {
		s.log("INFO", fmt.Sprintf("[Pipeline] [PRIORITY] Template '%s' detected (Full template sync). Flattened settings count: %d", subName, len(settings)), id, taskLabel)
		// Логуємо наявність критичних для користувача полів (DEBUG)
		if v, ok := settings["montageIntroVideoPaths"]; ok {
			s.log("INFO", fmt.Sprintf("[Pipeline] [DEBUG] Montage Intro Paths found in template: %v", v), id, taskLabel)
		}

		pSettings.SyncFromMap(settings)
		s.log("INFO", "[Pipeline] [PRIORITY] Sync completed. Template overrides applied successfully.", id, taskLabel)
	} else {
		// Якщо шаблон не обрано, використовуємо налаштування з панелі пайплайну з додаванням
		// специфічних для завдання прапорців (наприклад, skippedStages).
		pSettings.SyncFromMap(settings)
		s.log("INFO", fmt.Sprintf("[Pipeline] Using pipeline panel settings. Flattened keys: %d", len(settings)), id, taskLabel)
	}

	// Захист від "протікання" глобальних налаштувань у задачі зі старих шаблонів.
	// SyncFromMap (json.Unmarshal) не скидає поля відсутні у JSON шаблону, тому поля
	// що з'явились нещодавно можуть успадковувати глобальне значення.
	if v, ok := settings["imageVideoDistribution"]; !ok || v == "" {
		pSettings.ImageVideoDistribution = "sequential"
	}

	if taskType != "translate" && taskType != "rewrite" && taskType != "voiceover" {
		return "", fmt.Errorf("task type %s not implemented", taskType)
	}

	finalDir := s.ResolveFinalDir(taskName, taskType, subName, settings)
	s.log("INFO", fmt.Sprintf("[Pipeline] Final directory resolved: %s", finalDir), id, taskLabel)

	// DEBUG: Зберігаємо фінальні налаштування у папку задачі для аналізу
	if err := os.MkdirAll(finalDir, 0755); err == nil {
		// 1. Raw Map (те, що пройшло через flattenSettings)
		if rawData, err := json.MarshalIndent(settings, "", "  "); err == nil {
			_ = os.WriteFile(filepath.Join(finalDir, "debug_pipeline_input_map.json"), rawData, 0644)
		}
		// 2. Struct (те, що було реально застосовано до пайплайну)
		if structData, err := json.MarshalIndent(pSettings, "", "  "); err == nil {
			_ = os.WriteFile(filepath.Join(finalDir, "debug_pipeline_applied_settings.json"), structData, 0644)
		}
		s.log("INFO", "[Pipeline] [DEBUG] Saved settings dumps to final directory", id, taskLabel)
	}
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
			data := s.CheckExistingFiles(id, finalDir, taskType, settings, false)
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
		if !shouldSkipText && pSettings.TranslateControlEnabled && (taskType == "translate" || taskType == "rewrite") && orSuccess {
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

	// Determine if image stage needs subtitle.srt (subtitle_duration mode).
	// In that case we must serialize: run voice+subtitle first, then image.
	imageVideoDistribution, _ := settings["imageVideoDistribution"].(string)
	if imageVideoDistribution == "" {
		imageVideoDistribution = pSettings.ImageVideoDistribution
	}
	needsSubtitleFirst := imageVideoDistribution == "subtitle_duration" && !shouldSkipSubtitle
	if imageVideoDistribution == "subtitle_duration" {
		pSettings.ImageSyncEnabled = true
		pSettings.ImageGooglerVideoEnabled = true
	}

	// 3 & 4. Voiceover and Image Generation Stages logic in parallel
	var stagesWg sync.WaitGroup
	var voiceErr error
	var imageErr error
	var subtitleErr error

	// Channel used to signal that subtitle stage is done (for subtitle_duration mode).
	subtitleDone := make(chan struct{})

	stagesWg.Add(1)
	go func() {
		defer stagesWg.Done()
		if shouldSkipVoice {
			s.log("INFO", "[Pipeline] Skipping voiceover stage, using existing file.", id, taskLabel)
			dur, _ := utils.GetAudioDuration(filepath.Join(finalDir, "voice.mp3"))
			s.emitStageStatus(id, "voice", "completed", dur)
		} else {
			voiceErr = s.ProcessVoiceover(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings)
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
		// Signal that subtitle (and voice) are done, regardless of errors.
		close(subtitleDone)
	}()

	stagesWg.Add(1)
	go func() {
		defer stagesWg.Done()
		// In subtitle_duration mode we must wait for subtitle.srt before generating images.
		if needsSubtitleFirst {
			select {
			case <-subtitleDone:
			case <-s.ctx.Done():
				return
			}
			if subtitleErr != nil || voiceErr != nil {
				imageErr = fmt.Errorf("skipping image stage: voice/subtitle failed")
				s.emitStageStatus(id, "image", "failed")
				return
			}
		}
		imageErr = s.ProcessImage(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings, taskName, templateDir)
		if imageErr != nil {
			s.log("ERROR", fmt.Sprintf("[Pipeline] Image stage failed: %v", imageErr), id, taskLabel)
			return
		}

		// Image Control
		iImageEnabled, ok := settings["imageEnabled"].(bool)
		if !ok {
			iImageEnabled = pSettings.ImageEnabled
		}
		if pSettings.ImageControlEnabled && !shouldSkipImage && iImageEnabled {
			s.emitStageStatus(id, "image", "waiting")
			s.log("INFO", "[Control] Waiting for user image/video review...", id, taskLabel)

			resChan := make(chan string)
			s.pendingControl.Store(id+"_image", resChan)

			if s.OnRequestImageControl != nil {
				limit := s.settings.GetRemotePreviewLimit()
				previewFiles := s.collectImageControlPreviewPaths(finalDir, limit, id, taskLabel)
				s.OnRequestImageControl(id, previewFiles)
			}

			// Block goroutine until result received or timeout/context cancel
			select {
			case action := <-resChan:
				if action == "cancel" || action == "reject" || action == "regenerate" {
					s.log("ERROR", "[Control] Media rejected or regeneration requested by user.", id, taskLabel)
					imageErr = fmt.Errorf("media rejected in control phase")
					s.emitStageStatus(id, "image", "failed")
					return
				}
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

	// 5. Subtitle Barrier: Release and Wait
	subtitleStageFinished = true
	s.subtitleBarrier.Lock()
	s.subtitleBarrier.inFlightCount--
	s.subtitleBarrier.cond.Broadcast()

	s.log("INFO", "[Pipeline] Waiting for all other tasks in queue to finish subtitles...", id, taskLabel)
	for s.subtitleBarrier.inFlightCount > 0 {
		s.subtitleBarrier.cond.Wait()
	}
	s.subtitleBarrier.Unlock()
	s.log("INFO", "[Pipeline] All subtitles finished, proceeding to montage.", id, taskLabel)

	// 6. Montage Stage
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
		safeName := utils.SanitizeFilename(stage.Name)
		if safeName == "" {
			safeName = "custom_stage_" + stage.ID
		}
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
		case "originalText":
			sourceContent = originalText
		default:
			sourceContent = processedText
		}

		fullPrompt := strings.ReplaceAll(stage.Prompt, "{input}", sourceContent)
		useModel := stage.Model
		if useModel == "" || useModel == "default" {
			useModel = defaultModel
		}
		useTemp := stage.Temperature
		if useTemp == 0 {
			useTemp = defaultTemp
		}
		useMaxTokens := stage.MaxTokens
		if useMaxTokens == 0 {
			useMaxTokens = 2000
		}

		result, err := s.openRouter.Chat(id, taskLabel, "custom", keyName, apiKey, useModel, fullPrompt, useTemp, useMaxTokens)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Custom] Stage %s failed: %v", stage.Name, err), id, taskLabel)
			continue
		}

		err = os.WriteFile(savePath, []byte(result), 0644)
		if err != nil {
			s.log("ERROR", fmt.Sprintf("[Custom] Failed to save result for %s: %v", stage.Name, err), id, taskLabel)
		} else {
			s.log("SUCCESS", fmt.Sprintf("[Custom] Stage %s completed, saved to %s", stage.Name, savePath), id, taskLabel)
		}
	}
}

// flattenSettings розгортає вкладену структуру шаблону в плоску мапу,
// враховуючи префікси для вкладених блоків (montage, voiceover тощо)
func (s *PipelineService) flattenSettings(m map[string]interface{}) map[string]interface{} {
	res := make(map[string]interface{})

	var flatten func(map[string]interface{}, string)
	flatten = func(current map[string]interface{}, prefix string) {
		for k, v := range current {
			// Визначаємо фінальний ключ з урахуванням префікса
			fullKey := k
			if prefix != "" && !strings.HasPrefix(strings.ToLower(k), strings.ToLower(prefix)) {
				// Якщо префікс є (наприклад, "montage"), а ключ "enabled",
				// робимо "montageEnabled" (перша літера ключа стає великою)
				if len(k) > 0 {
					fullKey = prefix + strings.ToUpper(k[:1]) + k[1:]
				} else {
					fullKey = prefix
				}
			}

			// Спеціальна обробка блоків, які в struct PipelineSettings є плоскими з префіксом
			if sub, ok := v.(map[string]interface{}); ok {
				lowerK := strings.ToLower(k)
				switch lowerK {
				case "montage", "voiceover", "image", "subtitle", "translate", "rewrite", "control", "stages":
					// Для "stages" та "control" префікс не потрібен, бо вони мапляться вручну або мають власні назви
					newPrefix := k
					if lowerK == "stages" || lowerK == "control" {
						newPrefix = ""
					}

					// Додаємо також мапінги для самих назв блоків (ImageEnabled тощо)
					switch lowerK {
					case "stages":
						if val, ok := sub["image"].(bool); ok {
							res["imageEnabled"] = val
						}
						if val, ok := sub["voiceover"].(bool); ok {
							res["voiceoverEnabled"] = val
						}
						if val, ok := sub["subtitle"].(bool); ok {
							res["subtitleEnabled"] = val
						}
						if val, ok := sub["translate"].(bool); ok {
							res["translateEnabled"] = val
						}
						if val, ok := sub["montage"].(bool); ok {
							res["montageEnabled"] = val
						}
						if val, ok := sub["rewrite"].(bool); ok {
							res["rewriteEnabled"] = val
						}
					case "control":
						if val, ok := sub["image"].(bool); ok {
							res["imageControlEnabled"] = val
						}
						if val, ok := sub["translate"].(bool); ok {
							res["translateControlEnabled"] = val
						}
						if val, ok := sub["montage"].(bool); ok {
							res["montageControlEnabled"] = val
						}
					}

					flatten(sub, newPrefix)
				default:
					// Звичайна вкладеність (без префікса блоку)
					flatten(sub, prefix)
				}
			} else {
				// Пряме значення
				res[fullKey] = v
			}
		}
	}

	flatten(m, "")

	// DEBUG: Логуємо наявність важливих ключів після розплющення
	criticalKeys := []string{"montageIntroVideoPaths", "montageOverlayTriggers", "montageOverlayEnabled", "montageOverlayTriggersEnabled"}
	for _, ck := range criticalKeys {
		if val, ok := res[ck]; ok {
			s.log("INFO", fmt.Sprintf("[Pipeline] [FLATTEN] Found critical key '%s': %v", ck, val))
		}
	}

	return res
}

// SubmitControlResult resumes a paused pipeline with updated text
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

// SubmitImageControlResult resumes a paused pipeline after image review
func (s *PipelineService) SubmitImageControlResult(id string, action string) {
	if val, ok := s.pendingControl.Load(id + "_image"); ok {
		ch := val.(chan string)
		if action == "" {
			action = "confirm"
		}
		ch <- action
	}
}

// SubmitMontageControlResult resumes a paused pipeline after montage review
func (s *PipelineService) SubmitMontageControlResult(id string, planData string) {
	if val, ok := s.pendingControl.Load(id + "_montage"); ok {
		ch := val.(chan string)
		ch <- planData
	}
}

func (s *PipelineService) UpdateSubtitleSemaphore(newSize int, engine ...string) {
	s.subtitleSemMu.Lock()
	defer s.subtitleSemMu.Unlock()
	eng := "standard"
	if len(engine) > 0 {
		eng = engine[0]
	}
	switch eng {
	case "amd":
		if newSize == s.subtitleAmdSemSize {
			return
		}
		s.subtitleAmdSem = make(chan struct{}, newSize)
		s.subtitleAmdSemSize = newSize
	case "whisperx":
		if newSize == s.subtitleWhisperXSemSize {
			return
		}
		s.subtitleWhisperXSem = make(chan struct{}, newSize)
		s.subtitleWhisperXSemSize = newSize
	default:
		if newSize == s.subtitleSemSize {
			return
		}
		s.subtitleSem = make(chan struct{}, newSize)
		s.subtitleSemSize = newSize
	}
}

func (s *PipelineService) getSubtitleSem(engine string) chan struct{} {
	s.subtitleSemMu.Lock()
	defer s.subtitleSemMu.Unlock()
	switch engine {
	case "amd":
		return s.subtitleAmdSem
	case "whisperx":
		return s.subtitleWhisperXSem
	default:
		return s.subtitleSem
	}
}

func (s *PipelineService) UpdateMontageSemaphore(newSize int) {
	s.montageSemMu.Lock()
	defer s.montageSemMu.Unlock()
	if newSize == s.montageSemSize {
		return
	}
	s.montageSem = make(chan struct{}, newSize)
	s.montageSemSize = newSize
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

func (s *PipelineService) CheckExistingFiles(id string, finalDir string, taskType string, settings map[string]interface{}, skipExtra bool) ExistingFilesData {
	data := ExistingFilesData{ID: id, FoundStages: []string{}}
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
	voiceFiles := []string{"voice.mp3", "voice.wav", "Voice.mp3", "Voice.wav"}
	for _, f := range voiceFiles {
		p := filepath.Join(finalDir, f)
		if info, err := os.Stat(p); err == nil && !info.IsDir() {
			data.FoundStages = append(data.FoundStages, "voice")
			if !skipExtra {
				dur, err := utils.GetAudioDuration(p)
				if err == nil {
					data.VoiceDuration = dur
				}
			}
			break
		}
	}

	subFiles := []string{"subtitle.srt", "subtitle.ass", "subtitles.srt", "subtitles.ass", "Subtitle.srt", "Subtitle.ass"}
	for _, f := range subFiles {
		p := filepath.Join(finalDir, f)
		if info, err := os.Stat(p); err == nil && !info.IsDir() {
			data.FoundStages = append(data.FoundStages, "subtitle")
			break
		}
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
	return data
}

func (s *PipelineService) SubmitExistingFilesResult(id string, skipStages []string) {
	if val, ok := s.pendingSkip.Load(id); ok {
		ch := val.(chan []string)
		ch <- skipStages
	}
}

func (s *PipelineService) ResolveFinalDir(taskName string, taskType string, subName string, settings map[string]interface{}) string {
	pSettings := s.settings.GetPipelineSettings()

	// Збираємо всі можливі базові шляхи
	basePaths := []string{}

	// 1. Шлях із налаштувань конкретного завдання
	if out, ok := settings[taskType+"OutputPath"].(string); ok && out != "" {
		basePaths = append(basePaths, out)
	}

	// 2. Специфічні шляхи типів із глобальних налаштувань
	if taskType == "rewrite" && pSettings.RewriteOutputPath != "" {
		basePaths = append(basePaths, pSettings.RewriteOutputPath)
	} else if taskType != "rewrite" && pSettings.TranslateOutputPath != "" {
		basePaths = append(basePaths, pSettings.TranslateOutputPath)
	}

	// 3. Загальний шлях виводу
	if pSettings.OutputPath != "" {
		basePaths = append(basePaths, pSettings.OutputPath)
	}

	// 4. Системний шлях "Відео"
	home, _ := os.UserHomeDir()
	sysVideos := ""
	if runtime.GOOS == "darwin" {
		sysVideos = filepath.Join(home, "Movies")
	} else {
		sysVideos = filepath.Join(home, "Videos")
	}
	basePaths = append(basePaths, sysVideos)

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

	// Варіанти папок для перевірки
	candidates := []string{
		safeTemplateDir + " - " + safeTaskName + " - " + safeTemplateDir, // [NEW] Паттерн користувача: Template - Task - Template
		safeTemplateDir + " - " + safeTaskName,                           // Новий стандарт (плоский, санітизовані назви)
		safeTaskName + " - " + safeTemplateDir,                           // Альтернативний плоский (Task - Template)
		templateDir + " - " + taskName,                                   // Старі оригінальні назви (для зворотної сумісності)
		taskName + " - " + templateDir,                                   // Альтернативний старий (для зворотної сумісності)
		filepath.Join(safeTaskName, safeTemplateDir),                     // Старий вкладений (Task/Template, санітизований)
		safeTaskName, // Просто назва (санітизована)
		taskName,     // Просто назва (оригінал)
	}

	// Використовуємо map для унікальних шляхів, щоб не перевіряти двічі
	checkedPaths := make(map[string]bool)

	for _, basePath := range basePaths {
		if basePath == "" || checkedPaths[basePath] {
			continue
		}
		checkedPaths[basePath] = true

		// Перевіряємо, чи існує сама база (якщо ні - пробуємо створити, крім системних)
		if _, err := os.Stat(basePath); os.IsNotExist(err) && basePath != sysVideos {
			_ = os.MkdirAll(basePath, 0755)
		}

		for i, candidate := range candidates {
			fullPath := filepath.Join(basePath, candidate)
			if runtime.GOOS != "windows" {
				fullPath = strings.ReplaceAll(fullPath, "\\", "/")
			}

			if _, err := os.Stat(fullPath); err == nil {
				s.log("INFO", fmt.Sprintf("[Resolve] Знайдено існуючу папку в %s (%d): %s", basePath, i+1, fullPath))
				return fullPath
			}
		}
	}

	// Якщо нічого не знайдено, повертаємо шлях за замовчуванням у ПЕРШОМУ доступному базовому шляху
	targetBase := basePaths[0]
	// Якщо перший шлях недоступний (наприклад, диск не підключено), шукаємо перший існуючий
	for _, b := range basePaths {
		if _, err := os.Stat(b); err == nil {
			targetBase = b
			break
		}
	}

	defaultPath := filepath.Join(targetBase, safeTemplateDir+" - "+safeTaskName)
	if runtime.GOOS != "windows" {
		defaultPath = strings.ReplaceAll(defaultPath, "\\", "/")
	}
	return defaultPath
}

// collectImageControlPreviewPaths збирає до limit абсолютних шляхів до медіа для прев'ю віддаленого контролю.
// Згенеровані кадри лежать у finalDir/images; додатково перевіряємо корінь finalDir.
func (s *PipelineService) collectImageControlPreviewPaths(finalDir string, limit int, id string, taskLabel string) []string {
	if limit <= 0 {
		return nil
	}
	extOK := map[string]bool{
		".png": true, ".jpg": true, ".jpeg": true, ".webp": true,
		".mp4": true, ".webm": true, ".gif": true,
	}
	var out []string
	addFromDir := func(dir string) {
		if len(out) >= limit {
			return
		}
		entries, err := os.ReadDir(dir)
		if err != nil {
			return
		}
		var names []string
		for _, f := range entries {
			if f.IsDir() {
				continue
			}
			ext := strings.ToLower(filepath.Ext(f.Name()))
			if !extOK[ext] {
				continue
			}
			names = append(names, f.Name())
		}
		sort.Slice(names, func(i, j int) bool {
			return utils.NaturalLess(names[i], names[j])
		})
		for _, n := range names {
			if len(out) >= limit {
				break
			}
			out = append(out, filepath.Join(dir, n))
		}
	}
	imagesDir := filepath.Join(finalDir, "images")
	if st, err := os.Stat(imagesDir); err == nil && st.IsDir() {
		addFromDir(imagesDir)
	}
	if len(out) < limit {
		addFromDir(finalDir)
	}
	if len(out) == 0 {
		s.log("WARN", fmt.Sprintf("[Control] Не знайдено медіафайлів для прев'ю (очікувались у %s/images або в корені)", finalDir), id, taskLabel)
	}
	return out
}
