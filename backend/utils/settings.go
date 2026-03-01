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

type OverlayTrigger struct {
	Phrase string `json:"phrase"`
	Path   string `json:"path"`
	X      int    `json:"x"`
	Y      int    `json:"y"`
}

type CustomStage struct {
	ID         string `json:"id"`
	Name       string `json:"name"`
	Prompt     string `json:"prompt"`
	DataSource string `json:"dataSource"` // original, processed, taskName
	Enabled    bool   `json:"enabled"`
}

type PipelineSettings struct {
	TranslateModel                string  `json:"translateModel,omitempty"`
	TranslatePrompt               string  `json:"translatePrompt,omitempty"`
	TranslateTemperature          float64 `json:"translateTemperature,omitempty"`
	TranslateMaxTokens            int     `json:"translateMaxTokens,omitempty"`
	TranslateCollapsed            bool    `json:"translateCollapsed"`
	TranslateOpenRouterKeyID      string  `json:"translateOpenRouterKeyID,omitempty"`
	RewriteModel                  string  `json:"rewriteModel,omitempty"`
	RewritePrompt                 string  `json:"rewritePrompt,omitempty"`
	RewriteTemperature            float64 `json:"rewriteTemperature,omitempty"`
	RewriteMaxTokens              int     `json:"rewriteMaxTokens,omitempty"`
	RewriteCollapsed              bool    `json:"rewriteCollapsed"`
	RewriteOpenRouterKeyID        string  `json:"rewriteOpenRouterKeyID,omitempty"`
	SidebarWidth                  int     `json:"sidebarWidth,omitempty"`
	TranslateEnabled              bool    `json:"translateEnabled"`
	RewriteEnabled                bool    `json:"rewriteEnabled"`
	ApiCollapsed                  bool    `json:"apiCollapsed"`
	TranslateOutputPath           string  `json:"translateOutputPath,omitempty"`
	RewriteOutputPath             string  `json:"rewriteOutputPath,omitempty"`
	PathCollapsed                 bool    `json:"pathCollapsed"`
	TranslatePipelineName         string  `json:"translatePipelineName,omitempty"`
	RewritePipelineName           string  `json:"rewritePipelineName,omitempty"`
	TemplatesCollapsed            bool    `json:"templatesCollapsed"`
	TranslateTemplatesCollapsed   bool    `json:"translateTemplatesCollapsed"`
	RewriteTemplatesCollapsed     bool    `json:"rewriteTemplatesCollapsed"`
	VoiceoverElevenLabsBotKeyID   string  `json:"voiceoverElevenLabsBotKeyID,omitempty"`
	VoiceoverCollapsed            bool    `json:"voiceoverCollapsed"`
	VoiceoverEnabled              bool    `json:"voiceoverEnabled"`
	VoiceoverPipelineName         string  `json:"voiceoverPipelineName,omitempty"`
	VoiceoverTemplatesCollapsed   bool    `json:"voiceoverTemplatesCollapsed"`
	VoiceoverService              string  `json:"voiceoverService,omitempty"`
	VoiceoverTemplate             string  `json:"voiceoverTemplate,omitempty"`
	VoiceoverVoiceMakerKeyID      string  `json:"voiceoverVoiceMakerKeyID,omitempty"`
	VoiceMakerVoiceID             string  `json:"voiceMakerVoiceID,omitempty"`
	VoiceMakerLanguageCode        string  `json:"voiceMakerLanguageCode,omitempty"`
	VoiceMakerCharLimit           int     `json:"voiceMakerCharLimit,omitempty"`
	ElevenLabsUnlimVoiceID        string  `json:"elevenLabsUnlimVoiceID,omitempty"`
	ElevenLabsUnlimStability      float64 `json:"elevenLabsUnlimStability,omitempty"`
	ElevenLabsUnlimSimilarity     float64 `json:"elevenLabsUnlimSimilarity,omitempty"`
	ElevenLabsUnlimStyle          float64 `json:"elevenLabsUnlimStyle,omitempty"`
	ElevenLabsUnlimSpeakerBoost   bool    `json:"elevenLabsUnlimSpeakerBoost,omitempty"`
	VoiceoverElevenLabsUnlimKeyID string  `json:"voiceoverElevenLabsUnlimKeyID,omitempty"`
	VoiceoverElevenLabsUAKeyID    string  `json:"voiceoverElevenLabsUAKeyID,omitempty"`
	ElevenLabsUAVoiceID           string  `json:"elevenLabsUAVoiceID,omitempty"`
	ElevenLabsUAStability         float64 `json:"elevenLabsUAStability,omitempty"`
	ElevenLabsUASimilarity        float64 `json:"elevenLabsUASimilarity,omitempty"`
	ElevenLabsUAStyle             float64 `json:"elevenLabsUAStyle,omitempty"`
	ElevenLabsUASpeakerBoost      bool    `json:"elevenLabsUASpeakerBoost,omitempty"`
	ElevenLabsUAModel             string  `json:"elevenLabsUAModel,omitempty"`
	EdgeTTSVoiceID                string  `json:"edgeTTSVoiceID,omitempty"`
	EdgeTTSRate                   string  `json:"edgeTTSRate,omitempty"`
	EdgeTTSPitch                  string  `json:"edgeTTSPitch,omitempty"`
	EdgeTTSVolume                 string  `json:"edgeTTSVolume,omitempty"`
	TranslateControlEnabled       bool    `json:"translateControlEnabled"`
	ImageControlEnabled           bool    `json:"imageControlEnabled"`
	ControlCollapsed              bool    `json:"controlCollapsed"`

	// Subtitle settings
	SubtitleEnabled     bool   `json:"subtitleEnabled"`
	SubtitleCollapsed   bool   `json:"subtitleCollapsed"`
	SubtitleService     string `json:"subtitleService,omitempty"`
	SubtitleModel       string `json:"subtitleModel,omitempty"`
	SubtitleAmdLanguage string `json:"subtitleAmdLanguage,omitempty"`
	SubtitleMaxLen      int    `json:"subtitleMaxLen,omitempty"`
	SubtitleMaxWords    int    `json:"subtitleMaxWords,omitempty"`
	SubtitleColor       string `json:"subtitleColor,omitempty"`
	SubtitleSize        int    `json:"subtitleSize,omitempty"`
	SubtitleFont        string `json:"subtitleFont,omitempty"`
	SubtitleFadeEnabled bool   `json:"subtitleFadeEnabled"`
	SubtitleFadeIn      int    `json:"subtitleFadeIn,omitempty"`
	SubtitleFadeOut     int    `json:"subtitleFadeOut,omitempty"`

	// Image settings
	ImageEnabled                   bool    `json:"imageEnabled"`
	ImageService                   string  `json:"imageService,omitempty"`
	ImageModel                     string  `json:"imageModel,omitempty"`
	ImageWidth                     int     `json:"imageWidth,omitempty"`
	ImageHeight                    int     `json:"imageHeight,omitempty"`
	ImageNoLogo                    bool    `json:"imageNoLogo"`
	ImageEnhance                   bool    `json:"imageEnhance"`
	ImagePrompt                    string  `json:"imagePrompt,omitempty"`
	ImagePollinationsKeyID         string  `json:"imagePollinationsKeyID,omitempty"`
	ImageGooglerModel              string  `json:"imageGooglerModel,omitempty"`
	ImageGooglerAspectRatio        string  `json:"imageGooglerAspectRatio,omitempty"`
	ImageGooglerRemixEnabled       bool    `json:"imageGooglerRemixEnabled"`
	ImageGooglerReferenceImage     string  `json:"imageGooglerReferenceImage,omitempty"`
	ImageGooglerRemixStrictMode    bool    `json:"imageGooglerRemixStrictMode"`
	ImageGooglerVideoEnabled       bool    `json:"imageGooglerVideoEnabled"`
	ImageGooglerVideoModel         string  `json:"imageGooglerVideoModel,omitempty"`
	ImageGooglerVideoMode          string  `json:"imageGooglerVideoMode,omitempty"`
	ImageGooglerVideoCount         int     `json:"imageGooglerVideoCount,omitempty"`
	ImageGooglerVideoUpscale       bool    `json:"imageGooglerVideoUpscale"`
	ImagePipelineName              string  `json:"imagePipelineName,omitempty"`
	ImageTemplatesCollapsed        bool    `json:"imageTemplatesCollapsed"`
	ImageCollapsed                 bool    `json:"imageCollapsed"`
	ImageGenerationMethod          string  `json:"imageGenerationMethod,omitempty"`
	ImageGroupSentences            bool    `json:"imageGroupSentences"`
	ImageSentenceLimit             int     `json:"imageSentenceLimit,omitempty"`
	ImageInitialSentenceCount      int     `json:"imageInitialSentenceCount,omitempty"`
	ImagePromptModel               string  `json:"imagePromptModel,omitempty"`
	ImageMode                      string  `json:"imageMode,omitempty"`
	ImageMemoryType                string  `json:"imageMemoryType,omitempty"`
	ImageMemoryChars               int     `json:"imageMemoryChars,omitempty"`
	ImagePromptTemperature         float64 `json:"imagePromptTemperature,omitempty"`
	ImagePromptMaxTokens           int     `json:"imagePromptMaxTokens,omitempty"`
	ElevenLabsImageKeyID           string  `json:"elevenLabsImageKeyID,omitempty"`
	ElevenLabsImagePortrait        bool    `json:"elevenLabsImagePortrait"`
	ElevenLabsImageAspectRatio     string  `json:"elevenLabsImageAspectRatio,omitempty"`
	ImageSyncEnabled               bool    `json:"imageSyncEnabled"`
	ImageDetermineCharacters       bool    `json:"imageDetermineCharacters"`
	ImageDetermineCharactersPrompt string  `json:"imageDetermineCharactersPrompt,omitempty"`

	// Keep outputPath for migration if needed

	OutputPath string `json:"outputPath,omitempty"`

	MontageEnabled                bool             `json:"montageEnabled"`
	MontageCollapsed              bool             `json:"montageCollapsed"`
	MontageSwayFactor             float64          `json:"montageSwayFactor"`
	MontageTransitionDuration     float64          `json:"montageTransitionDuration"`
	MontageTransitionEffect       string           `json:"montageTransitionEffect"`
	MontageZoomFactor             float64          `json:"montageZoomFactor"`
	MontageEncodingPreset         string           `json:"montageEncodingPreset"`
	MontageBitrate                int              `json:"montageBitrate"`
	MontageResolution             string           `json:"montageResolution"`
	MontageFPS                    int              `json:"montageFPS"`
	MontageUpscaleFactor          float64          `json:"montageUpscaleFactor"`
	MontageVideoCodec             string           `json:"montageVideoCodec"`
	MontageThreadsPerProcess      int              `json:"montageThreadsPerProcess"`
	MontageProcessPriority        string           `json:"montageProcessPriority"`
	MontageCPUCores               int              `json:"montageCPUCores"`
	MontageIntroVideoEnabled      bool             `json:"montageIntroVideoEnabled"`
	MontageIntroVideoPath         string           `json:"montageIntroVideoPath,omitempty"`
	MontageWatermarkEnabled       bool             `json:"montageWatermarkEnabled"`
	MontageWatermarkPath          string           `json:"montageWatermarkPath,omitempty"`
	MontageWatermarkPosition      string           `json:"montageWatermarkPosition"` // top-left, top-right, bottom-left, bottom-right, center
	MontageWatermarkOpacity       float64          `json:"montageWatermarkOpacity"`
	MontageWatermarkSize          int              `json:"montageWatermarkSize"` // percentage of width
	MontageWatermarkOnIntro       bool             `json:"montageWatermarkOnIntro"`
	MontageOverlayEnabled         bool             `json:"montageOverlayEnabled"`
	MontageOverlayPath            string           `json:"montageOverlayPath,omitempty"`
	MontageOverlayOnIntro         bool             `json:"montageOverlayOnIntro"`
	MontageOverlayTriggersEnabled bool             `json:"montageOverlayTriggersEnabled"`
	MontageOverlayTriggers        []OverlayTrigger `json:"montageOverlayTriggers,omitempty"`

	CustomStages          []CustomStage `json:"customStages,omitempty"`
	CustomStagesEnabled   bool          `json:"customStagesEnabled"`
	CustomStagesCollapsed bool          `json:"customStagesCollapsed"`
}

