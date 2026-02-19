package pipeline

import (
	"os"
	"path/filepath"
)

// EnsureDirectory creates the task directory structure
func (s *PipelineService) EnsureDirectory(outPath string, taskName string, templateDir string) (string, error) {
	finalDir := filepath.Join(outPath, taskName, templateDir)
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
