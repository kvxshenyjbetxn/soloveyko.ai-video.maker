package utils

import (
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"fmt"
	"soloveyko/backend/bin"
)

// EnsureEngine перевіряє наявність бінарного файлу в системній папці
// і розпаковує його з вбудованих ресурсів, якщо він відсутній.
func EnsureEngine(name string) (string, error) {
	binaryName := name
	if runtime.GOOS == "windows" {
		binaryName += ".exe"
	}

	// Отримуємо шлях до папки з бінарниками в конфигу користувача
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}
	binDir := filepath.Join(configDir, "Soloveyko", "bin")
	targetPath := filepath.Join(binDir, binaryName)

	// Якщо файл вже існує, повертаємо шлях
	if info, err := os.Stat(targetPath); err == nil {
		// Переконуємося, що права на виконання встановлені, але тільки якщо вони не 0755
		// Це трохи швидше ніж постійний Chmod на маку
		if runtime.GOOS != "windows" && info.Mode().Perm() != 0755 {
			os.Chmod(targetPath, 0755)
		}
		return targetPath, nil
	}

	// Якщо папки немає - створюємо
	if err := os.MkdirAll(binDir, 0755); err != nil {
		return "", err
	}

	// Розпаковуємо з вбудованої FS
	src, err := bin.Files.Open(binaryName)
	if err != nil {
		return "", err // Файл не поклали в папку перед компіляцією
	}
	defer src.Close()

	dst, err := os.OpenFile(targetPath, os.O_CREATE|os.O_WRONLY, 0755)
	if err != nil {
		return "", err
	}
	defer dst.Close()

	if _, err := io.Copy(dst, src); err != nil {
		return "", err
	}
	dst.Close() // Закриваємо перед Chmod

	// Примусово ставимо права на виконання для Mac/Linux
	if runtime.GOOS != "windows" {
		os.Chmod(targetPath, 0755)
	}

	return targetPath, nil
}

// EnsureExifTool перевіряє наявність ExifTool і розпаковує його, якщо потрібно.
func EnsureExifTool() (string, error) {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}
	binDir := filepath.Join(configDir, "Soloveyko", "bin")

	switch runtime.GOOS {
	case "windows":
		// Для Windows шукаємо в підпапці, яку ми розпакуємо
		targetDir := filepath.Join(binDir, "exiftool_win")
		targetExe := filepath.Join(targetDir, "exiftool.exe")

		if _, err := os.Stat(targetExe); err == nil {
			return targetExe, nil
		}

		// Розпаковуємо
		os.MkdirAll(binDir, 0755)
		src, err := bin.Files.Open("exiftool_win.zip")
		if err != nil {
			return "", err
		}
		defer src.Close()

		tempZip := filepath.Join(binDir, "exiftool_temp.zip")
		dst, err := os.Create(tempZip)
		if err != nil {
			return "", err
		}

		if _, err := io.Copy(dst, src); err != nil {
			dst.Close()
			return "", err
		}
		dst.Close()

		if err := Unzip(tempZip, binDir); err != nil {
			os.Remove(tempZip)
			return "", err
		}
		os.Remove(tempZip)

		// Перевіряємо чи з'явилася папка і чи треба перейменувати exiftool(-k).exe
		// (Користувач міг запакувати як є)
		exifK := filepath.Join(targetDir, "exiftool(-k).exe")
		if _, err := os.Stat(exifK); err == nil {
			os.Rename(exifK, targetExe)
		}

		if _, err := os.Stat(targetExe); err == nil {
			return targetExe, nil
		}
	case "darwin":
		targetDir := filepath.Join(binDir, "exiftool_mac")
		targetExe := filepath.Join(targetDir, "exiftool")

		if _, err := os.Stat(targetExe); err == nil {
			return targetExe, nil
		}

		// Розпаковуємо
		os.MkdirAll(binDir, 0755)
		src, err := bin.Files.Open("exiftool_mac.zip")
		if err != nil {
			// Якщо немає зіпа, шукаємо в системі
			if p, err := exec.LookPath("exiftool"); err == nil {
				return p, nil
			}
			return "", err
		}
		defer src.Close()

		tempZip := filepath.Join(binDir, "exiftool_temp_mac.zip")
		dst, err := os.Create(tempZip)
		if err != nil {
			return "", err
		}

		if _, err := io.Copy(dst, src); err != nil {
			dst.Close()
			return "", err
		}
		dst.Close()

		if err := Unzip(tempZip, binDir); err != nil {
			os.Remove(tempZip)
			return "", err
		}
		os.Remove(tempZip)

		// Ставимо права на виконання
		if _, err := os.Stat(targetExe); err == nil {
			os.Chmod(targetExe, 0755)
			return targetExe, nil
		}

		// На Mac також шукаємо системний якщо розпаковка не дала результату
		if p, err := exec.LookPath("exiftool"); err == nil {
			return p, nil
		}
	}

	return "", nil
}
// Виправлено: Логіка детекції шляху тепер перевіряє декілька варіантів (Mac/Win), 
// щоб коректно знаходити WhisperX навіть за наявності вкладених папок в архіві.
func GetWhisperXExePath() string {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	binDir := filepath.Join(configDir, "Soloveyko", "bin")

	possiblePaths := []string{
		filepath.Join(binDir, "whisperx_cli.exe"),
		filepath.Join(binDir, "whisperx_aligner_win", "whisperx_cli.exe"),
		filepath.Join(binDir, "whisperx-win", "whisperx_cli.exe"),
		filepath.Join(binDir, "whisperx-mac", "whisperx_cli"),
	}

	for _, p := range possiblePaths {
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}

	return ""
}

