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

	// Спробуємо розпарсити як об'єкт {"used": X}, {"count": X} або {"current_usage": X}
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
		if val, ok := obj["current_usage"]; ok {
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

type RemixImageRequest struct {
	Prompt          string           `json:"prompt"`
	ReferenceImages []ReferenceImage `json:"reference_images"`
	AspectRatio     string           `json:"aspect_ratio,omitempty"`
	StrictMode      bool             `json:"strict_mode"`
}

type ReferenceImage struct {
	Category string `json:"category"`
	Image    string `json:"image"`
}

// GenerateImage генерує картинку за допомогою Googler з автоматичними повторами
func (s *GooglerService) GenerateImage(apiKey string, model string, prompt string, aspectRatio string, outputPath string) error {
	imgSem, _ := s.ensureSemaphores()
	imgSem <- struct{}{}
	defer func() { <-imgSem }()

	// Fallback list
	allModels := []string{"whisk", "flow", "grok", "gemini"}
	startIndex := -1
	for i, m := range allModels {
		if m == model {
			startIndex = i
			break
		}
	}
	if startIndex == -1 {
		allModels = append([]string{model}, allModels...)
		startIndex = 0
	}

	var lastErr error
	for i := startIndex; i < len(allModels); i++ {
		currentModel := allModels[i]

		// Map aspect ratio for current model
		apiRatio := aspectRatio
		if currentModel == "grok" {
			switch apiRatio {
			case "IMAGE_ASPECT_RATIO_LANDSCAPE":
				apiRatio = "16:9"
			case "IMAGE_ASPECT_RATIO_PORTRAIT":
				apiRatio = "9:16"
			case "IMAGE_ASPECT_RATIO_SQUARE":
				apiRatio = "1:1"
			}
		} else {
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

		for attempt := 1; ; attempt++ {
			if attempt > 1 {
				waitTime := 5 * time.Second
				isRateLimit := lastErr != nil && (strings.Contains(lastErr.Error(), "(429)") || strings.Contains(strings.ToLower(lastErr.Error()), "rate limit"))

				if isRateLimit {
					waitTime = 5 * time.Minute
					if s.OnLog != nil {
						s.OnLog("WARN", fmt.Sprintf("[Googler] Image (%s) rate limit exceeded (429). Waiting 5 minutes before retry %d (infinite mode)...", currentModel, attempt))
					}
				} else {
					if attempt > 3 {
						break // Exit retry loop for non-429 errors after 3 attempts
					}
					if s.OnLog != nil {
						s.OnLog("INFO", fmt.Sprintf("[Googler] Retrying %s (%d/3) in 5s...", currentModel, attempt))
					}
				}
				time.Sleep(waitTime)
			}

			err := s.generateImageOnce(apiKey, currentModel, prompt, apiRatio, outputPath)
			if err == nil {
				return nil
			}

			lastErr = err
			if !s.isRetryable(err) {
				break
			}
		}

		if i < len(allModels)-1 {
			if s.OnLog != nil {
				s.OnLog("WARN", fmt.Sprintf("[Googler] %s failed -> Falling back to %s", currentModel, allModels[i+1]))
			}
			time.Sleep(2 * time.Second)
		}
	}

	return lastErr
}

func (s *GooglerService) isRetryable(err error) bool {
	if err == nil {
		return false
	}
	errMsg := strings.ToLower(err.Error())
	return strings.Contains(errMsg, "timeout") ||
		strings.Contains(errMsg, "no image") ||
		strings.Contains(errMsg, "no video") ||
		strings.Contains(errMsg, "internal error") ||
		strings.Contains(errMsg, "rate limit") ||
		strings.Contains(errMsg, "429") ||
		strings.Contains(errMsg, "connection") ||
		strings.Contains(errMsg, "500") ||
		strings.Contains(errMsg, "502") ||
		strings.Contains(errMsg, "503") ||
		strings.Contains(errMsg, "504")
}

func (s *GooglerService) generateImageOnce(apiKey string, model string, prompt string, apiRatio string, outputPath string) error {
	client := &http.Client{Timeout: 300 * time.Second}
	var url string
	var reqBody interface{}

	// Визначаємо ендпоінт та тіло запиту на основі моделі
	switch model {
	case "flow":
		url = fmt.Sprintf("%s/v4/flow/image/generate?api_key=%s", s.baseUrl, apiKey)
		reqBody = FlowImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "whisk":
		url = fmt.Sprintf("%s/v4/whisk/image/generate?api_key=%s", s.baseUrl, apiKey)
		reqBody = GenericImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "grok":
		url = fmt.Sprintf("%s/v4/grok/image/generate?api_key=%s", s.baseUrl, apiKey)
		reqBody = GenericImageRequest{
			Prompt:      prompt,
			AspectRatio: apiRatio,
		}
	case "gemini":
		// Gemini (Imagen 4) тепер у v4. Згідно googler.json, вона не приймає aspect_ratio в v4
		url = fmt.Sprintf("%s/v4/gemini/image/generate?api_key=%s", s.baseUrl, apiKey)
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

		statusUrl := fmt.Sprintf("%s/v4/operations/%s?api_key=%s", s.baseUrl, opResp.OperationID, apiKey)
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

// RemixImage генерує картинку на основі референсів (Style/Subject/Scene) з автоматичними повторами та фалбеком
func (s *GooglerService) RemixImage(apiKey string, prompt string, referenceImages []ReferenceImage, aspectRatio string, strictMode bool, outputPath string) error {
	imgSem, _ := s.ensureSemaphores()
	imgSem <- struct{}{}
	defer func() { <-imgSem }()
	var lastErr error
	for attempt := 1; ; attempt++ {
		if attempt > 1 {
			waitTime := 5 * time.Second
			isRateLimit := lastErr != nil && (strings.Contains(lastErr.Error(), "(429)") || strings.Contains(strings.ToLower(lastErr.Error()), "rate limit"))

			if isRateLimit {
				waitTime = 5 * time.Minute
				if s.OnLog != nil {
					s.OnLog("WARN", fmt.Sprintf("[Googler] Remix rate limit exceeded (429). Waiting 5 minutes before retry %d (infinite mode)...", attempt))
				}
			} else {
				if attempt > 3 {
					break // Exit retry loop for non-429 errors after 3 attempts
				}
				if s.OnLog != nil {
					s.OnLog("INFO", fmt.Sprintf("[Googler] Retrying remix (%d/3) in 5s...", attempt))
				}
			}
			time.Sleep(waitTime)
		}

		err := s.remixImageOnce(apiKey, prompt, referenceImages, aspectRatio, strictMode, outputPath)
		if err == nil {
			return nil
		}
		lastErr = err

		if !s.isRetryable(err) {
			break
		}
	}

	// Fallback to standard Image generation with next models
	if s.OnLog != nil {
		s.OnLog("WARN", "[Googler] Remix failed -> Falling back to standard Flow generation")
	}
	return s.GenerateImage(apiKey, "flow", prompt, aspectRatio, outputPath)
}

func (s *GooglerService) remixImageOnce(apiKey string, prompt string, referenceImages []ReferenceImage, aspectRatio string, strictMode bool, outputPath string) error {
	client := &http.Client{Timeout: 300 * time.Second}
	url := fmt.Sprintf("%s/v4/whisk/image/remix?api_key=%s", s.baseUrl, apiKey)

	reqBody := RemixImageRequest{
		Prompt:          prompt,
		ReferenceImages: referenceImages,
		AspectRatio:     aspectRatio,
		StrictMode:      strictMode,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	if s.OnLogData != nil {
		s.OnLogData("Googler Image Remix Request", fmt.Sprintf("PROMPT: %s\nRATIO: %s\nSTRICT: %v\nREFS: %d", prompt, aspectRatio, strictMode, len(referenceImages)))
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
		return fmt.Errorf("Googler Remix API failed (%d): %s", resp.StatusCode, string(body))
	}

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

		statusUrl := fmt.Sprintf("%s/v4/operations/%s?api_key=%s", s.baseUrl, opResp.OperationID, apiKey)
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
			return fmt.Errorf("Googler remix task failed: %s", stResp.Error)
		}
	}

	return fmt.Errorf("Googler remix timeout after 5 minutes")
}

// GenerateVideo генерує відео за допомогою Googler (text-to-video або image-to-video) з автоматичними повторами та фалбеком
func (s *GooglerService) GenerateVideo(apiKey string, model string, prompt string, imageBase64 string, aspectRatio string, upscale bool, outputPath string) error {
	_, vidSem := s.ensureSemaphores()
	vidSem <- struct{}{}
	defer func() { <-vidSem }()

	// Fallback list
	allModels := []string{"whisk", "flow", "grok", "gemini"}
	startIndex := -1
	for i, m := range allModels {
		if m == model {
			startIndex = i
			break
		}
	}
	if startIndex == -1 {
		allModels = append([]string{model}, allModels...)
		startIndex = 0
	}

	var lastErr error
	for i := startIndex; i < len(allModels); i++ {
		currentModel := allModels[i]

		for attempt := 1; ; attempt++ {
			if attempt > 1 {
				waitTime := 5 * time.Second
				isRateLimit := lastErr != nil && (strings.Contains(lastErr.Error(), "(429)") || strings.Contains(strings.ToLower(lastErr.Error()), "rate limit"))

				if isRateLimit {
					waitTime = 5 * time.Minute
					if s.OnLog != nil {
						s.OnLog("WARN", fmt.Sprintf("[Googler] Video (%s) rate limit exceeded (429). Waiting 5 minutes before retry %d (infinite mode)...", currentModel, attempt))
					}
				} else {
					if attempt > 3 {
						break // Exit retry loop for non-429 errors after 3 attempts
					}
					if s.OnLog != nil {
						s.OnLog("INFO", fmt.Sprintf("[Googler] Retrying video (%s) [%d/3] in 5s...", currentModel, attempt))
					}
				}
				time.Sleep(waitTime)
			}

			err := s.generateVideoOnce(apiKey, currentModel, prompt, imageBase64, aspectRatio, upscale, outputPath)
			if err == nil {
				return nil
			}

			lastErr = err
			if !s.isRetryable(err) {
				break
			}
		}

		if i < len(allModels)-1 {
			if s.OnLog != nil {
				s.OnLog("WARN", fmt.Sprintf("[Googler] Video %s failed -> Falling back to %s", currentModel, allModels[i+1]))
			}
			time.Sleep(2 * time.Second)
		}
	}

	return lastErr
}

func (s *GooglerService) generateVideoOnce(apiKey string, model string, prompt string, imageBase64 string, aspectRatio string, upscale bool, outputPath string) error {
	client := &http.Client{Timeout: 300 * time.Second}
	var url string
	var reqBody interface{}

	// Map aspect ratio
	apiRatio := aspectRatio
	if model == "grok" {
		switch apiRatio {
		case "IMAGE_ASPECT_RATIO_LANDSCAPE":
			apiRatio = "16:9"
		case "IMAGE_ASPECT_RATIO_PORTRAIT":
			apiRatio = "9:16"
		case "IMAGE_ASPECT_RATIO_SQUARE":
			apiRatio = "1:1"
		}
	} else {
		if !strings.HasPrefix(apiRatio, "VIDEO_ASPECT_RATIO_") {
			switch apiRatio {
			case "16:9", "IMAGE_ASPECT_RATIO_LANDSCAPE":
				apiRatio = "VIDEO_ASPECT_RATIO_LANDSCAPE"
			case "9:16", "IMAGE_ASPECT_RATIO_PORTRAIT":
				apiRatio = "VIDEO_ASPECT_RATIO_PORTRAIT"
			default:
				apiRatio = "VIDEO_ASPECT_RATIO_LANDSCAPE"
			}
		}
	}

	if imageBase64 != "" {
		// Image to video
		switch model {
		case "flow":
			url = fmt.Sprintf("%s/v4/flow/video/from-ingredients?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":           prompt,
				"reference_images": []string{imageBase64},
				"aspect_ratio":     apiRatio,
			}
		case "whisk":
			url = fmt.Sprintf("%s/v4/whisk/video/from-image?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":      prompt,
				"input_image": imageBase64,
			}
		case "grok":
			url = fmt.Sprintf("%s/v4/grok/video/from-image?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":  prompt,
				"image":   imageBase64,
				"upscale": upscale,
			}
		case "gemini":
			url = fmt.Sprintf("%s/v4/gemini/video/generate?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":           prompt,
				"reference_images": []string{imageBase64},
			}
		default:
			return fmt.Errorf("unknown video model: %s", model)
		}
	} else {
		// Text to video
		switch model {
		case "flow":
			url = fmt.Sprintf("%s/v4/flow/video/from-text?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":       prompt,
				"aspect_ratio": apiRatio,
			}
		case "whisk":
			url = fmt.Sprintf("%s/v4/whisk/video/from-text?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt": prompt,
			}
		case "grok":
			url = fmt.Sprintf("%s/v4/grok/video/from-text?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt":       prompt,
				"aspect_ratio": apiRatio,
				"upscale":      upscale,
			}
		case "gemini":
			url = fmt.Sprintf("%s/v4/gemini/video/generate?api_key=%s", s.baseUrl, apiKey)
			reqBody = map[string]interface{}{
				"prompt": prompt,
			}
		default:
			return fmt.Errorf("unknown video model: %s", model)
		}
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return err
	}

	if s.OnLogData != nil {
		s.OnLogData("Googler Video Request", fmt.Sprintf("MODEL: %s\nPROMPT: %s\nRATIO: %s", model, prompt, apiRatio))
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
		return fmt.Errorf("Googler Video API failed (%d): %s", resp.StatusCode, string(body))
	}

	var opResp OperationResponse
	if err := json.NewDecoder(resp.Body).Decode(&opResp); err != nil {
		return err
	}

	if opResp.OperationID == "" {
		return fmt.Errorf("no operation_id returned")
	}

	// Polling
	maxRetries := 120 // 10 minutes max for video
	for i := 0; i < maxRetries; i++ {
		time.Sleep(5 * time.Second)

		statusUrl := fmt.Sprintf("%s/v4/operations/%s?api_key=%s", s.baseUrl, opResp.OperationID, apiKey)
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
			var base64Data string
			switch v := stResp.Result.(type) {
			case string:
				base64Data = v
			case []interface{}:
				if len(v) > 0 {
					if st, ok := v[0].(string); ok {
						base64Data = st
					}
				}
			}

			if base64Data == "" {
				return fmt.Errorf("empty result in success status")
			}

			return utils.SaveBase64Image(base64Data, outputPath)
		}

		if stResp.Status == "error" {
			return fmt.Errorf("Googler video task failed: %s", stResp.Error)
		}
	}

	return fmt.Errorf("Googler video timeout after 10 minutes")
}
