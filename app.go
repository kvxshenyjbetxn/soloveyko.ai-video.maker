package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"soloveyko/backend/api"
	"soloveyko/backend/pipeline"
	"soloveyko/backend/utils"
	"strings"
	"sync"
	"time"

	"github.com/gen2brain/beeep"
	wruntime "github.com/wailsapp/wails/v2/pkg/runtime"
)

// App struct
type App struct {
	ctx             context.Context
	settings        *utils.SettingsService
	stats           *utils.StatsService
	openRouter      *api.OpenRouterService
	pollinations    *api.PollinationsService
	elevenLabs      *api.ElevenLabsBotService
	elevenLabsUnlim *api.ElevenLabsUnlimService
	voiceMaker      *api.VoiceMakerService
	googler         *api.GooglerService
	elevenLabsImage *api.ElevenLabsImageService
	elevenLabsUA    *api.ElevenLabsUAService
	assemblyAI      *api.AssemblyAIService
	templates       *utils.TemplateService
	pipeline        *pipeline.PipelineService
	galleryManager  *utils.GalleryManager
	fileLogger      *utils.FileLogger
	localWhisper    *pipeline.LocalWhisperService
	amdWhisper      *pipeline.AmdWhisperService
	edgeTTS         *api.EdgeTTSService
	history         *utils.HistoryService
	fullHistory     *utils.FullHistoryService
	productionStats *utils.ProductionStatsService
	googleParser    *api.GoogleParserService
	authService     *api.AuthService
	telegramService *api.TelegramService
	updater         *utils.UpdateManager
	workerCtx       context.Context
	workerCancel    context.CancelFunc
}

// NewApp creates a new App application struct
func NewApp() *App {
	settings := utils.NewSettingsService()
	orService := api.NewOpenRouterService(settings)
	app := &App{
		settings:        settings,
		fileLogger:      utils.NewFileLogger(),
		stats:           utils.NewStatsService(),
		openRouter:      orService,
		pollinations:    api.NewPollinationsService(settings),
		elevenLabs:      api.NewElevenLabsBotService(settings),
		elevenLabsUnlim: api.NewElevenLabsUnlimService(settings),
		voiceMaker:      api.NewVoiceMakerService(settings),
		googler:         api.NewGooglerService(settings),
		elevenLabsImage: api.NewElevenLabsImageService(settings),
		elevenLabsUA:    api.NewElevenLabsUAService(settings),
		assemblyAI:      api.NewAssemblyAIService(settings),
		templates:       utils.NewTemplateService(),
		googleParser:    api.NewGoogleParserService(),
		authService:     api.NewAuthService(),
		telegramService: api.NewTelegramService(),
		updater:         utils.NewUpdateManager(utils.AppVersion),
	}
	app.galleryManager = utils.NewGalleryManager()
	app.localWhisper = pipeline.NewLocalWhisperService()
	app.amdWhisper = pipeline.NewAmdWhisperService()
	app.edgeTTS = api.NewEdgeTTSService()
	app.history = utils.NewHistoryService()
	app.fullHistory = utils.NewFullHistoryService()
	app.productionStats = utils.NewProductionStatsService()

	app.pipeline = pipeline.NewPipelineService(settings, app.openRouter, app.elevenLabs, app.elevenLabsUnlim, app.elevenLabsUA, app.voiceMaker, app.pollinations, app.googler, app.elevenLabsImage, app.localWhisper, app.amdWhisper, app.edgeTTS, app.assemblyAI)

	app.pipeline.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.pipeline.OnStageStatus = func(id string, stage string, status string, message string) {
		app.EmitStageStatus(id, stage, status, message)
	}

	app.pipeline.OnTaskStatus = func(id string, status string, progress int) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "taskStatus", id, status, progress)
		}
	}

	app.pipeline.OnImageGenerated = func(taskName, templateName, imageName, imgPath, prompt string) {
		app.galleryManager.AddImage(taskName, templateName, imageName, imgPath, prompt)
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "galleryUpdate")
		}
	}

	app.pipeline.OnImageDeleted = func(imgPath string) {
		app.galleryManager.RemoveImage(imgPath)
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "galleryUpdate")
		}
	}

	app.pipeline.OnTextResult = func(id string, resultText string) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "textResult", id, len([]rune(resultText)))
		}
	}

	app.pipeline.OnRequestControl = func(id string, text string) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "requestControl", id, text)
		}
	}
	app.pipeline.OnRequestImageControl = func(id string) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "requestImageControl", id)
		}
	}
	app.pipeline.OnRequestMontageControl = func(id string, planData string) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "requestMontageControl", id, planData)
		}
	}
	app.pipeline.OnRequestExistingFilesCheck = func(data pipeline.ExistingFilesData) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "requestExistingFilesCheck", data)
		}
	}
	app.pipeline.OnPipelineSuccess = func(id string, taskName string, taskType string, subName string, original string, processed string, settings map[string]interface{}, duration float64) {
		tpls := []string{}
		if subName != "" {
			tpls = append(tpls, subName)
		}

		stages := []string{}
		switch taskType {
		case "translate":
			stages = append(stages, "translate")
		case "rewrite":
			stages = append(stages, "rewrite")
		}

		if val, ok := settings["voiceoverEnabled"].(bool); ok && val {
			stages = append(stages, "voiceover")
		}
		if val, ok := settings["imageEnabled"].(bool); ok && val {
			stages = append(stages, "image")
		}
		if val, ok := settings["subtitleEnabled"].(bool); ok && val {
			stages = append(stages, "subtitles")
		}
		if val, ok := settings["montageEnabled"].(bool); ok && val {
			stages = append(stages, "montage")
		}

		_ = app.fullHistory.AddEntry(taskName, taskType, tpls, stages, original, processed, duration)

		// Record stats only if it was a video production (montage stage carried out)
		if val, ok := settings["montageEnabled"].(bool); ok && val {
			app.productionStats.RecordCompletion(taskType, duration)
		}
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "fullHistoryUpdate")
		}
	}

	orService.OnRequestStart = func(id string, taskLabel string, taskType string, keyName string, model string, temp float64, tokens int) {
		app.LogToUI("INFO", fmt.Sprintf("[OpenRouter] [%s] Request | Key: %s | Model: %s | Temp: %.2f | Max Tokens: %v", strings.Title(taskType), keyName, model, temp, tokens), id, taskLabel)
		// Емітуємо подію, щоб фронтенд знав, що завдання ДІЙСНО почало обробку
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "taskStatus", id, "processing", 10)
		}
	}

	orService.OnLogData = func(category string, data string) {
		if app.fileLogger != nil {
			app.fileLogger.LogData(category, data)
		}
	}

	orService.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.elevenLabs.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.elevenLabsUnlim.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.elevenLabsUA.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.assemblyAI.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.googler.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.googler.OnLogData = func(category string, data string) {
		if app.fileLogger != nil {
			app.fileLogger.LogData(category, data)
		}
	}

	app.elevenLabsImage.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.elevenLabsImage.OnLogData = func(category string, data string) {
		if app.fileLogger != nil {
			app.fileLogger.LogData(category, data)
		}
	}

	return app
}

// GetSubtitleMaxConnections повертає ліміт одночасних запитів Субтитрів
func (a *App) GetSubtitleMaxConnections() int {
	return a.settings.GetSubtitleMaxConnections()
}

// SaveSubtitleMaxConnections встановлює ліміт одночасних запитів Субтитрів
func (a *App) SaveSubtitleMaxConnections(max int) error {
	err := a.settings.SetSubtitleMaxConnections(max)
	if err == nil {
		a.pipeline.UpdateSubtitleSemaphore(max, "standard")
	}
	return err
}

// GetSubtitleAmdMaxConnections повертає ліміт одночасних запитів Субтитрів AMD
func (a *App) GetSubtitleAmdMaxConnections() int {
	return a.settings.GetSubtitleAmdMaxConnections()
}

// SaveSubtitleAmdMaxConnections встановлює ліміт одночасних запитів Субтитрів AMD
func (a *App) SaveSubtitleAmdMaxConnections(max int) error {
	err := a.settings.SetSubtitleAmdMaxConnections(max)
	if err == nil {
		a.pipeline.UpdateSubtitleSemaphore(max, "amd")
	}
	return err
}

// GetSubtitleWhisperXMaxConnections повертає ліміт одночасних запитів WhisperX
func (a *App) GetSubtitleWhisperXMaxConnections() int {
	return a.settings.GetSubtitleWhisperXMaxConnections()
}

// SaveSubtitleWhisperXMaxConnections встановлює ліміт одночасних запитів WhisperX
func (a *App) SaveSubtitleWhisperXMaxConnections(max int) error {
	err := a.settings.SetSubtitleWhisperXMaxConnections(max)
	if err == nil {
		a.pipeline.UpdateSubtitleSemaphore(max, "whisperx")
	}
	return err
}

// GetMontageMaxConnections повертає ліміт одночасних запитів Монтажу
func (a *App) GetMontageMaxConnections() int {
	return a.settings.GetMontageMaxConnections()
}

