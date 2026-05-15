package utils

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
)

type footagePoolState struct {
	NextIndex map[string]int `json:"nextIndex"` // pool key → next index
}

// FootagePool tracks global round-robin position for each unique footage set.
// Thread-safe and persists state across restarts.
type FootagePool struct {
	mu       sync.Mutex
	state    footagePoolState
	filePath string
}

func NewFootagePool(configDir string) *FootagePool {
	fp := &FootagePool{
		filePath: filepath.Join(configDir, "footage_pool.json"),
		state:    footagePoolState{NextIndex: make(map[string]int)},
	}
	fp.load()
	return fp
}

func (fp *FootagePool) load() {
	data, err := os.ReadFile(fp.filePath)
	if err != nil {
		return
	}
	_ = json.Unmarshal(data, &fp.state)
	if fp.state.NextIndex == nil {
		fp.state.NextIndex = make(map[string]int)
	}
}

func (fp *FootagePool) save() {
	data, err := json.MarshalIndent(fp.state, "", "  ")
	if err != nil {
		return
	}
	_ = os.WriteFile(fp.filePath, data, 0644)
}

// poolKey returns a stable identifier for a set of footage paths (order-independent).
func poolKey(paths []string) string {
	sorted := make([]string, len(paths))
	copy(sorted, paths)
	sort.Strings(sorted)
	return strings.Join(sorted, "\n")
}

// ClaimNext returns `count` footage paths in round-robin order starting from the current
// global position, then advances the position by 1.
// This guarantees each task starts at a different footage file regardless of how many files it claims.
// Safe for concurrent use across goroutines and across restarts.
func (fp *FootagePool) ClaimNext(paths []string, count int) []string {
	if len(paths) == 0 || count <= 0 {
		return nil
	}

	fp.mu.Lock()
	defer fp.mu.Unlock()

	key := poolKey(paths)
	startIdx := fp.state.NextIndex[key]

	result := make([]string, count)
	for i := 0; i < count; i++ {
		result[i] = paths[(startIdx+i)%len(paths)]
	}

	// Always advance by 1 so the next task starts from the next footage file,
	// regardless of how many files this task claimed.
	fp.state.NextIndex[key] = (startIdx + 1) % len(paths)
	fp.save()

	return result
}

// ClaimForDuration claims footage files in round-robin order until their total duration
// covers targetDur seconds. getDur should return file duration in seconds (0 = unknown).
// If duration cannot be determined, falls back to claiming len(paths) files.
// Always advances the global position by 1 after claiming.
func (fp *FootagePool) ClaimForDuration(paths []string, targetDur float64, getDur func(string) float64) []string {
	if len(paths) == 0 {
		return nil
	}

	fp.mu.Lock()
	defer fp.mu.Unlock()

	key := poolKey(paths)
	startIdx := fp.state.NextIndex[key]

	var result []string
	totalDur := 0.0
	i := 0
	canProbe := true

	for totalDur < targetDur {
		if i >= len(paths)*200 { // safety: 200 full cycles maximum
			break
		}
		p := paths[(startIdx+i)%len(paths)]
		dur := getDur(p)
		result = append(result, p)
		i++
		if dur <= 0 {
			canProbe = false
			break
		}
		totalDur += dur
	}

	// If we couldn't probe durations at all, fall back to one full set of paths
	if !canProbe && len(result) < len(paths) {
		for j := len(result); j < len(paths); j++ {
			result = append(result, paths[(startIdx+j)%len(paths)])
		}
	}

	fp.state.NextIndex[key] = (startIdx + 1) % len(paths)
	fp.save()
	return result
}
