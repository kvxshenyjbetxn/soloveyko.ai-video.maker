package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type AuthRequest struct {
	Key        string `json:"key"`
	HardwareID string `json:"hardware_id,omitempty"`
}

type AuthResponse struct {
	Valid             bool   `json:"valid"`
	ExpiresAt         string `json:"expires_at"`
	HardwareBound     bool   `json:"hardware_bound"`
	SubscriptionLevel int    `json:"subscription_level"`
	IsUnlimited       bool   `json:"is_unlimited"`
	TelegramID        int64  `json:"telegram_id"`
}

type AuthService struct {
	BaseURL    string
	HTTPClient *http.Client
}

func NewAuthService() *AuthService {
	return &AuthService{
		BaseURL: "https://new-project-combain-server-production.up.railway.app",
		HTTPClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (s *AuthService) ValidateKey(key string, hardwareID string) (*AuthResponse, error) {
	reqBody := AuthRequest{
		Key:        key,
		HardwareID: hardwareID,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	url := fmt.Sprintf("%s/validate_key/", s.BaseURL)
	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := s.HTTPClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var errResp map[string]interface{}
		json.NewDecoder(resp.Body).Decode(&errResp)
		if detail, ok := errResp["detail"].(string); ok {
			return nil, fmt.Errorf("%s", detail)
		}
		return nil, fmt.Errorf("server returned status code: %d", resp.StatusCode)
	}

	var authResp AuthResponse
	if err := json.NewDecoder(resp.Body).Decode(&authResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return &authResp, nil
}
