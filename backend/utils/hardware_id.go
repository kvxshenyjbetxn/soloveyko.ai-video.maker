package utils

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"runtime"
)

// GetHardwareID retrieves the unique hardware identifier for the device (Platform UUID)
// and returns its SHA-256 hash.
func GetHardwareID() string {
	var hwID string

	if runtime.GOOS == "windows" {
		cmd := exec.Command("reg", "query", `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography`, "/v", "MachineGuid")
		out, err := cmd.CombinedOutput()
		if err == nil {
			re := regexp.MustCompile(`[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}`)
			matches := re.FindStringSubmatch(string(out))
			if len(matches) > 0 {
				hwID = matches[0]
			}
		}
	} else if runtime.GOOS == "darwin" {
		cmd := exec.Command("ioreg", "-rd1", "-c", "IOPlatformExpertDevice")
		out, err := cmd.CombinedOutput()
		if err == nil {
			re := regexp.MustCompile(`"IOPlatformUUID" = "([^"]+)"`)
			matches := re.FindStringSubmatch(string(out))
			if len(matches) > 1 {
				hwID = matches[1]
			}
		}
	}

	if hwID == "" {
		host, _ := os.Hostname()
		hwID = fmt.Sprintf("%s-%s-%s", host, runtime.GOOS, runtime.GOARCH)
	}

	hasher := sha256.New()
	hasher.Write([]byte(hwID))
	return hex.EncodeToString(hasher.Sum(nil))
}
