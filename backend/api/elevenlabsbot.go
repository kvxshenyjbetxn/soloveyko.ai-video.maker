package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type ElevenLabsBotService struct {
	settings *utils.SettingsService
	baseUrl  string
	OnLog    func(level string, message string)
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

	// Documentation says X-API-Key is required
	req.Header["X-API-Key"] = []string{apiKey}
	req.Header.Set("Accept", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)

	if resp.StatusCode != http.StatusOK {
		if s.OnLog != nil {
			s.OnLog("ERROR", fmt.Sprintf("[ElevenLabsBot] Balance check error %d: %s", resp.StatusCode, string(body)))
		}
		return 0, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	var balanceRes ElevenLabsUserResponse
	if err := json.Unmarshal(body, &balanceRes); err != nil {
		return 0, err
	}

	return float64(balanceRes.Balance), nil
}

// GetTemplates отримує список шаблонів голосів
func (s *ElevenLabsBotService) GetTemplates(apiKey string) ([]string, error) {
	if apiKey == "" {
		return nil, fmt.Errorf("API key is empty")
	}

	if s.OnLog != nil {
		s.OnLog("INFO", fmt.Sprintf("[ElevenLabsBot] Fetching voice templates..."))
	}

	client := &http.Client{Timeout: 15 * time.Second}
	req, err := http.NewRequest("GET", s.baseUrl+"/templates", nil)
	if err != nil {
		return nil, err
	}

	// Documentation says X-API-Key is required
	req.Header["X-API-Key"] = []string{apiKey}
	req.Header.Set("Accept", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		if s.OnLog != nil {
			s.OnLog("ERROR", fmt.Sprintf("[ElevenLabsBot] Connection error: %v", err))
		}
		return nil, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)

	if resp.StatusCode != http.StatusOK {
		if s.OnLog != nil {
			s.OnLog("ERROR", fmt.Sprintf("[ElevenLabsBot] API error %d: %s", resp.StatusCode, string(body)))
		}
		return nil, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	// Use a temporary buffer to allow multiple decoding attempts if needed
	var rawData json.RawMessage
	if err := json.Unmarshal(body, &rawData); err != nil {
		if s.OnLog != nil {
			s.OnLog("ERROR", fmt.Sprintf("[ElevenLabsBot] JSON parse error: %v | Body: %s", err, string(body)))
		}
		return nil, err
	}

	var results []string

	// 1. Try as simple slice of strings
	var strSlice []string
	if err := json.Unmarshal(rawData, &strSlice); err == nil {
		results = strSlice
	}

	// 2. Try as slice of objects
	if len(results) == 0 {
		var objSlice []map[string]interface{}
		if err := json.Unmarshal(rawData, &objSlice); err == nil {
			for _, item := range objSlice {
				if name, ok := item["name"].(string); ok {
					results = append(results, name)
				} else if name, ok := item["label"].(string); ok {
					results = append(results, name)
				} else if name, ok := item["title"].(string); ok {
					results = append(results, name)
				} else if id, ok := item["template_id"].(string); ok {
					results = append(results, id)
				} else if id, ok := item["id"].(string); ok {
					results = append(results, id)
				}
			}
		}
	}

	// 3. Try as object with "templates" or "data" field
	if len(results) == 0 {
		var tObj map[string]interface{}
		if err := json.Unmarshal(rawData, &tObj); err == nil {
			fields := []string{"templates", "data", "items"}
			for _, f := range fields {
				if val, ok := tObj[f]; ok {
					if slice, ok := val.([]interface{}); ok {
						for _, item := range slice {
							if s, ok := item.(string); ok {
								results = append(results, s)
							} else if m, ok := item.(map[string]interface{}); ok {
								if name, ok := m["name"].(string); ok {
									results = append(results, name)
								} else if name, ok := m["label"].(string); ok {
									results = append(results, name)
								} else if name, ok := m["title"].(string); ok {
									results = append(results, name)
								}
							}
						}
					}
				}
				if len(results) > 0 {
					break
				}
			}
		}
	}

	// 4. Try as a map (key is template name)
	if len(results) == 0 {
		var tMap map[string]interface{}
		if err := json.Unmarshal(rawData, &tMap); err == nil {
			for k := range tMap {
				results = append(results, k)
			}
		}
	}

	if len(results) > 0 {
		if s.OnLog != nil {
			s.OnLog("SUCCESS", fmt.Sprintf("[ElevenLabsBot] Successfully loaded %d voice templates", len(results)))
		}
		return results, nil
	}

	if s.OnLog != nil {
		s.OnLog("ERROR", fmt.Sprintf("[ElevenLabsBot] Unexpected templates format: %s", string(body)))
	}
	return nil, fmt.Errorf("unexpected templates format")
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsBotService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsBotAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsBotService) GetAPIKey() string {
	return s.settings.GetElevenLabsBotAPIKey()
}
