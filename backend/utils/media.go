package utils

import (
	"encoding/base64"
	"fmt"
	"os"
	"strconv"
	"strings"
)

// SaveBase64Image декодує base64 строку (з префіксом або без) та зберігає у файл
func SaveBase64Image(base64Data string, outputPath string) error {
	// Видаляємо префікс "data:image/png;base64," якщо він є
	if i := strings.Index(base64Data, ","); i != -1 {
		base64Data = base64Data[i+1:]
	}

	data, err := base64.StdEncoding.DecodeString(base64Data)
	if err != nil {
		return fmt.Errorf("failed to decode base64: %v", err)
	}

	return os.WriteFile(outputPath, data, 0644)
}

// GetAudioDuration повертає тривалість аудіофайлу у форматі "0:00" або "0 сек"
// Використовує вбудований ffprobe.
func GetAudioDuration(path string) (string, error) {
	ffprobePath, err := EnsureEngine("ffprobe")
	if err != nil {
		// Якщо вбудованого немає, спробуємо системний (як запасний варіант)
		ffprobePath = "ffprobe"
	}

	// ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 path
	out, err := runHiddenCommand(ffprobePath, "-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", path)
	if err != nil {
		return "", err
	}

	durationStr := strings.TrimSpace(string(out))
	seconds, err := strconv.ParseFloat(durationStr, 64)
	if err != nil {
		return "", err
	}

	return FormatDuration(seconds), nil
}

// FormatDuration форматує секунди у зручний для читання вигляд
func FormatDuration(seconds float64) string {
	minutes := int(seconds) / 60
	secs := int(seconds) % 60
	if minutes > 0 {
		return fmt.Sprintf("%d:%02d", minutes, secs)
	}
	return fmt.Sprintf("%d сек", secs)
}
