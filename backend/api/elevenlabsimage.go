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
	settings  *utils.SettingsService
	baseUrl   string
	sem       chan struct{}
	limit     int
	mu        sync.Mutex
	OnLog     func(level string, message string, details ...string)
	OnLogData func(category string, data string)
}

func NewElevenLabsImageService(settings *utils.SettingsService) *ElevenLabsImageService {
	return &ElevenLabsImageService{
		settings: settings,
		baseUrl:  "https://voiceapi.csv666.ru/api/v2/image",
	}
}

func (s *ElevenLabsImageService) ensureSemaphore() chan struct{} {
	s.mu.Lock()
	defer s.mu.Unlock()

	max := s.settings.GetElevenLabsImageMaxConnections()
	if max > 3 {
		max = 3 // API v2 limit
	}
	if s.sem == nil || s.limit != max {
		s.sem = make(chan struct{}, max)
		s.limit = max
	}

	return s.sem
}

type ImageCreateRequest struct {
	Prompt           string `json:"prompt"`
	AspectRatio      string `json:"aspect_ratio,omitempty"`
	PromptUpsampling bool   `json:"prompt_upsampling"`
	GenerationMode   string `json:"generation_mode"`
	SaveThumbnail    bool   `json:"save_thumbnail"`
	NumImages        int    `json:"num_images"`
}

type ImageTaskResponse struct {
	TaskID  int    `json:"task_id"`
	Status  string `json:"status"`
	Message string `json:"message"`
}

type ImageStatusResponse struct {
	TaskID       int     `json:"task_id"`
	Status       string  `json:"status"` // queued, in_progress, completed, failed, cancelled
	Progress     float64 `json:"progress"`
	ErrorMessage string  `json:"error_message,omitempty"`
}

type ImageResultResponse struct {
	ImageBase64 string `json:"image_base64"`
	Images      []struct {
		ImageBase64 string `json:"image_base64"`
	} `json:"images"`
}

type ImageErrorResponse struct {
	Detail string `json:"detail"`
	Error  string `json:"error,omitempty"`
}

// GenerateImage генерує картинку за допомогою ElevenLabs Image v2 (Асинхронно)
func (s *ElevenLabsImageService) GenerateImage(apiKey string, prompt string, aspectRatio string, outputPath string) error {
	sem := s.ensureSemaphore()
	sem <- struct{}{}
	defer func() { <-sem }()

	client := &http.Client{Timeout: 30 * time.Second}

	// 1. Створити задачу
	createUrl := fmt.Sprintf("%s/generate", s.baseUrl)
	reqBody := ImageCreateRequest{
		Prompt:           prompt,
		AspectRatio:      aspectRatio,
		PromptUpsampling: false,
		GenerationMode:   "quality",
		SaveThumbnail:    false,
		NumImages:        1,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	if s.OnLogData != nil {
		s.OnLogData("ElevenLabs Image v2 Request", fmt.Sprintf("PROMPT: %s\nRATIO: %s", prompt, aspectRatio))
	}

	req, err := http.NewRequest("POST", createUrl, bytes.NewBuffer(jsonData))
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

	if resp.StatusCode != http.StatusAccepted && resp.StatusCode != http.StatusOK {
		var errResp ImageErrorResponse
		json.NewDecoder(resp.Body).Decode(&errResp)
		msg := errResp.Detail
		if msg == "" {
			msg = errResp.Error
		}
		return fmt.Errorf("ElevenLabs Image API (v2) Create failed (%d): %s", resp.StatusCode, msg)
	}

	var task ImageTaskResponse
	if err := json.NewDecoder(resp.Body).Decode(&task); err != nil {
		return err
	}

	if s.OnLog != nil {
		s.OnLog("INFO", fmt.Sprintf("[ElevenLabs Image] Task created: %d, status: %s", task.TaskID, task.Status))
	}

	// 2. Опитування статусу
	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	timeout := time.After(300 * time.Second)
	statusUrl := fmt.Sprintf("%s/tasks/%d/status", s.baseUrl, task.TaskID)

	for {
		select {
		case <-timeout:
			return fmt.Errorf("ElevenLabs Image generation timed out after 5 minutes")
		case <-ticker.C:
			sReq, err := http.NewRequest("GET", statusUrl, nil)
			if err != nil {
				continue
			}
			sReq.Header.Set("X-API-Key", apiKey)

			sResp, err := client.Do(sReq)
			if err != nil {
				continue
			}

			if sResp.StatusCode != http.StatusOK {
				sResp.Body.Close()
				continue
			}

			var status ImageStatusResponse
			err = json.NewDecoder(sResp.Body).Decode(&status)
			sResp.Body.Close()
			if err != nil {
				continue
			}

			if status.Status == "completed" {
				goto retrieveResult
			} else if status.Status == "failed" || status.Status == "cancelled" {
				return fmt.Errorf("ElevenLabs Image task failed or cancelled: %s", status.ErrorMessage)
			}

			if s.OnLog != nil {
				s.OnLog("INFO", fmt.Sprintf("[ElevenLabs Image] Task %d progress: %.2f", task.TaskID, status.Progress))
			}
		}
	}

retrieveResult:
	// 3. Отримання результату
	resultUrl := fmt.Sprintf("%s/tasks/%d/result?image_base64=true", s.baseUrl, task.TaskID)
	rReq, err := http.NewRequest("GET", resultUrl, nil)
	if err != nil {
		return err
	}
	rReq.Header.Set("X-API-Key", apiKey)

	rResp, err := client.Do(rReq)
	if err != nil {
		return err
	}
	defer rResp.Body.Close()

	if rResp.StatusCode != http.StatusOK {
		return fmt.Errorf("ElevenLabs Image failed to retrieve result (%d)", rResp.StatusCode)
	}

	var res ImageResultResponse
	if err := json.NewDecoder(rResp.Body).Decode(&res); err != nil {
		return err
	}

	b64 := res.ImageBase64
	if b64 == "" && len(res.Images) > 0 {
		b64 = res.Images[0].ImageBase64
	}

	if b64 == "" {
		return fmt.Errorf("API v2 returned empty image data")
	}

	err = utils.SaveBase64Image(b64, outputPath)
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
