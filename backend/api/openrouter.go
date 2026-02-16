package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type OpenRouterService struct {
	settings *utils.SettingsService
}

func NewOpenRouterService(settings *utils.SettingsService) *OpenRouterService {
	return &OpenRouterService{
		settings: settings,
	}
}

type OpenRouterModel struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}

type ModelsResponse struct {
	Data []OpenRouterModel `json:"data"`
}

type CreditsResponse struct {
	Data struct {
		TotalCredits float64 `json:"total_credits"`
		TotalUsage   float64 `json:"total_usage"`
	} `json:"data"`
}

// GetOpenRouterCredits check balance
func (s *OpenRouterService) GetOpenRouterCredits(apiKey string) (float64, error) {
	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", "https://openrouter.ai/api/v1/credits", nil)
	if err != nil {
		return 0, err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("HTTP-Referer", "http://localhost:3000")
	req.Header.Set("X-Title", "Soloveyko AI Video Maker")

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return 0, fmt.Errorf("API request failed with status: %d, body: %s", resp.StatusCode, string(body))
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return 0, err
	}

	var creditsResponse CreditsResponse
	if err := json.Unmarshal(body, &creditsResponse); err != nil {
		return 0, err
	}

	return creditsResponse.Data.TotalCredits - creditsResponse.Data.TotalUsage, nil // Returning balance (Credits - Usage)
}

// GetOpenRouterAvailableModels fetch models from OpenRouter
func (s *OpenRouterService) GetOpenRouterAvailableModels() ([]OpenRouterModel, error) {
	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Get("https://openrouter.ai/api/v1/models")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API request failed with status: %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var modelsResponse ModelsResponse
	if err := json.Unmarshal(body, &modelsResponse); err != nil {
		return nil, err
	}

	return modelsResponse.Data, nil
}

// SaveOpenRouterAPIKey saves API key
func (s *OpenRouterService) SaveOpenRouterAPIKey(apiKey string) error {
	return s.settings.SetOpenRouterAPIKey(apiKey)
}

// GetOpenRouterAPIKey gets API key
func (s *OpenRouterService) GetOpenRouterAPIKey() string {
	return s.settings.GetOpenRouterAPIKey()
}

// SaveOpenRouterModels saves list of model IDs
func (s *OpenRouterService) SaveOpenRouterModels(models []string) error {
	return s.settings.SetOpenRouterModels(models)
}

// GetOpenRouterSavedModels gets list of saved model IDs
func (s *OpenRouterService) GetOpenRouterSavedModels() []string {
	return s.settings.GetOpenRouterModels()
}