type Settings struct {
	Language                      string           `json:"language"`
	Theme                         string           `json:"theme"`
	AccentColor                   string           `json:"accentColor"`
	OpenRouterAPIKey              string           `json:"openRouterAPIKey"`
	OpenRouterKeys                []NamedAPIKey    `json:"openRouterKeys"`
	OpenRouterModels              []string         `json:"openRouterModels"`
	PollinationsAPIKey            string           `json:"pollinationsAPIKey"`
	PollinationsKeys              []NamedAPIKey    `json:"pollinationsKeys"`
	PollinationsModels            []string         `json:"pollinationsModels"`
	ElevenLabsBotAPIKey           string           `json:"elevenLabsBotAPIKey"`
	ElevenLabsBotKeys             []NamedAPIKey    `json:"elevenLabsBotKeys"`
	ElevenLabsUnlimAPIKey         string           `json:"elevenLabsUnlimAPIKey"`
	ElevenLabsUnlimKeys           []NamedAPIKey    `json:"elevenLabsUnlimKeys"`
	ElevenLabsUAKeys              []NamedAPIKey    `json:"elevenLabsUAKeys"`
	VoiceMakerAPIKey              string           `json:"voiceMakerAPIKey"`
	VoiceMakerKeys                []NamedAPIKey    `json:"voiceMakerKeys"`
	VoiceMakerBalance             float64          `json:"voiceMakerBalance"`
	GooglerAPIKey                 string           `json:"googlerAPIKey"`
	ElevenLabsImageAPIKey         string           `json:"elevenLabsImageAPIKey"`
	ElevenLabsUAAPIKey            string           `json:"elevenLabsUAAPIKey"`
	AssemblyAIAPIKey              string           `json:"assemblyAIAPIKey"`
	OpenRouterMaxConnections      int              `json:"openRouterMaxConnections"`
	GooglerMaxImageConnections    int              `json:"googlerMaxImageConnections"`
	GooglerMaxVideoConnections    int              `json:"googlerMaxVideoConnections"`
	ElevenLabsBotAlertThreshold   float64          `json:"elevenLabsBotAlertThreshold"`
	ElevenLabsUnlimAlertThreshold float64          `json:"elevenLabsUnlimAlertThreshold"`
	ElevenLabsUAAlertThreshold    float64          `json:"elevenLabsUAAlertThreshold"`
	VoiceMakerAlertThreshold      float64          `json:"voiceMakerAlertThreshold"`
	OpenRouterAlertThreshold      float64          `json:"openRouterAlertThreshold"`
	GooglerVideoAlertThreshold    float64          `json:"googlerVideoAlertThreshold"`
	GooglerImageAlertThreshold    float64          `json:"googlerImageAlertThreshold"`
	ElevenLabsImageKeys           []NamedAPIKey    `json:"elevenLabsImageKeys"`
	ElevenLabsImageMaxConnections int              `json:"elevenLabsImageMaxConnections"`
	SubtitleMaxConnections        int              `json:"subtitleMaxConnections"`
	MontageMaxConnections         int              `json:"montageMaxConnections"`
	MontageMode                   string           `json:"montageMode"`
	Pipeline                      PipelineSettings `json:"pipeline"`
	GoogleSheetURL                string           `json:"googleSheetURL"`
	GoogleFilter                  string           `json:"googleFilter"`
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
				VoiceoverEnabled:     false,
				SubtitleMaxLen:       40,
				SubtitleMaxWords:     10,
				SubtitleColor:        "#ffffff",
				SubtitleSize:         24,
				SubtitleFont:         "Arial",
				SubtitleFadeEnabled:  true,
				SubtitleFadeIn:       300,
				SubtitleFadeOut:      300,
				SidebarWidth:         320,
				MontageEnabled:       false,
				MontageCollapsed:     true,
				ImageMode:            "normal",
				ImageMemoryType:      "primitive",
				ImageMemoryChars:     1000,
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
	if settings.GooglerMaxImageConnections <= 0 {
		settings.GooglerMaxImageConnections = 25
	}
	if settings.GooglerMaxVideoConnections <= 0 {
		settings.GooglerMaxVideoConnections = 10
	}
	if settings.ElevenLabsImageMaxConnections <= 0 {
		settings.ElevenLabsImageMaxConnections = 25
	}
	if settings.SubtitleMaxConnections <= 0 {
		settings.SubtitleMaxConnections = 2
	}
	if settings.MontageMaxConnections <= 0 {
		settings.MontageMaxConnections = 1
	}
	if settings.MontageMode == "" {
		settings.MontageMode = "standard"
	}
	if settings.Pipeline.MontageTransitionDuration <= 0 {
		settings.Pipeline.MontageTransitionDuration = 0.5
	}
	if settings.Pipeline.MontageTransitionEffect == "" {
		settings.Pipeline.MontageTransitionEffect = "fade"
	}
	if settings.Pipeline.MontageSwayFactor == 0 {
		settings.Pipeline.MontageSwayFactor = 1.0
	}
	if settings.Pipeline.MontageZoomFactor == 0 {
		settings.Pipeline.MontageZoomFactor = 1.0
	}
	if settings.Pipeline.MontageEncodingPreset == "" {
		settings.Pipeline.MontageEncodingPreset = "medium"
	}
	if settings.Pipeline.MontageBitrate <= 0 {
		settings.Pipeline.MontageBitrate = 15
	}
	if settings.Pipeline.MontageResolution == "" {
		settings.Pipeline.MontageResolution = "1080p"
	}
	if settings.Pipeline.MontageFPS <= 0 {
		settings.Pipeline.MontageFPS = 30
	}
	if settings.Pipeline.MontageUpscaleFactor <= 0 {
		settings.Pipeline.MontageUpscaleFactor = 2.0
	}
	if settings.Pipeline.MontageVideoCodec == "" {
		settings.Pipeline.MontageVideoCodec = "cpu"
	}
	if settings.Pipeline.MontageProcessPriority == "" {
		settings.Pipeline.MontageProcessPriority = "normal"
	}
	// MontageCPUCores = 0 means all cores (default)
	// MontageThreadsPerProcess = 0 means auto (not set), so no default override needed

	if settings.Pipeline.ImageMode == "" {
		settings.Pipeline.ImageMode = "normal"
	}
	if settings.Pipeline.ImageMemoryType == "" {
		settings.Pipeline.ImageMemoryType = "primitive"
	}
	if settings.Pipeline.ImageMemoryChars <= 0 {
		settings.Pipeline.ImageMemoryChars = 1000
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
	// Міграція для ElevenLabsUnlimKeys
	if len(settings.ElevenLabsUnlimKeys) == 0 && settings.ElevenLabsUnlimAPIKey != "" {
		settings.ElevenLabsUnlimKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.ElevenLabsUnlimAPIKey,
			},
		}
	}
	// Міграція для ElevenLabsUAKeys
	if len(settings.ElevenLabsUAKeys) == 0 && settings.ElevenLabsUAAPIKey != "" {
		settings.ElevenLabsUAKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.ElevenLabsUAAPIKey,
			},
		}
	}
	// Міграція для VoiceMakerKeys
	if len(settings.VoiceMakerKeys) == 0 && settings.VoiceMakerAPIKey != "" {
		settings.VoiceMakerKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.VoiceMakerAPIKey,
			},
		}
	}
	// Міграція для PollinationsKeys
	if len(settings.PollinationsKeys) == 0 && settings.PollinationsAPIKey != "" {
		settings.PollinationsKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.PollinationsAPIKey,
			},
		}
	}
	// Міграція для ElevenLabsImageKeys
	if len(settings.ElevenLabsImageKeys) == 0 && settings.ElevenLabsImageAPIKey != "" {
		settings.ElevenLabsImageKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  settings.ElevenLabsImageAPIKey,
			},
		}
	}
	if settings.Pipeline.SubtitleMaxWords <= 0 {
		settings.Pipeline.SubtitleMaxWords = 10
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
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.PollinationsKeys) == 0 {
		settings.PollinationsKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}
	return s.SaveSettings(settings)
}

