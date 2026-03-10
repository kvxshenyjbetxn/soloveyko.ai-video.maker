package utils

import (
	"io"
	"os"
	"path/filepath"
	"runtime"
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