// Додано: Нова функція для автоматизації встановлення WhisperX, яка об'єднує завантаження 
// та розпакування з єдиним прогресом для UI.
func DownloadAndInstallWhisperX(progressFunc func(string, float64)) error {
	var url string
	switch runtime.GOOS {
	case "windows":
		url = "https://github.com/kvxshenyjbetxn/video.maker.releases/releases/download/whisperx/whisperx-win.zip"
	case "darwin":
		url = "https://github.com/kvxshenyjbetxn/video.maker.releases/releases/download/whisperx/whisperx-mac.zip"
	default:
		return fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}

	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}
	binDir := filepath.Join(configDir, "Soloveyko", "bin")
	os.MkdirAll(binDir, 0755)

	tempZip := filepath.Join(binDir, "whisperx_download.zip")
	defer os.Remove(tempZip)

	// 1. Завантаження
	err = DownloadWithProgress(url, tempZip, func(percent float64) {
		if progressFunc != nil {
			// Перші 80% - завантаження
			progressFunc("Завантаження...", percent*0.8)
		}
	})
	if err != nil {
		return err
	}

	// 2. Розпакування
	err = UnzipWithProgress(tempZip, binDir, func(percent float64) {
		if progressFunc != nil {
			// Останні 20% - розпакування (від 80% до 100%)
			progressFunc("Розпакування...", 80+(percent*0.2))
		}
	})

	return err
}

// DownloadWithProgress виконує HTTP Get запит та відстежує кількість отриманих байтів 
// для розрахунку відсотка прогресу завантаження.
func DownloadWithProgress(url, dest string, progressFunc func(float64)) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	out, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer out.Close()

	total := resp.ContentLength
	counter := &WriteCounter{
		Total:        total,
		OnProgress:   progressFunc,
	}

	if _, err = io.Copy(out, io.TeeReader(resp.Body, counter)); err != nil {
		return err
	}

	return nil
}

// WriteCounter допоміжний тип для підрахунку байтів
type WriteCounter struct {
	Total      int64
	Downloaded int64
	OnProgress func(float64)
}

func (wc *WriteCounter) Write(p []byte) (int, error) {
	n := len(p)
	wc.Downloaded += int64(n)
	if wc.Total > 0 && wc.OnProgress != nil {
		percent := float64(wc.Downloaded) / float64(wc.Total) * 100
		wc.OnProgress(percent)
	}
	return n, nil
}