// GetPollinationsKeys повертає список іменованих ключів Pollinations
func (s *SettingsService) GetPollinationsKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.PollinationsKeys
}

// SetPollinationsKeys зберігає список іменованих ключів Pollinations
func (s *SettingsService) SetPollinationsKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.PollinationsKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.PollinationsAPIKey = keys[0].Key
	}

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
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.ElevenLabsUnlimKeys) == 0 {
		settings.ElevenLabsUnlimKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}
	return s.SaveSettings(settings)
}

// GetElevenLabsUnlimKeys повертає список іменованих ключів ElevenLabsUnlim
func (s *SettingsService) GetElevenLabsUnlimKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.ElevenLabsUnlimKeys
}

// SetElevenLabsUnlimKeys зберігає список іменованих ключів ElevenLabsUnlim
func (s *SettingsService) SetElevenLabsUnlimKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUnlimKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.ElevenLabsUnlimAPIKey = keys[0].Key
	}

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
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.VoiceMakerKeys) == 0 {
		settings.VoiceMakerKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}
	return s.SaveSettings(settings)
}

// GetVoiceMakerKeys повертає список іменованих ключів VoiceMaker
func (s *SettingsService) GetVoiceMakerKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.VoiceMakerKeys
}

// SetVoiceMakerKeys зберігає список іменованих ключів VoiceMaker
func (s *SettingsService) SetVoiceMakerKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.VoiceMakerKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.VoiceMakerAPIKey = keys[0].Key
	}

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
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.ElevenLabsImageKeys) == 0 {
		settings.ElevenLabsImageKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}
	return s.SaveSettings(settings)
}

