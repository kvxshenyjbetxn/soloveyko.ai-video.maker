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

	// Callbacks for UI updates
	OnLog            func(level string, message string, details ...string)
	OnStageStatus    func(id string, stage string, status string)
	OnTextResult     func(id string, resultText string)
	OnRequestControl func(id string, text string)

	pendingControl sync.Map // Map taskID -> chan string
}

// NewPipelineService creates a new PipelineService
func NewPipelineService(
	settings *utils.SettingsService,
	openRouter *api.OpenRouterService,
	elevenLabs *api.ElevenLabsBotService,
	elevenLabsUnlim *api.ElevenLabsUnlimService,
) *PipelineService {
	return &PipelineService{
		settings:        settings,
		openRouter:      openRouter,
		elevenLabs:      elevenLabs,
		elevenLabsUnlim: elevenLabsUnlim,
	}
}

// SetContext sets the runtime context
func (s *PipelineService) SetContext(ctx context.Context) {
	s.ctx = ctx
}

// ProcessTask handles the execution of a single pipeline task
func (s *PipelineService) ProcessTask(id string, taskNumber int, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
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

func (s *PipelineService) emitStageStatus(id string, stage string, status string) {
	if s.OnStageStatus != nil {
		s.OnStageStatus(id, stage, status)
	}
}

func (s *PipelineService) runPipeline(id string, taskLabel string, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	pSettings := s.settings.GetPipelineSettings()

	if taskType != "translate" && taskType != "rewrite" && taskType != "voiceover" {
		return "", fmt.Errorf("task type %s not implemented", taskType)
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

	// 3. Voiceover Stage
	err = s.ProcessVoiceover(id, taskLabel, processedText, finalDir, settings, &pSettings)
	if err != nil {
		// We don't necessarily want to fail the whole pipeline if voiceover fails,
		// but for now we follow old logic which just logged it.
	}

	return processedText, nil
}

// SubmitControlResult resumes a paused pipeline with updated text
func (s *PipelineService) SubmitControlResult(id string, text string) {
	if val, ok := s.pendingControl.Load(id); ok {
		ch := val.(chan string)
		ch <- text
	}
}
