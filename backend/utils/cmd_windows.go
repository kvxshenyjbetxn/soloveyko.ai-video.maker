//go:build windows

package utils

import (
	"context"
	"os/exec"
	"syscall"
	"time"
)

// runHiddenCommand виконує команду без створення вікна консолі на Windows
func runHiddenCommand(name string, args ...string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, name, args...)
	cmd.SysProcAttr = &syscall.SysProcAttr{
		HideWindow:    true,
		CreationFlags: 0x08000000, // CREATE_NO_WINDOW
	}
	return cmd.Output()
}