// SaveMontageMaxConnections встановлює ліміт одночасних запитів Монтажу
func (a *App) SaveMontageMaxConnections(max int) error {
	err := a.settings.SetMontageMaxConnections(max)
	if err == nil {
		a.pipeline.UpdateMontageSemaphore(max)
	}
	return err
}

// GetMontageMode повертає режим монтажу
func (a *App) GetMontageMode() string {
	return a.settings.GetMontageMode()
}

// SaveMontageMode встановлює режим монтажу
func (a *App) SaveMontageMode(mode string) error {
	return a.settings.SetMontageMode(mode)
}

// GetSystemStats повертає поточну статистику системи
func (a *App) GetSystemStats() (*utils.SystemStats, error) {
	return a.stats.GetSystemStats()
}

// GeneratePreview здійснює швидкий рендер фрагмента для попереднього перегляду
func (a *App) GeneratePreview(settings map[string]interface{}) (string, error) {
	previewDir := a.settings.GetPreviewDir()
	
	// Створюємо папку, якщо не існує
	if err := os.MkdirAll(previewDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create preview directory: %v", err)
	}

	// Очищуємо папку перед генерацією (окрім images та voice.mp3)
	entries, _ := os.ReadDir(previewDir)
	for _, entry := range entries {
		name := entry.Name()
		if name == "images" || name == "voice.mp3" {
			continue
		}
		os.RemoveAll(filepath.Join(previewDir, name))
	}

	// Переконуємось, що папка images існує
	if err := os.MkdirAll(filepath.Join(previewDir, "images"), 0755); err != nil {
		return "", fmt.Errorf("failed to create images directory in preview: %v", err)
	}

	a.LogToUI("INFO", "[Preview] Starting quick preview generation...", "preview_task", "Preview")

	// Створюємо базові PipelineSettings з поточних налаштувань
	currentSettings, _ := a.settings.LoadSettings()
	pSettings := currentSettings.Pipeline

	// Оновлюємо pSettings з тими, що прийшли з фронтенда (з вкладки Preview)
	// Це дозволяє бачити зміни в реальному часі після натискання ОК
	// Оновлюємо субтитри
	if val, ok := settings["subtitleEnabled"].(bool); ok { pSettings.SubtitleEnabled = val }
	if val, ok := settings["subtitleService"].(string); ok { pSettings.SubtitleService = val }
	if val, ok := settings["subtitleModel"].(string); ok { pSettings.SubtitleModel = val }
	if val, ok := settings["subtitleFont"].(string); ok { pSettings.SubtitleFont = val }
	if val, ok := settings["subtitleSize"].(float64); ok { pSettings.SubtitleSize = int(val) }
	if val, ok := settings["subtitleColor"].(string); ok { pSettings.SubtitleColor = val }
	if val, ok := settings["subtitleOutlineColor"].(string); ok { pSettings.SubtitleOutlineColor = val }
	if val, ok := settings["subtitleOutlineWidth"].(float64); ok { pSettings.SubtitleOutlineWidth = val }
	if val, ok := settings["subtitleShadowColor"].(string); ok { pSettings.SubtitleShadowColor = val }
	if val, ok := settings["subtitleShadowWidth"].(float64); ok { pSettings.SubtitleShadowWidth = val }
	if val, ok := settings["subtitleBlur"].(float64); ok { pSettings.SubtitleBlur = val }
	if val, ok := settings["subtitleUppercase"].(bool); ok { pSettings.SubtitleUppercase = val }
	if val, ok := settings["subtitlePosition"].(string); ok { pSettings.SubtitlePosition = val }
	if val, ok := settings["subtitleMarginV"].(float64); ok { pSettings.SubtitleMarginV = int(val) }
	if val, ok := settings["subtitleAnimation"].(string); ok { pSettings.SubtitleAnimation = val }
	if val, ok := settings["subtitleFadeEnabled"].(bool); ok { pSettings.SubtitleFadeEnabled = val }
	if val, ok := settings["subtitleFadeIn"].(float64); ok { pSettings.SubtitleFadeIn = int(val) }
	if val, ok := settings["subtitleFadeOut"].(float64); ok { pSettings.SubtitleFadeOut = int(val) }
	if val, ok := settings["subtitleKaraokeEffect"].(bool); ok { pSettings.SubtitleKaraokeEffect = val }
	if val, ok := settings["subtitleKaraokeColor"].(string); ok { pSettings.SubtitleKaraokeColor = val }
	if val, ok := settings["subtitleKaraokeMode"].(string); ok { pSettings.SubtitleKaraokeMode = val }
	if val, ok := settings["subtitleKaraokeScale"].(float64); ok { pSettings.SubtitleKaraokeScale = val }
	if val, ok := settings["subtitleKaraokeSpeed"].(float64); ok { pSettings.SubtitleKaraokeSpeed = int(val) }
	if val, ok := settings["subtitleMaxLen"].(float64); ok { pSettings.SubtitleMaxLen = int(val) }
	if val, ok := settings["subtitleMaxWords"].(float64); ok { pSettings.SubtitleMaxWords = int(val) }
	if val, ok := settings["subtitleWhisperxLanguage"].(string); ok { pSettings.SubtitleWhisperxLanguage = val }
	if val, ok := settings["subtitleAmdLanguage"].(string); ok { pSettings.SubtitleAmdLanguage = val }
	
	// Оновлюємо монтаж
	if val, ok := settings["montageSwayFactor"].(float64); ok { pSettings.MontageSwayFactor = val }
	if val, ok := settings["montageZoomFactor"].(float64); ok { pSettings.MontageZoomFactor = val }
	if val, ok := settings["montageTransitionDuration"].(float64); ok { pSettings.MontageTransitionDuration = val }
	if val, ok := settings["montageTransitionEffect"].(string); ok { pSettings.MontageTransitionEffect = val }
	if val, ok := settings["montageOrientation"].(string); ok { pSettings.MontageOrientation = val }
	if val, ok := settings["montageWatermarkEnabled"].(bool); ok { pSettings.MontageWatermarkEnabled = val }
	if val, ok := settings["montageWatermarkPath"].(string); ok { pSettings.MontageWatermarkPath = val }
	if val, ok := settings["montageWatermarkPosition"].(string); ok { pSettings.MontageWatermarkPosition = val }
	if val, ok := settings["montageWatermarkOpacity"].(float64); ok { pSettings.MontageWatermarkOpacity = val }
	if val, ok := settings["montageWatermarkSize"].(float64); ok { pSettings.MontageWatermarkSize = int(val) }
	if val, ok := settings["montageIntroVideoEnabled"].(bool); ok { pSettings.MontageIntroVideoEnabled = val }
	if val, ok := settings["montageIntroVideoPath"].(string); ok { pSettings.MontageIntroVideoPath = val }
	if val, ok := settings["montageIntroVideoPaths"].([]interface{}); ok {
		paths := make([]string, 0, len(val))
		for _, v := range val {
			if s, ok := v.(string); ok && s != "" {
				paths = append(paths, s)
			}
		}
		pSettings.MontageIntroVideoPaths = paths
	}

	// Preview Specific Limits
	if val, ok := settings["previewLimitSeconds"].(float64); ok && val > 0 {
		settings["previewLimitSeconds"] = val
	}
	if val, ok := settings["previewImageMax"].(float64); ok {
		settings["previewImageMax"] = int(val)
	}
	if val, ok := settings["previewVideoMax"].(float64); ok {
		settings["previewVideoMax"] = int(val)
	}

	// [PREVIEW] Respect sidebar settings instead of forcing speed
	// Use settings from left panel

	// 1. Обробка субтитрів
	 voicePath := filepath.Join(previewDir, "voice.mp3")
	 if _, err := os.Stat(voicePath); os.IsNotExist(err) {
		 return "", fmt.Errorf("voice.mp3 not found in preview folder. Please add it for preview.")
	 }

	 err := a.pipeline.ProcessSubtitle("preview_task", "Preview", previewDir, settings, &pSettings)
	 if err != nil {
		 a.LogToUI("ERROR", fmt.Sprintf("[Preview] Subtitle stage failed: %v", err))
		 return "", err
	 }

	 // 2. Обробка монтажу
	 err = a.pipeline.ProcessMontage("preview_task", "Preview", previewDir, settings, &pSettings, "Preview", "")
	 if err != nil {
		 a.LogToUI("ERROR", fmt.Sprintf("[Preview] Montage stage failed: %v", err))
		 return "", err
	 }

	 finalVideo := filepath.Join(previewDir, "final.mp4")
	 if _, err := os.Stat(finalVideo); err != nil {
		 // Debug: list files to see what was actually generated
		 files, _ := os.ReadDir(previewDir)
		 var foundFiles []string
		 for _, f := range files {
			 if !f.IsDir() && strings.HasSuffix(f.Name(), ".mp4") {
				 foundFiles = append(foundFiles, f.Name())
			 }
		 }
		 return "", fmt.Errorf("final video was not generated. Found MP4s: %v", foundFiles)
	 }

	 a.LogToUI("SUCCESS", "[Preview] Preview generated successfully!")
	 return finalVideo, nil
}

// GetPreviewPath повертає шлях до папки прев'ю
func (a *App) GetPreviewPath() string {
	return a.settings.GetPreviewDir()
}

