package pipeline

import (
	"archive/zip"
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"soloveyko/backend/bin"
	"soloveyko/backend/utils"

	wruntime "github.com/wailsapp/wails/v2/pkg/runtime"
)

const modelBaseURL = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

type LocalWhisperService struct {
	ctx          context.Context
	modelsFolder string
}

func NewLocalWhisperService() *LocalWhisperService {
	configDir, err := os.UserConfigDir()
	if err != nil {
		configDir = "."
	}

	appConfigDir := filepath.Join(configDir, "Soloveyko")
	modelsDir := filepath.Join(appConfigDir, "models")

	os.MkdirAll(modelsDir, os.ModePerm)

	return &LocalWhisperService{
		modelsFolder: modelsDir,
	}
}

func (s *LocalWhisperService) SetContext(ctx context.Context) {
	s.ctx = ctx
}

func (s *LocalWhisperService) CheckModel(modelName string) bool {
	fileName := fmt.Sprintf("ggml-%s.bin", modelName)
	modelPath := filepath.Join(s.modelsFolder, fileName)

	if _, err := os.Stat(modelPath); err == nil {
		return true
	}
	return false
}

func (s *LocalWhisperService) GetModelPath(modelName string) (string, error) {
	fileName := fmt.Sprintf("ggml-%s.bin", modelName)
	modelPath := filepath.Join(s.modelsFolder, fileName)

	if _, err := os.Stat(modelPath); err == nil {
		return modelPath, nil
	}

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Початок завантаження моделі " + modelName,
			"percent": 0.0,
		})
	}

	err := s.downloadModel(modelName, modelPath)
	if err != nil {
		return "", fmt.Errorf("помилка завантаження моделі: %w", err)
	}

	return modelPath, nil
}

func (s *LocalWhisperService) downloadModel(modelName, destPath string) error {
	fileName := fmt.Sprintf("ggml-%s.bin", modelName)
	downloadURL := fmt.Sprintf("%s/%s", modelBaseURL, fileName)

	resp, err := http.Get(downloadURL)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		return fmt.Errorf("huggingface повернув статус %d", resp.StatusCode)
	}

	tempFilePath := destPath + ".part"
	out, err := os.Create(tempFilePath)
	if err != nil {
		return err
	}

	totalSize := resp.ContentLength
	var downloaded int64
	buf := make([]byte, 32*1024)

	for {
		n, err := resp.Body.Read(buf)
		if n > 0 {
			out.Write(buf[0:n])
			downloaded += int64(n)

			if totalSize > 0 && s.ctx != nil {
				percent := float64(downloaded) / float64(totalSize) * 100
				wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
					"status":  fmt.Sprintf("Завантаження: %.1f%%", percent),
					"percent": percent,
				})
			}
		}
		if err == io.EOF {
			break
		}
		if err != nil {
			out.Close()
			return err
		}
	}
	out.Close()

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Завантаження завершено",
			"percent": 100.0,
		})
	}

	return os.Rename(tempFilePath, destPath)
}

func (s *LocalWhisperService) findWhisperExe(dir string) string {
	var found string
	filepath.Walk(dir, func(p string, info os.FileInfo, err error) error {
		if err == nil && !info.IsDir() {
			name := strings.ToLower(info.Name())
			// Windows: шукаємо .exe
			if runtime.GOOS == "windows" {
				if name == "whisper-cli.exe" || name == "whisper.exe" || name == "main.exe" {
					if name == "whisper-cli.exe" {
						found = p
						return filepath.SkipDir
					}
					if found == "" {
						found = p
					}
				}
			} else {
				// Mac/Linux: шукаємо назви без розширення або з назвою whisper
				if name == "whisper-cli" || name == "whisper" || name == "main" {
					if name == "whisper-cli" {
						found = p
						return filepath.SkipDir
					}
					if found == "" {
						found = p
					}
				}
			}
		}
		return nil
	})

	return found
}

func (s *LocalWhisperService) unzip(src, dest string) error {
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
		// Встановлення прав доступу після копіювання, особливо для виконуваних файлів
		os.Chmod(fpath, f.Mode())
	}
	return nil
}

