package utils

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sync"
)

type NamedAPIKey struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	Key  string `json:"key"`
}

type PipelineSettings struct {
	TranslateModel              string  `json:"translateModel,omitempty"`
	TranslatePrompt             string  `json:"translatePrompt,omitempty"`
	TranslateTemperature        float64 `json:"translateTemperature,omitempty"`
	TranslateMaxTokens          int     `json:"translateMaxTokens,omitempty"`
	TranslateCollapsed          bool    `json:"translateCollapsed"`
	TranslateOpenRouterKeyID    string  `json:"translateOpenRouterKeyID,omitempty"`
	TranslateElevenLabsBotKeyID string  `json:"translateElevenLabsBotKeyID,omitempty"`
	RewriteModel                string  `json:"rewriteModel,omitempty"`
	RewritePrompt               string  `json:"rewritePrompt,omitempty"`
	RewriteTemperature          float64 `json:"rewriteTemperature,omitempty"`
	RewriteMaxTokens            int     `json:"rewriteMaxTokens,omitempty"`
	RewriteCollapsed            bool    `json:"rewriteCollapsed"`
	RewriteOpenRouterKeyID      string  `json:"rewriteOpenRouterKeyID,omitempty"`
	RewriteElevenLabsBotKeyID   string  `json:"rewriteElevenLabsBotKeyID,omitempty"`
	SidebarWidth                int     `json:"sidebarWidth,omitempty"`
	TranslateEnabled            bool    `json:"translateEnabled"`
	RewriteEnabled              bool    `json:"rewriteEnabled"`
	ApiCollapsed                bool    `json:"apiCollapsed"`
	TranslateOutputPath         string  `json:"translateOutputPath,omitempty"`
	RewriteOutputPath           string  `json:"rewriteOutputPath,omitempty"`
	PathCollapsed               bool    `json:"pathCollapsed"`
	TranslatePipelineName       string  `json:"translatePipelineName,omitempty"`
	RewritePipelineName         string  `json:"rewritePipelineName,omitempty"`
	TemplatesCollapsed          bool    `json:"templatesCollapsed"`
	TranslateTemplatesCollapsed bool    `json:"translateTemplatesCollapsed"`
	RewriteTemplatesCollapsed   bool    `json:"rewriteTemplatesCollapsed"`
	VoiceoverOpenRouterKeyID    string  `json:"voiceoverOpenRouterKeyID,omitempty"`
	VoiceoverElevenLabsBotKeyID string  `json:"voiceoverElevenLabsBotKeyID,omitempty"`
	VoiceoverModel              string  `json:"voiceoverModel,omitempty"`
	VoiceoverPrompt             string  `json:"voiceoverPrompt,omitempty"`
	VoiceoverTemperature        float64 `json:"voiceoverTemperature,omitempty"`
	VoiceoverMaxTokens          int     `json:"voiceoverMaxTokens,omitempty"`
	VoiceoverCollapsed          bool    `json:"voiceoverCollapsed"`
	VoiceoverEnabled            bool    `json:"voiceoverEnabled"`
	VoiceoverOutputPath         string  `json:"voiceoverOutputPath,omitempty"`
	VoiceoverPipelineName       string  `json:"voiceoverPipelineName,omitempty"`
	VoiceoverTemplatesCollapsed bool    `json:"voiceoverTemplatesCollapsed"`
	// Keep outputPath for migration if needed
	OutputPath string `json:"outputPath,omitempty"`
}

