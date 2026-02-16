package utils

import (
	"context"
	"os/exec"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/mem"
)

type DiskInfo struct {
	Device      string  `json:"device"`
	Mountpoint  string  `json:"mountpoint"`
	Total       uint64  `json:"total"`
	Free        uint64  `json:"free"`
	Used        uint64  `json:"used"`
	UsedPercent float64 `json:"usedPercent"`
}

type SystemStats struct {
	CPUPercent float64    `json:"cpuPercent"`
	RAMTotal   uint64     `json:"ramTotal"`
	RAMUsed    uint64     `json:"ramUsed"`
	RAMPercent float64    `json:"ramPercent"`
	GPUInfo    string     `json:"gpuInfo"`
	GPUPercent float64    `json:"gpuPercent"`
	Disks      []DiskInfo `json:"disks"`
}

type StatsService struct{}

func NewStatsService() *StatsService {
	return &StatsService{}
}

func (s *StatsService) GetSystemStats() (*SystemStats, error) {
	stats := &SystemStats{}

	// CPU
	cpuPercents, err := cpu.Percent(0, false)
	if err == nil && len(cpuPercents) > 0 {
		stats.CPUPercent = cpuPercents[0]
	}

	// RAM
	v, err := mem.VirtualMemory()
	if err == nil {
		stats.RAMTotal = v.Total
		stats.RAMUsed = v.Used
		stats.RAMPercent = v.UsedPercent
	}

	// Disks
	partitions, err := disk.Partitions(false)
	if err == nil {
		for _, partition := range partitions {
			if strings.HasPrefix(partition.Mountpoint, "/dev") || strings.HasPrefix(partition.Mountpoint, "/sys") || strings.HasPrefix(partition.Mountpoint, "/proc") {
				continue
			}
			usage, err := disk.Usage(partition.Mountpoint)
			if err == nil && usage.Total > 0 {
				stats.Disks = append(stats.Disks, DiskInfo{
					Device:      partition.Device,
					Mountpoint:  partition.Mountpoint,
					Total:       usage.Total,
					Free:        usage.Free,
					Used:        usage.Used,
					UsedPercent: usage.UsedPercent,
				})
			}
		}
	}

	// GPU
	if runtime.GOOS == "windows" {
		stats.GPUInfo = getWindowsGPUInfo()
		stats.GPUPercent = getWindowsGPULoad()
	} else if runtime.GOOS == "darwin" {
		stats.GPUInfo = getMacGPUInfo()
		stats.GPUPercent = 0
	} else {
		stats.GPUInfo = "N/A"
	}

	return stats, nil
}

// runHiddenCommand виконує команду без створення вікна консолі на Windows
func runHiddenCommand(name string, args ...string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, name, args...)
	if runtime.GOOS == "windows" {
		cmd.SysProcAttr = &syscall.SysProcAttr{
			HideWindow:    true,
			CreationFlags: 0x08000000, // CREATE_NO_WINDOW
		}
	}
	return cmd.Output()
}

func getWindowsGPULoad() float64 {
	// 1. Try NVIDIA SMI
	out, err := runHiddenCommand("nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits")
	if err == nil {
		val, err := strconv.ParseFloat(strings.TrimSpace(string(out)), 64)
		if err == nil {
			return val
		}
	}

	// 2. Try PowerShell
	psCmd := "$v = Get-Counter '\\GPU Engine(*)\\Utilization Percentage' -ErrorAction SilentlyContinue; if ($v) { ($v.CounterSamples | Measure-Object -Property CookedValue -Max).Maximum } else { 0 }"
	out, err = runHiddenCommand("powershell", "-Command", psCmd)
	if err == nil {
		val, err := strconv.ParseFloat(strings.TrimSpace(string(out)), 64)
		if err == nil {
			return val
		}
	}

	// 3. Try WMIC
	out, err = runHiddenCommand("wmic", "path", "Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine", "get", "UtilizationPercentage")
	if err == nil {
		lines := strings.Split(string(out), "\n")
		var maxLoad float64
		for _, line := range lines {
			val, err := strconv.ParseFloat(strings.TrimSpace(line), 64)
			if err == nil && val > maxLoad {
				maxLoad = val
			}
		}
		if maxLoad > 0 {
			return maxLoad
		}
	}

	return 0
}

func getWindowsGPUInfo() string {
	out, err := runHiddenCommand("wmic", "path", "win32_VideoController", "get", "name")
	if err == nil {
		lines := strings.Split(string(out), "\n")
		for _, line := range lines {
			trimmed := strings.TrimSpace(line)
			if trimmed != "" && trimmed != "Name" {
				return trimmed
			}
		}
	}
	return "Generic GPU"
}

func getMacGPUInfo() string {
	out, err := runHiddenCommand("system_profiler", "SPDisplaysDataType")
	if err == nil {
		strOut := string(out)
		if strings.Contains(strOut, "Chipset Model:") {
			parts := strings.Split(strOut, "Chipset Model:")
			if len(parts) > 1 {
				lines := strings.Split(parts[1], "\n")
				return strings.TrimSpace(lines[0])
			}
		}
	}
	return "Apple GPU"
}
