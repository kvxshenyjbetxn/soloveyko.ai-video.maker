package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"soloveyko/backend/utils"
	"strings"
	"sync"
	"time"
)

type GooglerService struct {
	settings  *utils.SettingsService
	baseUrl   string
	imgSem    chan struct{}
	vidSem    chan struct{}
	imgLimit  int
	vidLimit  int
	mu        sync.Mutex
	OnLog     func(level string, message string, details ...string)
	OnLogData func(category string, data string)
}

func NewGooglerService(settings *utils.SettingsService) *GooglerService {
	return &GooglerService{
		settings: settings,
		baseUrl:  "https://googler.fast-gen.ai/api",
	}
}

func (s *GooglerService) ensureSemaphores() (chan struct{}, chan struct{}) {
	s.mu.Lock()
	defer s.mu.Unlock()

	imgMax := s.settings.GetGooglerMaxImageConnections()
	if s.imgSem == nil || s.imgLimit != imgMax {
		s.imgSem = make(chan struct{}, imgMax)
		s.imgLimit = imgMax
	}

	vidMax := s.settings.GetGooglerMaxVideoConnections()
	if s.vidSem == nil || s.vidLimit != vidMax {
		s.vidSem = make(chan struct{}, vidMax)
		s.vidLimit = vidMax
	}

	return s.imgSem, s.vidSem
}

type FlexibleFloat64 float64

func (f *FlexibleFloat64) UnmarshalJSON(data []byte) error {
	// Спробуємо розпарсити як число
	var n float64
	if err := json.Unmarshal(data, &n); err == nil {
		*f = FlexibleFloat64(n)
		return nil
	}

	// Спробуємо розпарсити як об'єкт {"used": X} або {"count": X}
	var obj map[string]float64
	if err := json.Unmarshal(data, &obj); err == nil {
		if val, ok := obj["used"]; ok {
			*f = FlexibleFloat64(val)
			return nil
		}
		if val, ok := obj["count"]; ok {
			*f = FlexibleFloat64(val)
			return nil
		}
	}

	*f = 0
	return nil
}

type GooglerAccountLimits struct {
	ImgGenPerHourLimit            FlexibleFloat64 `json:"img_gen_per_hour_limit"`
	VideoGenPerHourLimit          FlexibleFloat64 `json:"video_gen_per_hour_limit"`
	ImgGenerationThreadsAllowed   FlexibleFloat64 `json:"img_generation_threads_allowed"`
	VideoGenerationThreadsAllowed FlexibleFloat64 `json:"video_generation_threads_allowed"`
	PromptTokensPerHourLimit      FlexibleFloat64 `json:"prompt_tokens_per_hour_limit"`
}

type GooglerActiveThreads struct {
	ImageThreads FlexibleFloat64 `json:"image_threads"`
	VideoThreads FlexibleFloat64 `json:"video_threads"`
}

type GooglerHourlyUsage struct {
	ImageGeneration  FlexibleFloat64 `json:"image_generation"`
	VideoGeneration  FlexibleFloat64 `json:"video_generation"`
	PromptGeneration FlexibleFloat64 `json:"prompt_generation"`
}

type GooglerCurrentUsage struct {
	HourlyUsage   GooglerHourlyUsage   `json:"hourly_usage"`
	ActiveThreads GooglerActiveThreads `json:"active_threads"`
}

type GooglerUsageResponse struct {
	ApiKey         string               `json:"api_key"`
	AccountLimits  GooglerAccountLimits `json:"account_limits"`
	CurrentUsage   GooglerCurrentUsage  `json:"current_usage"`
	UsageWindow    string               `json:"usage_window"`
	ActivationDate FlexibleFloat64      `json:"activation_date"`
	ExpirationDate FlexibleFloat64      `json:"expiration_date"`
}

