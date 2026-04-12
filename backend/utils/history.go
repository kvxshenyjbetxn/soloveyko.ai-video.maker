package utils

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"
)

type HistoryEntry struct {
	ID               string                 `json:"id"`
	TaskName         string                 `json:"taskName"`
	Type             string                 `json:"type"`
	Templates        []string               `json:"templates"`
	Content          string                 `json:"content"`
	SubName          string                 `json:"subName,omitempty"`
	SettingsSnapshot map[string]interface{} `json:"settingsSnapshot,omitempty"`
	Timestamp        string                 `json:"timestamp"` // RFC3339 string
}

type HistoryService struct {
	historyPath string
	mu          sync.RWMutex
}

func NewHistoryService() *HistoryService {
	// Отримуємо директорію конфігурації для користувача
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	// Створюємо шлях до папки data програми
	appDataDir := filepath.Join(configDir, "Soloveyko", "data")

	// Створюємо директорію, якщо не існує
	_ = os.MkdirAll(appDataDir, 0755)

	return &HistoryService{
		historyPath: filepath.Join(appDataDir, "history.json"),
	}
}

// LoadHistory завантажує історію з файлу
func (s *HistoryService) LoadHistory() ([]HistoryEntry, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if _, err := os.Stat(s.historyPath); os.IsNotExist(err) {
		return []HistoryEntry{}, nil
	}

	data, err := os.ReadFile(s.historyPath)
	if err != nil {
		return nil, err
	}

	var history []HistoryEntry
	err = json.Unmarshal(data, &history)
	if err != nil {
		return []HistoryEntry{}, nil
	}

	return history, nil
}

// SaveHistory зберігає історію у файл
func (s *HistoryService) SaveHistory(history []HistoryEntry) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	data, err := json.MarshalIndent(history, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(s.historyPath, data, 0644)
}

// AddEntry додає новий запис до історії та виконує очистку застарілих записів
func (s *HistoryService) AddEntry(name string, taskType string, templates []string, content string) error {
	return s.AddEntryDetailed(name, taskType, templates, content, "", nil)
}

// AddEntryDetailed adds a history entry with optional sub-name and exact settings snapshot.
func (s *HistoryService) AddEntryDetailed(name string, taskType string, templates []string, content string, subName string, settingsSnapshot map[string]interface{}) error {
	history, err := s.LoadHistory()
	if err != nil {
		// Якщо не вдалося завантажити, починаємо з порожньої історії
		history = []HistoryEntry{}
	}

	entry := HistoryEntry{
		ID:               time.Now().Format("20060102150405"),
		TaskName:         name,
		Type:             taskType,
		Templates:        templates,
		Content:          content,
		SubName:          subName,
		SettingsSnapshot: cloneHistorySettingsSnapshot(settingsSnapshot),
		Timestamp:        time.Now().Format(time.RFC3339),
	}

	history = append(history, entry)

	// Очистка застарілих (старше 2 днів)
	history = s.Cleanup(history)

	return s.SaveHistory(history)
}

func cloneHistorySettingsSnapshot(settingsSnapshot map[string]interface{}) map[string]interface{} {
	if len(settingsSnapshot) == 0 {
		return nil
	}

	data, err := json.Marshal(settingsSnapshot)
	if err != nil {
		return nil
	}

	var cloned map[string]interface{}
	if err := json.Unmarshal(data, &cloned); err != nil {
		return nil
	}

	return cloned
}

// Cleanup видаляє записи старші за 2 дні
func (s *HistoryService) Cleanup(history []HistoryEntry) []HistoryEntry {
	twoDaysAgo := time.Now().AddDate(0, 0, -2)
	var filtered []HistoryEntry
	for _, entry := range history {
		t, err := time.Parse(time.RFC3339, entry.Timestamp)
		if err == nil && t.After(twoDaysAgo) {
			filtered = append(filtered, entry)
		}
	}

	// Сортуємо: нові згори
	sort.Slice(filtered, func(i, j int) bool {
		ti, _ := time.Parse(time.RFC3339, filtered[i].Timestamp)
		tj, _ := time.Parse(time.RFC3339, filtered[j].Timestamp)
		return ti.After(tj)
	})

	return filtered
}

// GetHistory повертає відфільтровану історію
func (s *HistoryService) GetHistory() ([]HistoryEntry, error) {
	history, err := s.LoadHistory()
	if err != nil {
		return nil, err
	}

	// Про всяк випадок ще раз чистимо та сортуємо
	return s.Cleanup(history), nil
}
