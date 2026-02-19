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
	orService := api.NewOpenRouterService(settings)
	app := &App{
		settings:        settings,
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

	orService.OnRequestStart = func(id string, taskLabel string, taskType string, keyName string, model string, temp float64, tokens int) {
		app.LogToUI("INFO", fmt.Sprintf("[OpenRouter] [%s] Request | Key: %s | Model: %s | Temp: %.2f | Max Tokens: %v", strings.Title(taskType), keyName, model, temp, tokens), id, taskLabel)
		// Емітуємо подію, щоб фронтенд знав, що завдання ДІЙСНО почало обробку
		if app.ctx != nil {
			wruntime.EventsEmit(app.ctx, "taskStatus", id, "processing", 10)
		}
	}

	app.elevenLabs.OnLog = func(level string, message string, details ...string) {
		app.LogToUI(level, message, details...)
	}

	app.elevenLabsUA.OnLog = func(level string, message string) {
		app.LogToUI(level, message)
	}

	return app
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
func (a *App) LogToUI(level string, message string, details ...string) {
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

// EmitStageStatus emits a stage status event to the frontend
func (a *App) EmitStageStatus(id string, stage string, status string) {
	if a.ctx != nil {
		wruntime.EventsEmit(a.ctx, "stageStatus", id, stage, status)
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

// ProcessTask handles the execution of a single pipeline task
func (a *App) ProcessTask(id string, taskNumber int, taskType string, content string, settings map[string]interface{}, taskName string, subName string) (string, error) {
	displayTaskName := taskName
	if len([]rune(displayTaskName)) > 10 {
		displayTaskName = string([]rune(displayTaskName)[:10]) + "..."
	}

	label := displayTaskName
	if subName != "" {
		label = fmt.Sprintf("%s - %s", displayTaskName, subName)
	}
	taskLabel := fmt.Sprintf("%s (#%d)", label, taskNumber)
	// 1. Get Pipeline Settings for Output Path and Keys
	pSettings := a.settings.GetPipelineSettings()

	var apiKey string
	var model, prompt string
	var temp, tokens float64
	var pipelineName string
	var outPath string

	if taskType == "translate" || taskType == "rewrite" || taskType == "voiceover" {
		// Get actual API Keys and Paths
		keyID, _ := settings[taskType+"OpenRouterKeyID"].(string)
		outPath, _ = settings[taskType+"OutputPath"].(string)

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

		// 1. Process Text (OpenRouter)
		var processedText string = content
		var orSuccess bool = false

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

		if shouldProcessText {
			// Handle OpenRouter Keys
			keys := a.settings.GetOpenRouterKeys()
			for _, k := range keys {
				if k.ID == keyID {
					apiKey = k.Key
					break
				}
			}
			if apiKey == "" && len(keys) > 0 {
				apiKey = keys[0].Key
			}

			if apiKey != "" {
				a.EmitStageStatus(id, "text", "running")
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

				var fullPrompt string
				if strings.Contains(prompt, "{{content}}") {
					fullPrompt = strings.ReplaceAll(prompt, "{{content}}", content)
				} else {
					fullPrompt = prompt + "\n\n" + content
				}

				result, err := a.openRouter.Chat(id, taskLabel, taskType, keyName, apiKey, model, fullPrompt, temp, int(tokens))
				if err != nil {
					a.LogToUI("ERROR", fmt.Sprintf("[OpenRouter] [%s] Error: %v", strings.Title(taskType), err), id, taskLabel)
					a.EmitStageStatus(id, "text", "failed")
					return "", err
				}
				processedText = result
				orSuccess = true
				a.LogToUI("SUCCESS", fmt.Sprintf("[OpenRouter] [%s] Success: Result received", strings.Title(taskType)), id, taskLabel)
				a.EmitStageStatus(id, "text", "completed")
			} else {
				a.LogToUI("WARN", fmt.Sprintf("[OpenRouter] [%s] API key not found, skipping text processing", strings.Title(taskType)), id, taskLabel)
				a.EmitStageStatus(id, "text", "completed") // Mark as completed even if skipped due to missing API key
			}
		} else {
			// Якщо етап тексту вимкнено - він одразу зелений (використовуємо оригінал)
			a.EmitStageStatus(id, "text", "completed")
		}

		// Determine Directory Structure
		templateDir := subName
		if templateDir == "" {
			templateDir = pipelineName
			if templateDir == "" {
				templateDir = "Default"
			}
		}
		finalDir := filepath.Join(outPath, taskName, templateDir)
		err := os.MkdirAll(finalDir, 0755)
		if err != nil {
			a.LogToUI("ERROR", fmt.Sprintf("[FileSystem] Failed to create directory: %v", err), id, taskLabel)
			return "", err
		}

		// Save Text Result (if processed or explicitly requested)
		if orSuccess || shouldProcessText || taskType != "voiceover" {
			fileName := "result.txt"
			switch taskType {
			case "translate":
				fileName = "translation.txt"
			case "rewrite":
				fileName = "rewrite.txt"
			}
			filePath := filepath.Join(finalDir, fileName)
			os.WriteFile(filePath, []byte(processedText), 0644)
		}

		// 2. Voiceover Stage
		var vEnabled bool
		if val, ok := settings["voiceoverEnabled"].(bool); ok {
			vEnabled = val
		} else {
			// If not in task settings, use global
			vEnabled = pSettings.VoiceoverEnabled
		}

		if vEnabled {
			vService, _ := settings["voiceoverService"].(string)
			if vService == "" {
				vService = pSettings.VoiceoverService
			}
			vTemplate, _ := settings["voiceoverTemplate"].(string)
			if vTemplate == "" {
				vTemplate = pSettings.VoiceoverTemplate
			}
			vKeyID, _ := settings["voiceoverElevenLabsBotKeyID"].(string)
			if vKeyID == "" {
				vKeyID = pSettings.VoiceoverElevenLabsBotKeyID
			}

			a.LogToUI("INFO", fmt.Sprintf("[Pipeline] Voiceover stage started. Service: %s, Template: %s", vService, vTemplate), id, taskLabel)

			if vService == "elevenlabsbot" {
				if vTemplate == "" {
					a.LogToUI("ERROR", "[ElevenLabsBot] Voice template is not selected!", id, taskLabel)
				} else {
					// Fetch API Key for Voiceover
					vApiKey := ""
					vKeys := a.settings.GetElevenLabsBotKeys()
					for _, k := range vKeys {
						if k.ID == vKeyID {
							vApiKey = k.Key
							break
						}
					}
					if vApiKey == "" && len(vKeys) > 0 {
						vApiKey = vKeys[0].Key
					}

					if vApiKey != "" {
						a.EmitStageStatus(id, "voice", "running")
						voiceFilePath := filepath.Join(finalDir, "voice.mp3")
						err := a.elevenLabs.Synthesize(vApiKey, processedText, vTemplate, voiceFilePath, id, taskLabel)
						if err != nil {
							a.LogToUI("ERROR", fmt.Sprintf("[ElevenLabsBot] Synthesis Error: %v", err), id, taskLabel)
							a.EmitStageStatus(id, "voice", "failed")
						} else {
							a.LogToUI("SUCCESS", "[ElevenLabsBot] Success: Voice saved to voice.mp3", id, taskLabel)
							a.EmitStageStatus(id, "voice", "completed")
						}
					} else {
						a.LogToUI("ERROR", "[ElevenLabsBot] API key not found for voiceover", id, taskLabel)
						a.EmitStageStatus(id, "voice", "failed")
					}
				}
			} else if vService != "" {
				a.LogToUI("WARN", fmt.Sprintf("[Pipeline] Service %s is not yet implemented for auto-synthesis", vService), id, taskLabel)
			} else {
				a.LogToUI("ERROR", "[Pipeline] Voiceover service is not selected!", id, taskLabel)
			}
		} else {
			a.LogToUI("INFO", "[Pipeline] Voiceover stage is disabled, skipping.", id, taskLabel)
		}

		return processedText, nil
	}

	return "", fmt.Errorf("task type %s not implemented", taskType)
}
