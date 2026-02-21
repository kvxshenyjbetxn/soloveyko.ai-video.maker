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
	PrepareHiddenCmd(cmd)
	return cmd.Output()
}

// PrepareHiddenCmd налаштовує команду так, щоб вона не відкривала вікно консолі на Windows
func PrepareHiddenCmd(cmd *exec.Cmd) {
	if cmd.SysProcAttr == nil {
		cmd.SysProcAttr = &syscall.SysProcAttr{}
	}
	cmd.SysProcAttr.HideWindow = true
	cmd.SysProcAttr.CreationFlags |= 0x08000000 // CREATE_NO_WINDOW
}
