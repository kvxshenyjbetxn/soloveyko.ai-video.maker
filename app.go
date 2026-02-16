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
	ctx          context.Context
	settings     *utils.SettingsService
	stats        *utils.StatsService
	openRouter   *api.OpenRouterService
	pollinations *api.PollinationsService
}

// NewApp creates a new App application struct
func NewApp() *App {
	settings := utils.NewSettingsService()
	return &App{
		settings:     settings,
		stats:        utils.NewStatsService(),
		openRouter:   api.NewOpenRouterService(settings),
		pollinations: api.NewPollinationsService(settings),
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
