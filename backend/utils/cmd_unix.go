//go:build !windows

package utils

import (
	"context"
	"os/exec"
	"time"
)

// runHiddenCommand виконує команду стандартним способом на Unix-системах,
// оскільки вікна консолі там не створюються автоматично.
func runHiddenCommand(name string, args ...string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, name, args...)
	return cmd.Output()
}

// PrepareHiddenCmd на Unix-системах нічого не робить
func PrepareHiddenCmd(cmd *exec.Cmd) {}
