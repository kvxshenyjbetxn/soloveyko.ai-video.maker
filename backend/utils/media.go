package utils

import (
	"fmt"
	"strconv"
	"strings"
)

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