// GetElevenLabsImageKeys повертає список іменованих ключів ElevenLabsImage
func (s *SettingsService) GetElevenLabsImageKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.ElevenLabsImageKeys
}

// SetElevenLabsImageKeys зберігає список іменованих ключів ElevenLabsImage
func (s *SettingsService) SetElevenLabsImageKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsImageKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.ElevenLabsImageAPIKey = keys[0].Key
	}

	return s.SaveSettings(settings)
}

// GetElevenLabsImageMaxConnections повертає ліміт одночасних запитів ElevenLabs Image
func (s *SettingsService) GetElevenLabsImageMaxConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 25
	}
	if settings.ElevenLabsImageMaxConnections <= 0 {
		return 25
	}
	return settings.ElevenLabsImageMaxConnections
}

// SetElevenLabsImageMaxConnections встановлює ліміт одночасних запитів ElevenLabs Image
func (s *SettingsService) SetElevenLabsImageMaxConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.ElevenLabsImageMaxConnections = max
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
	// Оновлюємо також іменовані ключі, якщо вони порожні
	if len(settings.ElevenLabsUAKeys) == 0 {
		settings.ElevenLabsUAKeys = []NamedAPIKey{
			{
				ID:   "default",
				Name: "Default",
				Key:  apiKey,
			},
		}
	}
	return s.SaveSettings(settings)
}

