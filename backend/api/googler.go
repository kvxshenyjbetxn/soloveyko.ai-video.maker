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

type FlexibleFloat64 float64

func (f *FlexibleFloat64) UnmarshalJSON(data []byte) error {
	// Спробуємо розпарсити як число
	var n float64
	if err := json.Unmarshal(data, &n); err == nil {
		*f = FlexibleFloat64(n)
		return nil
	}

	// Спробуємо розпарсити як об'єкт {"used": X} або {"count": X}
	var obj map[string]float64
	if err := json.Unmarshal(data, &obj); err == nil {
		if val, ok := obj["used"]; ok {
			*f = FlexibleFloat64(val)
			return nil
		}
		if val, ok := obj["count"]; ok {
			*f = FlexibleFloat64(val)
			return nil
		}
	}

	*f = 0
	return nil
}

type GooglerAccountLimits struct {
	ImgGenPerHourLimit            FlexibleFloat64 `json:"img_gen_per_hour_limit"`
	VideoGenPerHourLimit          FlexibleFloat64 `json:"video_gen_per_hour_limit"`
	ImgGenerationThreadsAllowed   FlexibleFloat64 `json:"img_generation_threads_allowed"`
	VideoGenerationThreadsAllowed FlexibleFloat64 `json:"video_generation_threads_allowed"`
	PromptTokensPerHourLimit      FlexibleFloat64 `json:"prompt_tokens_per_hour_limit"`
}

type GooglerActiveThreads struct {
	ImageThreads FlexibleFloat64 `json:"image_threads"`
	VideoThreads FlexibleFloat64 `json:"video_threads"`
}

type GooglerHourlyUsage struct {
	ImageGeneration  FlexibleFloat64 `json:"image_generation"`
	VideoGeneration  FlexibleFloat64 `json:"video_generation"`
	PromptGeneration FlexibleFloat64 `json:"prompt_generation"`
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
	ActivationDate FlexibleFloat64      `json:"activation_date"`
	ExpirationDate FlexibleFloat64      `json:"expiration_date"`
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
