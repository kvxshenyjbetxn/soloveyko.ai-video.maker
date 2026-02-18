package utils

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

type PipelineTemplate struct {
	ID        string                 `json:"id"`
	Type      string                 `json:"type"` // "translate" or "rewrite"
	Name      string                 `json:"name"`
	CreatedAt int64                  `json:"createdAt"`
	Settings  map[string]interface{} `json:"settings"`
}

type TemplateService struct {
	templatesDir string
	mu           sync.RWMutex
}

func NewTemplateService() *TemplateService {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	appConfigDir := filepath.Join(configDir, "Soloveyko")
	appTemplatesDir := filepath.Join(appConfigDir, "templates")
	os.MkdirAll(appTemplatesDir, 0755)

	s := &TemplateService{
		templatesDir: appTemplatesDir,
	}

	// Migration from old templates.json
	oldPath := filepath.Join(appConfigDir, "templates.json")
	if _, err := os.Stat(oldPath); err == nil {
		data, err := os.ReadFile(oldPath)
		if err == nil {
			var oldTemplates []PipelineTemplate
			if err := json.Unmarshal(data, &oldTemplates); err == nil {
				for _, t := range oldTemplates {
					if t.Type == "" {
						t.Type = "translate" // Default for old ones
					}
					s.saveSingleTemplate(t)
				}
				os.Rename(oldPath, oldPath+".bak")
			}
		}
	}

	return s
}

func (s *TemplateService) LoadTemplates() ([]PipelineTemplate, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	files, err := os.ReadDir(s.templatesDir)
	if err != nil {
		return nil, err
	}

	templates := []PipelineTemplate{}
	for _, file := range files {
		if file.IsDir() || filepath.Ext(file.Name()) != ".json" {
			continue
		}

		data, err := os.ReadFile(filepath.Join(s.templatesDir, file.Name()))
		if err != nil {
			continue
		}

		var tpl PipelineTemplate
		if err := json.Unmarshal(data, &tpl); err == nil {
			templates = append(templates, tpl)
		}
	}

	return templates, nil
}

func (s *TemplateService) AddTemplate(tplType string, name string, data map[string]interface{}) (*PipelineTemplate, error) {
	if name == "" {
		templates, _ := s.LoadTemplates()
		count := 0
		for _, t := range templates {
			if t.Type == tplType {
				count++
			}
		}
		name = fmt.Sprintf("Pipeline %d", count+1)
	}

	id := fmt.Sprintf("%d", time.Now().UnixNano())
	newTemplate := PipelineTemplate{
		ID:        id,
		Type:      tplType,
		Name:      name,
		CreatedAt: time.Now().Unix(),
		Settings:  data,
	}

	err := s.saveSingleTemplate(newTemplate)
	if err != nil {
		return nil, err
	}

	return &newTemplate, nil
}

func (s *TemplateService) saveSingleTemplate(tpl PipelineTemplate) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	data, err := json.MarshalIndent(tpl, "", "  ")
	if err != nil {
		return err
	}

	// Clean up old files for this ID if name changed
	files, _ := os.ReadDir(s.templatesDir)
	for _, f := range files {
		if !f.IsDir() && filepath.Ext(f.Name()) == ".json" {
			path := filepath.Join(s.templatesDir, f.Name())
			content, err := os.ReadFile(path)
			if err == nil {
				var oldTpl PipelineTemplate
				if json.Unmarshal(content, &oldTpl) == nil && oldTpl.ID == tpl.ID {
					if f.Name() != s.getFileName(tpl) {
						os.Remove(path)
					}
				}
			}
		}
	}

	return os.WriteFile(filepath.Join(s.templatesDir, s.getFileName(tpl)), data, 0644)
}

func (s *TemplateService) getFileName(tpl PipelineTemplate) string {
	cleanName := ""
	for _, r := range tpl.Name {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '-' || r == '_' || r >= 0x0400 { // Allow cyrillic
			cleanName += string(r)
		} else {
			cleanName += "_"
		}
	}
	if cleanName == "" {
		cleanName = tpl.ID
	}
	return fmt.Sprintf("%s_%s.json", tpl.Type, cleanName)
}

func (s *TemplateService) DeleteTemplate(id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	files, err := os.ReadDir(s.templatesDir)
	if err != nil {
		return err
	}

	deleted := false
	for _, file := range files {
		if file.IsDir() || filepath.Ext(file.Name()) != ".json" {
			continue
		}

		fullPath := filepath.Join(s.templatesDir, file.Name())
		data, err := os.ReadFile(fullPath)
		if err != nil {
			continue
		}

		var tpl PipelineTemplate
		if err := json.Unmarshal(data, &tpl); err == nil {
			if tpl.ID == id {
				os.Remove(fullPath)
				deleted = true
			}
		}
	}

	if !deleted {
		return fmt.Errorf("template with ID %s not found", id)
	}

	return nil
}

func (s *TemplateService) UpdateTemplate(id string, name string, data map[string]interface{}) error {
	templates, err := s.LoadTemplates()
	if err != nil {
		return err
	}

	for _, t := range templates {
		if t.ID == id {
			t.Name = name
			t.Settings = data
			return s.saveSingleTemplate(t)
		}
	}

	return fmt.Errorf("template not found")
}