type Settings struct {
	Language                      string           `json:"language"`
	Theme                         string           `json:"theme"`
	AccentColor                   string           `json:"accentColor"`
	OpenRouterAPIKey              string           `json:"openRouterAPIKey"`
	OpenRouterKeys                []NamedAPIKey    `json:"openRouterKeys"`
	OpenRouterModels              []string         `json:"openRouterModels"`
	PollinationsAPIKey            string           `json:"pollinationsAPIKey"`
	PollinationsModels            []string         `json:"pollinationsModels"`
	ElevenLabsBotAPIKey           string           `json:"elevenLabsBotAPIKey"`
	ElevenLabsBotKeys             []NamedAPIKey    `json:"elevenLabsBotKeys"`
	ElevenLabsUnlimAPIKey         string           `json:"elevenLabsUnlimAPIKey"`
	VoiceMakerAPIKey              string           `json:"voiceMakerAPIKey"`
	VoiceMakerBalance             float64          `json:"voiceMakerBalance"`
	GooglerAPIKey                 string           `json:"googlerAPIKey"`
	ElevenLabsImageAPIKey         string           `json:"elevenLabsImageAPIKey"`
	ElevenLabsUAAPIKey            string           `json:"elevenLabsUAAPIKey"`
	AssemblyAIAPIKey              string           `json:"assemblyAIAPIKey"`
	OpenRouterMaxConnections      int              `json:"openRouterMaxConnections"`
	ElevenLabsBotAlertThreshold   float64          `json:"elevenLabsBotAlertThreshold"`
	ElevenLabsUnlimAlertThreshold float64          `json:"elevenLabsUnlimAlertThreshold"`
	VoiceMakerAlertThreshold      float64          `json:"voiceMakerAlertThreshold"`
	OpenRouterAlertThreshold      float64          `json:"openRouterAlertThreshold"`
	GooglerVideoAlertThreshold    float64          `json:"googlerVideoAlertThreshold"`
	GooglerImageAlertThreshold    float64          `json:"googlerImageAlertThreshold"`
	Pipeline                      PipelineSettings `json:"pipeline"`
}

type SettingsService struct {
	configPath string
	mu         sync.RWMutex
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
	s.mu.RLock()
	defer s.mu.RUnlock()

	// Якщо файл не існує, повертаємо налаштування за замовчуванням
	if _, err := os.Stat(s.configPath); os.IsNotExist(err) {
		return &Settings{
			Language:    "uk",
			Theme:       "dark",
			AccentColor: "#ff00c3", // Наш фірмовий рожевий
			OpenRouterModels: []string{
				"google/gemini-2.5-flash",
				"z-ai/glm-4.5-air:free",
			},
			Pipeline: PipelineSettings{
				TranslateModel:       "google/gemini-2.5-flash",
				TranslateTemperature: 1.0,
				TranslateEnabled:     true,
				RewriteModel:         "google/gemini-2.5-flash",
				RewriteTemperature:   1.0,
				RewriteEnabled:       true,
				VoiceoverModel:       "google/gemini-2.5-flash",
				VoiceoverTemperature: 1.0,
				VoiceoverEnabled:     false,
				SidebarWidth:         320,
			},
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

	// Дефолтні значення, якщо поле відсутнє в конфізі
	if settings.Theme == "" {
		settings.Theme = "dark"
	}
	if settings.AccentColor == "" {
		settings.AccentColor = "#ff00c3"
	}
	if settings.OpenRouterMaxConnections <= 0 {
		settings.OpenRouterMaxConnections = 5
	}
	// Якщо список моделей взагалі nil (поле відсутнє в JSON), додаємо дефолтні.
	// Якщо список порожній [], але не nil (користувач все видалив), не чіпаємо.
	if settings.OpenRouterModels == nil {
		settings.OpenRouterModels = []string{
			"google/gemini-2.5-flash",
			"z-ai/glm-4.5-air:free",
		}
	}
	if settings.Pipeline.TranslateModel == "" && len(settings.OpenRouterModels) > 0 {
		settings.Pipeline.TranslateModel = settings.OpenRouterModels[0]
	}
	// Міграція для OpenRouterKeys
	if len(settings.OpenRouterKeys) == 0 && settings.OpenRouterAPIKey != "" {
		settings.OpenRouterKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.OpenRouterAPIKey,
			},
		}
	}
	// Міграція для ElevenLabsBotKeys
	if len(settings.ElevenLabsBotKeys) == 0 && settings.ElevenLabsBotAPIKey != "" {
		settings.ElevenLabsBotKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.ElevenLabsBotAPIKey,
			},
		}
	}

	return &settings, nil
}

