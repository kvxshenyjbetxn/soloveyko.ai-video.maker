package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"soloveyko/backend/utils"
	"time"
)

type ElevenLabsBotService struct {
	settings *utils.SettingsService
	baseUrl  string
	OnLog    func(level string, message string, details ...string)
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
		s.OnLog("INFO", "[ElevenLabsBot] Fetching voice templates...")
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

// TaskCreateRequest структура для створення завдання
type TaskCreateRequest struct {
	Text         string `json:"text"`
	TemplateName string `json:"template_name"`
}

// TaskCreateResponse відповідь при створенні завдання
type TaskCreateResponse struct {
	TaskID int64 `json:"task_id"`
}

// TaskStatusResponse відповідь про статус завдання
type TaskStatusResponse struct {
	Status    string `json:"status"`
	ErrorText string `json:"error_text,omitempty"`
}

// CreateTask створює нове завдання на синтез
func (s *ElevenLabsBotService) CreateTask(apiKey string, text string, templateName string) (string, error) {
	if apiKey == "" {
		return "", fmt.Errorf("API key is empty")
	}

	reqBody := TaskCreateRequest{
		Text:         text,
		TemplateName: templateName,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	client := &http.Client{Timeout: 30 * time.Second}
	req, err := http.NewRequest("POST", s.baseUrl+"/tasks", bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header["X-API-Key"] = []string{apiKey}
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		return "", fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res TaskCreateResponse
	if err := json.Unmarshal(body, &res); err != nil {
		return "", fmt.Errorf("failed to parse response: %v | Body: %s", err, string(body))
	}

	return fmt.Sprintf("%d", res.TaskID), nil
}

// GetTaskStatus перевіряє статус обробки
func (s *ElevenLabsBotService) GetTaskStatus(apiKey string, taskID string) (string, error) {
	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/tasks/%s/status", s.baseUrl, taskID), nil)
	if err != nil {
		return "", err
	}

	req.Header["X-API-Key"] = []string{apiKey}

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res TaskStatusResponse
	if err := json.Unmarshal(body, &res); err != nil {
		return "", err
	}

	return res.Status, nil
}

// DownloadResult завантажує готовий файл
func (s *ElevenLabsBotService) DownloadResult(apiKey string, taskID string, filePath string) error {
	client := &http.Client{Timeout: 60 * time.Second}
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/tasks/%s/result", s.baseUrl, taskID), nil)
	if err != nil {
		return err
	}

	req.Header["X-API-Key"] = []string{apiKey}

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	out, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

// Synthesize виконує повний цикл синтезу голосу
func (s *ElevenLabsBotService) Synthesize(apiKey string, text string, templateName string, outputPath string, id string, taskLabel string) error {
	if s.OnLog != nil {
		s.OnLog("INFO", "[ElevenLabsBot] Starting voice synthesis...", id, taskLabel)
	}

	taskID, err := s.CreateTask(apiKey, text, templateName)
	if err != nil {
		return err
	}

	if s.OnLog != nil {
		s.OnLog("INFO", fmt.Sprintf("[ElevenLabsBot] Task created: %s. Polling status...", taskID), id, taskLabel)
	}

	// Опитування статусу
	maxAttempts := 60 // 5 хвилин (60 * 5 сек)
	for i := 0; i < maxAttempts; i++ {
		status, err := s.GetTaskStatus(apiKey, taskID)
		if err != nil {
			return err
		}

		if s.OnLog != nil {
			s.OnLog("INFO", fmt.Sprintf("[ElevenLabsBot] Task %s status: %s", taskID, status), id, taskLabel)
		}

		switch status {
		case "ending_processed", "ending":
			if s.OnLog != nil {
				s.OnLog("INFO", fmt.Sprintf("[ElevenLabsBot] Synthesis completed (status: %s). Downloading...", status), id, taskLabel)
			}
			// Якщо статус ending, даємо секунду на закриття файлу сервером
			if status == "ending" {
				time.Sleep(2 * time.Second)
			}
			return s.DownloadResult(apiKey, taskID, outputPath)
		case "error", "error_handled":
			return fmt.Errorf("synthesis failed with status: %s", status)
		}

		time.Sleep(5 * time.Second)
	}

	return fmt.Errorf("synthesis timeout")
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsBotService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsBotAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsBotService) GetAPIKey() string {
	return s.settings.GetElevenLabsBotAPIKey()
}