// GetUsage отримує статистику використання акаунту
func (s *GooglerService) GetUsage(apiKey string) (*GooglerUsageResponse, error) {
	if apiKey == "" {
		return nil, fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	// Спробуємо v3 ендпоінт, як у спеці
	url := fmt.Sprintf("%s/v3/account/usage?api_key=%s", s.baseUrl, apiKey)
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("X-API-Key", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		fmt.Printf("Googler API Request Error: %v\n", err)
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var errData map[string]interface{}
		json.NewDecoder(resp.Body).Decode(&errData)
		fmt.Printf("Googler API Error Response: %d - %v\n", resp.StatusCode, errData)
		return nil, fmt.Errorf("API error: %d", resp.StatusCode)
	}

	var usage GooglerUsageResponse
	if err := json.NewDecoder(resp.Body).Decode(&usage); err != nil {
		fmt.Printf("Googler API Decode Error: %v\n", err)
		return nil, err
	}

	return &usage, nil
}

// SaveAPIKey зберігає API ключ
func (s *GooglerService) SaveAPIKey(apiKey string) error {
	return s.settings.SetGooglerAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *GooglerService) GetAPIKey() string {
	return s.settings.GetGooglerAPIKey()
}

type GenericImageRequest struct {
	Prompt      string `json:"prompt"`
	AspectRatio string `json:"aspect_ratio,omitempty"`
}

type FlowImageRequest struct {
	Prompt      string `json:"prompt"`
	AspectRatio string `json:"aspect_ratio,omitempty"`
	Model       string `json:"model,omitempty"`
}

type OperationResponse struct {
	Success       bool   `json:"success"`
	OperationID   string `json:"operation_id"`
	OperationType string `json:"operation_type"`
	Status        string `json:"status"`
}

type OperationStatusResponse struct {
	OperationID string      `json:"operation_id"`
	Status      string      `json:"status"`
	Result      interface{} `json:"result"` // can be string or []string
	Error       string      `json:"error,omitempty"`
}

// GenerateImage генерує картинку за допомогою Googler з автоматичними повторами
func (s *GooglerService) GenerateImage(apiKey string, model string, prompt string, aspectRatio string, outputPath string) error {
	imgSem, _ := s.ensureSemaphores()
	imgSem <- struct{}{}
	defer func() { <-imgSem }()

	// Мапінг аспект-ратіо під вимоги API Googler
	apiRatio := aspectRatio
	if model == "grok" {
		// Grok використовує короткі назви (16:9, 1:1 і т.д.)
		switch apiRatio {
		case "IMAGE_ASPECT_RATIO_LANDSCAPE":
			apiRatio = "16:9"
		case "IMAGE_ASPECT_RATIO_PORTRAIT":
			apiRatio = "9:16"
		case "IMAGE_ASPECT_RATIO_SQUARE":
			apiRatio = "1:1"
		}
	} else {
		// Flow, Whisk та інші використовують довгі назви
		if !strings.HasPrefix(apiRatio, "IMAGE_ASPECT_RATIO_") {
			switch apiRatio {
			case "16:9":
				apiRatio = "IMAGE_ASPECT_RATIO_LANDSCAPE"
			case "9:16":
				apiRatio = "IMAGE_ASPECT_RATIO_PORTRAIT"
			case "1:1":
				apiRatio = "IMAGE_ASPECT_RATIO_SQUARE"
			default:
				apiRatio = "IMAGE_ASPECT_RATIO_LANDSCAPE"
			}
		}
	}

	maxRetries := 3
	var lastErr error

	for attempt := 1; attempt <= maxRetries; attempt++ {
		if attempt > 1 {
			if s.OnLog != nil {
				s.OnLog("INFO", fmt.Sprintf("[Googler] Retrying image generation (attempt %d/%d)...", attempt, maxRetries))
			}
			time.Sleep(10 * time.Second)
		}

		err := s.generateImageOnce(apiKey, model, prompt, apiRatio, outputPath)
		if err == nil {
			return nil
		}

		lastErr = err
		errMsg := strings.ToLower(err.Error())

		// Ретрай тільки для мережевих помилок або тимчасових помилок сервера
		shouldRetry := strings.Contains(errMsg, "timeout") ||
			strings.Contains(errMsg, "no images were generated") ||
			strings.Contains(errMsg, "internal error") ||
			strings.Contains(errMsg, "rate limit") ||
			strings.Contains(errMsg, "connection")

		if !shouldRetry {
			break
		}
	}

	return lastErr
}

func (s *GooglerService) generateImageOnce(apiKey string, model string, prompt string, apiRatio string, outputPath string) error {
	client := &http.Client{Timeout: 300 * time.Second}
	var url string
	var reqBody interface{}

	// Визначаємо ендпоінт та тіло запиту на основі моделі
	switch model {
	case "flow":
		url = fmt.Sprintf("%s/v4/flow/image/generate", s.baseUrl)
		reqBody = FlowImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "whisk":
		url = fmt.Sprintf("%s/v4/whisk/image/generate", s.baseUrl)
		reqBody = GenericImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "grok":
		url = fmt.Sprintf("%s/v4/grok/image/generate", s.baseUrl)
		reqBody = GenericImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "gemini":
		// Gemini (Imagen 4) тепер у v4. Згідно googler.json, вона не приймає aspect_ratio в v4
		url = fmt.Sprintf("%s/v4/gemini/image/generate", s.baseUrl)
		reqBody = map[string]interface{}{
			"prompt": prompt,
		}
	default:
		return fmt.Errorf("unknown model: %s", model)
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	if s.OnLogData != nil {
		s.OnLogData("Googler Image Request", fmt.Sprintf("MODEL: %s\nPROMPT: %s\nRATIO: %s", model, prompt, apiRatio))
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
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("Googler API failed (%d): %s", resp.StatusCode, string(body))
	}

	// v4 (всі моделі тепер тут) повертають operation_id

	// Асинхронна обробка для v4
	var opResp OperationResponse
	if err := json.NewDecoder(resp.Body).Decode(&opResp); err != nil {
		return err
	}

	if opResp.OperationID == "" {
		return fmt.Errorf("no operation_id returned")
	}

	// Polling
	maxRetries := 60 // 5 minutes (5s * 60)
	for i := 0; i < maxRetries; i++ {
		time.Sleep(5 * time.Second)

		statusUrl := fmt.Sprintf("%s/v4/operations/%s", s.baseUrl, opResp.OperationID)
		sReq, err := http.NewRequest("GET", statusUrl, nil)
		if err != nil {
			return err
		}
		sReq.Header.Set("X-API-Key", apiKey)

		sResp, err := client.Do(sReq)
		if err != nil {
			continue // try again
		}

		var stResp OperationStatusResponse
		if err := json.NewDecoder(sResp.Body).Decode(&stResp); err != nil {
			sResp.Body.Close()
			continue
		}
		sResp.Body.Close()

		if stResp.Status == "success" {
			// Result can be string or []string (for grok)
			var base64Data string
			switch v := stResp.Result.(type) {
			case string:
				base64Data = v
			case []interface{}:
				if len(v) > 0 {
					if s, ok := v[0].(string); ok {
						base64Data = s
					}
				}
			}

			if base64Data == "" {
				return fmt.Errorf("empty result in success status")
			}

			return utils.SaveBase64Image(base64Data, outputPath)
		}

		if stResp.Status == "error" {
			return fmt.Errorf("Googler task failed: %s", stResp.Error)
		}
	}

	return fmt.Errorf("Googler timeout after 5 minutes")
}
