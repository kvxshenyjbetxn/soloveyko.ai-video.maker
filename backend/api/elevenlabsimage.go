package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"soloveyko/backend/utils"
	"sync"
	"time"
)

type ElevenLabsImageService struct {
	settings *utils.SettingsService
	baseUrl  string
	sem      chan struct{}
	limit    int
	mu       sync.Mutex
	OnLog    func(level string, message string, details ...string)
}

func NewElevenLabsImageService(settings *utils.SettingsService) *ElevenLabsImageService {
	return &ElevenLabsImageService{
		settings: settings,
		baseUrl:  "https://voiceapi.csv666.ru/api/v1/image",
	}
}

func (s *ElevenLabsImageService) ensureSemaphore() chan struct{} {
	s.mu.Lock()
	defer s.mu.Unlock()

	max := s.settings.GetElevenLabsImageMaxConnections()
	if s.sem == nil || s.limit != max {
		s.sem = make(chan struct{}, max)
		s.limit = max
	}

	return s.sem
}

type ImageCreateRequest struct {
	Prompt      string `json:"prompt"`
	AspectRatio string `json:"aspect_ratio,omitempty"`
}

type ImageResultResponse struct {
	ImageB64 string `json:"image_b64"`
}

type ImageErrorResponse struct {
	Detail    string `json:"detail"`
	ErrorCode string `json:"error_code,omitempty"`
}

// GenerateImage генерує картинку за допомогою ElevenLabs Image
func (s *ElevenLabsImageService) GenerateImage(apiKey string, prompt string, aspectRatio string, outputPath string) error {
	sem := s.ensureSemaphore()
	sem <- struct{}{}
	defer func() { <-sem }()

	client := &http.Client{Timeout: 300 * time.Second}
	url := fmt.Sprintf("%s/create", s.baseUrl)

	reqBody := ImageCreateRequest{
		Prompt:      prompt,
		AspectRatio: aspectRatio,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-API-Key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var errResp ImageErrorResponse
		json.NewDecoder(resp.Body).Decode(&errResp)
		return fmt.Errorf("ElevenLabs Image API failed (%d): %s", resp.StatusCode, errResp.Detail)
	}

	var res ImageResultResponse
	if err := json.NewDecoder(resp.Body).Decode(&res); err != nil {
		return err
	}

	if res.ImageB64 == "" {
		return fmt.Errorf("API returned empty image data")
	}

	err = utils.SaveBase64Image(res.ImageB64, outputPath)
	return err
}

type ElevenLabsImageUsage struct {
	ActiveThreads int `json:"active_threads"`
	MaxThreads    int `json:"max_threads"`
}

// GetUsage повертає поточне використання потоків
func (s *ElevenLabsImageService) GetUsage() ElevenLabsImageUsage {
	active := 0
	limit := s.settings.GetElevenLabsImageMaxConnections()

	s.mu.Lock()
	if s.sem != nil {
		active = len(s.sem)
	}
	s.mu.Unlock()

	return ElevenLabsImageUsage{
		ActiveThreads: active,
		MaxThreads:    limit,
	}
}

// SaveAPIKey зберігає API ключ
func (s *ElevenLabsImageService) SaveAPIKey(apiKey string) error {
	return s.settings.SetElevenLabsImageAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *ElevenLabsImageService) GetAPIKey() string {
	return s.settings.GetElevenLabsImageAPIKey()
}
