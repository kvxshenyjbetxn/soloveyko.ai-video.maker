package main

import (
	"context"
	"os/exec"
	"runtime"
	"soloveyko/backend/utils"
)

// App struct
type App struct {
	ctx      context.Context
	settings *utils.SettingsService
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{
		settings: utils.NewSettingsService(),
	}
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
