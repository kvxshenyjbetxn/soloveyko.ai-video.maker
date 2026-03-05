package utils

import (
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
	ReleaseDate string `json:"release_date"`
}

type UpdateManager struct {
	CurrentVersion string
	ManifestURL    string
}

func NewUpdateManager(currentVersion string) *UpdateManager {
	return &UpdateManager{
		CurrentVersion: currentVersion,
		ManifestURL:    "https://new-project-combain-server-production.up.railway.app/latest_version",
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
	exeName := filepath.Base(exePath)

	userConfig, _ := os.UserConfigDir()
	updateWorkDir := filepath.Join(userConfig, "SoloveykoAI", "update")
	os.RemoveAll(updateWorkDir)
	os.MkdirAll(updateWorkDir, 0755)

	extractDir := filepath.Join(updateWorkDir, "files")
	os.MkdirAll(extractDir, 0755)

	batchPath := filepath.Join(updateWorkDir, "run.bat")

	batchContent := fmt.Sprintf(`@echo off
setlocal enabledelayedexpansion
title Soloveyko AI Update Tool

echo.
echo ========================================
echo   Soloveyko AI Update System
echo ========================================
echo.

echo [1/4] Closing application...
taskkill /F /IM "%s" /T > nul 2>&1
timeout /t 2 /nobreak > nul

echo [2/4] Extracting files...
powershell -NoProfile -Command "Expand-Archive -LiteralPath '%s' -DestinationPath '%s' -Force"
if errorlevel 1 goto error

echo [3/4] Installing...
:: Robocopy копіює все з розпакованої папки в папку програми
robocopy "%s" "%s" /E /IS /MOVE /R:3 /W:2 > nul
if errorlevel 8 goto error

echo [4/4] Starting Soloveyko AI...
:: Використовуємо PowerShell для надійного пошуку та запуску EXE (на випадок вкладених папок)
powershell -NoProfile -Command "$exe = Get-ChildItem -Path '%s' -Filter '%s' -Recurse | Select-Object -First 1; if ($exe) { Start-Process $exe.FullName }"

echo.
echo [DONE] Update successful!
goto cleanup

:error
echo.
echo !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
echo !! ERROR: Installation failed         !!
echo !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
echo.
pause
exit

:cleanup
:: Видаляємо темп через 3 секунди фоном
start /b "" cmd /c "timeout /t 3 > nul & rd /s /q \"%s\""
exit
`, exeName, pkgPath, extractDir, extractDir, destDir, destDir, exeName, updateWorkDir)

	err := os.WriteFile(batchPath, []byte(batchContent), 0644)
	if err != nil {
		return err
	}

	// Запускаємо батнік як прихований процес
	cmd := exec.Command("cmd", "/c", batchPath)
	PrepareHiddenCmd(cmd)
	return cmd.Start()
}

func (m *UpdateManager) applyMacOS(pkgPath, exePath string) error {
	// На Mac exePath зазвичай вказує на Contents/MacOS/soloveyko всередині .app
	appPath := exePath
	if idx := strings.Index(exePath, ".app/Contents/MacOS"); idx != -1 {
		appPath = exePath[:idx+4]
	}

	shPath := filepath.Join(os.TempDir(), "soloveyko_update.sh")

	// Створюємо скрипт для Mac з розширеним логуванням та надійнішою перевіркою
	shContent := fmt.Sprintf(`#!/bin/bash
exec > /tmp/soloveyko_update.log 2>&1
echo "Update started: $(date)"
echo "Archive: %s"
echo "Target: %s"

# Чекаємо виходу програми (до 5 секунд)
for i in {1..5}; do
    if ! pgrep -f "%s" > /dev/null; then break; fi
    echo "Waiting for app to close..."
    sleep 1
done

EXTRACT_DIR="/tmp/soloveyko_extract_$(date +%%s)"
mkdir -p "$EXTRACT_DIR"

echo "Extracting archive..."
# Використовуємо ditto (Mac native) якщо є, або unzip
if command -v ditto >/dev/null 2>&1; then
    ditto -x -k "%s" "$EXTRACT_DIR"
else
    unzip -q -o "%s" -d "$EXTRACT_DIR"
fi

# Шукаємо .app бандл (до 4 рівнів глибини, ігноруючи __MACOSX)
NEW_APP=$(find "$EXTRACT_DIR" -maxdepth 4 -name "*.app" -type d | grep -v "__MACOSX" | head -n 1)

if [ -n "$NEW_APP" ]; then
    echo "Found bundle in archive: $NEW_APP"
    
    # Робимо бінарник всередині бандла виконуваним (на випадок якщо zip робився на Windows)
    find "$NEW_APP/Contents/MacOS" -type f -exec chmod +x {} + 2>/dev/null
    
    echo "Replacing application bundle..."
    rm -rf "%s"
    cp -R "$NEW_APP" "%s"
    
    echo "Clearing quarantine flag..."
    xattr -rd com.apple.quarantine "%s" 2>/dev/null
    
    sync
    echo "Launching new version..."
    open "%s"
else
    echo "No .app bundle found in archive, searching for binary..."
    # Спроба знайти будь-який виконуваний файл або просто будь-який файл як бінарник
    NEW_BIN=$(find "$EXTRACT_DIR" -maxdepth 4 -type f -perm +111 | head -n 1)
    if [ -z "$NEW_BIN" ]; then NEW_BIN=$(find "$EXTRACT_DIR" -maxdepth 4 -type f | head -n 1); fi
    
    if [ -n "$NEW_BIN" ]; then
        echo "Found potential binary: $NEW_BIN"
        cp -af "$NEW_BIN" "%s"
        chmod +x "%s"
        sync
        open "%s"
    else
        echo "ERROR: No suitable update files found in archive"
    fi
fi

echo "Cleaning up..."
rm -rf "$EXTRACT_DIR"
echo "Update finished."
`, pkgPath, appPath, filepath.Base(exePath), pkgPath, pkgPath, appPath, appPath, appPath, appPath, exePath, exePath, appPath)

	err := os.WriteFile(shPath, []byte(shContent), 0755)
	if err != nil {
		return err
	}

	cmd := exec.Command("sh", shPath)
	return cmd.Start()
}
