package utils

import (
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

type FileLogger struct {
	logFile *os.File
	mu      sync.Mutex
}

func NewFileLogger() *FileLogger {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	appDir := filepath.Join(configDir, "Soloveyko")
	logsDir := filepath.Join(appDir, "logs")
	os.MkdirAll(logsDir, 0755)

	timestamp := time.Now().Format("2006-01-02_15-04-05")
	logPath := filepath.Join(logsDir, fmt.Sprintf("session_%s.log", timestamp))

	fl := &FileLogger{}
	file, err := os.OpenFile(logPath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0666)
	if err == nil {
		fl.logFile = file
		fl.Log("INFO", "=== NEW SESSION STARTED ===")

		// Clean up and log the result
		cleaned := fl.cleanupOldLogs(logsDir)
		if cleaned > 0 {
			fl.Log("INFO", fmt.Sprintf("Cleaned up %d log files older than 7 days", cleaned))
		}
	}

	return fl
}

func (l *FileLogger) Log(level string, message string, details ...string) {
	l.mu.Lock()
	defer l.mu.Unlock()

	if l.logFile == nil {
		return
	}

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	tLabel := ""
	if len(details) > 1 {
		tLabel = details[1]
	}

	logLine := ""
	if tLabel != "" {
		logLine = fmt.Sprintf("[%s] [%s] (%s) %s\n", timestamp, level, tLabel, message)
	} else {
		logLine = fmt.Sprintf("[%s] [%s] %s\n", timestamp, level, message)
	}

	l.logFile.WriteString(logLine)
}

func (l *FileLogger) LogData(category string, data string) {
	l.mu.Lock()
	defer l.mu.Unlock()

	if l.logFile == nil {
		return
	}

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	divider := "================================================================================\n"
	header := fmt.Sprintf("[%s] [DETAILED DATA] [%s]\n", timestamp, category)

	l.logFile.WriteString("\n" + divider)
	l.logFile.WriteString(header)
	l.logFile.WriteString(divider)
	l.logFile.WriteString(data + "\n")
	l.logFile.WriteString(divider + "\n")
}

func (l *FileLogger) cleanupOldLogs(logsDir string) int {
	files, err := os.ReadDir(logsDir)
	if err != nil {
		return 0
	}

	now := time.Now()
	sevenDays := 7 * 24 * time.Hour
	cleanedCount := 0

	for _, file := range files {
		if file.IsDir() {
			continue
		}
		info, err := file.Info()
		if err != nil {
			continue
		}

		// Don't delete the current log file
		if info.Name() == filepath.Base(l.logFile.Name()) {
			continue
		}

		if now.Sub(info.ModTime()) > sevenDays {
			err := os.Remove(filepath.Join(logsDir, file.Name()))
			if err == nil {
				cleanedCount++
			}
		}
	}

	return cleanedCount
}

func (l *FileLogger) Close() {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.logFile != nil {
		l.logFile.Close()
		l.logFile = nil
	}
}
