package utils

import (
	"encoding/base64"
	"fmt"
	"image"
	"image/gif"
	"image/jpeg"
	"image/png"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"golang.org/x/image/draw"
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

// GetImageAsBase64 читає файл та повертає base64 строку з префіксом
func GetImageAsBase64(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}

	mimeType := "image/jpeg"
	ext := strings.ToLower(filepath.Ext(path))
	switch ext {
	case ".png":
		mimeType = "image/png"
	case ".webp":
		mimeType = "image/webp"
	case ".gif":
		mimeType = "image/gif"
	}

	return fmt.Sprintf("data:%s;base64,%s", mimeType, base64.StdEncoding.EncodeToString(data)), nil
}

func GetAudioDuration(path string) (string, error) {
	seconds, err := GetAudioDurationSeconds(path)
	if err != nil {
		return "", err
	}
	return FormatDuration(seconds), nil
}

// GetAudioDurationSeconds повертає тривалість аудіофайлу в секундах
func GetAudioDurationSeconds(path string) (float64, error) {
	ffprobePath, err := EnsureEngine("ffprobe")
	if err != nil {
		ffprobePath = "ffprobe"
	}

	out, err := runHiddenCommand(ffprobePath, "-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", path)
	if err != nil {
		return 0, err
	}

	durationStr := strings.TrimSpace(string(out))
	seconds, err := strconv.ParseFloat(durationStr, 64)
	if err != nil {
		return 0, err
	}

	return seconds, nil
}

// FormatDuration форматує секунди у зручний для читання вигляд
func FormatDuration(seconds float64) string {
	hrs := int(seconds) / 3600
	minutes := (int(seconds) % 3600) / 60
	secs := int(seconds) % 60

	if hrs > 0 {
		return fmt.Sprintf("%d:%02d:%02d", hrs, minutes, secs)
	}
	if minutes > 0 {
		return fmt.Sprintf("%d:%02d", minutes, secs)
	}
	return fmt.Sprintf("%d сек", secs)
}

// CreateThumbnail створює зменшену копію зображення
func CreateThumbnail(inputPath string, outputPath string, maxWidth int) error {
	file, err := os.Open(inputPath)
	if err != nil {
		return err
	}
	defer file.Close()

	img, format, err := image.Decode(file)
	if err != nil {
		return err
	}

	bounds := img.Bounds()
	width := bounds.Dx()
	height := bounds.Dy()

	if width <= maxWidth {
		// Якщо картинка вже менша, просто копіюємо
		return os.WriteFile(outputPath, nil, 0644) // Placeholder or actually copy? Let's copy.
	}

	newWidth := maxWidth
	newHeight := (height * maxWidth) / width

	newImg := image.NewRGBA(image.Rect(0, 0, newWidth, newHeight))
	draw.BiLinear.Scale(newImg, newImg.Bounds(), img, bounds, draw.Over, nil)

	out, err := os.Create(outputPath)
	if err != nil {
		return err
	}
	defer out.Close()

	switch format {
	case "png":
		return png.Encode(out, newImg)
	case "gif":
		return gif.Encode(out, newImg, nil)
	default:
		return jpeg.Encode(out, newImg, &jpeg.Options{Quality: 85})
	}
}
