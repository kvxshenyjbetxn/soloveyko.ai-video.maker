package utils

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sync"
	"time"
)

type DailyStat struct {
	Date          string  `json:"date"`          // YYYY-MM-DD
	VideoCount    int     `json:"videoCount"`    // Number of completed "montage" stages
	TotalDuration float64 `json:"totalDuration"` // Sum of durations in seconds
}

type ProductionStats struct {
	DailyStats    map[string]*DailyStat `json:"dailyStats"`
	TotalVideos   int                   `json:"totalVideos"`
	TotalDuration float64               `json:"totalDuration"`
}

type ProductionStatsService struct {
	filePath string
	mu       sync.RWMutex
	stats    ProductionStats
}

func NewProductionStatsService() *ProductionStatsService {
	configDir, err := os.UserConfigDir()
	if err != nil {
		homeDir, _ := os.UserHomeDir()
		configDir = homeDir
	}

	appDataDir := filepath.Join(configDir, "Soloveyko", "data")
	_ = os.MkdirAll(appDataDir, 0755)

	filePath := filepath.Join(appDataDir, "production_stats.json")

	s := &ProductionStatsService{
		filePath: filePath,
		stats: ProductionStats{
			DailyStats: make(map[string]*DailyStat),
		},
	}

	s.load()
	return s
}

func (s *ProductionStatsService) load() {
	s.mu.Lock()
	defer s.mu.Unlock()

	data, err := os.ReadFile(s.filePath)
	if err == nil {
		_ = json.Unmarshal(data, &s.stats)
	}
	if s.stats.DailyStats == nil {
		s.stats.DailyStats = make(map[string]*DailyStat)
	}
}

func (s *ProductionStatsService) save() {
	data, err := json.MarshalIndent(s.stats, "", "  ")
	if err == nil {
		_ = os.WriteFile(s.filePath, data, 0644)
	}
}

func (s *ProductionStatsService) RecordCompletion(taskType string, duration float64) {
	s.mu.Lock()
	defer s.mu.Unlock()

	date := time.Now().Format("2006-01-02")
	stat, ok := s.stats.DailyStats[date]
	if !ok {
		stat = &DailyStat{Date: date}
		s.stats.DailyStats[date] = stat
	}

	stat.VideoCount++
	stat.TotalDuration += duration

	s.stats.TotalVideos++
	s.stats.TotalDuration += duration

	s.save()
}

type UIStatsResponse struct {
	TotalVideos      int          `json:"totalVideos"`
	TotalDuration    float64      `json:"totalDuration"`
	AverageDuration  float64      `json:"averageDuration"`
	DailyData        []*DailyStat `json:"dailyData"`
	Last30DaysVideos int          `json:"last30DaysVideos"`
}

func (s *ProductionStatsService) GetStats(days int) *UIStatsResponse {
	s.mu.RLock()
	defer s.mu.RUnlock()

	var dailyData []*DailyStat
	now := time.Now()

	// Collect dates for sorting/filtering
	var dates []string
	for d := range s.stats.DailyStats {
		dates = append(dates, d)
	}

	// Create a slice of all daily stats sorted by date
	var allDaily []*DailyStat
	for _, d := range s.stats.DailyStats {
		allDaily = append(allDaily, d)
	}

	// Simple date sorting (string sort works for YYYY-MM-DD)
	sortDaily(allDaily)

	last30Videos := 0
	thirtyDaysAgo := now.AddDate(0, 0, -30).Format("2006-01-02")

	for _, d := range allDaily {
		if d.Date >= thirtyDaysAgo {
			last30Videos += d.VideoCount
		}

		if days > 0 {
			limitDate := now.AddDate(0, 0, -days).Format("2006-01-02")
			if d.Date >= limitDate {
				dailyData = append(dailyData, d)
			}
		} else {
			dailyData = append(dailyData, d)
		}
	}

	avg := 0.0
	if s.stats.TotalVideos > 0 {
		avg = s.stats.TotalDuration / float64(s.stats.TotalVideos)
	}

	return &UIStatsResponse{
		TotalVideos:      s.stats.TotalVideos,
		TotalDuration:    s.stats.TotalDuration,
		AverageDuration:  avg,
		DailyData:        dailyData,
		Last30DaysVideos: last30Videos,
	}
}

// ClearData resets all stats to zero and saves
func (s *ProductionStatsService) ClearData() {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.stats.DailyStats = make(map[string]*DailyStat)
	s.stats.TotalVideos = 0
	s.stats.TotalDuration = 0

	s.save()
}

// GenerateRandomData populates the stats with random data for the last 30 days for testing
func (s *ProductionStatsService) GenerateRandomData() {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.stats.DailyStats = make(map[string]*DailyStat)
	s.stats.TotalVideos = 0
	s.stats.TotalDuration = 0

	now := time.Now()
	for i := 30; i >= 0; i-- {
		date := now.AddDate(0, 0, -i).Format("2006-01-02")

		// Random 0 to 8 videos per day
		count := int(time.Now().UnixNano() % 8)
		if i == 0 {
			count = 3
		} // Ensure something for today

		totalDur := 0.0
		for j := 0; j < count; j++ {
			// Random duration 120-600 seconds
			dur := 120.0 + float64(time.Now().UnixNano()%480)
			totalDur += dur
		}

		s.stats.DailyStats[date] = &DailyStat{
			Date:          date,
			VideoCount:    count,
			TotalDuration: totalDur,
		}
		s.stats.TotalVideos += count
		s.stats.TotalDuration += totalDur
	}

	s.save()
}

// Helper to sort daily stats (internal use)
func sortDaily(data []*DailyStat) {
	// Simple bubble sort or similar if we don't want to import sort package more than needed
	for i := 0; i < len(data); i++ {
		for j := i + 1; j < len(data); j++ {
			if data[i].Date > data[j].Date {
				data[i], data[j] = data[j], data[i]
			}
		}
	}
}
