package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"soloveyko/backend/utils"
	"time"
)

type AssemblyAIService struct {
	settings *utils.SettingsService
	baseUrl  string

	OnLog func(level string, message string, details ...string)
}

func NewAssemblyAIService(settings *utils.SettingsService) *AssemblyAIService {
	return &AssemblyAIService{
		settings: settings,
		baseUrl:  "https://api.assemblyai.com/v2",
	}
}

// CheckConnection перевіряє валідність API ключа
func (s *AssemblyAIService) CheckConnection(apiKey string) error {
	if apiKey == "" {
		return fmt.Errorf("API key is empty")
	}

	client := &http.Client{Timeout: 10 * time.Second}
	req, err := http.NewRequest("GET", s.baseUrl+"/transcript", nil)
	if err != nil {
		return err
	}

	req.Header.Set("Authorization", apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusOK {
		return nil
	}

	if resp.StatusCode == http.StatusUnauthorized {
		return fmt.Errorf("invalid API key")
	}

	return fmt.Errorf("API error: %s", resp.Status)
}

// SaveAPIKey зберігає API ключ
func (s *AssemblyAIService) SaveAPIKey(apiKey string) error {
	return s.settings.SetAssemblyAIAPIKey(apiKey)
}

// GetAPIKey повертає збережений API ключ
func (s *AssemblyAIService) GetAPIKey() string {
	return s.settings.GetAssemblyAIAPIKey()
}

var assemblyaiSemaphore = make(chan struct{}, 5)

func (s *AssemblyAIService) SetContext(ctx context.Context) {
	// Not needed directly in api, but useful if we pass context down.
	// We'll accept ctx in Transcribe method per convention, or use a field.
}

func (s *AssemblyAIService) TranscribeFull(ctx context.Context, audioFilePath string) (string, string, error) {
	apiKey := s.GetAPIKey()
	if apiKey == "" {
		return "", "", fmt.Errorf("AssemblyAI API Key is not configured")
	}

	assemblyaiSemaphore <- struct{}{}
	defer func() { <-assemblyaiSemaphore }()

	// 1. Upload the audio file
	audioData, err := os.ReadFile(audioFilePath)
	if err != nil {
		return "", "", fmt.Errorf("помилка читання аудіо: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", s.baseUrl+"/upload", bytes.NewReader(audioData))
	if err != nil {
		return "", "", fmt.Errorf("помилка створення запиту на завантаження: %w", err)
	}
	req.Header.Set("Authorization", apiKey)

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return "", "", fmt.Errorf("помилка виконання запиту на завантаження: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return "", "", fmt.Errorf("помилка завантаження (статус %d): %s", resp.StatusCode, string(body))
	}

	var uploadResp struct {
		UploadURL string `json:"upload_url"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&uploadResp); err != nil {
		return "", "", fmt.Errorf("помилка декодування відповіді завантаження: %w", err)
	}

	// 2. Submit transcription request
	transcriptReqBody := map[string]interface{}{
		"audio_url":          uploadResp.UploadURL,
		"language_detection": true,
	}
	reqBodyBytes, _ := json.Marshal(transcriptReqBody)

	req, err = http.NewRequestWithContext(ctx, "POST", s.baseUrl+"/transcript", bytes.NewBuffer(reqBodyBytes))
	if err != nil {
		return "", "", fmt.Errorf("помилка створення запиту транскрибації: %w", err)
	}
	req.Header.Set("Authorization", apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err = client.Do(req)
	if err != nil {
		return "", "", fmt.Errorf("помилка виконання запиту транскрибації: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return "", "", fmt.Errorf("помилка початку транскрибації (статус %d): %s", resp.StatusCode, string(body))
	}

	var transcriptResp struct {
		ID string `json:"id"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&transcriptResp); err != nil {
		return "", "", fmt.Errorf("помилка декодування відповіді транскрибації: %w", err)
	}

	// 3. Poll for completion
	pollingURL := fmt.Sprintf("%s/transcript/%s", s.baseUrl, transcriptResp.ID)
	var fullBody []byte
	for {
		select {
		case <-ctx.Done():
			return "", "", fmt.Errorf("транскрибацію скасовано")
		default:
		}

		req, err = http.NewRequestWithContext(ctx, "GET", pollingURL, nil)
		if err != nil {
			return "", "", fmt.Errorf("помилка створення запиту перевірки статусу: %w", err)
		}
		req.Header.Set("Authorization", apiKey)

		resp, err = client.Do(req)
		if err != nil {
			return "", "", fmt.Errorf("помилка запиту статусу: %w", err)
		}

		bodyBytes, err := io.ReadAll(resp.Body)
		resp.Body.Close()
		if err != nil {
			return "", "", fmt.Errorf("помилка читання статусу: %w", err)
		}

		if resp.StatusCode != http.StatusOK {
			return "", "", fmt.Errorf("помилка перевірки статусу (%d): %s", resp.StatusCode, string(bodyBytes))
		}

		var pollResp map[string]interface{}
		if err := json.Unmarshal(bodyBytes, &pollResp); err != nil {
			return "", "", fmt.Errorf("помилка декодування статусу: %w", err)
		}

		status, _ := pollResp["status"].(string)
		if status == "completed" {
			fullBody = bodyBytes
			break
		} else if status == "error" {
			errMsg, _ := pollResp["error"].(string)
			return "", "", fmt.Errorf("помилка AssemblyAI під час транскрибації: %s", errMsg)
		}

		// Wait before polling again
		time.Sleep(3 * time.Second)
	}

	// 4. Download SRT
	srtURL := fmt.Sprintf("%s/transcript/%s/srt", s.baseUrl, transcriptResp.ID)
	req, err = http.NewRequestWithContext(ctx, "GET", srtURL, nil)
	if err != nil {
		return "", "", fmt.Errorf("помилка створення запиту SRT: %w", err)
	}
	req.Header.Set("Authorization", apiKey)

	resp, err = client.Do(req)
	if err != nil {
		return "", "", fmt.Errorf("помилка завантаження SRT: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return "", "", fmt.Errorf("помилка отримання SRT (статус %d): %s", resp.StatusCode, string(body))
	}

	srtBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", "", fmt.Errorf("помилка читання SRT: %w", err)
	}

	return string(srtBytes), string(fullBody), nil
}

func (s *AssemblyAIService) Transcribe(ctx context.Context, audioFilePath string) (string, error) {
	srt, _, err := s.TranscribeFull(ctx, audioFilePath)
	return srt, err
}