// SaveSettings зберігає налаштування у файл
func (s *SettingsService) SaveSettings(settings *Settings) error {
	s.mu.Lock()
	defer s.mu.Unlock()

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
		return err
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
		return err
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
		return err
	}

	settings.AccentColor = color
	return s.SaveSettings(settings)
}

// GetElevenLabsBotAlertThreshold повертає поріг попередження для ElevenLabsBot
func (s *SettingsService) GetElevenLabsBotAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.ElevenLabsBotAlertThreshold
}

// SetElevenLabsBotAlertThreshold зберігає поріг попередження для ElevenLabsBot
func (s *SettingsService) SetElevenLabsBotAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsBotAlertThreshold = threshold
	return s.SaveSettings(settings)
}

// GetElevenLabsUnlimAlertThreshold повертає поріг попередження для ElevenLabsUnlim
func (s *SettingsService) GetElevenLabsUnlimAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.ElevenLabsUnlimAlertThreshold
}

// SetElevenLabsUnlimAlertThreshold зберігає поріг попередження для ElevenLabsUnlim
func (s *SettingsService) SetElevenLabsUnlimAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUnlimAlertThreshold = threshold
	return s.SaveSettings(settings)
}

// GetVoiceMakerAlertThreshold повертає поріг попередження для VoiceMaker
func (s *SettingsService) GetVoiceMakerAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.VoiceMakerAlertThreshold
}

// SetVoiceMakerAlertThreshold зберігає поріг попередження для VoiceMaker
func (s *SettingsService) SetVoiceMakerAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.VoiceMakerAlertThreshold = threshold
	return s.SaveSettings(settings)
}

// GetOpenRouterAlertThreshold повертає поріг попередження для OpenRouter
func (s *SettingsService) GetOpenRouterAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.OpenRouterAlertThreshold
}

// SetOpenRouterAlertThreshold зберігає поріг попередження для OpenRouter
func (s *SettingsService) SetOpenRouterAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.OpenRouterAlertThreshold = threshold
	return s.SaveSettings(settings)
}

// GetGooglerVideoAlertThreshold повертає поріг попередження для Googler (відео)
func (s *SettingsService) GetGooglerVideoAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.GooglerVideoAlertThreshold
}

// SetGooglerVideoAlertThreshold зберігає поріг попередження для Googler (відео)
func (s *SettingsService) SetGooglerVideoAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.GooglerVideoAlertThreshold = threshold
	return s.SaveSettings(settings)
}

// GetGooglerImageAlertThreshold повертає поріг попередження для Googler (картинки)
func (s *SettingsService) GetGooglerImageAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.GooglerImageAlertThreshold
}

// SetGooglerImageAlertThreshold зберігає поріг попередження для Googler (картинки)
func (s *SettingsService) SetGooglerImageAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.GooglerImageAlertThreshold = threshold
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
		return err
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
		return err
	}

	settings.OpenRouterModels = models
	return s.SaveSettings(settings)
}

// GetOpenRouterKeys повертає список іменованих ключів OpenRouter
func (s *SettingsService) GetOpenRouterKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.OpenRouterKeys
}

// SetOpenRouterKeys зберігає список іменованих ключів OpenRouter
func (s *SettingsService) SetOpenRouterKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.OpenRouterKeys = keys
	// Оновлюємо старий ключ для сумісності з іншими частинами коду
	if len(keys) > 0 {
		settings.OpenRouterAPIKey = keys[0].Key
	}

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
		return err
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
		return err
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
		return err
	}

	settings.ElevenLabsBotAPIKey = apiKey
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.ElevenLabsBotKeys) == 0 {
		settings.ElevenLabsBotKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}

	return s.SaveSettings(settings)
}

// GetElevenLabsBotKeys повертає список іменованих ключів ElevenLabsBot
func (s *SettingsService) GetElevenLabsBotKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.ElevenLabsBotKeys
}

// SetElevenLabsBotKeys зберігає список іменованих ключів ElevenLabsBot
func (s *SettingsService) SetElevenLabsBotKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsBotKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.ElevenLabsBotAPIKey = keys[0].Key
	}

	return s.SaveSettings(settings)
}

