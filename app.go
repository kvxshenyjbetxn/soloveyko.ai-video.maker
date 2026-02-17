package main

import (
	"context"
	"os/exec"
	"runtime"
	"soloveyko/backend/api"
	"soloveyko/backend/utils"
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
	var cmd *exec.Cmd

	switch runtime.GOOS {
	case "windows":
		cmd = exec.Command("explorer", path)
	case "darwin":
		cmd = exec.Command("open", path)
	default:
		// Для Linux (на випадок якщо знадобиться)
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

// OpenRouter Methods

// GetOpenRouterCredits returns the user's credits balance from OpenRouter
func (a *App) GetOpenRouterCredits(apiKey string) (float64, error) {
	return a.openRouter.GetOpenRouterCredits(apiKey)
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
	return a.elevenLabs.GetBalance(apiKey)
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
	return a.elevenLabsUnlim.GetBalance(apiKey)
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
	return a.voiceMaker.GetBalance(apiKey)
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
	return a.googler.GetUsage(apiKey)
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
	return a.assemblyAI.CheckConnection(apiKey)
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
