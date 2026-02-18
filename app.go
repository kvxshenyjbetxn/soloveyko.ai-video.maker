package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"soloveyko/backend/api"
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
}

// NewApp creates a new App application struct
func NewApp() *App {
	settings := utils.NewSettingsService()
	return &App{
		settings:        settings,
		stats:           utils.NewStatsService(),
		openRouter:      api.NewOpenRouterService(settings),
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
}

// GetSystemStats повертає поточну статистику системи
func (a *App) GetSystemStats() (*utils.SystemStats, error) {
	return a.stats.GetSystemStats()
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

// LogToUI emits a log event to the frontend
func (a *App) LogToUI(level string, message string) {
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "log", level, message)
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
func (a *App) AddTemplate(tplType string, name string, data utils.PipelineSettings) (*utils.PipelineTemplate, error) {
	return a.templates.AddTemplate(tplType, name, data)
}

// DeleteTemplate видаляє шаблон пайплайну
func (a *App) DeleteTemplate(id string) error {
	return a.templates.DeleteTemplate(id)
}

// UpdateTemplate оновлює шаблон пайплайну
func (a *App) UpdateTemplate(id string, name string, data utils.PipelineSettings) error {
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

// SaveVoiceMakerBalance saves last known balance
func (a *App) SaveVoiceMakerBalance(balance float64) error {
	return a.settings.SetVoiceMakerBalance(balance)
}

// GetVoiceMakerSavedBalance gets last saved balance
func (a *App) GetVoiceMakerSavedBalance() float64 {
	return a.settings.GetVoiceMakerBalance()
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

// ElevenLabsUA Methods

// SaveElevenLabsUAAPIKey saves API key
func (a *App) SaveElevenLabsUAAPIKey(apiKey string) error {
	return a.elevenLabsUA.SaveAPIKey(apiKey)
}

// GetElevenLabsUAAPIKey gets API key
func (a *App) GetElevenLabsUAAPIKey() string {
	return a.elevenLabsUA.GetAPIKey()
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

// Pipeline Methods

// GetPipelineSettings returns pipeline configuration
func (a *App) GetPipelineSettings() utils.PipelineSettings {
	return a.settings.GetPipelineSettings()
}

// SavePipelineSettings saves pipeline configuration
func (a *App) SavePipelineSettings(pipeline utils.PipelineSettings) error {
	return a.settings.SavePipelineSettings(pipeline)
}

// ProcessTask handles the execution of a single pipeline task
func (a *App) ProcessTask(taskType string, content string, settings map[string]interface{}, taskName string) (string, error) {
	// 1. Get Pipeline Settings for Output Path and Keys
	pSettings := a.settings.GetPipelineSettings()

	var apiKey string
	var model, prompt string
	var temp, tokens float64
	var pipelineName string
	var outPath string

	if taskType == "translate" || taskType == "rewrite" {
		// Get actual API Key
		keyID, _ := settings[taskType+"OpenRouterKeyID"].(string)
		outPath, _ = settings["outputPath"].(string)

		if outPath == "" {
			outPath = pSettings.OutputPath
		}
		keys := a.settings.GetOpenRouterKeys()
		for _, k := range keys {
			if k.ID == keyID {
				apiKey = k.Key
				break
			}
		}

		if apiKey == "" && len(keys) > 0 {
			apiKey = keys[0].Key // Fallback to first key
		}

		if apiKey == "" {
			return "", fmt.Errorf("API key not found")
		}

		model, _ = settings[taskType+"Model"].(string)
		prompt, _ = settings[taskType+"Prompt"].(string)
		temp, _ = settings[taskType+"Temperature"].(float64)
		tokens, _ = settings[taskType+"MaxTokens"].(float64)
		pipelineName, _ = settings[taskType+"PipelineName"].(string)

		if pipelineName == "" {
			pipelineName = "Default"
		}

		keyName := "Default/First"
		for _, k := range keys {
			if k.ID == keyID {
				keyName = k.Name
				break
			}
		}

		// Log Request
		a.LogToUI("INFO", fmt.Sprintf("[OpenRouter] [%s] Request | Key: %s | Model: %s | Temp: %.2f | Max Tokens: %v", strings.Title(taskType), keyName, model, temp, tokens))

		var fullPrompt string
		if strings.Contains(prompt, "{{content}}") {
			fullPrompt = strings.ReplaceAll(prompt, "{{content}}", content)
		} else {
			fullPrompt = prompt + "\n\n" + content
		}

		result, err := a.openRouter.Chat(apiKey, model, fullPrompt, temp, int(tokens))
		if err != nil {
			a.LogToUI("ERROR", fmt.Sprintf("[OpenRouter] [%s] Error: %v", strings.Title(taskType), err))
			return "", err
		}

		// Log Result
		a.LogToUI("SUCCESS", fmt.Sprintf("[OpenRouter] [%s] Success: Result received", strings.Title(taskType)))

		// Save to file with new structure: OutputPath / TaskName / PipelineName / fileName
		if outPath != "" {
			finalDir := filepath.Join(outPath, taskName, pipelineName)
			err := os.MkdirAll(finalDir, 0755)
			if err == nil {
				fileName := "result.txt"
				if taskType == "translate" {
					fileName = "translation.txt"
				} else if taskType == "rewrite" {
					fileName = "rewrite.txt"
				}
				filePath := filepath.Join(finalDir, fileName)
				os.WriteFile(filePath, []byte(result), 0644)
			} else {
				a.LogToUI("ERROR", fmt.Sprintf("[FileSystem] Failed to create directory: %v", err))
			}
		}

		return result, nil
	}

	return "", fmt.Errorf("task type %s not implemented", taskType)
}
