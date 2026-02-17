package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type ElevenLabsBotService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewElevenLabsBotService(settings *utils.SettingsService) *ElevenLabsBotService {
	return &ElevenLabsBotService{
		settings: settings,
		baseUrl:  "https://voiceapi.csv666.ru",
	}
}

type ElevenLabsUserResponse struct {
	TelegramID  int64  `json:"telegram_id"`
	Balance     int    `json:"balance"`
	BalanceText string `json:"balance_text"`
}

// GetBalance отримує поточний баланс користувача
func (s *ElevenLabsBotService) GetBalance(apiKey string) (float64, error) {
	if apiKey == "" {
		return 0, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", s.baseUrl+"/balance", nil)
	if err != nil {
		return 0, err
	}

	req.Header.Set("X-API-Key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return 0, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	var balanceRes ElevenLabsUserResponse
	if err := json.NewDecoder(resp.Body).Decode(&balanceRes); err != nil {
		return 0, err
	}

	return float64(balanceRes.Balance), nil
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsBotService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsBotAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsBotService) GetAPIKey() string {
	return s.settings.GetElevenLabsBotAPIKey()
}
