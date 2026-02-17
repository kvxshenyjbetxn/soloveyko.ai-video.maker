package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type GooglerService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewGooglerService(settings *utils.SettingsService) *GooglerService {
	return &GooglerService{
		settings: settings,
		baseUrl:  "https://googler.fast-gen.ai/api",
	}
}

type GooglerAccountLimits struct {
	ImgGenPerHourLimit            float64 `json:"img_gen_per_hour_limit"`
	VideoGenPerHourLimit          float64 `json:"video_gen_per_hour_limit"`
	ImgGenerationThreadsAllowed   float64 `json:"img_generation_threads_allowed"`
	VideoGenerationThreadsAllowed float64 `json:"video_generation_threads_allowed"`
	PromptTokensPerHourLimit      float64 `json:"prompt_tokens_per_hour_limit"`
}

type GooglerActiveThreads struct {
	ImageThreads float64 `json:"image_threads"`
	VideoThreads float64 `json:"video_threads"`
}

type GooglerHourlyUsage struct {
	ImageGeneration  float64 `json:"image_generation"`
	VideoGeneration  float64 `json:"video_generation"`
	PromptGeneration float64 `json:"prompt_generation"`
}

type GooglerCurrentUsage struct {
	HourlyUsage   GooglerHourlyUsage   `json:"hourly_usage"`
	ActiveThreads GooglerActiveThreads `json:"active_threads"`
}

type GooglerUsageResponse struct {
	ApiKey         string               `json:"api_key"`
	AccountLimits  GooglerAccountLimits `json:"account_limits"`
	CurrentUsage   GooglerCurrentUsage  `json:"current_usage"`
	UsageWindow    string               `json:"usage_window"`
	ActivationDate float64              `json:"activation_date"`
	ExpirationDate float64              `json:"expiration_date"`
}

// GetUsage отримує статистику використання акаунту
func (s *GooglerService) GetUsage(apiKey string) (*GooglerUsageResponse, error) {
	if apiKey == "" {
		return nil, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	// Спробуємо v3 ендпоінт, як у спеці
	url := fmt.Sprintf("%s/v3/account/usage?api_key=%s", s.baseUrl, apiKey)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("X-API-Key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		fmt.Printf("Googler API Request Error: %v\n", err)
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var errData map[string]interface{}
		json.NewDecoder(resp.Body).Decode(&errData)
		fmt.Printf("Googler API Error Response: %d - %v\n", resp.StatusCode, errData)
		return nil, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	var usage GooglerUsageResponse
	if err := json.NewDecoder(resp.Body).Decode(&usage); err != nil {
		fmt.Printf("Googler API Decode Error: %v\n", err)
		return nil, err
	}

	return &usage, nil
}

// SaveAPIKey зберігає API ключ
func (s *GooglerService) SaveAPIKey(apiKey string) error {
	return s.settings.SetGooglerAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *GooglerService) GetAPIKey() string {
	return s.settings.GetGooglerAPIKey()
}