// GetPreviewAudioDuration повертає тривалість аудіо в папці прев'ю
func (a *App) GetPreviewAudioDuration() (float64, error) {
	previewDir := a.settings.GetPreviewDir()
	audioPath := filepath.Join(previewDir, "voice.mp3")
	if _, err := os.Stat(audioPath); err != nil {
		return 0, nil
	}
	dur, err := utils.GetAudioDurationSeconds(audioPath)
	if err != nil {
		return 0, err
	}
	return dur, nil
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
	a.pipeline.SetContext(ctx)

	// Set app name for native notifications
	beeep.AppName = utils.AppName

	// Register native file drop handler
	wruntime.OnFileDrop(ctx, func(x, y int, paths []string) {
		// Emit custom event to frontend with absolute paths
		wruntime.EventsEmit(ctx, "files:dropped", map[string]interface{}{
			"x":     x,
			"y":     y,
			"paths": paths,
		})
	})

	// Розпаковуємо всі бінарники одразу при старті в фоні (без блокування UI)
	go func() {
		utils.EnsureEngine("ffprobe")
		utils.EnsureExifTool() // Розпаковка ExifTool для метаданих
		if a.localWhisper != nil {
			a.localWhisper.EnsureFFmpeg()
			a.localWhisper.EnsureWhisperCLI()
		}
	}()

	// Start worker mode heartbeat if enabled
	if a.settings.GetWorkerModeEnabled() {
		a.workerCtx, a.workerCancel = context.WithCancel(context.Background())
		go a.startHeartbeatLoop()
		go a.startTaskPollingLoop()
	}
}

// ToggleWorkerMode вмикає або вимикає режим воркера
func (a *App) ToggleWorkerMode(enabled bool) error {
	err := a.settings.SetWorkerModeEnabled(enabled)
	if err != nil {
		return err
	}

	if enabled {
		if a.workerCancel == nil {
			a.workerCtx, a.workerCancel = context.WithCancel(context.Background())
			go a.startHeartbeatLoop()
			go a.startTaskPollingLoop()
		}
	} else {
		if a.workerCancel != nil {
			a.workerCancel()
			a.workerCancel = nil
			a.workerCtx = nil
		}
	}
	return nil
}

// GetWorkerStatus повертає чи активний зараз режим воркера (чи йде Heartbeat)
func (a *App) GetWorkerStatus() bool {
	return a.workerCancel != nil
}

func (a *App) startHeartbeatLoop() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	hwID := utils.GetHardwareID()

	hostname, _ := os.Hostname()

	// Перший запуск одразу
	if key := a.settings.GetAppAccessKey(); key != "" {
		a.sendHeartbeat(key, hwID, hostname, "active")
	}

	for {
		select {
		case <-a.workerCtx.Done():
			// Відправляємо статус offline перед виходом
			if key := a.settings.GetAppAccessKey(); key != "" {
				a.sendHeartbeat(key, hwID, hostname, "offline")
			}
			return
		case <-ticker.C:
			if key := a.settings.GetAppAccessKey(); key != "" {
				a.sendHeartbeat(key, hwID, hostname, "active")
			}
		}
	}
}

func (a *App) startTaskPollingLoop() {
	ticker := time.NewTicker(15 * time.Second)
	defer ticker.Stop()

	hwID := utils.GetHardwareID()

	for {
		select {
		case <-a.workerCtx.Done():
			return
		case <-ticker.C:
			if key := a.settings.GetAppAccessKey(); key != "" {
				a.pollAndExecuteTask(key, hwID)
			}
		}
	}
}

func (a *App) workerCancelDone() <-chan struct{} {
	// Helper to handle nil check if needed, but here we can just use a internal context
	// Actually we should use a shared context for worker mode
	return nil // placeholder, will fix with proper context management
}

