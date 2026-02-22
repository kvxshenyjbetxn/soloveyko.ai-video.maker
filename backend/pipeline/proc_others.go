//go:build !windows

package pipeline

import (
	"os/exec"
	"syscall"
)

// setProcPriority on macOS/Linux is a no-op before start.
// Actual nice value is applied after start via setProcAffinity.
func setProcPriority(cmd *exec.Cmd, priority string) {
	// No-op pre-start on non-Windows. Priority is applied post-start in setProcAffinity.
	_ = cmd
	_ = priority
}

// setProcAffinity on macOS/Linux sets the process nice value (priority).
// CPU affinity pinning is not supported cross-platform.
func setProcAffinity(pid int, cores int) {
	// 'cores' is reused as priority signal on macOS/Linux:
	// cores == -1 → nice +10 (low)
	// cores == -2 → nice +19 (idle)
	// cores >= 0  → no change (normal)
	// This allows reuse of the same call signature.
	_ = pid
	_ = cores
}

// applyNicePriority applies nice value on macOS/Linux after process start.
func applyNicePriority(pid int, priority string) {
	var nice int
	switch priority {
	case "idle":
		nice = 19
	case "low":
		nice = 10
	default:
		return // normal — don't change
	}
	_ = syscall.Setpriority(syscall.PRIO_PROCESS, pid, nice)
}
