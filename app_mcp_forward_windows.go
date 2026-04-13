//go:build windows

package main

import (
	"fmt"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
)

func (a *App) launchMCPForwardScript() error {
	scriptPath, err := resolveMCPForwardScriptPath()
	if err != nil {
		return err
	}

	if pids, err := findMCPForwardPIDs(); err == nil && len(pids) > 0 {
		return nil
	}

	a.mcpForwardMu.Lock()
	defer a.mcpForwardMu.Unlock()

	if a.mcpForwardCmd != nil && a.mcpForwardCmd.Process != nil {
		if err := a.mcpForwardCmd.Process.Signal(syscall.Signal(0)); err == nil {
			return nil
		}
		a.mcpForwardCmd = nil
	}

	cmd := exec.Command("cmd.exe", "/c", "start", "", "/d", filepath.Dir(scriptPath), scriptPath)
	cmd.Dir = filepath.Dir(scriptPath)

	if err := cmd.Start(); err != nil {
		return err
	}
	a.mcpForwardCmd = cmd

	go func(startedCmd *exec.Cmd) {
		err := startedCmd.Wait()

		a.mcpForwardMu.Lock()
		if a.mcpForwardCmd == startedCmd {
			a.mcpForwardCmd = nil
		}
		a.mcpForwardMu.Unlock()

		if err != nil && a.ctx != nil {
			a.LogToUI("WARN", fmt.Sprintf("[MCP] Forward process exited: %v", err))
		}
	}(cmd)

	return nil
}

func killMCPForwardCmd(cmd *exec.Cmd) error {
	if cmd == nil || cmd.Process == nil {
		return nil
	}
	if err := killMCPForwardPID(cmd.Process.Pid); err != nil {
		return err
	}
	return nil
}

func killMCPForwardPID(pid int) error {
	if pid <= 0 {
		return nil
	}
	if err := exec.Command("taskkill", "/PID", fmt.Sprintf("%d", pid), "/T", "/F").Run(); err != nil {
		return err
	}
	return nil
}

func findMCPForwardPIDs() ([]int, error) {
	psCommand := fmt.Sprintf(
		`Get-CimInstance Win32_Process | Where-Object { $_.Name -ieq 'ssh.exe' -and $_.CommandLine -like '*%s*' } | Select-Object -ExpandProperty ProcessId`,
		mcpForwardTunnelSignature,
	)

	output, err := exec.Command(
		"powershell.exe",
		"-NoProfile",
		"-NonInteractive",
		"-Command",
		psCommand,
	).CombinedOutput()
	if err != nil {
		trimmed := strings.TrimSpace(string(output))
		if trimmed == "" {
			return nil, err
		}
		return nil, fmt.Errorf("%w: %s", err, trimmed)
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	pids := make([]int, 0, len(lines))
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		pid, convErr := strconv.Atoi(line)
		if convErr != nil {
			continue
		}
		pids = append(pids, pid)
	}

	return pids, nil
}

func signalZero() syscall.Signal {
	return syscall.Signal(0)
}