func (s *LocalWhisperService) EnsureWhisperCLI() (string, error) {
	configDir, _ := os.UserConfigDir()
	binFolder := filepath.Join(configDir, "Soloveyko", "bin")
	os.MkdirAll(binFolder, os.ModePerm)

	// Якщо вже є розпаковано - беремо його
	if runtime.GOOS == "windows" {
		if exePath := s.findWhisperExe(binFolder); exePath != "" {
			return exePath, nil
		}
	} else {
		if exePath := s.findWhisperExe(binFolder); exePath != "" {
			return exePath, nil
		}
		// Перевіряємо в системі для лінухів/маків якщо нема розпакованого
		for _, name := range []string{"whisper-cli", "whisper-cpp", "main", "whisper"} {
			if _, err := exec.LookPath(name); err == nil {
				return name, nil
			}
		}
	}

	// Отже, немає. Розпаковуємо з вбудованих файлів (embedded)
	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Розпакування whisper та ffmpeg...",
			"percent": 50.0,
		})
	}

	filesToExtract := []string{}
	switch runtime.GOOS {
	case "windows":
		filesToExtract = append(filesToExtract, "whisper.zip", "ffmpeg.exe")
	default:
		filesToExtract = append(filesToExtract, "whisper", "ffmpeg")
	}

	for _, name := range filesToExtract {
		targetPath := filepath.Join(binFolder, name)
		if _, err := os.Stat(targetPath); err == nil {
			continue // вже розпаковано і існує
		}

		data, err := bin.Files.ReadFile(name)
		if err != nil {
			// Якщо файлу немає у вбудованих ресурсах (наприклад, для Mac), ігноруємо
			continue
		}

		os.WriteFile(targetPath, data, 0755)
		// Для Mac/Linux примусово встановлюємо права на виконання
		if runtime.GOOS != "windows" {
			os.Chmod(targetPath, 0755)
		}

		// Якщо це ZIP архів (для вінди) - розпакуємо
		if strings.HasSuffix(name, ".zip") {
			err := s.unzip(targetPath, binFolder)
			if err == nil {
				// Видаляємо архів після розпакування
				os.Remove(targetPath)

				// Перейменовуємо папку Release у whisper (зазвичай whisper розпаковується туди)
				releaseDir := filepath.Join(binFolder, "Release")
				whisperDir := filepath.Join(binFolder, "whisper")
				if stat, err := os.Stat(releaseDir); err == nil && stat.IsDir() {
					os.RemoveAll(whisperDir) // видаляємо стару якщо є
					os.Rename(releaseDir, whisperDir)
				}
			}
		}
	}

	// Спробуємо знайти знову
	if exePath := s.findWhisperExe(binFolder); exePath != "" {
		return exePath, nil
	}

	return "", fmt.Errorf("не вдалося знайти або розпакувати whisper.cpp")
}

func (s *LocalWhisperService) EnsureFFmpeg() (string, error) {
	configDir, _ := os.UserConfigDir()
	binFolder := filepath.Join(configDir, "Soloveyko", "bin")

	ffmpegName := "ffmpeg"
	if runtime.GOOS == "windows" {
		ffmpegName = "ffmpeg.exe"
	}

	ffmpegPath := filepath.Join(binFolder, ffmpegName)
	if _, err := os.Stat(ffmpegPath); err == nil {
		return ffmpegPath, nil
	}

	// Якщо розпакованого в Soloveyko/bin немає, перевіряємо чи є системний ffmpeg
	if _, err := exec.LookPath(ffmpegName); err == nil {
		return ffmpegName, nil
	}

	return "", fmt.Errorf("ffmpeg не знайдено ні вбудованого, ні в системі")
}

func (s *LocalWhisperService) TranscribeBase(audioFilePath string, modelName string, maxLen int, threads int) (string, error) {
	modelPath, err := s.GetModelPath(modelName)
	if err != nil {
		return "", err
	}

	whisperExe, err := s.EnsureWhisperCLI()
	if err != nil {
		return "", fmt.Errorf("помилка ініціалізації whisper: %w", err)
	}

	ffmpegExe, err := s.EnsureFFmpeg()
	if err != nil {
		return "", fmt.Errorf("ffmpeg не знайдено. Будь ласка, переконайтесь що бінарник ffmpeg є в наявності: %w", err)
	}

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Конвертація аудіо (ffmpeg)...",
			"percent": 100.0,
		})
	}

	// Use unique temp files to avoid collisions during concurrent batch processing
	tempSuffix := strings.ReplaceAll(filepath.Base(audioFilePath), ".", "_")
	if len(tempSuffix) > 20 {
		tempSuffix = tempSuffix[:20]
	}
	tempSuffix += "_" + utils.RandomString(5)

	wavTempFile := filepath.Join(os.TempDir(), fmt.Sprintf("whisper_local_%s_in.wav", tempSuffix))
	defer os.Remove(wavTempFile)
	_ = os.Remove(wavTempFile)

	ffmpegCmd := exec.CommandContext(s.ctx, ffmpegExe, "-y", "-i", audioFilePath, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", wavTempFile)
	utils.PrepareHiddenCmd(ffmpegCmd)
	if err := ffmpegCmd.Run(); err != nil {
		return "", fmt.Errorf("помилка конвертації аудіо: %w", err)
	}

	if s.ctx != nil {
		wruntime.EventsEmit(s.ctx, "download_progress", map[string]interface{}{
			"status":  "Транскрибація виконується...",
			"percent": 100.0,
		})
	}

	outputSrtBase := filepath.Join(os.TempDir(), fmt.Sprintf("whisper_local_%s_out", tempSuffix))
	defer os.Remove(outputSrtBase + ".srt")
	_ = os.Remove(outputSrtBase + ".srt")

	maxLenStr := fmt.Sprintf("%d", maxLen)
	if maxLen <= 0 {
		maxLenStr = "40"
	}

	whisperArgs := []string{"-m", modelPath, "-l", "auto", "-f", wavTempFile, "-osrt", "-of", outputSrtBase, "-ml", maxLenStr, "-sow"}
	if threads > 0 {
		whisperArgs = append([]string{"-t", fmt.Sprintf("%d", threads)}, whisperArgs...)
	}
	whisperCmd := exec.CommandContext(s.ctx, whisperExe, whisperArgs...)
	utils.PrepareHiddenCmd(whisperCmd)
	cmdOut, err := whisperCmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("помилка whisper (%v): %s", err, string(cmdOut))
	}

	srtBytes, err := os.ReadFile(outputSrtBase + ".srt")
	if err != nil {
		return "", fmt.Errorf("не вдалося прочитати файл субтитрів: %w", err)
	}

	return string(srtBytes), nil
}
