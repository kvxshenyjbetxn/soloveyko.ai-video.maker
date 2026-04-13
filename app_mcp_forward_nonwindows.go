//go:build !windows

package main

import (
	"fmt"
	"os/exec"
	"syscall"
)

func (a *App) launchMCPForwardScript() error {
	return fmt.Errorf("MCP forward auto-launch is supported only on Windows")
}

func killMCPForwardCmd(cmd *exec.Cmd) error {
	return nil
}

func killMCPForwardPID(pid int) error {
	return nil
}

func findMCPForwardPIDs() ([]int, error) {
	return nil, nil
}

func signalZero() syscall.Signal {
	return syscall.Signal(0)
}
