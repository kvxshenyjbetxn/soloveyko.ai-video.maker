package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type PollinationsService struct {
	settings *utils.SettingsService
}

func NewPollinationsService(settings *utils.SettingsService) *PollinationsService {
	return &PollinationsService{
		settings: settings,
	}
}

// GetPollinationsImageModels fetches available image models from Pollinations.ai
func (s *PollinationsService) GetPollinationsImageModels() ([]string, error) {
	client := &http.Client{Timeout: 10 * time.Second}
	url := "https://gen.pollinations.ai/image/models"
	fmt.Println("Fetching models from:", url)

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	apiKey := s.GetPollinationsAPIKey()
	if apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+apiKey)
	}

	resp, err := client.Do(req)
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

	var models []string
	if err := json.Unmarshal(body, &models); err == nil {
		return models, nil
	}

	var modelsObjects []map[string]interface{}
	if err := json.Unmarshal(body, &modelsObjects); err == nil {
		var names []string
		for _, m := range modelsObjects {
			if name, ok := m["name"].(string); ok {
				names = append(names, name)
			} else if id, ok := m["id"].(string); ok {
				names = append(names, id)
			}
		}
		return names, nil
	}

	return nil, fmt.Errorf("failed to parse models response: %s", string(body))
}

// SavePollinationsAPIKey saves API key
func (s *PollinationsService) SavePollinationsAPIKey(apiKey string) error {
	return s.settings.SetPollinationsAPIKey(apiKey)
}

// GetPollinationsAPIKey gets API key
func (s *PollinationsService) GetPollinationsAPIKey() string {
	return s.settings.GetPollinationsAPIKey()
}

// SavePollinationsModels saves list of model IDs
func (s *PollinationsService) SavePollinationsModels(models []string) error {
	return s.settings.SetPollinationsModels(models)
}

// GetPollinationsSavedModels gets list of saved model IDs
func (s *PollinationsService) GetPollinationsSavedModels() []string {
	return s.settings.GetPollinationsModels()
}
