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

type ElevenLabsUAService struct {
	settings *utils.SettingsService
	baseUrl  string
	OnLog    func(level string, message string, details ...string)
}

func NewElevenLabsUAService(settings *utils.SettingsService) *ElevenLabsUAService {
	return &ElevenLabsUAService{
		settings: settings,
		baseUrl:  "https://11tts.net/v1",
	}
}

type ElevenLabsUABalanceResponse struct {
	CharacterLimit      int `json:"character_limit"`
	CharactersUsed      int `json:"characters_used"`
	CharactersRemaining int `json:"characters_remaining"`
}

// GetBalance повертає баланс символів користувача
func (s *ElevenLabsUAService) GetBalance(apiKey string) (float64, error) {
	if apiKey == "" {
		return 0, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	url := fmt.Sprintf("%s/user/balance", s.baseUrl)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return 0, err
	}

	req.Header.Set("xi-api-key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return 0, fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res ElevenLabsUABalanceResponse
	if err := json.Unmarshal(body, &res); err != nil {
		return 0, err
	}

	return float64(res.CharactersRemaining), nil
}

type ElevenLabsUAVoiceSettings struct {
	Stability       float64 `json:"stability"`
	SimilarityBoost float64 `json:"similarity_boost"`
	Style           float64 `json:"style"`
	UseSpeakerBoost bool    `json:"use_speaker_boost"`
}

type ElevenLabsUACreateRequest struct {
	Text          string                     `json:"text"`
	VoiceID       string                     `json:"voice_id"`
	ModelID       string                     `json:"model_id,omitempty"`
	VoiceSettings *ElevenLabsUAVoiceSettings `json:"voice_settings,omitempty"`
}

type ElevenLabsUACreateResponse struct {
	ID     interface{} `json:"id"` // Може бути int або string
	Status string      `json:"status"`
}

type ElevenLabsUAStatusResponse struct {
	ID             interface{} `json:"id"`
	Status         string      `json:"status"`
	AudioUrl       string      `json:"audio_url,omitempty"`
	CharacterCount int         `json:"character_count,omitempty"`
	ErrorMessage   string      `json:"error_message,omitempty"`
}

// CreateTask створює завдання на генерацію
func (s *ElevenLabsUAService) CreateTask(apiKey string, text string, voiceID string, modelID string, voiceSettings *ElevenLabsUAVoiceSettings) (string, error) {
	if apiKey == "" {
		return "", fmt.Errorf("API key is empty")
	}

	reqBody := ElevenLabsUACreateRequest{
		Text:          text,
		VoiceID:       voiceID,
		ModelID:       modelID,
		VoiceSettings: voiceSettings,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	client := &http.Client{Timeout: 30 * time.Second}
	url := fmt.Sprintf("%s/text-to-speech/%s?output_format=mp3_44100_128", s.baseUrl, voiceID)
	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("xi-api-key", apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if s.OnLog != nil {
		s.OnLog("DEBUG", fmt.Sprintf("[ElevenLabsUA] CreateTask response (%d): %s", resp.StatusCode, string(body)))
	}

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusAccepted {
		return "", fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res map[string]interface{}
	if err := json.Unmarshal(body, &res); err != nil {
		return "", fmt.Errorf("failed to parse response: %v | Body: %s", err, string(body))
	}

	// Спробуємо різні ключі ID
	idKeys := []string{"id", "task_id", "uuid", "request_id"}
	for _, key := range idKeys {
		if val, ok := res[key]; ok && val != nil {
			return fmt.Sprintf("%v", val), nil
		}
	}

	return "", fmt.Errorf("could not find task ID in response: %s", string(body))
}

// GetTaskStatus перевіряє статус
func (s *ElevenLabsUAService) GetTaskStatus(apiKey string, taskID string) (*ElevenLabsUAStatusResponse, error) {
	client := &http.Client{Timeout: 15 * time.Second}
	url := fmt.Sprintf("%s/text-to-speech/%s/status", s.baseUrl, taskID)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("xi-api-key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var res ElevenLabsUAStatusResponse
	if err := json.Unmarshal(body, &res); err != nil {
		return nil, err
	}

	return &res, nil
}

// DownloadFile завантажує файл за посиланням
func (s *ElevenLabsUAService) DownloadFile(url string, filePath string) error {
	client := &http.Client{Timeout: 300 * time.Second}
	resp, err := client.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("download error %d", resp.StatusCode)
	}

	out, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

// Synthesize виконує повний цикл озвучення
func (s *ElevenLabsUAService) Synthesize(apiKey string, text string, voiceID string, modelID string, voiceSettings *ElevenLabsUAVoiceSettings, outputPath string, id string, taskLabel string) error {
	if s.OnLog != nil {
		s.OnLog("INFO", "[ElevenLabsUA] Starting voice synthesis...", id, taskLabel)
	}

	if modelID == "" {
		modelID = "eleven_multilingual_v2"
	}

	taskID, err := s.CreateTask(apiKey, text, voiceID, modelID, voiceSettings)
	if err != nil {
		return err
	}

	if s.OnLog != nil {
		s.OnLog("INFO", fmt.Sprintf("[ElevenLabsUA] Task created: %s. Polling status...", taskID), id, taskLabel)
	}

	maxAttempts := 120
	for i := 0; i < maxAttempts; i++ {
		statusRes, err := s.GetTaskStatus(apiKey, taskID)
		if err != nil {
			return err
		}

		if s.OnLog != nil {
			s.OnLog("INFO", fmt.Sprintf("[ElevenLabsUA] Task %s status: %s", taskID, statusRes.Status), id, taskLabel)
		}

		switch statusRes.Status {
		case "success":
			if statusRes.AudioUrl == "" {
				return fmt.Errorf("synthesis finished but audio_url is empty")
			}
			if s.OnLog != nil {
				s.OnLog("INFO", "[ElevenLabsUA] Synthesis completed. Downloading...", id, taskLabel)
			}
			return s.DownloadFile(statusRes.AudioUrl, outputPath)
		case "failed":
			if statusRes.ErrorMessage != "" {
				return fmt.Errorf("synthesis failed: %s", statusRes.ErrorMessage)
			}
			return fmt.Errorf("synthesis failed")
		}

		// Поступове збільшення інтервалу опитування (backoff)
		pollInterval := 5 * time.Second
		if i < 5 { // Перші 5 спроб - кожні 2 секунди
			pollInterval = 2 * time.Second
		}
		time.Sleep(pollInterval)
	}

	return fmt.Errorf("synthesis timeout")
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsUAService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsUAAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsUAService) GetAPIKey() string {
	return s.settings.GetElevenLabsUAAPIKey()
}
