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

type VoiceMakerService struct {
	settings *utils.SettingsService
	baseUrl  string
	OnLog    func(level string, message string, details ...string)
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
	SampleRate   string `json:"SampleRate,omitempty"`
	ResponseType string `json:"ResponseType,omitempty"`
}

type VoiceMakerResponse struct {
	Success        bool   `json:"success"`
	Path           string `json:"path"`
	UsedChars      int    `json:"usedChars"`
	RemainChars    int    `json:"remainChars"`
	RemainKeyChars int    `json:"remainKeyChars"`
	Message        string `json:"message"`
}

type VoicemakerVoice struct {
	Engine       string `json:"Engine"`
	VoiceId      string `json:"VoiceId"`
	VoiceGender  string `json:"VoiceGender"`
	VoiceWebname string `json:"VoiceWebname"`
	Country      string `json:"Country"`
	Language     string `json:"Language"`
	LanguageName string `json:"LanguageName"`
}

type VoicemakerListResponse struct {
	Success bool `json:"success"`
	Data    struct {
		VoicesList []VoicemakerVoice `json:"voices_list"`
	} `json:"data"`
}

// GetBalance отримує поточний баланс користувача
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

// GetVoicesList отримує список голосів від API
func (s *VoiceMakerService) GetVoicesList(apiKey string) ([]VoicemakerVoice, error) {
	if apiKey == "" {
		return nil, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 30 * time.Second}
	req, err := http.NewRequest("POST", s.baseUrl+"/voice/list", bytes.NewBuffer([]byte("{}")))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var listResp VoicemakerListResponse
	if err := json.NewDecoder(resp.Body).Decode(&listResp); err != nil {
		return nil, err
	}

	if !listResp.Success {
		return nil, fmt.Errorf("failed to fetch voice list")
	}

	return listResp.Data.VoicesList, nil
}

// Synthesize виконує синтез тексту в аудіо
func (s *VoiceMakerService) Synthesize(apiKey string, text string, voiceId string, languageCode string, outputPath string, id string, taskLabel string) error {
	payload := VoiceMakerConvertRequest{
		Engine:       "neural", // За замовчуванням neural, можна змінити якщо треба
		VoiceId:      voiceId,
		LanguageCode: languageCode,
		Text:         text,
		OutputFormat: "mp3",
	}

	// Визначаємо Engine на основі VoiceId
	if len(voiceId) > 3 {
		enginePrefix := voiceId[:3]
		switch enginePrefix {
		case "ai1":
			payload.Engine = "standard"
		case "ai2":
			payload.Engine = "neural"
		case "ai3":
			payload.Engine = "neural"
		case "ai4":
			payload.Engine = "neural"
		case "pro":
			payload.Engine = "neural"
		}
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	client := &http.Client{Timeout: 60 * time.Second}
	req, err := http.NewRequest("POST", s.baseUrl+"/voice/convert", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	var vmResp VoiceMakerResponse
	if err := json.NewDecoder(resp.Body).Decode(&vmResp); err != nil {
		return err
	}

	if !vmResp.Success {
		return fmt.Errorf("Voicemaker error: %s", vmResp.Message)
	}

	// Завантаження файлу
	audioResp, err := http.Get(vmResp.Path)
	if err != nil {
		return err
	}
	defer audioResp.Body.Close()

	if audioResp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to download audio from %s", vmResp.Path)
	}

	out, err := os.Create(outputPath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, audioResp.Body)
	return err
}

// SaveAPIKey зберігає API ключ
func (s *VoiceMakerService) SaveAPIKey(apiKey string) error {
	return s.settings.SetVoiceMakerAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *VoiceMakerService) GetAPIKey() string {
	return s.settings.GetVoiceMakerAPIKey()
}
