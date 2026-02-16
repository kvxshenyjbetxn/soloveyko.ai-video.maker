package utils

import (
	"os/exec"
	"runtime"
	"strconv"
	"strings"

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
			// Skip special partitions
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

func getWindowsGPULoad() float64 {
	// 1. Try NVIDIA SMI (Fastest & most accurate for NVIDIA)
	out, err := exec.Command("nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits").Output()
	if err == nil {
		val, err := strconv.ParseFloat(strings.TrimSpace(string(out)), 64)
		if err == nil {
			return val
		}
	}

	// 2. Try PowerShell approach for generic GPU load
	// We use the "Utilization Percentage" counter which is most common
	psCmd := "$v = Get-Counter '\\GPU Engine(*)\\Utilization Percentage' -ErrorAction SilentlyContinue; if ($v) { ($v.CounterSamples | Measure-Object -Property CookedValue -Max).Maximum } else { 0 }"
	out, err = exec.Command("powershell", "-Command", psCmd).Output()
	if err == nil {
		val, err := strconv.ParseFloat(strings.TrimSpace(string(out)), 64)
		if err == nil {
			return val
		}
	}

	// 3. Try WMIC (Very robust, works on almost all Windows)
	wmicCmd := "path Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine get UtilizationPercentage"
	out, err = exec.Command("wmic", strings.Split(wmicCmd, " ")...).Output()
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
	// Get GPU Name using WMIC
	out, err := exec.Command("wmic", "path", "win32_VideoController", "get", "name").Output()
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
	out, err := exec.Command("system_profiler", "SPDisplaysDataType").Output()
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