// GetElevenLabsUAKeys повертає список іменованих ключів ElevenLabsUA
func (s *SettingsService) GetElevenLabsUAKeys() []NamedAPIKey {
	settings, err := s.LoadSettings()
	if err != nil {
		return []NamedAPIKey{}
	}
	return settings.ElevenLabsUAKeys
}

// SetElevenLabsUAKeys зберігає список іменованих ключів ElevenLabsUA
func (s *SettingsService) SetElevenLabsUAKeys(keys []NamedAPIKey) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUAKeys = keys
	// Оновлюємо старий ключ для сумісності
	if len(keys) > 0 {
		settings.ElevenLabsUAAPIKey = keys[0].Key
	}

	return s.SaveSettings(settings)
}

// GetElevenLabsUAAlertThreshold повертає поріг попередження для ElevenLabsUA
func (s *SettingsService) GetElevenLabsUAAlertThreshold() float64 {
	settings, err := s.LoadSettings()
	if err != nil {
		return 0
	}
	return settings.ElevenLabsUAAlertThreshold
}

// SetElevenLabsUAAlertThreshold зберігає поріг попередження для ElevenLabsUA
func (s *SettingsService) SetElevenLabsUAAlertThreshold(threshold float64) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.ElevenLabsUAAlertThreshold = threshold
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

