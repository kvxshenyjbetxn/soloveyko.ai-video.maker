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

type ElevenLabsUnlimService struct {
	settings *utils.SettingsService
	baseUrl  string
	OnLog    func(level string, message string, details ...string)
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

	client := &http.Client{Timeout: 30 * time.Second}
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
		return -1, nil
	}

	return float64(stats.RemainingCharacters), nil
}

type VoicerSynthesizeRequest struct {
	Text               string                 `json:"text"`
	VoiceID            string                 `json:"voice_id,omitempty"`
	ModelID            string                 `json:"model_id,omitempty"`
	SplitType          string                 `json:"split_type,omitempty"`
	SplitOutput        bool                   `json:"split_output,omitempty"`
	AutoPauseEnabled   bool                   `json:"auto_pause_enabled,omitempty"`
	AutoPauseDuration  float64                `json:"auto_pause_duration,omitempty"`
	AutoPauseFrequency int                    `json:"auto_pause_frequency,omitempty"`
	VoiceSettings      map[string]interface{} `json:"voice_settings,omitempty"`
}

type VoicerSynthesizeResponse struct {
	TaskID  string `json:"task_id"`
	Status  string `json:"status,omitempty"`
	Message string `json:"message,omitempty"`
}

type VoicerStatusResponse struct {
	Status    string  `json:"status"`
	Progress  float64 `json:"progress,omitempty"`
	ErrorText string  `json:"error,omitempty"`
}

func (s *ElevenLabsUnlimService) CreateTask(apiKey string, text string, voiceID string, voiceSettings map[string]interface{}) (string, error) {
	reqBody := VoicerSynthesizeRequest{
		Text:          text,
		VoiceID:       voiceID,
		ModelID:       "eleven_multilingual_v2",
		VoiceSettings: voiceSettings,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	// Збільшено таймаут до 120 секунд для створення задачі
	client := &http.Client{Timeout: 120 * time.Second}
	req, err := http.NewRequest("POST", s.baseUrl+"/voice/synthesize", bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusAccepted && resp.StatusCode != http.StatusCreated {
		return "", fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res VoicerSynthesizeResponse
	if err := json.Unmarshal(body, &res); err != nil {
		return "", fmt.Errorf("failed to parse response: %v | Body: %s", err, string(body))
	}

	return res.TaskID, nil
}

func (s *ElevenLabsUnlimService) GetTaskStatus(apiKey string, taskID string) (string, error) {
	client := &http.Client{Timeout: 30 * time.Second}
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/voice/status/%s", s.baseUrl, taskID), nil)
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res VoicerStatusResponse
	if err := json.Unmarshal(body, &res); err == nil && res.Status != "" {
		if res.Status == "failed" {
			if res.ErrorText != "" {
				return "failed", fmt.Errorf("API error: %s", res.ErrorText)
			}
			return "failed", nil
		}
		return res.Status, nil
	}

	var statusStr string
	if err := json.Unmarshal(body, &statusStr); err == nil {
		return statusStr, nil
	}

	return string(bytes.Trim(body, "\" ")), nil
}

func (s *ElevenLabsUnlimService) DownloadResult(apiKey string, taskID string, filePath string) error {
	client := &http.Client{Timeout: 600 * time.Second}
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/voice/download/%s", s.baseUrl, taskID), nil)
	if err != nil {
		return err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)

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

func (s *ElevenLabsUnlimService) Synthesize(apiKey string, text string, voiceID string, voiceSettings map[string]interface{}, outputPath string, id string, taskLabel string) error {
	if s.OnLog != nil {
		s.OnLog("INFO", "[ElevenLabsUnlim] Starting voice synthesis...", id, taskLabel)
	}

	taskID, err := s.CreateTask(apiKey, text, voiceID, voiceSettings)
	if err != nil {
		return err
	}

	if s.OnLog != nil {
		s.OnLog("INFO", fmt.Sprintf("[ElevenLabsUnlim] Task created: %s. Polling status...", taskID), id, taskLabel)
	}

	maxAttempts := 120
	for i := 0; i < maxAttempts; i++ {
		status, err := s.GetTaskStatus(apiKey, taskID)
		if err != nil {
			return err
		}

		if s.OnLog != nil {
			s.OnLog("INFO", fmt.Sprintf("[ElevenLabsUnlim] Task %s status: %s", taskID, status), id, taskLabel)
		}

		switch status {
		case "completed":
			if s.OnLog != nil {
				s.OnLog("INFO", "[ElevenLabsUnlim] Synthesis completed. Downloading...", id, taskLabel)
			}
			return s.DownloadResult(apiKey, taskID, outputPath)
		case "failed":
			return fmt.Errorf("synthesis failed (status: failed)")
		}

		time.Sleep(5 * time.Second)
	}

	return fmt.Errorf("synthesis timeout")
}

func (s *ElevenLabsUnlimService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsUnlimAPIKey(apiKey)
}

func (s *ElevenLabsUnlimService) GetAPIKey() string {
	return s.settings.GetElevenLabsUnlimAPIKey()
}