// GetElevenLabsUnlimAPIKey повертає API ключ ElevenLabsUnlim
func (s *SettingsService) GetElevenLabsUnlimAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.ElevenLabsUnlimAPIKey
}

// SetElevenLabsUnlimAPIKey зберігає API ключ ElevenLabsUnlim
func (s *SettingsService) SetElevenLabsUnlimAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUnlimAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetVoiceMakerAPIKey повертає API ключ VoiceMaker
func (s *SettingsService) GetVoiceMakerAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.VoiceMakerAPIKey
}

// SetVoiceMakerAPIKey зберігає API ключ VoiceMaker
func (s *SettingsService) SetVoiceMakerAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.VoiceMakerAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetVoiceMakerBalance повертає останній збережений баланс VoiceMaker
func (s *SettingsService) GetVoiceMakerBalance() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.VoiceMakerBalance
}

// SetVoiceMakerBalance зберігає баланс VoiceMaker
func (s *SettingsService) SetVoiceMakerBalance(balance float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.VoiceMakerBalance = balance
	return s.SaveSettings(settings)
}

// GetGooglerAPIKey повертає API ключ Googler
func (s *SettingsService) GetGooglerAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.GooglerAPIKey
}

// SetGooglerAPIKey зберігає API ключ Googler
func (s *SettingsService) SetGooglerAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.GooglerAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetElevenLabsImageAPIKey повертає API ключ ElevenLabsImage
func (s *SettingsService) GetElevenLabsImageAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.ElevenLabsImageAPIKey
}

// SetElevenLabsImageAPIKey зберігає API ключ ElevenLabsImage
func (s *SettingsService) SetElevenLabsImageAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsImageAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetElevenLabsUAAPIKey повертає API ключ ElevenLabsUA
func (s *SettingsService) GetElevenLabsUAAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.ElevenLabsUAAPIKey
}

// SetElevenLabsUAAPIKey зберігає API ключ ElevenLabsUA
func (s *SettingsService) SetElevenLabsUAAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUAAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetAssemblyAIAPIKey повертає API ключ AssemblyAI
func (s *SettingsService) GetAssemblyAIAPIKey() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.AssemblyAIAPIKey
}

// SetAssemblyAIAPIKey зберігає API ключ AssemblyAI
func (s *SettingsService) SetAssemblyAIAPIKey(apiKey string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.AssemblyAIAPIKey = apiKey
	return s.SaveSettings(settings)
}

// GetOpenRouterMaxConnections повертає ліміт одночасних запитів
func (s *SettingsService) GetOpenRouterMaxConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 5
	}
	if settings.OpenRouterMaxConnections <= 0 {
		return 5
	}
	return settings.OpenRouterMaxConnections
}

// SetOpenRouterMaxConnections встановлює ліміт одночасних запитів
func (s *SettingsService) SetOpenRouterMaxConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.OpenRouterMaxConnections = max
	return s.SaveSettings(settings)
}

// GetPipelineSettings повертає налаштування пайплайну
func (s *SettingsService) GetPipelineSettings() PipelineSettings {
	settings, err := s.LoadSettings()
	if err != nil {
		return PipelineSettings{
			TranslateTemperature: 0.7,
			RewriteTemperature:   0.7,
		}
	}
	// Якщо налаштування порожні, повертаємо дефолтні
	if settings.Pipeline.TranslateTemperature == 0 {
		settings.Pipeline.TranslateTemperature = 1.0
	}
	if settings.Pipeline.RewriteTemperature == 0 {
		settings.Pipeline.RewriteTemperature = 1.0
	}
	if settings.Pipeline.VoiceoverTemperature == 0 {
		settings.Pipeline.VoiceoverTemperature = 1.0
	}
	if settings.Pipeline.SidebarWidth == 0 {
		settings.Pipeline.SidebarWidth = 320
		settings.Pipeline.TranslateEnabled = true
		settings.Pipeline.RewriteEnabled = true
	}
	return settings.Pipeline
}

// SavePipelineSettings зберігає налаштування пайплайну
func (s *SettingsService) SavePipelineSettings(pipeline PipelineSettings) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.Pipeline = pipeline
	return s.SaveSettings(settings)
}