func (a *App) pollAndExecuteTask(key, hwID string) {
	if key == "" {
		return
	}

	key = strings.TrimSpace(key)
	url := fmt.Sprintf("%s/tasks/claim?key=%s&hardware_id=%s", 
		"https://new-project-combain-server-production.up.railway.app", url.QueryEscape(key), hwID)
	
	resp, err := http.Get(url)
	if err != nil || resp.StatusCode != http.StatusOK {
		if resp != nil {
			resp.Body.Close()
		}
		return
	}
	defer resp.Body.Close()

	var task struct {
		ID       string `json:"id"`
		TaskName string `json:"task_name"`
		Payload  string `json:"payload"`
		Settings string `json:"settings"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&task); err != nil {
		return
	}

	a.LogToUI("INFO", fmt.Sprintf("[Worker] Claimed remote task: %s", task.TaskName))

	var settings map[string]interface{}
	json.Unmarshal([]byte(task.Settings), &settings)

	// Execute task
	id := task.ID
	if id == "" {
		id = fmt.Sprintf("remote_%d", time.Now().Unix())
	}

	// We'll use task type from settings or payload if needed, 
	// for now assume "translate" as a safe fallback or extract from naming
	taskType := "translate" // This should ideally be in the RemoteTask model

	_, err = a.pipeline.ProcessTask(id, 1, taskType, task.Payload, settings, task.TaskName, "")
	
	status := "completed"
	result := ""
	if err != nil {
		status = "failed"
		result = err.Error()
		a.LogToUI("ERROR", fmt.Sprintf("[Worker] Remote task failed: %v", err))
	} else {
		a.LogToUI("SUCCESS", fmt.Sprintf("[Worker] Remote task completed: %s", task.TaskName))
	}

	// Report result
	a.sendTaskResult(key, task.ID, status, result)
}

func (a *App) sendTaskResult(key, taskID, status, result string) {
	key = strings.TrimSpace(key)
	url := fmt.Sprintf("%s/tasks/result", "https://new-project-combain-server-production.up.railway.app")
	payload := map[string]interface{}{
		"key":     key,
		"task_id": taskID,
		"status":  status,
		"result":  result,
	}
	jsonData, _ := json.Marshal(payload)
	http.Post(url, "application/json", bytes.NewBuffer(jsonData))
}

func (a *App) sendHeartbeat(key string, hwID string, name string, status string) {
	key = strings.TrimSpace(key)
	if key == "" {
		return
	}

	url := fmt.Sprintf("%s/worker/heartbeat", "https://new-project-combain-server-production.up.railway.app")
	
	payload := map[string]string{
		"key":         key,
		"hardware_id": hwID,
		"name":        name,
		"status":      status,
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return
	}

	resp, err := http.Post(url, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		// Log error to UI if needed
		return
	}
	defer resp.Body.Close()
}

// GetAvailableWorkers отримує список активних воркерів з сервера
func (a *App) GetAvailableWorkers() ([]map[string]interface{}, error) {
	key := a.settings.GetAppAccessKey()
	if key == "" {
		return nil, fmt.Errorf("app key is missing")
	}

	key = strings.TrimSpace(key)
	url := fmt.Sprintf("%s/workers?key=%s", "https://new-project-combain-server-production.up.railway.app", url.QueryEscape(key))
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("server returned status: %d", resp.StatusCode)
	}

	var workers []map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&workers); err != nil {
		return nil, err
	}

	hwID := utils.GetHardwareID()
	var filtered []map[string]interface{}
	for _, w := range workers {
		if wID, ok := w["hardware_id"].(string); ok && wID != hwID {
			filtered = append(filtered, w)
		}
	}

	return filtered, nil
}

// SendRemoteTask відправляє завдання на сервер для віддаленого виконання
func (a *App) SendRemoteTask(name, payload string, settings map[string]interface{}) error {
	key := a.settings.GetAppAccessKey()
	if key == "" {
		return fmt.Errorf("app key is missing")
	}

	hwID := utils.GetHardwareID()
	settingsJSON, _ := json.Marshal(settings)

	url := fmt.Sprintf("%s/tasks", "https://new-project-combain-server-production.up.railway.app")
	
	reqBody := map[string]string{
		"key":         key,
		"hardware_id": hwID,
		"name":        name,
		"payload":     payload,
		"settings":    string(settingsJSON),
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	resp, err := http.Post(url, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("server error: %d", resp.StatusCode)
	}

	a.LogToUI("SUCCESS", fmt.Sprintf("[Remote] Task '%s' sent to render farm", name))
	return nil
}

// LogToUI emits a log event to the frontend
func (a *App) LogToUI(level string, message string, details ...string) {
	if a.fileLogger != nil {
		a.fileLogger.Log(level, message, details...)
	}

	if a.ctx != nil {
		tID := ""
		tLabel := ""
		if len(details) > 0 {
			tID = details[0]
		}
		if len(details) > 1 {
			tLabel = details[1]
		}
		wruntime.EventsEmit(a.ctx, "log", level, message, tID, tLabel)
	}
}

// LogFromUI is a simple wrapper for LogToUI to be used from the frontend
// to avoid issues with variadic arguments in Wails reflection
func (a *App) LogFromUI(level string, message string) {
	a.LogToUI(level, message)
}

// EmitStageStatus emits a stage status event to the frontend
func (a *App) EmitStageStatus(id string, stage string, status string, message string) {
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "stageStatus", id, stage, status, message)
	}
}

// GetAppVersion returns the current application version
func (a *App) GetAppVersion() string {
	return utils.AppVersion
}

// GetOS returns the current operating system
func (a *App) GetOS() string {
	return runtime.GOOS
}

// Check for updates checks for application updates
func (a *App) CheckForUpdates(manifestURL string) (*utils.UpdateManifest, error) {
	return a.updater.Check(manifestURL)
}

// DownloadUpdate downloads the update package and reports progress via events
func (a *App) DownloadUpdate(url string) (string, error) {
	progressChan := make(chan int)

	// Start progress monitoring in a goroutine
	go func() {
		for progress := range progressChan {
			if a.ctx != nil {
				wruntime.EventsEmit(a.ctx, "updateProgress", progress)
			}
		}
	}()

	defer close(progressChan)

	var pkgPath string
	var err error

	if runtime.GOOS == "darwin" {
		pkgPath, err = a.updater.DownloadToDownloads(url, progressChan)
	} else {
		pkgPath, err = a.updater.Download(url, progressChan)
	}

	if err != nil {
		return "", err
	}

	return pkgPath, nil
}

// ApplyUpdate runs the update script and restarts the app
func (a *App) ApplyUpdate(pkgPath string) error {
	err := a.updater.Apply(pkgPath)
	if err != nil {
		return err
	}

	// Exit the app so the update script can replace the binary
	os.Exit(0)
	return nil
}

// GetLanguage повертає поточну мову з налаштувань
func (a *App) GetLanguage() string {
	return a.settings.GetLanguage()
}

// SetLanguage встановлює мову та зберігає у файл
func (a *App) SetLanguage(language string) error {
	return a.settings.SetLanguage(language)
}

// GetTheme повертає поточну тему з налаштувань
func (a *App) GetTheme() string {
	return a.settings.GetTheme()
}

// SetTheme встановлює тему та зберігає у файл
func (a *App) SetTheme(theme string) error {
	return a.settings.SetTheme(theme)
}

// GetAccentColor повертає поточний колір акценту
func (a *App) GetAccentColor() string {
	return a.settings.GetAccentColor()
}

// SetAccentColor встановлює колір акценту та зберігає у файл
func (a *App) SetAccentColor(color string) error {
	return a.settings.SetAccentColor(color)
}

// IsFirstRun повертає чи це перший запуск програми
func (a *App) IsFirstRun() bool {
	return a.settings.IsFirstRun()
}

// SetFirstRun встановлює статус першого запуску
func (a *App) SetFirstRun(firstRun bool) error {
	return a.settings.SetFirstRun(firstRun)
}

// GetShowWelcome повертає чи потрібно показувати вікно привітання
func (a *App) GetShowWelcome() bool {
	return a.settings.GetShowWelcome()
}

// SetShowWelcome встановлює чи потрібно показувати вікно привітання
func (a *App) SetShowWelcome(show bool) error {
	return a.settings.SetShowWelcome(show)
}

// OpenConfigDir відкриває папку з конфігурацією в системному провіднику
func (a *App) OpenConfigDir() {
	path := a.settings.GetConfigDir()
	a.OpenPath(path)
}

// SetGeneralWhisperEngine встановлює обраний двигун транскрипції
func (a *App) SetGeneralWhisperEngine(engine string) error {
	return a.settings.SetGeneralWhisperEngine(engine)
}

// SetGeneralMontageCodec встановлює обраний кодек для монтажу
func (a *App) SetGeneralMontageCodec(codec string) error {
	return a.settings.SetGeneralMontageCodec(codec)
}

// OpenPath opens the specified path in the system file explorer
func (a *App) OpenPath(path string) {
	if path == "" {
		return
	}

	// Normalize path separators if needed (handling Windows paths on Unix)
	if runtime.GOOS != "windows" {
		path = strings.ReplaceAll(path, "\\", "/")
	}

	// Clean the path to resolve redundant separators and ..
	path = filepath.Clean(path)

	a.LogToUI("INFO", fmt.Sprintf("[System] Opening path: %s", path))

	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "windows":
		cmd = exec.Command("explorer", path)
	case "darwin":
		cmd = exec.Command("open", path)
	default:
		cmd = exec.Command("xdg-open", path)
	}

	if cmd != nil {
		go func() {
			err := cmd.Run()
			if err != nil {
				a.LogToUI("ERROR", fmt.Sprintf("[System] Failed to open path: %v", err))
			}
		}()
	}
}

// GetConfigPath повертає шлях до файлу налаштувань (для дебагу)
func (a *App) GetConfigPath() string {
	return a.settings.GetConfigPath()
}

// GetTemplates повертає список шаблонів пайплайнів
func (a *App) GetTemplates() ([]utils.PipelineTemplate, error) {
	return a.templates.LoadTemplates()
}

// AddTemplate додає новий шаблон пайплайну
func (a *App) AddTemplate(tplType string, name string, data map[string]interface{}) (*utils.PipelineTemplate, error) {
	return a.templates.AddTemplate(tplType, name, data)
}

// DeleteTemplate видаляє шаблон пайплайну
func (a *App) DeleteTemplate(id string) error {
	return a.templates.DeleteTemplate(id)
}

// UpdateTemplate оновлює шаблон пайплайну
func (a *App) UpdateTemplate(id string, name string, data map[string]interface{}) error {
	return a.templates.UpdateTemplate(id, name, data)
}

// SelectDirectory opens a directory dialog and returns the selected path
func (a *App) SelectDirectory() (string, error) {
	return wruntime.OpenDirectoryDialog(a.ctx, wruntime.OpenDialogOptions{
		Title: "Виберіть папку для збереження",
	})
}

// GetDefaultVideosPath returns the system default videos folder
func (a *App) GetDefaultVideosPath() string {
	home, err := os.UserHomeDir()
	if err != nil {
		return ""
	}

	switch runtime.GOOS {
	case "windows":
		return filepath.Join(home, "Videos")
	case "darwin":
		return filepath.Join(home, "Movies")
	default:
		return filepath.Join(home, "Videos")
	}
}

// OpenRouter Methods

// GetOpenRouterCredits returns the user's credits balance from OpenRouter
func (a *App) GetOpenRouterCredits(apiKey string) (float64, error) {
	balance, err := a.openRouter.GetOpenRouterCredits(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[OpenRouter] Balance check failed: %v", err))
		return 0, err
	}
	a.LogToUI("SUCCESS", fmt.Sprintf("[OpenRouter] Balance updated: $%.4f", balance))
	return balance, nil
}

// GetOpenRouterAvailableModels returns the list of available models from OpenRouter
func (a *App) GetOpenRouterAvailableModels() ([]api.OpenRouterModel, error) {
	return a.openRouter.GetOpenRouterAvailableModels()
}

// SaveOpenRouterAPIKey saves API key
func (a *App) SaveOpenRouterAPIKey(apiKey string) error {
	return a.openRouter.SaveOpenRouterAPIKey(apiKey)
}

// GetOpenRouterAPIKey gets API key
func (a *App) GetOpenRouterAPIKey() string {
	return a.openRouter.GetOpenRouterAPIKey()
}

// SaveOpenRouterModels saves list of model IDs
func (a *App) SaveOpenRouterModels(models []string) error {
	return a.openRouter.SaveOpenRouterModels(models)
}

// GetOpenRouterSavedModels gets list of saved model IDs
func (a *App) GetOpenRouterSavedModels() []string {
	return a.openRouter.GetOpenRouterSavedModels()
}

// GetOpenRouterKeys returns the list of named API keys
func (a *App) GetOpenRouterKeys() []utils.NamedAPIKey {
	return a.settings.GetOpenRouterKeys()
}

// SaveOpenRouterKeys saves the list of named API keys
func (a *App) SaveOpenRouterKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetOpenRouterKeys(keys)
}

// Pollinations Methods

// GetPollinationsImageModels fetches available image models from Pollinations.ai
func (a *App) GetPollinationsImageModels() ([]string, error) {
	return a.pollinations.GetPollinationsImageModels()
}

// SavePollinationsAPIKey saves API key
func (a *App) SavePollinationsAPIKey(apiKey string) error {
	return a.pollinations.SavePollinationsAPIKey(apiKey)
}

// GetPollinationsAPIKey gets API key
func (a *App) GetPollinationsAPIKey() string {
	return a.pollinations.GetPollinationsAPIKey()
}

// SavePollinationsModels saves list of model IDs
func (a *App) SavePollinationsModels(models []string) error {
	return a.pollinations.SavePollinationsModels(models)
}

// GetPollinationsSavedModels gets list of saved model IDs
func (a *App) GetPollinationsSavedModels() []string {
	return a.pollinations.GetPollinationsSavedModels()
}

// GetPollinationsKeys returns the list of named API keys
func (a *App) GetPollinationsKeys() []utils.NamedAPIKey {
	return a.settings.GetPollinationsKeys()
}

// SavePollinationsKeys saves the list of named API keys
func (a *App) SavePollinationsKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetPollinationsKeys(keys)
}

// ElevenLabsBot Methods

// GetElevenLabsBotBalance returns the user's balance from ElevenLabsBot
func (a *App) GetElevenLabsBotBalance(apiKey string) (float64, error) {
	balance, err := a.elevenLabs.GetBalance(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[ElevenLabsBot] Balance check failed: %v", err))
		return 0, err
	}
	a.LogToUI("SUCCESS", fmt.Sprintf("[ElevenLabsBot] Balance updated: %.0f tokens", balance))
	return balance, nil
}

// SaveElevenLabsBotAPIKey saves API key
func (a *App) SaveElevenLabsBotAPIKey(apiKey string) error {
	return a.elevenLabs.SaveAPIKey(apiKey)
}

// GetElevenLabsBotAPIKey gets API key
func (a *App) GetElevenLabsBotAPIKey() string {
	return a.elevenLabs.GetAPIKey()
}

// GetElevenLabsBotKeys returns the list of named API keys
func (a *App) GetElevenLabsBotKeys() []utils.NamedAPIKey {
	return a.settings.GetElevenLabsBotKeys()
}

// SaveElevenLabsBotKeys saves the list of named API keys
func (a *App) SaveElevenLabsBotKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetElevenLabsBotKeys(keys)
}

// ElevenLabsUnlim Methods

// GetElevenLabsUnlimBalance returns the user's balance from ElevenLabsUnlim
func (a *App) GetElevenLabsUnlimBalance(apiKey string) (float64, error) {
	balance, err := a.elevenLabsUnlim.GetBalance(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[ElevenLabsUnlim] Balance check failed: %v", err))
		return 0, err
	}
	if balance == -1 {
		a.LogToUI("SUCCESS", "[ElevenLabsUnlim] Balance updated: Unlimited")
	} else {
		a.LogToUI("SUCCESS", fmt.Sprintf("[ElevenLabsUnlim] Balance updated: %.0f tokens", balance))
	}
	return balance, nil
}

// SaveElevenLabsUnlimAPIKey saves API key
func (a *App) SaveElevenLabsUnlimAPIKey(apiKey string) error {
	return a.elevenLabsUnlim.SaveAPIKey(apiKey)
}

// GetElevenLabsUnlimAPIKey gets API key
func (a *App) GetElevenLabsUnlimAPIKey() string {
	return a.elevenLabsUnlim.GetAPIKey()
}

// GetElevenLabsUnlimKeys returns the list of named API keys
func (a *App) GetElevenLabsUnlimKeys() []utils.NamedAPIKey {
	return a.settings.GetElevenLabsUnlimKeys()
}

// SaveElevenLabsUnlimKeys saves the list of named API keys
func (a *App) SaveElevenLabsUnlimKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetElevenLabsUnlimKeys(keys)
}

// VoiceMaker Methods

// GetVoiceMakerBalance returns the user's balance from VoiceMaker (via test request)
func (a *App) GetVoiceMakerBalance(apiKey string) (float64, error) {
	balance, err := a.voiceMaker.GetBalance(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[VoiceMaker] Balance check failed: %v", err))
		return 0, err
	}
	a.LogToUI("SUCCESS", fmt.Sprintf("[VoiceMaker] Balance updated: %.0f units", balance))
	return balance, nil
}

// SaveVoiceMakerAPIKey saves API key
func (a *App) SaveVoiceMakerAPIKey(apiKey string) error {
	return a.voiceMaker.SaveAPIKey(apiKey)
}

// GetVoiceMakerAPIKey gets API key
func (a *App) GetVoiceMakerAPIKey() string {
	return a.voiceMaker.GetAPIKey()
}

// GetVoiceMakerKeys returns the list of named API keys
func (a *App) GetVoiceMakerKeys() []utils.NamedAPIKey {
	return a.settings.GetVoiceMakerKeys()
}

// SaveVoiceMakerKeys saves the list of named API keys
func (a *App) SaveVoiceMakerKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetVoiceMakerKeys(keys)
}

// GetVoiceMakerVoices returns the list of voices for a given key
func (a *App) GetVoiceMakerVoices(apiKey string) ([]api.VoicemakerVoice, error) {
	voices, err := a.voiceMaker.GetVoicesList(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[VoiceMaker] Failed to fetch voices: %v", err))
		return nil, err
	}
	a.LogToUI("SUCCESS", fmt.Sprintf("[VoiceMaker] Successfully fetched %d voices", len(voices)))
	return voices, nil
}

// Googler Methods

// GetGooglerUsage returns account usage stats
func (a *App) GetGooglerUsage(apiKey string) (*api.GooglerUsageResponse, error) {
	usage, err := a.googler.GetUsage(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[Googler] Usage check failed: %v", err))
		return nil, err
	}
	a.LogToUI("SUCCESS", "[Googler] API Status: Online (Usage data received)")
	return usage, nil
}

// SaveGooglerAPIKey saves API key
func (a *App) SaveGooglerAPIKey(apiKey string) error {
	return a.googler.SaveAPIKey(apiKey)
}

// GetGooglerAPIKey gets API key
func (a *App) GetGooglerAPIKey() string {
	return a.googler.GetAPIKey()
}

// ElevenLabsImage Methods

// SaveElevenLabsImageAPIKey saves API key
func (a *App) SaveElevenLabsImageAPIKey(apiKey string) error {
	return a.elevenLabsImage.SaveAPIKey(apiKey)
}

// GetElevenLabsImageAPIKey gets API key
func (a *App) GetElevenLabsImageAPIKey() string {
	return a.elevenLabsImage.GetAPIKey()
}

// GetElevenLabsImageKeys returns the list of named API keys
func (a *App) GetElevenLabsImageKeys() []utils.NamedAPIKey {
	return a.settings.GetElevenLabsImageKeys()
}

// SaveElevenLabsImageKeys saves the list of named API keys
func (a *App) SaveElevenLabsImageKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetElevenLabsImageKeys(keys)
}

// GetElevenLabsImageMaxConnections повертає ліміт одночасних запитів ElevenLabs Image
func (a *App) GetElevenLabsImageMaxConnections() int {
	return a.settings.GetElevenLabsImageMaxConnections()
}

// SaveElevenLabsImageMaxConnections встановлює ліміт одночасних запитів ElevenLabs Image
func (a *App) SaveElevenLabsImageMaxConnections(max int) error {
	return a.settings.SetElevenLabsImageMaxConnections(max)
}

// GetElevenLabsImageUsage повертає статистику використання ElevenLabs Image
func (a *App) GetElevenLabsImageUsage() api.ElevenLabsImageUsage {
	return a.elevenLabsImage.GetUsage()
}

// ElevenLabsUA Methods

// GetElevenLabsUABalance returns the user's balance from ElevenLabsUA
func (a *App) GetElevenLabsUABalance(apiKey string) (float64, error) {
	balance, err := a.elevenLabsUA.GetBalance(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[ElevenLabsUA] Balance check failed: %v", err))
		return 0, err
	}
	a.LogToUI("SUCCESS", fmt.Sprintf("[ElevenLabsUA] Balance updated: %.0f characters", balance))
	return balance, nil
}

// SaveElevenLabsUAAPIKey saves API key
func (a *App) SaveElevenLabsUAAPIKey(apiKey string) error {
	return a.elevenLabsUA.SaveAPIKey(apiKey)
}

// GetElevenLabsUAAPIKey gets API key
func (a *App) GetElevenLabsUAAPIKey() string {
	return a.elevenLabsUA.GetAPIKey()
}

// GetElevenLabsUAKeys returns the list of named API keys
func (a *App) GetElevenLabsUAKeys() []utils.NamedAPIKey {
	return a.settings.GetElevenLabsUAKeys()
}

// SaveElevenLabsUAKeys saves the list of named API keys
func (a *App) SaveElevenLabsUAKeys(keys []utils.NamedAPIKey) error {
	return a.settings.SetElevenLabsUAKeys(keys)
}

// GetElevenLabsUAAlertThreshold gets alert threshold
func (a *App) GetElevenLabsUAAlertThreshold() float64 {
	return a.settings.GetElevenLabsUAAlertThreshold()
}

// SaveElevenLabsUAAlertThreshold saves alert threshold
func (a *App) SaveElevenLabsUAAlertThreshold(threshold float64) error {
	return a.settings.SetElevenLabsUAAlertThreshold(threshold)
}

// AssemblyAI Methods

// CheckAssemblyAIConnection checks if the API key is valid
func (a *App) CheckAssemblyAIConnection(apiKey string) error {
	err := a.assemblyAI.CheckConnection(apiKey)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[AssemblyAI] Connection failed: %v", err))
		return err
	}
	a.LogToUI("SUCCESS", "[AssemblyAI] Connection successful")
	return nil
}

// SaveAssemblyAIAPIKey saves API key
func (a *App) SaveAssemblyAIAPIKey(apiKey string) error {
	return a.assemblyAI.SaveAPIKey(apiKey)
}

// GetAssemblyAIAPIKey gets API key
func (a *App) GetAssemblyAIAPIKey() string {
	return a.assemblyAI.GetAPIKey()
}

// Threshold Methods

// GetElevenLabsBotAlertThreshold gets alert threshold
func (a *App) GetElevenLabsBotAlertThreshold() float64 {
	return a.settings.GetElevenLabsBotAlertThreshold()
}

// SaveElevenLabsBotAlertThreshold saves alert threshold
func (a *App) SaveElevenLabsBotAlertThreshold(threshold float64) error {
	return a.settings.SetElevenLabsBotAlertThreshold(threshold)
}

// GetElevenLabsUnlimAlertThreshold gets alert threshold
func (a *App) GetElevenLabsUnlimAlertThreshold() float64 {
	return a.settings.GetElevenLabsUnlimAlertThreshold()
}

// SaveElevenLabsUnlimAlertThreshold saves alert threshold
func (a *App) SaveElevenLabsUnlimAlertThreshold(threshold float64) error {
	return a.settings.SetElevenLabsUnlimAlertThreshold(threshold)
}

// GetVoiceMakerAlertThreshold gets alert threshold
func (a *App) GetVoiceMakerAlertThreshold() float64 {
	return a.settings.GetVoiceMakerAlertThreshold()
}

// SaveVoiceMakerAlertThreshold saves alert threshold
func (a *App) SaveVoiceMakerAlertThreshold(threshold float64) error {
	return a.settings.SetVoiceMakerAlertThreshold(threshold)
}

// GetOpenRouterAlertThreshold gets alert threshold
func (a *App) GetOpenRouterAlertThreshold() float64 {
	return a.settings.GetOpenRouterAlertThreshold()
}

// SaveOpenRouterAlertThreshold saves alert threshold
func (a *App) SaveOpenRouterAlertThreshold(threshold float64) error {
	return a.settings.SetOpenRouterAlertThreshold(threshold)
}

// GetOpenRouterMaxConnections повертає ліміт одночасних запитів
func (a *App) GetOpenRouterMaxConnections() int {
	return a.settings.GetOpenRouterMaxConnections()
}

// SaveOpenRouterMaxConnections встановлює ліміт одночасних запитів
func (a *App) SaveOpenRouterMaxConnections(max int) error {
	return a.settings.SetOpenRouterMaxConnections(max)
}

// GetGooglerVideoAlertThreshold gets alert threshold
func (a *App) GetGooglerVideoAlertThreshold() float64 {
	return a.settings.GetGooglerVideoAlertThreshold()
}

// SaveGooglerVideoAlertThreshold saves alert threshold
func (a *App) SaveGooglerVideoAlertThreshold(threshold float64) error {
	return a.settings.SetGooglerVideoAlertThreshold(threshold)
}

// GetGooglerImageAlertThreshold gets alert threshold
func (a *App) GetGooglerImageAlertThreshold() float64 {
	return a.settings.GetGooglerImageAlertThreshold()
}

// SaveGooglerImageAlertThreshold saves alert threshold
func (a *App) SaveGooglerImageAlertThreshold(threshold float64) error {
	return a.settings.SetGooglerImageAlertThreshold(threshold)
}

// Auth Methods

// ValidateKey validates the provided key against the manager bot
func (a *App) ValidateKey(key string) (*api.AuthResponse, error) {
	hwID := utils.GetHardwareID()
	return a.authService.ValidateKey(key, hwID)
}

// GetSavedAuthKey returns the saved Access Key
func (a *App) GetSavedAuthKey() string {
	return a.settings.GetAppAccessKey()
}

// SaveAuthKey saves the Access Key to settings
func (a *App) SaveAuthKey(key string) error {
	return a.settings.SetAppAccessKey(key)
}

// ClearAuthKey clears the Access Key from settings
func (a *App) ClearAuthKey() error {
	return a.settings.SetAppAccessKey("")
}

// GetMyHardwareID повертає Hardware ID цього ПК для ідентифікації
func (a *App) GetMyHardwareID() string {
	return utils.GetHardwareID()
}

// GetGooglerMaxImageConnections повертає ліміт одночасних запитів Googler (Image)
func (a *App) GetGooglerMaxImageConnections() int {
	return a.settings.GetGooglerMaxImageConnections()
}

// SaveGooglerMaxImageConnections встановлює ліміт одночасних запитів Googler (Image)
func (a *App) SaveGooglerMaxImageConnections(max int) error {
	return a.settings.SetGooglerMaxImageConnections(max)
}

// GetGooglerMaxVideoConnections повертає ліміт одночасних запитів Googler (Video)
func (a *App) GetGooglerMaxVideoConnections() int {
	return a.settings.GetGooglerMaxVideoConnections()
}

// SaveGooglerMaxVideoConnections встановлює ліміт одночасних запитів Googler (Video)
func (a *App) SaveGooglerMaxVideoConnections(max int) error {
	return a.settings.SetGooglerMaxVideoConnections(max)
}

// Pipeline Methods

// GetPipelineSettings returns pipeline configuration
func (a *App) GetPipelineSettings() utils.PipelineSettings {
	return a.settings.GetPipelineSettings()
}

// SavePipelineSettings saves pipeline configuration
func (a *App) SavePipelineSettings(pipeline utils.PipelineSettings) error {
	return a.settings.SavePipelineSettings(pipeline)
}

// GetElevenLabsBotVoiceTemplates returns the list of voice templates for a given API key
func (a *App) GetElevenLabsBotVoiceTemplates(apiKey string) ([]api.VoiceTemplate, error) {
	return a.elevenLabs.GetTemplates(apiKey)
}

// GetEdgeTTSVoices returns the list of available Microsoft Edge TTS voices
func (a *App) GetEdgeTTSVoices() ([]api.EdgeTTSVoice, error) {
	return a.edgeTTS.GetVoices()
}

// AmdWhisper Methods

// IsAmdWhisperInstalled checks if the binary is already extracted
func (a *App) IsAmdWhisperInstalled() bool {
	return a.amdWhisper.IsInstalled()
}

// InstallAmdWhisper extracts the whisper-amd.zip
func (a *App) InstallAmdWhisper() error {
	a.LogToUI("INFO", "[AmdWhisper] Початок інсталяції сервісу AMD...")
	err := a.amdWhisper.Install()
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[AmdWhisper] Помилка інсталяції: %v", err))
		return err
	}
	a.LogToUI("SUCCESS", "[AmdWhisper] Сервіс AMD успішно встановлено")
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "amdInstalled")
	}
	return nil
}

// GetAmdWhisperModels returns the list of available models for AMD Whisper
func (a *App) GetAmdWhisperModels() ([]string, error) {
	return a.amdWhisper.GetAvailableModels()
}

// ProcessTask handles the execution of a single pipeline task
func (a *App) ProcessTask(id string, taskNumber int, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	return a.pipeline.ProcessTask(id, taskNumber, taskType, content, settings, taskName, subName)
}

// SubmitControlResult resumes a paused task with edited text (legacy/simple confirm)
func (a *App) SubmitControlResult(taskId string, content string) {
	a.pipeline.SubmitControlResult(taskId, content)
}

// SendControlAction sends a complex action to a paused task
func (a *App) SendControlAction(id string, action string, text string, settings map[string]interface{}) {
	a.pipeline.SubmitControlAction(id, &pipeline.ControlAction{
		Action:   action,
		Text:     text,
		Settings: settings,
	})
}

// CancelQueue cancels all currently running pipeline tasks
func (a *App) CancelQueue() {
	a.pipeline.CancelProcessing()
	a.LogToUI("WARN", "[Queue] Queue cancellation requested. All pending tasks will fail.")
}

// ResetQueueCancellation allows starting new tasks after a previous cancellation
func (a *App) ResetQueueCancellation() {
	a.pipeline.ResetCancellation()
}

// CheckExistingTask checks if a folder already exists and contains relevant files
// CheckExistingTasks checks multiple tasks (usually from multiple templates) for existing files
func (a *App) CheckExistingTasks(tasks []map[string]interface{}) ([]pipeline.ExistingFilesData, error) {
	results := make([]pipeline.ExistingFilesData, 0)
	var mu sync.Mutex
	var wg sync.WaitGroup

	for _, t := range tasks {
		wg.Add(1)
		go func(taskData map[string]interface{}) {
			defer wg.Done()

			taskName, _ := taskData["taskName"].(string)
			taskType, _ := taskData["taskType"].(string)
			subName, _ := taskData["subName"].(string)
			settings, _ := taskData["settings"].(map[string]interface{})

			if taskName == "" {
				return
			}

			// We use ResolveFinalDir which now handles backward compatibility more efficiently
			finalDir := a.pipeline.ResolveFinalDir(taskName, taskType, subName, settings)

			if _, err := os.Stat(finalDir); err == nil {
				// For the UI check, we DO want full info (skipExtra = false)
				// but since we are running in parallel, it will be much faster.
				data := a.pipeline.CheckExistingFiles("check", finalDir, taskType, settings, false)
				if len(data.FoundStages) > 0 {
					data.ID = subName
					mu.Lock()
					results = append(results, data)
					mu.Unlock()
				}
			}
		}(t)
	}
	wg.Wait()

	if len(results) > 0 {
		return results, nil
	}
	return nil, nil
}

func (a *App) CheckExistingTask(taskName string, taskType string, settings map[string]interface{}, subName string) (*pipeline.ExistingFilesData, error) {
	if taskName == "" {
		return nil, nil
	}

	finalDir := a.pipeline.ResolveFinalDir(taskName, taskType, subName, settings)
	a.LogToUI("INFO", fmt.Sprintf("[Check] Checking directory: %s", finalDir))

	if _, err := os.Stat(finalDir); err == nil {
		data := a.pipeline.CheckExistingFiles("check", finalDir, taskType, settings, false)
		a.LogToUI("INFO", fmt.Sprintf("[Check] Found stages: %v", data.FoundStages))
		if len(data.FoundStages) > 0 {
			return &data, nil
		}
	} else {
		a.LogToUI("INFO", "[Check] Directory does not exist")
	}
	return nil, nil
}

// ResolveTaskDir returns the final directory for a task
func (a *App) ResolveTaskDir(taskName string, taskType string, subName string, settings map[string]interface{}) string {
	return a.pipeline.ResolveFinalDir(taskName, taskType, subName, settings)
}

// SubmitImageControlResult resumes a paused task after image review
func (a *App) SubmitImageControlResult(taskId string) {
	a.pipeline.SubmitImageControlResult(taskId)
}

// SubmitMontageControlResult resumes a paused task after montage review
func (a *App) SubmitMontageControlResult(taskId string, result string) {
	a.pipeline.SubmitMontageControlResult(taskId, result)
}

// SubmitExistingFilesResult resumes a task after existing files check
func (a *App) SubmitExistingFilesResult(id string, skipStages []string) {
	a.pipeline.SubmitExistingFilesResult(id, skipStages)
}

// PrepareMontageBatch prepares the synchronization for a batch of tasks with montage control
func (a *App) PrepareMontageBatch(taskIDs []string) {
	a.pipeline.PrepareMontageBatch(taskIDs)
}

// GetGalleryImages scans output directories and returns gallery data
func (a *App) GetGalleryImages() []utils.GalleryTask {
	return a.galleryManager.GetGalleryData()
}

// RegenerateGalleryImage regenerates a single image in the gallery
func (a *App) RegenerateGalleryImage(imgPath string, prompt string, service string, settings map[string]interface{}) (string, error) {
	return a.pipeline.RegenerateImage(imgPath, prompt, service, settings)
}

// DeleteGalleryImage removes an image from session memory and deletes the file from disk
func (a *App) DeleteGalleryImage(imgPath string) bool {
	// 1. Remove from Memory
	a.galleryManager.RemoveImage(imgPath)

	// 2. Remove from Disk
	err := os.Remove(imgPath)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[Gallery] Failed to delete file: %v", err))
		return false
	}

	a.LogToUI("SUCCESS", fmt.Sprintf("[Gallery] Image deleted from disk: %s", filepath.Base(imgPath)))
	wruntime.EventsEmit(a.ctx, "galleryUpdate")
	return true
}

// ClearGallery clears all images from memory
func (a *App) ClearGallery() {
	a.galleryManager.Clear()
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "galleryUpdate")
	}
}

// DeleteGalleryImages removes multiple images
func (a *App) DeleteGalleryImages(imgPaths []string) int {
	successCount := 0
	for _, path := range imgPaths {
		if a.DeleteGalleryImage(path) {
			successCount++
		}
	}
	if successCount > 0 {
		wruntime.EventsEmit(a.ctx, "galleryUpdate")
	}
	return successCount
}

// SelectImage opens a file dialog to select an image file
func (a *App) SelectImage() (string, error) {
	return wruntime.OpenFileDialog(a.ctx, wruntime.OpenDialogOptions{
		Title: "Select Reference Image",
		Filters: []wruntime.FileFilter{
			{DisplayName: "Images", Pattern: "*.jpg;*.jpeg;*.png;*.webp"},
		},
	})
}

// SelectVideo opens a file dialog to select a video file
func (a *App) SelectVideo() (string, error) {
	return wruntime.OpenFileDialog(a.ctx, wruntime.OpenDialogOptions{
		Title: "Select Video File",
		Filters: []wruntime.FileFilter{
			{DisplayName: "Videos", Pattern: "*.mp4;*.mov;*.avi;*.mkv;*.webm"},
		},
	})
}

// AddToHistory adds a new entry to the task history
func (a *App) AddToHistory(name string, taskType string, templates []string, content string) error {
	err := a.history.AddEntry(name, taskType, templates, content)
	if err == nil && a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "historyUpdate")
	}
	return err
}

// GetHistory returns the task history (last 2 days)
func (a *App) GetHistory() ([]utils.HistoryEntry, error) {
	return a.history.GetHistory()
}

// Full History Methods (30 days)

func (a *App) GetFullHistory() ([]utils.HistoryMetadata, error) {
	return a.fullHistory.GetEntries()
}

func (a *App) GetFullHistoryEntry(id string) (*utils.FullHistoryEntry, error) {
	return a.fullHistory.GetEntry(id)
}

func (a *App) DeleteFullHistoryEntry(id string) error {
	return a.fullHistory.DeleteEntry(id)
}

func (a *App) AddFullHistoryEntry(name string, taskType string, templates []string, stages []string, original string, processed string, duration float64) error {
	return a.fullHistory.AddEntry(name, taskType, templates, stages, original, processed, duration)
}

// GetImageAsBase64 returns base64 content of an image file for preview
func (a *App) GetImageAsBase64(path string) (string, error) {
	return utils.GetImageAsBase64(path)
}

// CheckSubtitleModel checks if local model exists
func (a *App) CheckSubtitleModel(modelName string) bool {
	return a.localWhisper.CheckModel(modelName)
}

// DownloadSubtitleModel downloads the local model
func (a *App) DownloadSubtitleModel(modelName string) error {
	a.localWhisper.SetContext(a.ctx)
	_, err := a.localWhisper.GetModelPath(modelName)
	return err
}
func (a *App) GetProductionStats(days int) *utils.UIStatsResponse {
	return a.productionStats.GetStats(days)
}

func (a *App) ClearProductionStats() {
	a.productionStats.ClearData()
}

// Google Parser Methods

func (a *App) GetGoogleSheetURL() string {
	return a.settings.GetGoogleSheetURL()
}

func (a *App) SaveGoogleSheetURL(url string) error {
	return a.settings.SetGoogleSheetURL(url)
}

func (a *App) GetGoogleFilter() string {
	return a.settings.GetGoogleFilter()
}

func (a *App) SaveGoogleFilter(filter string) error {
	return a.settings.SetGoogleFilter(filter)
}

func (a *App) GetGoogleMonitorMappings() []utils.GoogleMonitorMapping {
	return a.settings.GetGoogleMonitorMappings()
}

func (a *App) SaveGoogleMonitorMappings(mappings []utils.GoogleMonitorMapping) error {
	return a.settings.SetGoogleMonitorMappings(mappings)
}

func (a *App) GetGoogleMonitorDisplayColumns() []string {
	return a.settings.GetGoogleMonitorDisplayColumns()
}

func (a *App) SaveGoogleMonitorDisplayColumns(columns []string) error {
	return a.settings.SetGoogleMonitorDisplayColumns(columns)
}

func (a *App) GetGoogleMonitorTaskNameColumn() string {
	return a.settings.GetGoogleMonitorTaskNameColumn()
}

func (a *App) SaveGoogleMonitorTaskNameColumn(column string) error {
	return a.settings.SetGoogleMonitorTaskNameColumn(column)
}

func (a *App) GetGoogleSheets() []utils.GoogleSheetConfig {
	return a.settings.GetGoogleSheets()
}

func (a *App) SaveGoogleSheets(sheets []utils.GoogleSheetConfig) error {
	return a.settings.SetGoogleSheets(sheets)
}

type MultiSheetResult struct {
	ID      string                `json:"id"`
	Name    string                `json:"name"`
	Results []api.GoogleParserRow `json:"results"`
	Error   string                `json:"error,omitempty"`
}

func (a *App) ParseAllGoogleSheets() ([]MultiSheetResult, error) {
	configs := a.settings.GetGoogleSheets()
	results := make([]MultiSheetResult, len(configs))
	var wg sync.WaitGroup

	for i, cfg := range configs {
		if cfg.URL == "" {
			results[i] = MultiSheetResult{ID: cfg.ID, Name: cfg.Name}
			continue
		}

		wg.Add(1)
		go func(idx int, c utils.GoogleSheetConfig) {
			defer wg.Done()
			res, err := a.googleParser.ParseWithFilter(c.URL, c.Filter, c.IgnoreRows)
			errStr := ""
			if err != nil {
				errStr = err.Error()
				a.LogToUI("ERROR", fmt.Sprintf("[Google] Parsing failed for %s: %v", c.Name, err), "google", "Google Parser")
			} else {
				a.LogToUI("SUCCESS", fmt.Sprintf("[Google] Parsed %s: found %d items", c.Name, len(res)), "google", "Google Parser")
			}

			results[idx] = MultiSheetResult{
				ID:      c.ID,
				Name:    c.Name,
				Results: res,
				Error:   errStr,
			}
		}(i, cfg)
	}

	wg.Wait()
	return results, nil
}

func (a *App) ParseGoogleSheet(cfg utils.GoogleSheetConfig) ([]api.GoogleParserRow, error) {
	if cfg.URL == "" {
		return nil, fmt.Errorf("Google Sheet URL is not configured")
	}

	a.LogToUI("INFO", fmt.Sprintf("[Google] Testing table: %s (Filter: %s)", cfg.Name, cfg.Filter), "google", "Google Parser")
	results, err := a.googleParser.ParseWithFilter(cfg.URL, cfg.Filter, cfg.IgnoreRows)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[Google] Parsing failed: %v", err), "google", "Google Parser")
		return nil, err
	}

	return results, nil
}

func (a *App) FetchGoogleDocContent(url string) (string, error) {
	if url == "" {
		return "", nil
	}
	return a.googleParser.FetchDoc(url)
}

// Telegram Notifications Methods

func (a *App) GetTelegramNotificationsEnabled() bool {
	return a.settings.GetTelegramNotificationsEnabled()
}

func (a *App) SaveTelegramNotificationsEnabled(enabled bool) error {
	return a.settings.SetTelegramNotificationsEnabled(enabled)
}

func (a *App) GetTelegramChatID() string {
	return a.settings.GetTelegramChatID()
}

func (a *App) SaveTelegramChatID(chatID string) error {
	return a.settings.SetTelegramChatID(chatID)
}

func (a *App) SendTelegramNotification(chatID string, text string) error {
	return a.telegramService.SendNotification(chatID, text)
}

func (a *App) TestTelegramNotification(chatID string) error {
	return a.telegramService.SendNotification(chatID, "🔔 *Тестове сповіщення*\n\nСповіщення від Soloveyko.AI Video Maker успішно налаштовані!")
}

// System Notifications Methods

func (a *App) GetSystemNotificationsEnabled() bool {
	return a.settings.GetSystemNotificationsEnabled()
}

func (a *App) SaveSystemNotificationsEnabled(enabled bool) error {
	return a.settings.SetSystemNotificationsEnabled(enabled)
}

func (a *App) getIconPath() string {
	// Possible locations for the icon
	paths := []string{
		filepath.Join("build", "windows", "icon.ico"),
		"icon.ico",
		"icon.png",
	}

	for _, p := range paths {
		if _, err := os.Stat(p); err == nil {
			abs, _ := filepath.Abs(p)
			return abs
		}
	}

	return ""
}

// SendSystemNotification sends a native notification using the operating system's API
func (a *App) SendSystemNotification(title, text string) error {
	if !a.GetSystemNotificationsEnabled() {
		return nil
	}

	// Remove markdown-like bold (**) if any
	cleanTitle := strings.ReplaceAll(title, "**", "")
	cleanTitle = strings.ReplaceAll(cleanTitle, "*", "")

	cleanText := strings.ReplaceAll(text, "**", "")
	cleanText = strings.ReplaceAll(cleanText, "*", "")

	// Providing the icon path helps identity the application name on Windows
	return beeep.Notify(cleanTitle, cleanText, a.getIconPath())
}

func (a *App) TestSystemNotification() error {
	return a.SendSystemNotification("System Notifications", "🔔 Тестове сповіщення успішно налаштоване!")
}

// WhisperX Management Methods

// IsWhisperXInstalled checks if WhisperX is installed in the user's bin folder
func (a *App) IsWhisperXInstalled() bool {
	configDir := a.settings.GetConfigDir()
	binDir := filepath.Join(configDir, "bin")

	folderName := "whisperx-win"
	if runtime.GOOS == "darwin" {
		folderName = "whisperx-mac"
	}

	targetDir := filepath.Join(binDir, folderName)
	if _, err := os.Stat(targetDir); err == nil {
		return true
	}
	return false
}

// DownloadWhisperX downloads and installs WhisperX engine
func (a *App) DownloadWhisperX() error {
	url := "https://github.com/kvxshenyjbetxn/video.maker.releases/releases/download/whisperx/whisperx-win.zip"
	if runtime.GOOS == "darwin" {
		url = "https://github.com/kvxshenyjbetxn/video.maker.releases/releases/download/whisperx/whisperx-mac.zip"
	}

	a.LogToUI("INFO", "[WhisperX] Starting engine download...")

	progressChan := make(chan int)
	go func() {
		for progress := range progressChan {
			if a.ctx != nil {
				wruntime.EventsEmit(a.ctx, "whisperxDownloadProgress", progress)
			}
		}
	}()

	pkgPath, err := a.updater.Download(url, progressChan)
	close(progressChan) // Close after download finishes
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[WhisperX] Download failed: %v", err))
		return err
	}
	defer os.Remove(pkgPath)

	a.LogToUI("INFO", "[WhisperX] Extracting engine...")

	configDir := a.settings.GetConfigDir()
	binDir := filepath.Join(configDir, "bin")
	os.MkdirAll(binDir, 0755)

	err = a.updater.Unzip(pkgPath, binDir)
	if err != nil {
		a.LogToUI("ERROR", fmt.Sprintf("[WhisperX] Extraction failed: %v", err))
		return err
	}

	// For Mac, ensure binary is executable
	if runtime.GOOS == "darwin" {
		exePath := filepath.Join(binDir, "whisperx-mac", "whisperx_cli")
		os.Chmod(exePath, 0755)
	}

	a.LogToUI("SUCCESS", "[WhisperX] Engine installed successfully!")
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "whisperxInstalled")
	}

	return nil
}

type ImportMediaData struct {
	Path           string  `json:"path"`
	Duration       float64 `json:"duration"`
	IsVideo        bool    `json:"isVideo"`
	ActualDuration float64 `json:"actualDuration"`
}

func (a *App) getMediaMetadata(targetPath string, ext string) (*ImportMediaData, error) {
	videoExts := map[string]bool{".mp4": true, ".mkv": true, ".mov": true, ".avi": true, ".webm": true}
	imageExts := map[string]bool{".jpg": true, ".jpeg": true, ".png": true, ".webp": true}

	isVideo := videoExts[ext]
	isImage := imageExts[ext]

	if !isVideo && !isImage {
		return nil, fmt.Errorf("unsupported file type: %s", ext)
	}

	data := &ImportMediaData{
		Path:    targetPath,
		IsVideo: isVideo,
	}

	if isVideo {
		ffprobePath, _ := utils.EnsureEngine("ffprobe")
		if ffprobePath != "" {
			dur, _ := a.getFFprobeDuration(ffprobePath, targetPath)
			data.Duration = dur
			data.ActualDuration = dur
		}
	} else {
		data.Duration = 2.0
		data.ActualDuration = 0
	}

	a.LogToUI("SUCCESS", fmt.Sprintf("[Import] Metadata detected: %s", filepath.Base(targetPath)))
	return data, nil
}

func (a *App) ImportMediaFile(taskID string, taskName string, taskType string, subName string, settings map[string]interface{}, sourcePath string) (*ImportMediaData, error) {
	if sourcePath == "" {
		return nil, fmt.Errorf("source path is empty")
	}

	if _, err := os.Stat(sourcePath); err != nil {
		return nil, fmt.Errorf("source file not found: %v", err)
	}

	finalDir := a.pipeline.ResolveFinalDir(taskName, taskType, subName, settings)
	importDir := filepath.Join(finalDir, "imports")
	if _, err := os.Stat(importDir); os.IsNotExist(err) {
		os.MkdirAll(importDir, 0755)
	}

	ext := strings.ToLower(filepath.Ext(sourcePath))
	fileName := fmt.Sprintf("import_%d%s", time.Now().UnixNano(), ext)
	targetPath := filepath.Join(importDir, fileName)

	sourceFile, err := os.Open(sourcePath)
	if err != nil {
		return nil, err
	}
	defer sourceFile.Close()

	destFile, err := os.Create(targetPath)
	if err != nil {
		return nil, err
	}
	defer destFile.Close()

	_, err = io.Copy(destFile, sourceFile)
	if err != nil {
		return nil, err
	}

	return a.getMediaMetadata(targetPath, ext)
}

func (a *App) getFFprobeDuration(ffprobePath, path string) (float64, error) {
	cmd := exec.Command(ffprobePath, "-v", "error", "-show_entries", "format=duration",
		"-of", "default=noprint_wrappers=1:nokey=1", path)
	utils.PrepareHiddenCmd(cmd)

	out, err := cmd.Output()
	if err != nil {
		return 0, err
	}
	var dur float64
	_, err = fmt.Sscanf(strings.TrimSpace(string(out)), "%f", &dur)
	return dur, err
}

// SelectFiles opens a native file dialog and returns selected paths
func (a *App) SelectFiles() ([]string, error) {
	return wruntime.OpenMultipleFilesDialog(a.ctx, wruntime.OpenDialogOptions{
		Title: "Select Media Files",
		Filters: []wruntime.FileFilter{
			{DisplayName: "Media Files", Pattern: "*.jpg;*.jpeg;*.png;*.webp;*.mp4;*.mkv;*.mov;*.avi;*.webm"},
		},
	})
}

func (a *App) ReadFile(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(data), nil
}
