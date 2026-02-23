package utils

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"
)

type FullHistoryEntry struct {
	ID            string   `json:"id"`
	TaskName      string   `json:"taskName"`
	Type          string   `json:"type"`
	Templates     []string `json:"templates"`
	Stages        []string `json:"stages"`
	OriginalText  string   `json:"originalText"`
	ProcessedText string   `json:"processedText"`
	Timestamp     int64    `json:"timestamp"` // Unix timestamp
	FormattedDate string   `json:"formattedDate"`
}

type FullHistoryService struct {
	baseDir string
	mu      sync.RWMutex
}

func NewFullHistoryService() *FullHistoryService {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	appDataDir := filepath.Join(configDir, "Soloveyko", "data", "full_history")
	_ = os.MkdirAll(appDataDir, 0755)

	return &FullHistoryService{
		baseDir: appDataDir,
	}
}

// AddEntry saves a full history entry as a separate JSON file
func (s *FullHistoryService) AddEntry(name string, taskType string, templates []string, stages []string, original string, processed string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	now := time.Now()
	id := fmt.Sprintf("%d_%d", now.Unix(), now.Nanosecond()/1e6)

	entry := FullHistoryEntry{
		ID:            id,
		TaskName:      name,
		Type:          taskType,
		Templates:     templates,
		Stages:        stages,
		OriginalText:  original,
		ProcessedText: processed,
		Timestamp:     now.Unix(),
		FormattedDate: now.Format(time.RFC3339),
	}

	fileName := fmt.Sprintf("%s.json", id)
	filePath := filepath.Join(s.baseDir, fileName)

	data, err := json.MarshalIndent(entry, "", "  ")
	if err != nil {
		return err
	}

	err = os.WriteFile(filePath, data, 0644)
	if err != nil {
		return err
	}

	// Always cleanup after adding
	go s.Cleanup()

	return nil
}

// GetEntries returns all history entries (metadata only for list, without full texts to keep memory low)
type HistoryMetadata struct {
	ID            string   `json:"id"`
	TaskName      string   `json:"taskName"`
	Type          string   `json:"type"`
	Templates     []string `json:"templates"`
	Stages        []string `json:"stages"`
	Timestamp     int64    `json:"timestamp"`
	FormattedDate string   `json:"formattedDate"`
}

func (s *FullHistoryService) GetEntries() ([]HistoryMetadata, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	files, err := os.ReadDir(s.baseDir)
	if err != nil {
		return nil, err
	}

	var entries []HistoryMetadata
	for _, f := range files {
		if f.IsDir() || filepath.Ext(f.Name()) != ".json" {
			continue
		}

		path := filepath.Join(s.baseDir, f.Name())
		data, err := os.ReadFile(path)
		if err != nil {
			continue
		}

		var full FullHistoryEntry
		if err := json.Unmarshal(data, &full); err != nil {
			continue
		}

		entries = append(entries, HistoryMetadata{
			ID:            full.ID,
			TaskName:      full.TaskName,
			Type:          full.Type,
			Templates:     full.Templates,
			Stages:        full.Stages,
			Timestamp:     full.Timestamp,
			FormattedDate: full.FormattedDate,
		})
	}

	// Sort: newest first
	sort.Slice(entries, func(i, j int) bool {
		return entries[i].Timestamp > entries[j].Timestamp
	})

	return entries, nil
}

// GetEntry returns a single full entry by ID
func (s *FullHistoryService) GetEntry(id string) (*FullHistoryEntry, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	path := filepath.Join(s.baseDir, id+".json")
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var entry FullHistoryEntry
	if err := json.Unmarshal(data, &entry); err != nil {
		return nil, err
	}

	return &entry, nil
}

// DeleteEntry deletes a single entry file
func (s *FullHistoryService) DeleteEntry(id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	path := filepath.Join(s.baseDir, id+".json")
	return os.Remove(path)
}

// Cleanup removes entries older than 30 days
func (s *FullHistoryService) Cleanup() {
	s.mu.Lock()
	defer s.mu.Unlock()

	files, err := os.ReadDir(s.baseDir)
	if err != nil {
		return
	}

	thirtyDaysAgo := time.Now().AddDate(0, 0, -30).Unix()

	for _, f := range files {
		if f.IsDir() || filepath.Ext(f.Name()) != ".json" {
			continue
		}

		path := filepath.Join(s.baseDir, f.Name())
		data, err := os.ReadFile(path)
		if err != nil {
			continue
		}

		var entry struct {
			Timestamp int64 `json:"timestamp"`
		}
		if err := json.Unmarshal(data, &entry); err != nil {
			continue
		}

		if entry.Timestamp < thirtyDaysAgo {
			_ = os.Remove(path)
		}
	}
}