// GetGooglerMaxImageConnections повертає ліміт одночасних запитів Googler (Image)
func (s *SettingsService) GetGooglerMaxImageConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 25
	}
	if settings.GooglerMaxImageConnections <= 0 {
		return 25
	}
	return settings.GooglerMaxImageConnections
}

// SetGooglerMaxImageConnections встановлює ліміт одночасних запитів Googler (Image)
func (s *SettingsService) SetGooglerMaxImageConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.GooglerMaxImageConnections = max
	return s.SaveSettings(settings)
}

// GetGooglerMaxVideoConnections повертає ліміт одночасних запитів Googler (Video)
func (s *SettingsService) GetGooglerMaxVideoConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 10
	}
	if settings.GooglerMaxVideoConnections <= 0 {
		return 10
	}
	return settings.GooglerMaxVideoConnections
}

// SetGooglerMaxVideoConnections встановлює ліміт одночасних запитів Googler (Video)
func (s *SettingsService) SetGooglerMaxVideoConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.GooglerMaxVideoConnections = max
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
	if settings.Pipeline.SidebarWidth == 0 {
		settings.Pipeline.SidebarWidth = 320
		settings.Pipeline.TranslateEnabled = true
		settings.Pipeline.RewriteEnabled = true
	}
	if settings.Pipeline.VoiceMakerCharLimit <= 0 {
		settings.Pipeline.VoiceMakerCharLimit = 3000 // Дефолтне значення
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

// GetSubtitleMaxConnections повертає ліміт одночасних запитів Субтитрів
func (s *SettingsService) GetSubtitleMaxConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 2
	}
	return settings.SubtitleMaxConnections
}

