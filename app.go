package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"soloveyko/backend/api"
	"soloveyko/backend/pipeline"
	"soloveyko/backend/utils"
	"strings"

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
	}
	app.galleryManager = utils.NewGalleryManager()
	app.localWhisper = pipeline.NewLocalWhisperService()
	app.amdWhisper = pipeline.NewAmdWhisperService()
	app.edgeTTS = api.NewEdgeTTSService()
	app.history = utils.NewHistoryService()

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

	app.pipeline.OnImageGenerated = func(taskName, templateName, imageName, imgPath string) {
		app.galleryManager.AddImage(taskName, templateName, imageName, imgPath)
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
	app.pipeline.OnRequestExistingFilesCheck = func(data pipeline.ExistingFilesData) {
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "requestExistingFilesCheck", data)
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
		a.pipeline.UpdateSubtitleSemaphore(max)
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

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
	a.pipeline.SetContext(ctx)

	// Розпаковуємо всі бінарники одразу при старті в фоні (без блокування UI)
	go func() {
		utils.EnsureEngine("ffprobe")
		if a.localWhisper != nil {
			a.localWhisper.EnsureFFmpeg()
			a.localWhisper.EnsureWhisperCLI()
		}
	}()
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

// OpenConfigDir відкриває папку з конфігурацією в системному провіднику
func (a *App) OpenConfigDir() {
	path := a.settings.GetConfigDir()
	a.OpenPath(path)
}

// OpenPath opens the specified path in the system file explorer
func (a *App) OpenPath(path string) {
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
		cmd.Run()
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
func (a *App) GetElevenLabsBotVoiceTemplates(apiKey string) ([]string, error) {
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

// SubmitControlResult resumes a paused task with edited text
func (a *App) SubmitControlResult(taskId string, content string) {
	a.pipeline.SubmitControlResult(taskId, content)
}

// CheckExistingTask checks if a folder already exists and contains relevant files
// CheckExistingTasks checks multiple tasks (usually from multiple templates) for existing files
func (a *App) CheckExistingTasks(tasks []map[string]interface{}) ([]pipeline.ExistingFilesData, error) {
	results := make([]pipeline.ExistingFilesData, 0)
	for _, t := range tasks {
		taskName, _ := t["taskName"].(string)
		taskType, _ := t["taskType"].(string)
		subName, _ := t["subName"].(string)
		settings, _ := t["settings"].(map[string]interface{})

		if taskName == "" {
			continue
		}

		finalDir := a.pipeline.ResolveFinalDir(taskName, taskType, subName, settings)
		a.LogToUI("INFO", fmt.Sprintf("[Check] Checking directory for %s - %s: %s", taskName, subName, finalDir))

		if _, err := os.Stat(finalDir); err == nil {
			data := a.pipeline.CheckExistingFiles("check", finalDir, taskType)
			if len(data.FoundStages) > 0 {
				data.ID = subName // Use subName as ID to identify which template this is
				results = append(results, data)
			}
		}
	}

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
		data := a.pipeline.CheckExistingFiles("check", finalDir, taskType)
		a.LogToUI("INFO", fmt.Sprintf("[Check] Found stages: %v", data.FoundStages))
		if len(data.FoundStages) > 0 {
			return &data, nil
		}
	} else {
		a.LogToUI("INFO", "[Check] Directory does not exist")
	}
	return nil, nil
}

// SubmitImageControlResult resumes a paused task after image review
func (a *App) SubmitImageControlResult(taskId string) {
	a.pipeline.SubmitImageControlResult(taskId)
}

// SubmitExistingFilesResult resumes a task after existing files check
func (a *App) SubmitExistingFilesResult(id string, skipStages []string) {
	a.pipeline.SubmitExistingFilesResult(id, skipStages)
}

// GetGalleryImages scans output directories and returns gallery data
func (a *App) GetGalleryImages() []utils.GalleryTask {
	return a.galleryManager.GetGalleryData()
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
