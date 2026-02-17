package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type ElevenLabsUnlimService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewElevenLabsUnlimService(settings *utils.SettingsService) *ElevenLabsUnlimService {
	return &ElevenLabsUnlimService{
		settings: settings,
		baseUrl:  "https://voicer.mat3u.com/api/v1",
	}
}

type VoicerStatsResponse struct {
	SubscriptionType    string `json:"subscription_type"`
	TotalCharacters     int    `json:"total_characters"`
	UsedCharacters      int    `json:"used_characters"`
	RemainingCharacters int    `json:"remaining_characters"`
}

// GetBalance отримує поточний баланс користувача
func (s *ElevenLabsUnlimService) GetBalance(apiKey string) (float64, error) {
	if apiKey == "" {
		return 0, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", s.baseUrl+"/user/stats", nil)
	if err != nil {
		return 0, err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return 0, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	var stats VoicerStatsResponse
	if err := json.NewDecoder(resp.Body).Decode(&stats); err != nil {
		return 0, err
	}

	if stats.SubscriptionType == "unlimited" {
		return -1, nil // -1 означає безліміт
	}

	return float64(stats.RemainingCharacters), nil
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsUnlimService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsUnlimAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsUnlimService) GetAPIKey() string {
	return s.settings.GetElevenLabsUnlimAPIKey()
}
