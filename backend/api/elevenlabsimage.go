package api

import (
	"soloveyko/backend/utils"
)

type ElevenLabsImageService struct {
	settings *utils.SettingsService
}

func NewElevenLabsImageService(settings *utils.SettingsService) *ElevenLabsImageService {
	return &ElevenLabsImageService{
		settings: settings,
	}
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsImageService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsImageAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsImageService) GetAPIKey() string {
	return s.settings.GetElevenLabsImageAPIKey()
}
