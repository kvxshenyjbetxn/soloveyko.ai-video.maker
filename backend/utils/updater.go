package utils

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

type UpdateManifest struct {
	Version     string `json:"version"`
	URL         string `json:"url"`
	Notes       string `json:"notes"`
	Checksum    string `json:"checksum"` // SHA-256
	ReleaseDate string `json:"release_date"`
}

type UpdateManager struct {
	CurrentVersion string
	ManifestURL    string
}

func NewUpdateManager(currentVersion string) *UpdateManager {
	return &UpdateManager{
		CurrentVersion: currentVersion,
		ManifestURL:    "https://raw.githubusercontent.com/USER/REPO/main/updates.json", // ТPlaceholder, змініть на свій
	}
}

// CompareVersions повертає true, якщо v2 > v1
func CompareVersions(v1, v2 string) bool {
	v1 = strings.TrimPrefix(v1, "v")
	v2 = strings.TrimPrefix(v2, "v")

	p1 := strings.Split(v1, ".")
	p2 := strings.Split(v2, ".")

	for i := 0; i < len(p1) && i < len(p2); i++ {
		var n1, n2 int
		fmt.Sscanf(p1[i], "%d", &n1)
		fmt.Sscanf(p2[i], "%d", &n2)

		if n1 < n2 {
			return true
		}
		if n1 > n2 {
			return false
		}
	}
	return len(p1) < len(p2)
}

func (m *UpdateManager) Check(manifestURL string) (*UpdateManifest, error) {
	if manifestURL != "" {
		m.ManifestURL = manifestURL
	}

	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Get(m.ManifestURL)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("server returned status %d", resp.StatusCode)
	}

	var manifests map[string]UpdateManifest
	if err := json.NewDecoder(resp.Body).Decode(&manifests); err != nil {
		// Спробуємо як один об'єкт, якщо це не мапа за ОС
		resp, err = client.Get(m.ManifestURL)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		var single UpdateManifest
		if err := json.NewDecoder(resp.Body).Decode(&single); err == nil {
			if CompareVersions(m.CurrentVersion, single.Version) {
				return &single, nil
			}
			return nil, nil
		}
		return nil, err
	}

	// Шукаємо для поточної ОС
	osKey := runtime.GOOS
	if manifest, ok := manifests[osKey]; ok {
		if CompareVersions(m.CurrentVersion, manifest.Version) {
			return &manifest, nil
		}
	}

	return nil, nil
}

func (m *UpdateManager) Download(url string, progressChan chan int) (string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("failed to download: status %d", resp.StatusCode)
	}

	tmpFile, err := os.CreateTemp("", "soloveyko-update-*.zip")
	if err != nil {
		return "", err
	}
	defer tmpFile.Close()

	size := resp.ContentLength
	buffer := make([]byte, 32*1024)
	var downloaded int64

	for {
		n, err := resp.Body.Read(buffer)
		if n > 0 {
			_, writeErr := tmpFile.Write(buffer[:n])
			if writeErr != nil {
				return "", writeErr
			}
			downloaded += int64(n)
			if size > 0 {
				progress := int(float64(downloaded) / float64(size) * 100)
				select {
				case progressChan <- progress:
				default:
				}
			}
		}
		if err == io.EOF {
			break
		}
		if err != nil {
			return "", err
		}
	}

	return tmpFile.Name(), nil
}

func VerifyChecksum(filePath, expectedChecksum string) error {
	if expectedChecksum == "" {
		return nil
	}

	f, err := os.Open(filePath)
	if err != nil {
		return err
	}
	defer f.Close()

	h := sha256.New()
	if _, err := io.Copy(h, f); err != nil {
		return err
	}

	actualChecksum := hex.EncodeToString(h.Sum(nil))
	if !strings.EqualFold(actualChecksum, expectedChecksum) {
		return fmt.Errorf("checksum mismatch: expected %s, got %s", expectedChecksum, actualChecksum)
	}

	return nil
}

func (m *UpdateManager) Apply(pkgPath string) error {
	exePath, err := os.Executable()
	if err != nil {
		return err
	}

	switch runtime.GOOS {
	case "windows":
		return m.applyWindows(pkgPath, exePath)
	case "darwin":
		return m.applyMacOS(pkgPath, exePath)
	default:
		return fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}
}

func (m *UpdateManager) applyWindows(pkgPath, exePath string) error {
	destDir := filepath.Dir(exePath)
	batchPath := filepath.Join(os.TempDir(), "soloveyko_update.bat")

	// Створюємо батнік для заміни файлів
	// 1. Чекаємо поки процес завершиться
	// 2. Розпаковуємо (потрібен powershell для нативності)
	// 3. Замінюємо
	// 4. Запускаємо нову версію
	// 5. Видаляємо себе

	batchContent := fmt.Sprintf(`@echo off
timeout /t 2 /nobreak > nul
powershell -Command "Expand-Archive -Path '%s' -DestinationPath '%s' -Force"
if errorlevel 1 goto error
start "" "%s"
del /f /q "%s"
exit
:error
echo Update failed. Please try again or download manually.
pause
exit
`, pkgPath, destDir, exePath, batchPath)

	err := os.WriteFile(batchPath, []byte(batchContent), 0644)
	if err != nil {
		return err
	}

	cmd := exec.Command("cmd", "/c", "start", "/min", batchPath)
	return cmd.Start()
}

func (m *UpdateManager) applyMacOS(pkgPath, exePath string) error {
	// Для Mac зазвичай додаток у .app бандлі
	// exePath повертає шлях до бінарника всередині MacOS/
	// Нам потрібно замінити весь бандл або просто бінарник, якщо це портативна версія

	shPath := filepath.Join(os.TempDir(), "soloveyko_update.sh")
	destDir := filepath.Dir(exePath)

	shContent := fmt.Sprintf(`#!/bin/bash
sleep 2
unzip -o "%s" -d "%s"
open "%s"
rm -- "$0"
`, pkgPath, destDir, exePath)

	err := os.WriteFile(shPath, []byte(shContent), 0755)
	if err != nil {
		return err
	}

	cmd := exec.Command("sh", shPath)
	return cmd.Start()
}
