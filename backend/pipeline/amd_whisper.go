package pipeline

import (
	"archive/zip"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"soloveyko/backend/bin"
	"soloveyko/backend/utils"

	wruntime "github.com/wailsapp/wails/v2/pkg/runtime"
)

type AmdWhisperService struct {
	ctx context.Context
}

func NewAmdWhisperService() *AmdWhisperService {
	return &AmdWhisperService{}
}

func (s *AmdWhisperService) SetContext(ctx context.Context) {
	s.ctx = ctx
}

// GetBinPath returns the path to the amd-whisper bin folder in user config
func (s *AmdWhisperService) GetBinPath() string {
	configDir, _ := os.UserConfigDir()
	return filepath.Join(configDir, "Soloveyko", "bin", "whisper-amd")
}

// IsInstalled checks if the binary is already extracted
func (s *AmdWhisperService) IsInstalled() bool {
	exePath := filepath.Join(s.GetBinPath(), "main.exe")
	_, err := os.Stat(exePath)
	return err == nil
}

// Install extracts the whisper-amd.zip
func (s *AmdWhisperService) Install() error {
	if runtime.GOOS != "windows" {
		return fmt.Errorf("AMD Whisper is only available on Windows")
	}

	configDir, _ := os.UserConfigDir()
	binFolder := filepath.Join(configDir, "Soloveyko", "bin")
	os.MkdirAll(binFolder, os.ModePerm)

	zipData, err := bin.Files.ReadFile("whisper-amd.zip")
	if err != nil {
		return fmt.Errorf("failed to read whisper-amd.zip: %w", err)
	}

	zipPath := filepath.Join(binFolder, "whisper-amd-temp.zip")
	if err := os.WriteFile(zipPath, zipData, 0644); err != nil {
		return fmt.Errorf("failed to save zip: %w", err)
	}
	defer os.Remove(zipPath)

	// Extract
	err = s.unzip(zipPath, binFolder)
	if err != nil {
		return fmt.Errorf("failed to unzip: %w", err)
	}

	return nil
}

func (s *AmdWhisperService) unzip(src, dest string) error {
	r, err := zip.OpenReader(src)
	if err != nil {
		return err
	}
	defer r.Close()

	for _, f := range r.File {
		fpath := filepath.Join(dest, f.Name)
		if !strings.HasPrefix(fpath, filepath.Clean(dest)+string(os.PathSeparator)) {
			continue
		}
		if f.FileInfo().IsDir() {
			os.MkdirAll(fpath, os.ModePerm)
			continue
		}
		if err := os.MkdirAll(filepath.Dir(fpath), os.ModePerm); err != nil {
			return err
		}
		outFile, err := os.OpenFile(fpath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.Mode())
		if err != nil {
			return err
		}
		rc, err := f.Open()
		if err != nil {
			outFile.Close()
			return err
		}
		_, err = io.Copy(outFile, rc)
		outFile.Close()
		rc.Close()
		if err != nil {
			return err
		}
		os.Chmod(fpath, f.Mode())
	}
	return nil
}

func (s *AmdWhisperService) GetAvailableModels() ([]string, error) {
	modelsDir := filepath.Join(s.GetBinPath(), "models")
	if _, err := os.Stat(modelsDir); os.IsNotExist(err) {
		return []string{}, nil
	}

	files, err := os.ReadDir(modelsDir)
	if err != nil {
		return nil, err
	}

	var models []string
	for _, f := range files {
		if !f.IsDir() && strings.HasSuffix(f.Name(), ".bin") {
			models = append(models, f.Name())
		}
	}
	return models, nil
}

func (s *AmdWhisperService) Transcribe(audioFilePath string, modelName string, language string) (string, error) {
	if runtime.GOOS != "windows" {
		return "", fmt.Errorf("AMD Whisper is only available on Windows")
	}

	amdBinPath := s.GetBinPath()
	whisperExe := filepath.Join(amdBinPath, "main.exe")
	modelPath := filepath.Join(amdBinPath, "models", modelName)

	if _, err := os.Stat(modelPath); os.IsNotExist(err) {
		// Спробуємо знайти в стандартній папці моделей
		configDir, _ := os.UserConfigDir()
		standardModelPath := filepath.Join(configDir, "Soloveyko", "models", "ggml-"+modelName+".bin")
		if _, err := os.Stat(standardModelPath); err == nil {
			modelPath = standardModelPath
		} else {
			// Можливо передано повну назву ggml-base.bin
			standardModelPath = filepath.Join(configDir, "Soloveyko", "models", modelName)
			if _, err := os.Stat(standardModelPath); err == nil {
				modelPath = standardModelPath
			}
		}
	}

	if _, err := os.Stat(whisperExe); os.IsNotExist(err) {
		return "", fmt.Errorf("whisper-amd main.exe not found")
	}

	// For AMD Whisper, we might need ffmpeg as well to convert to 16kHz WAV
	// We can reuse EnsureFFmpeg from LocalWhisper or similar
	// But let's assume ffmpeg is already in Soloveyko/bin (it's ensured on startup)
	configDir, _ := os.UserConfigDir()
	ffmpegExe := filepath.Join(configDir, "Soloveyko", "bin", "ffmpeg.exe")

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Конвертація аудіо (ffmpeg)...",
			"percent": 100.0,
		})
	}

	wavTempFile := filepath.Join(os.TempDir(), "temp_whisper_amd_input.wav")
	defer os.Remove(wavTempFile)
	os.Remove(wavTempFile)

	ffmpegCmd := exec.CommandContext(s.ctx, ffmpegExe, "-y", "-i", audioFilePath, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", wavTempFile)
	utils.PrepareHiddenCmd(ffmpegCmd)
	if err := ffmpegCmd.Run(); err != nil {
		return "", fmt.Errorf("помилка конвертації аудіо для AMD Whisper: %w", err)
	}

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Транскрибація виконується (AMD)...",
			"percent": 100.0,
		})
	}

	outputSrtBase := filepath.Join(os.TempDir(), "temp_whisper_amd_output")
	defer os.Remove(outputSrtBase + ".srt")
	os.Remove(outputSrtBase + ".srt")

	if language == "" {
		language = "uk" // Default
	}

	// AMD Whisper CLI doesn't support -of in some versions.
	// By default it saves to [input_file].srt if -osrt is present.
	// Removing all extra parameters to test stability.
	args := []string{
		"-m", modelPath,
		"-l", language,
		"-f", wavTempFile,
		"-osrt",
	}

	whisperCmd := exec.CommandContext(s.ctx, whisperExe, args...)
	utils.PrepareHiddenCmd(whisperCmd)
	cmdOut, err := whisperCmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("помилка whisper-amd (%v): %s", err, string(cmdOut))
	}

	// Output file is [wavTempFile].srt OR [wavTempFile-without-extension].srt
	expectedSrt1 := wavTempFile + ".srt"
	expectedSrt2 := strings.TrimSuffix(wavTempFile, filepath.Ext(wavTempFile)) + ".srt"

	var srtBytes []byte
	var foundPath string

	if b, err := os.ReadFile(expectedSrt1); err == nil {
		srtBytes = b
		foundPath = expectedSrt1
	} else if b, err := os.ReadFile(expectedSrt2); err == nil {
		srtBytes = b
		foundPath = expectedSrt2
	}

	if srtBytes == nil {
		return "", fmt.Errorf("не вдалося знайти файл субтитрів AMD Whisper (перевірено %s та %s)", expectedSrt1, expectedSrt2)
	}

	if foundPath != "" {
		defer os.Remove(foundPath)
	}

	return string(srtBytes), nil
}
