package api

import (
	"soloveyko/backend/utils"
)

type ElevenLabsUAService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewElevenLabsUAService(settings *utils.SettingsService) *ElevenLabsUAService {
	return &ElevenLabsUAService{
		settings: settings,
		baseUrl:  "https://11tts.net/v1",
	}
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsUAService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsUAAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsUAService) GetAPIKey() string {
	return s.settings.GetElevenLabsUAAPIKey()
}
