package utils

import (
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

type GPUData struct {
	Name    string  `json:"name"`
	Percent float64 `json:"percent"`
}

type SystemStats struct {
	CPUPercent float64    `json:"cpuPercent"`
	RAMTotal   uint64     `json:"ramTotal"`
	RAMUsed    uint64     `json:"ramUsed"`
	RAMPercent float64    `json:"ramPercent"`
	GPUs       []GPUData  `json:"gpus"`
	Disks      []DiskInfo `json:"disks"`
}

type StatsService struct {
	cachedGPUNames []string
}

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
		seenMounts := make(map[string]bool)
		for _, partition := range partitions {
			m := partition.Mountpoint

			// Global filter for virtual/system filesystems
			fs := strings.ToLower(partition.Fstype)
			if fs == "devfs" || fs == "tmpfs" || fs == "autofs" || fs == "map" || fs == "nullfs" || fs == "procfs" || fs == "sysfs" {
				continue
			}

			// macOS specific filtering
			if runtime.GOOS == "darwin" {
				// Hide system-internal volumes that clutter the view
				if strings.HasPrefix(m, "/System/Volumes/") && m != "/System/Volumes/Data" {
					continue
				}
				if strings.HasPrefix(m, "/private/var/vm") || strings.Contains(m, "com.apple.TimeMachine") {
					continue
				}
				// If we have both / and /System/Volumes/Data (modern macOS split),
				// they show the same space. Keep / to avoid "double" disks.
				if m == "/System/Volumes/Data" {
					continue
				}
			}

			// Windows/Linux general filter
			if strings.HasPrefix(m, "/dev") || strings.HasPrefix(m, "/sys") || strings.HasPrefix(m, "/proc") {
				continue
			}

			if seenMounts[m] {
				continue
			}

			usage, err := disk.Usage(m)
			if err == nil && usage.Total > 0 {
				displayName := m
				if runtime.GOOS == "darwin" {
					if m == "/" {
						displayName = "Macintosh HD"
					} else if strings.HasPrefix(m, "/Volumes/") {
						displayName = strings.TrimPrefix(m, "/Volumes/")
					}
				}

				stats.Disks = append(stats.Disks, DiskInfo{
					Device:      partition.Device,
					Mountpoint:  displayName,
					Total:       usage.Total,
					Free:        usage.Free,
					Used:        usage.Used,
					UsedPercent: usage.UsedPercent,
				})
				seenMounts[m] = true
			}
		}
	}

	// GPU
	switch runtime.GOOS {
	case "windows":
		stats.GPUs = s.getWindowsGPUs()
	case "darwin":
		stats.GPUs = []GPUData{{Name: getMacGPUInfo(), Percent: 0}}
	default:
		stats.GPUs = []GPUData{{Name: "N/A", Percent: 0}}
	}

	return stats, nil
}

func (s *StatsService) getWindowsGPUs() []GPUData {
	var gpus []GPUData

	// 1. Try to get names (cached or via PowerShell)
	if len(s.cachedGPUNames) > 0 {
		for _, name := range s.cachedGPUNames {
			gpus = append(gpus, GPUData{Name: name, Percent: 0})
		}
	} else {
		psNamesCmd := "(Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name | Sort-Object -Unique)"
		out, err := runHiddenCommand("powershell", "-Command", psNamesCmd)
		if err == nil {
			lines := strings.Split(string(out), "\n")
			for _, line := range lines {
				name := strings.TrimSpace(line)
				// Cleanup name from invisible characters
				name = strings.Map(func(r rune) rune {
					if r < 32 || (r > 126 && r < 160) {
						return -1
					}
					return r
				}, name)

				if name != "" {
					// Avoid duplicates (system might still return similar names with different cases)
					isDuplicate := false
					normalizedCurrent := strings.ToLower(name)
					for _, existing := range s.cachedGPUNames {
						if strings.ToLower(existing) == normalizedCurrent {
							isDuplicate = true
							break
						}
					}

					if !isDuplicate {
						gpus = append(gpus, GPUData{Name: name, Percent: 0})
						s.cachedGPUNames = append(s.cachedGPUNames, name)
					}
				}
			}
		}
	}

	if len(gpus) == 0 {
		return []GPUData{{Name: "Generic GPU", Percent: 0}}
	}

	// 2. Try to get utilization metrics
	// NVIDIA SMI (accurate for NVIDIA)
	nvOut, nvErr := runHiddenCommand("nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits")
	if nvErr == nil {
		nvLines := strings.Split(strings.TrimSpace(string(nvOut)), "\n")
		nvIdx := 0
		for i := range gpus {
			if strings.Contains(strings.ToLower(gpus[i].Name), "nvidia") && nvIdx < len(nvLines) {
				val, err := strconv.ParseFloat(strings.TrimSpace(nvLines[nvIdx]), 64)
				if err == nil {
					gpus[i].Percent = val
				}
				nvIdx++
			}
		}
	}

	// For other GPUs (Intel, AMD) or if NVIDIA SMI failed, use PowerShell CIM
	// Note: It's hard to map UtilizationPercentage to specific GPU in WMI perfectly
	// because GPUEngine doesn't hold the Name, it holds LUIDs.
	// As a best effort, we'll get the max utilization for those that still have 0.
	psLoadCmd := "$v = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -ErrorAction SilentlyContinue; if ($v) { ($v | Measure-Object -Property UtilizationPercentage -Max).Maximum } else { 0 }"
	loadOut, loadErr := runHiddenCommand("powershell", "-Command", psLoadCmd)
	if loadErr == nil {
		maxLoad, err := strconv.ParseFloat(strings.TrimSpace(string(loadOut)), 64)
		if err == nil && maxLoad > 0 {
			for i := range gpus {
				if gpus[i].Percent == 0 {
					gpus[i].Percent = maxLoad
					// We apply maxLoad to all non-nvidia as a fallback
					// since we can't easily distinguish which one is working
					// without complex LUID mapping.
				}
			}
		}
	}

	return gpus
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
