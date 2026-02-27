package pipeline

import (
	"os"
	"path/filepath"
	"soloveyko/backend/utils"
)

// EnsureDirectory creates the task directory structure
func (s *PipelineService) EnsureDirectory(outPath string, taskName string, templateDir string) (string, error) {
	safeTask := utils.SanitizeFilename(taskName)
	safeTemplate := utils.SanitizeFilename(templateDir)
	finalDir := filepath.Join(outPath, safeTask, safeTemplate)
	err := os.MkdirAll(finalDir, 0755)
	return finalDir, err
}

// SaveTextResult saves the processed text to a file
func (s *PipelineService) SaveTextResult(finalDir string, taskType string, content string) error {
	fileName := "result.txt"
	switch taskType {
	case "translate":
		fileName = "translation.txt"
	case "rewrite":
		fileName = "rewrite.txt"
	}
	filePath := filepath.Join(finalDir, fileName)
	return os.WriteFile(filePath, []byte(content), 0644)
}

// LoadTextResult loads the processed text from a file
func (s *PipelineService) LoadTextResult(finalDir string, taskType string) (string, error) {
	fileName := "result.txt"
	switch taskType {
	case "translate":
		fileName = "translation.txt"
	case "rewrite":
		fileName = "rewrite.txt"
	}
	filePath := filepath.Join(finalDir, fileName)
	data, err := os.ReadFile(filePath)
	if err != nil {
		return "", err
	}
	return string(data), nil
}
