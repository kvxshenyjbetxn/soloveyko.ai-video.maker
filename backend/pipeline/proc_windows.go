//go:build windows

package pipeline

import (
	"os/exec"
	"runtime"
	"syscall"
)

// applyNicePriority is a no-op on Windows — priority is set via CreationFlags pre-start.
func applyNicePriority(pid int, priority string) { _ = pid; _ = priority }

// setProcPriority sets the Windows creation-flag priority class before cmd.Start()
func setProcPriority(cmd *exec.Cmd, priority string) {
	var flag uint32 = 0x00000020 // NORMAL_PRIORITY_CLASS
	switch priority {
	case "idle":
		flag = 0x00000040 // IDLE_PRIORITY_CLASS
	case "low":
		flag = 0x00004000 // BELOW_NORMAL_PRIORITY_CLASS
	case "high":
		flag = 0x00008000 // ABOVE_NORMAL_PRIORITY_CLASS
	}
	if cmd.SysProcAttr == nil {
		cmd.SysProcAttr = &syscall.SysProcAttr{}
	}
	cmd.SysProcAttr.CreationFlags = flag
}

// setProcAffinity limits FFmpeg to first N CPU cores using Windows SetProcessAffinityMask.
// Must be called AFTER cmd.Start().
func setProcAffinity(pid int, cores int) {
	if cores <= 0 {
		return
	}
	total := runtime.NumCPU()
	if cores > total {
		cores = total
	}
	// Bitmask: first 'cores' bits set (e.g. 4 cores → 0b00001111 = 15)
	mask := uint32((1 << uint(cores)) - 1)

	k32 := syscall.NewLazyDLL("kernel32.dll")
	openProc := k32.NewProc("OpenProcess")
	setAff := k32.NewProc("SetProcessAffinityMask")

	const PROCESS_ALL_ACCESS = 0x1F0FFF
	h, _, _ := openProc.Call(PROCESS_ALL_ACCESS, 0, uintptr(pid))
	if h != 0 {
		setAff.Call(h, uintptr(mask))
		syscall.CloseHandle(syscall.Handle(h))
	}
}
