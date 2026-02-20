package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"soloveyko/backend/utils"
	"sync"
	"time"
)

type PollinationsService struct {
	settings        *utils.SettingsService
	lastRequestTime time.Time
	mu              sync.Mutex
}

func NewPollinationsService(settings *utils.SettingsService) *PollinationsService {
	return &PollinationsService{
		settings: settings,
	}
}

// GenerateImage generates an image using Pollinations.ai and saves it to outputPath
func (s *PollinationsService) GenerateImage(apiKey string, prompt string, model string, width int, height int, nologo bool, enhance bool, outputPath string) error {
	s.mu.Lock()
	waitTime := 30 * time.Second
	if apiKey != "" {
		waitTime = 7 * time.Second
	}

	elapsed := time.Since(s.lastRequestTime)
	if elapsed < waitTime {
		sleepTime := waitTime - elapsed
		s.mu.Unlock()
		time.Sleep(sleepTime)
		s.mu.Lock()
	}
	s.lastRequestTime = time.Now()
	s.mu.Unlock()

	client := &http.Client{Timeout: 120 * time.Second}

	// Determine base URL based on API key presence
	var baseUrl string
	if apiKey != "" {
		baseUrl = fmt.Sprintf("https://gen.pollinations.ai/image/%s", utils.UrlEncode(prompt))
	} else {
		baseUrl = fmt.Sprintf("https://image.pollinations.ai/prompt/%s", utils.UrlEncode(prompt))
	}

	seed := time.Now().UnixMilli() % 10000000 // safe small integer for seed
	params := fmt.Sprintf("?width=%d&height=%d&seed=%d", width, height, seed)
	if model != "" {
		params += "&model=" + model
	}

	url := baseUrl + params

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return err
	}

	if apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+apiKey)
	}

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("API request failed with status %d: %s (URL: %s)", resp.StatusCode, string(bodyBytes), url)
	}

	out, err := os.Create(outputPath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

// GetPollinationsImageModels fetches available image models from Pollinations.ai
func (s *PollinationsService) GetPollinationsImageModels() ([]string, error) {
	client := &http.Client{Timeout: 10 * time.Second}
	url := "https://gen.pollinations.ai/image/models"

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

// GetPollinationsKeys returns list of named API keys
func (s *PollinationsService) GetPollinationsKeys() []utils.NamedAPIKey {
	return s.settings.GetPollinationsKeys()
}

// SetPollinationsKeys saves list of named API keys
func (s *PollinationsService) SetPollinationsKeys(keys []utils.NamedAPIKey) error {
	return s.settings.SetPollinationsKeys(keys)
}

// SavePollinationsModels saves list of model IDs
func (s *PollinationsService) SavePollinationsModels(models []string) error {
	return s.settings.SetPollinationsModels(models)
}

// GetPollinationsSavedModels gets list of saved model IDs
func (s *PollinationsService) GetPollinationsSavedModels() []string {
	return s.settings.GetPollinationsModels()
}