// SetSubtitleMaxConnections встановлює ліміт одночасних запитів Субтитрів
func (s *SettingsService) SetSubtitleMaxConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.SubtitleMaxConnections = max
	return s.SaveSettings(settings)
}

// GetMontageMaxConnections повертає ліміт одночасних запитів Монтажу
func (s *SettingsService) GetMontageMaxConnections() int {
	settings, err := s.LoadSettings()
	if err != nil {
		return 1
	}
	if settings.MontageMaxConnections <= 0 {
		return 1
	}
	return settings.MontageMaxConnections
}

// SetMontageMaxConnections встановлює ліміт одночасних запитів Монтажу
func (s *SettingsService) SetMontageMaxConnections(max int) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.MontageMaxConnections = max
	return s.SaveSettings(settings)
}

// GetMontageMode повертає режим монтажу (standard/experimental)
func (s *SettingsService) GetMontageMode() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return "standard"
	}
	return settings.MontageMode
}

// SetMontageMode встановлює режим монтажу
func (s *SettingsService) SetMontageMode(mode string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}

	settings.MontageMode = mode
	return s.SaveSettings(settings)
}

// GetGoogleSheetURL повертає URL гугл таблиці
func (s *SettingsService) GetGoogleSheetURL() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.GoogleSheetURL
}

// SetGoogleSheetURL зберігає URL гугл таблиці
func (s *SettingsService) SetGoogleSheetURL(url string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.GoogleSheetURL = url
	return s.SaveSettings(settings)
}

// GetGoogleFilter повертає фільтр для гугл таблиці
func (s *SettingsService) GetGoogleFilter() string {
	settings, err := s.LoadSettings()
	if err != nil {
		return ""
	}
	return settings.GoogleFilter
}

// SetGoogleFilter зберігає фільтр для гугл таблиці
func (s *SettingsService) SetGoogleFilter(filter string) error {
	settings, err := s.LoadSettings()
	if err != nil {
		return err
	}
	settings.GoogleFilter = filter
	return s.SaveSettings(settings)
}
