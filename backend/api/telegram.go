package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type TelegramService struct {
	// You can hardcode your bot token here or provide it via environment variables
	BotToken   string
	HTTPClient *http.Client
}

func NewTelegramService() *TelegramService {
	// Telegram Bot Token for @soloveyko_ai_notification_bot
	token := "8217593955:AAGN4TSpuQcwGXclUDwniKeWaBDoUfEuvN4"
	return &TelegramService{
		BotToken: token,
		HTTPClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

// SendNotification sends a markdown formatted message to the specified chat ID
func (s *TelegramService) SendNotification(chatID string, text string) error {
	if s.BotToken == "" || s.BotToken == "YOUR_TELEGRAM_BOT_TOKEN" {
		return fmt.Errorf("Telegram bot token is not configured in the source code")
	}

	url := fmt.Sprintf("https://api.telegram.org/bot%s/sendMessage", s.BotToken)

	payload := map[string]interface{}{
		"chat_id":    chatID,
		"text":       text,
		"parse_mode": "Markdown",
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := s.HTTPClient.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send message: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var errorResponse map[string]interface{}
		json.NewDecoder(resp.Body).Decode(&errorResponse)
		return fmt.Errorf("telegram API error: %v", errorResponse["description"])
	}

	return nil
}
