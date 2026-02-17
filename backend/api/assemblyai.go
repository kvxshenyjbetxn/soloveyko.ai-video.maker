package api

import (
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type AssemblyAIService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewAssemblyAIService(settings *utils.SettingsService) *AssemblyAIService {
	return &AssemblyAIService{
		settings: settings,
		baseUrl:  "https://api.assemblyai.com/v2",
	}
}

// CheckConnection перевіряє валідність API ключа
func (s *AssemblyAIService) CheckConnection(apiKey string) error {
	if apiKey == "" {
		return fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", s.baseUrl+"/transcript", nil)
	if err != nil {
		return err
	}

	req.Header.Set("Authorization", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusOK {
		return nil
	}

	if resp.StatusCode == http.StatusUnauthorized {
		return fmt.Errorf("invalid API key")
	}

	return fmt.Errorf("API error: %s", resp.Status)
}

// SaveAPIKey зберігає API ключ
func (s *AssemblyAIService) SaveAPIKey(apiKey string) error {
	return s.settings.SetAssemblyAIAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *AssemblyAIService) GetAPIKey() string {
	return s.settings.GetAssemblyAIAPIKey()
}
