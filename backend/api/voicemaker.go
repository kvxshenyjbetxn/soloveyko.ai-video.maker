package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"time"
)

type VoiceMakerService struct {
	settings *utils.SettingsService
	baseUrl  string
}

func NewVoiceMakerService(settings *utils.SettingsService) *VoiceMakerService {
	return &VoiceMakerService{
		settings: settings,
		baseUrl:  "https://developer.voicemaker.in/api/v1",
	}
}

type VoiceMakerConvertRequest struct {
	Engine       string `json:"Engine"`
	VoiceId      string `json:"VoiceId"`
	LanguageCode string `json:"LanguageCode"`
	Text         string `json:"Text"`
	OutputFormat string `json:"OutputFormat"`
}

type VoiceMakerResponse struct {
	Success        bool   `json:"success"`
	Path           string `json:"path"`
	UsedChars      int    `json:"usedChars"`
	RemainChars    int    `json:"remainChars"`
	RemainKeyChars int    `json:"remainKeyChars"`
	Message        string `json:"message"`
}

// GetBalance отримує поточний баланс користувача шляхом відправки тестового запиту
func (s *VoiceMakerService) GetBalance(apiKey string) (float64, error) {
	if apiKey == "" {
		return 0, fmt.Errorf("API key is empty")
	}

	payload := VoiceMakerConvertRequest{
		Engine:       "neural",
		VoiceId:      "ai3-Jony",
		LanguageCode: "en-US",
		Text:         "Test",
		OutputFormat: "mp3",
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return 0, err
	}

	client := &http.Client{Timeout: 15 * time.Second}
	req, err := http.NewRequest("POST", s.baseUrl+"/voice/convert", bytes.NewBuffer(jsonData))
	if err != nil {
		return 0, err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	var vmResp VoiceMakerResponse
	if err := json.NewDecoder(resp.Body).Decode(&vmResp); err != nil {
		return 0, err
	}

	if !vmResp.Success {
		return 0, fmt.Errorf("API error: %s", vmResp.Message)
	}

	return float64(vmResp.RemainChars), nil
}

// SaveAPIKey зберігає API ключ
func (s *VoiceMakerService) SaveAPIKey(apiKey string) error {
	return s.settings.SetVoiceMakerAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *VoiceMakerService) GetAPIKey() string {
	return s.settings.GetVoiceMakerAPIKey()
}
