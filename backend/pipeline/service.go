package pipeline

import (
	"context"
	"fmt"
	"soloveyko/backend/api"
	"soloveyko/backend/utils"
	"sync"
)

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

	// Callbacks for UI updates
	OnLog            func(level string, message string, details ...string)
	OnStageStatus    func(id string, stage string, status string, message string)
	OnTextResult     func(id string, resultText string)
	OnRequestControl func(id string, text string)
	OnTaskStatus     func(id string, status string, progress int)
	OnImageGenerated func(taskName string, templateName string, imageName string, path string)
	OnImageDeleted   func(imgPath string)

	pendingControl sync.Map // Map taskID -> chan string

	elevenLabsSem      chan struct{}
	elevenLabsUnlimSem chan struct{}
	elevenLabsUASem    chan struct{}
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
		elevenLabsSem:      make(chan struct{}, 5),
		elevenLabsUnlimSem: make(chan struct{}, 5),
		elevenLabsUASem:    make(chan struct{}, 5),
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

func (s *PipelineService) runPipeline(id string, taskLabel string, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	pSettings := s.settings.GetPipelineSettings()

	if taskType != "translate" && taskType != "rewrite" && taskType != "voiceover" {
		return "", fmt.Errorf("task type %s not implemented", taskType)
	}

	if s.OnTaskStatus != nil {
		s.OnTaskStatus(id, "running", 5)
	}

	// 1. Text Stage
	processedText, orSuccess, err := s.ProcessText(id, taskLabel, taskType, content, settings, &pSettings)
	if err != nil {
		return "", err
	}

	// Emit text result immediately after processing
	if s.OnTextResult != nil {
		s.OnTextResult(id, processedText)
	}

	// 1.5 Control Stage
	if pSettings.TranslateControlEnabled && (taskType == "translate" || taskType == "rewrite") && orSuccess {
		s.emitStageStatus(id, "text", "waiting")
		s.log("INFO", "[Control] Waiting for user translation review...", id, taskLabel)

		resChan := make(chan string)
		s.pendingControl.Store(id, resChan)

		if s.OnRequestControl != nil {
			s.OnRequestControl(id, processedText)
		}

		// Block until result received
		updatedText := <-resChan
		processedText = updatedText
		s.pendingControl.Delete(id)

		s.log("SUCCESS", "[Control] Translation approved by user", id, taskLabel)
		// Re-emit result length if it changed
		if s.OnTextResult != nil {
			s.OnTextResult(id, processedText)
		}
	}

	// 2. FS Stage - Prepare Directory
	outPath, _ := settings[taskType+"OutputPath"].(string)
	if outPath == "" {
		switch taskType {
		case "translate":
			outPath = pSettings.TranslateOutputPath
		case "rewrite":
			outPath = pSettings.RewriteOutputPath
		case "voiceover":
			outPath = pSettings.VoiceoverOutputPath
		}
	}
	if outPath == "" {
		outPath = pSettings.OutputPath
	}

	templateDir := subName
	if templateDir == "" {
		pipelineName, _ := settings[taskType+"PipelineName"].(string)
		templateDir = pipelineName
		if templateDir == "" {
			templateDir = "Default"
		}
	}

	finalDir, err := s.EnsureDirectory(outPath, taskName, templateDir)
	if err != nil {
		s.log("ERROR", fmt.Sprintf("[FileSystem] Failed to create directory: %v", err), id, taskLabel)
		return "", err
	}

	// Save Text Result
	shouldProcessText := false // Simple check for saving
	if taskType == "translate" || taskType == "rewrite" {
		shouldProcessText = true
	}

	if orSuccess || shouldProcessText || taskType != "voiceover" {
		s.SaveTextResult(finalDir, taskType, processedText)
	}

	// 3 & 4. Voiceover and Image Generation Stages logic in parallel
	var stageWg sync.WaitGroup
	var voiceErr error
	var imageErr error
	var subtitleErr error

	stageWg.Add(1)
	go func() {
		defer stageWg.Done()
		voiceErr = s.ProcessVoiceover(id, taskLabel, processedText, finalDir, settings, &pSettings)
		if voiceErr != nil {
			s.log("ERROR", fmt.Sprintf("[Pipeline] Voiceover stage failed: %v", voiceErr), id, taskLabel)
		} else if taskType == "voiceover" || taskType == "translate" || taskType == "rewrite" {
			// Після успішного створення озвучки запускаємо створення субтитрів
			subtitleErr = s.ProcessSubtitle(id, taskLabel, finalDir, settings, &pSettings)
			if subtitleErr != nil {
				s.log("ERROR", fmt.Sprintf("[Pipeline] Subtitle stage failed: %v", subtitleErr), id, taskLabel)
			}
		}
	}()

	stageWg.Add(1)
	go func() {
		defer stageWg.Done()
		imageErr = s.ProcessImage(id, taskLabel, taskType, processedText, finalDir, settings, &pSettings, taskName, templateDir)
		if imageErr != nil {
			s.log("ERROR", fmt.Sprintf("[Pipeline] Image stage failed: %v", imageErr), id, taskLabel)
		}
	}()

	stageWg.Wait()

	if voiceErr != nil || imageErr != nil {
		if voiceErr != nil {
			return processedText, voiceErr
		}
		return processedText, imageErr
	}

	return processedText, nil
}

func (s *PipelineService) flattenSettings(m map[string]interface{}) map[string]interface{} {
	res := make(map[string]interface{})
	for k, v := range m {
		if sub, ok := v.(map[string]interface{}); ok {
			for sk, sv := range sub {
				res[sk] = sv
			}
		} else {
			res[k] = v
		}
	}
	return res
}

// SubmitControlResult resumes a paused pipeline with updated text
func (s *PipelineService) SubmitControlResult(id string, text string) {
	if val, ok := s.pendingControl.Load(id); ok {
		ch := val.(chan string)
		ch <- text
	}
}
