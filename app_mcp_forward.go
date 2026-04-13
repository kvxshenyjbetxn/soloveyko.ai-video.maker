package main

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

const mcpForwardScriptName = "startVPS.bat"
const mcpForwardTunnelSignature = "127.0.0.1:39245:127.0.0.1:39245"

// GetMCPAutoForwardEnabled returns whether the MCP tunnel script should be launched automatically.
func (a *App) GetMCPAutoForwardEnabled() bool {
	return a.settings.GetMCPAutoForwardEnabled()
}

// SaveMCPAutoForwardEnabled persists whether the MCP tunnel script should be launched automatically.
func (a *App) SaveMCPAutoForwardEnabled(enabled bool) error {
	return a.settings.SetMCPAutoForwardEnabled(enabled)
}

// GetMCPForwardScriptPath returns the resolved path to the MCP tunnel batch script.
func (a *App) GetMCPForwardScriptPath() string {
	path, err := resolveMCPForwardScriptPath()
	if err != nil {
		return ""
	}
	return path
}

// GetMCPForwardStatus returns the current state of the auto-forward tunnel integration.
func (a *App) GetMCPForwardStatus() map[string]interface{} {
	status := map[string]interface{}{
		"supported":   runtime.GOOS == "windows",
		"enabled":     a.settings.GetMCPAutoForwardEnabled(),
		"os":          runtime.GOOS,
		"scriptFound": false,
		"scriptPath":  "",
		"running":     false,
		"pid":         0,
	}

	if path, err := resolveMCPForwardScriptPath(); err == nil {
		status["scriptFound"] = true
		status["scriptPath"] = path
	}

	a.mcpForwardMu.Lock()
	if a.mcpForwardCmd != nil && a.mcpForwardCmd.Process != nil {
		if err := a.mcpForwardCmd.Process.Signal(signalZero()); err == nil {
			status["running"] = true
			status["pid"] = a.mcpForwardCmd.Process.Pid
			a.mcpForwardMu.Unlock()
			return status
		}
		a.mcpForwardCmd = nil
	}
	a.mcpForwardMu.Unlock()

	pids, err := findMCPForwardPIDs()
	if err == nil && len(pids) > 0 {
		status["running"] = true
		status["pid"] = pids[0]
		status["instances"] = len(pids)
		return status
	}

	status["instances"] = 0
	return status
}

func (a *App) startMCPForwardIfEnabled() {
	if runtime.GOOS != "windows" {
		return
	}
	if !a.settings.GetMCPAutoForwardEnabled() {
		return
	}

	go func() {
		time.Sleep(1500 * time.Millisecond)
		if err := a.launchMCPForwardScript(); err != nil {
			a.LogToUI("ERROR", fmt.Sprintf("[MCP] Failed to launch %s: %v", mcpForwardScriptName, err))
			return
		}
		a.LogToUI("INFO", fmt.Sprintf("[MCP] Launched %s automatically", mcpForwardScriptName))
	}()
}

func (a *App) stopMCPForwardProcess() {
	a.mcpForwardMu.Lock()
	cmd := a.mcpForwardCmd
	a.mcpForwardCmd = nil
	a.mcpForwardMu.Unlock()

	if cmd != nil && cmd.Process != nil {
		_ = killMCPForwardCmd(cmd)
	}

	pids, err := findMCPForwardPIDs()
	if err != nil {
		return
	}
	for _, pid := range pids {
		_ = killMCPForwardPID(pid)
	}
}

func resolveMCPForwardScriptPath() (string, error) {
	candidates := []string{}

	if wd, err := os.Getwd(); err == nil && strings.TrimSpace(wd) != "" {
		candidates = append(candidates, filepath.Join(wd, mcpForwardScriptName))
	}

	if exePath, err := os.Executable(); err == nil && strings.TrimSpace(exePath) != "" {
		exeCandidate := filepath.Join(filepath.Dir(exePath), mcpForwardScriptName)
		duplicate := false
		for _, candidate := range candidates {
			if strings.EqualFold(candidate, exeCandidate) {
				duplicate = true
				break
			}
		}
		if !duplicate {
			candidates = append(candidates, exeCandidate)
		}
	}

	for _, candidate := range candidates {
		info, err := os.Stat(candidate)
		if err == nil && !info.IsDir() {
			return candidate, nil
		}
	}

	if len(candidates) == 0 {
		return "", fmt.Errorf("could not resolve candidate paths for %s", mcpForwardScriptName)
	}
	return "", fmt.Errorf("%s not found; searched: %s", mcpForwardScriptName, strings.Join(candidates, ", "))
}
