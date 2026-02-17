package utils

import (
	"encoding/json"
	"os"
	"path/filepath"
)

type Settings struct {
	Language            string   `json:"language"`
	Theme               string   `json:"theme"`
	AccentColor         string   `json:"accentColor"`
	OpenRouterAPIKey    string   `json:"openRouterAPIKey"`
	OpenRouterModels    []string `json:"openRouterModels"`
	PollinationsAPIKey  string   `json:"pollinationsAPIKey"`
	PollinationsModels  []string `json:"pollinationsModels"`
	ElevenLabsBotAPIKey string   `json:"elevenLabsBotAPIKey"`
}

type SettingsService struct {
	configPath string
}

func NewSettingsService() *SettingsService {
	// Отримуємо директорію конфігурації для користувача
	configDir, err := os.UserConfigDir()
	if err != nil {
		// Fallback на домашню директорію
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	// Створюємо шлях до папки програми
	appConfigDir := filepath.Join(configDir, "Soloveyko")

	// Створюємо директорію, якщо не існує
	os.MkdirAll(appConfigDir, 0755)

	return &SettingsService{
		configPath: filepath.Join(appConfigDir, "settings.json"),
	}
}

// LoadSettings завантажує налаштування з файлу
func (s *SettingsService) LoadSettings() (*Settings, error) {
	// Якщо файл не існує, повертаємо налаштування за замовчуванням
	if _, err := os.Stat(s.configPath); os.IsNotExist(err) {
		return &Settings{
			Language: "uk",
		}, nil
	}

	data, err := os.ReadFile(s.configPath)
	if err != nil {
		return nil, err
	}

	var settings Settings
	err = json.Unmarshal(data, &settings)
	if err != nil {
		return nil, err
	}

	// Дефолтні значення, якщо порожньо
	if settings.Theme == "" {
		settings.Theme = "dark"
	}
	if settings.AccentColor == "" {
		settings.AccentColor = "#0078d4"
	}

	return &settings, nil
}

// SaveSettings зберігає налаштування у файл
func (s *SettingsService) SaveSettings(settings *Settings) error {
	data, err := json.MarshalIndent(settings, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(s.configPath, data, 0644)
}

// GetLanguage повертає поточну мову
func (s *SettingsService) GetLanguage() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return "uk"
	}
	return settings.Language
}

// SetLanguage встановлює мову та зберігає налаштування
func (s *SettingsService) SetLanguage(language string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.Language = language
	return s.SaveSettings(settings)
}

// GetTheme повертає поточну тему
func (s *SettingsService) GetTheme() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return "dark"
	}
	return settings.Theme
}

// SetTheme встановлює тему та зберігає налаштування
func (s *SettingsService) SetTheme(theme string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.Theme = theme
	return s.SaveSettings(settings)
}

// GetAccentColor повертає поточний акцентний колір
func (s *SettingsService) GetAccentColor() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return "#0078d4"
	}
	return settings.AccentColor
}

// SetAccentColor встановлює колір та зберігає налаштування
func (s *SettingsService) SetAccentColor(color string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.AccentColor = color
	return s.SaveSettings(settings)
}

// GetConfigPath повертає шлях до файлу конфігурації (для дебагу)
func (s *SettingsService) GetConfigPath() string {
	return s.configPath
}

// GetConfigDir повертає шлях до папки конфігурації
func (s *SettingsService) GetConfigDir() string {
	return filepath.Dir(s.configPath)
}

// GetOpenRouterAPIKey повертає API ключ OpenRouter
func (s *SettingsService) GetOpenRouterAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.OpenRouterAPIKey
}

// SetOpenRouterAPIKey зберігає API ключ OpenRouter
func (s *SettingsService) SetOpenRouterAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.OpenRouterAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetOpenRouterModels повертає список збережених моделей OpenRouter
func (s *SettingsService) GetOpenRouterModels() []string {
	settings, err := s.LoadSettings()
	if err != nil {
		return []string{}
	}
	return settings.OpenRouterModels
}

// SetOpenRouterModels зберігає список моделей OpenRouter
func (s *SettingsService) SetOpenRouterModels(models []string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.OpenRouterModels = models
	return s.SaveSettings(settings)
}

// GetPollinationsAPIKey повертає API ключ Pollinations
func (s *SettingsService) GetPollinationsAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.PollinationsAPIKey
}

// SetPollinationsAPIKey зберігає API ключ Pollinations
func (s *SettingsService) SetPollinationsAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.PollinationsAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetPollinationsModels повертає список моделей Pollinations
func (s *SettingsService) GetPollinationsModels() []string {
	settings, err := s.LoadSettings()
	if err != nil {
		return []string{}
	}
	return settings.PollinationsModels
}

// SetPollinationsModels зберігає список моделей Pollinations
func (s *SettingsService) SetPollinationsModels(models []string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.PollinationsModels = models
	return s.SaveSettings(settings)
}

// GetElevenLabsBotAPIKey повертає API ключ ElevenLabsBot
func (s *SettingsService) GetElevenLabsBotAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.ElevenLabsBotAPIKey
}

// SetElevenLabsBotAPIKey зберігає API ключ ElevenLabsBot
func (s *SettingsService) SetElevenLabsBotAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		settings = &Settings{}
	}

	settings.ElevenLabsBotAPIKey = apiKey
	return s.SaveSettings(settings)
}
