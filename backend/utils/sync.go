package utils

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

type ImageTiming struct {
	Index    int     `json:"index"`
	Start    float64 `json:"start"`
	End      float64 `json:"end"`
	Duration float64 `json:"duration"`
}

type SrtBlock struct {
	Index int
	Start float64
	End   float64
	Text  string
}

func parseSrtTime(timeStr string) float64 {
	// Format: 00:00:01,000
	var h, m, s, ms int
	fmt.Sscanf(strings.Replace(timeStr, ",", ".", 1), "%d:%d:%d.%d", &h, &m, &s, &ms)
	return float64(h)*3600 + float64(m)*60 + float64(s) + float64(ms)/1000.0
}

func ParseSrt(content string) []SrtBlock {
	re := regexp.MustCompile(`(?m)^(\d+)\s*\r?\n(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})\r?\n([\s\S]*?)(?:\r?\n\r?\n|$)`)
	matches := re.FindAllStringSubmatch(content, -1)
	var blocks []SrtBlock
	for _, m := range matches {
		idx := 0
		fmt.Sscanf(m[1], "%d", &idx)
		blocks = append(blocks, SrtBlock{
			Index: idx,
			Start: parseSrtTime(m[2]),
			End:   parseSrtTime(m[3]),
			Text:  strings.TrimSpace(m[4]),
		})
	}
	return blocks
}

func normalizeText(t string) string {
	t = strings.ToLower(t)
	reg := regexp.MustCompile(`[^\w\s]`)
	t = reg.ReplaceAllString(t, "")
	return strings.Join(strings.Fields(t), " ")
}

func GetImageTimings(finalDir string, audioDur float64, totalImages int) ([]ImageTiming, error) {
	segmentsPath := filepath.Join(finalDir, "segments.json")
	srtPath := filepath.Join(finalDir, "subtitle.srt")

	// Fallback logic
	defaultTimings := func() []ImageTiming {
		var timings []ImageTiming
		if totalImages <= 0 {
			return timings
		}
		clipDur := audioDur / float64(totalImages)
		for i := 0; i < totalImages; i++ {
			timings = append(timings, ImageTiming{
				Index:    i,
				Start:    float64(i) * clipDur,
				End:      float64(i+1) * clipDur,
				Duration: clipDur,
			})
		}
		return timings
	}

	segmentsData, err := os.ReadFile(segmentsPath)
	if err != nil {
		return defaultTimings(), nil
	}

	var segments []string
	if err := json.Unmarshal(segmentsData, &segments); err != nil {
		return defaultTimings(), nil
	}

	srtData, err := os.ReadFile(srtPath)
	if err != nil {
		return defaultTimings(), nil
	}

	blocks := ParseSrt(string(srtData))
	if len(blocks) == 0 {
		return defaultTimings(), nil
	}

	var segmentTimings []ImageTiming
	lastBlockIdx := 0

	for i, seg := range segments {
		normSeg := normalizeText(seg)
		if normSeg == "" {
			continue
		}

		start := -1.0
		end := -1.0

		// Find start block
		for j := lastBlockIdx; j < len(blocks); j++ {
			normBlock := normalizeText(blocks[j].Text)
			// Check if segment starts in this block or block contains first few words
			words := strings.Fields(normSeg)
			if len(words) > 0 {
				firstWord := words[0]
				if strings.Contains(normBlock, firstWord) {
					start = blocks[j].Start
					lastBlockIdx = j
					break
				}
			}
		}

		if start == -1.0 {
			if len(segmentTimings) > 0 {
				start = segmentTimings[len(segmentTimings)-1].End
			} else {
				start = 0
			}
		}

		// Find end block
		// We look ahead to see where the segment ends
		for j := lastBlockIdx; j < len(blocks); j++ {
			normBlock := normalizeText(blocks[j].Text)
			words := strings.Fields(normSeg)
			if len(words) > 0 {
				lastWord := words[len(words)-1]
				if strings.Contains(normBlock, lastWord) {
					end = blocks[j].End
					lastBlockIdx = j
					// But we should check if its followed by other words of the segment in subsequent blocks?
					// For simplicity, we'll take the first one we find.
				}
			}
		}

		if end <= start {
			// If we didn't find end or its invalid, try to estimate from next segment start or end of audio
			if i < len(segments)-1 {
				// We'll fix it in the next pass or just use a default gap
				end = start + 5.0
			} else {
				end = audioDur
			}
		}

		segmentTimings = append(segmentTimings, ImageTiming{
			Index:    i,
			Start:    start,
			End:      end,
			Duration: end - start,
		})
	}

	// Refine timings to ensure no overlaps and no gaps
	for i := 0; i < len(segmentTimings); i++ {
		if i > 0 {
			if segmentTimings[i].Start < segmentTimings[i-1].End {
				segmentTimings[i].Start = segmentTimings[i-1].End
			}
			if segmentTimings[i].Start > segmentTimings[i-1].End {
				// Fill gap by extending previous
				segmentTimings[i-1].End = segmentTimings[i].Start
				segmentTimings[i-1].Duration = segmentTimings[i-1].End - segmentTimings[i-1].Start
			}
		}
		if segmentTimings[i].End <= segmentTimings[i].Start {
			segmentTimings[i].End = segmentTimings[i].Start + 0.1
		}
		segmentTimings[i].Duration = segmentTimings[i].End - segmentTimings[i].Start
	}

	// Ensure last one ends at audioDur
	if len(segmentTimings) > 0 {
		segmentTimings[len(segmentTimings)-1].End = audioDur
		segmentTimings[len(segmentTimings)-1].Duration = segmentTimings[len(segmentTimings)-1].End - segmentTimings[len(segmentTimings)-1].Start
	}

	// Now distribute timings if there are more images than segments (image_count > 1)
	// Or if some segments were grouped? Wait, segments correspond to prompts.
	// 1 segment = 1 prompt = (image_count) images.

	// Let's check how many images we actually have.
	// If totalImages != len(segments), we need to adapt.
	// Usually totalImages = len(segments) * image_count.

	imageCount := 1
	if len(segments) > 0 {
		imageCount = totalImages / len(segments)
	}
	if imageCount < 1 {
		imageCount = 1
	}

	var finalTimings []ImageTiming
	for _, st := range segmentTimings {
		subClipDur := st.Duration / float64(imageCount)
		for j := 0; j < imageCount; j++ {
			finalTimings = append(finalTimings, ImageTiming{
				Index:    len(finalTimings),
				Start:    st.Start + float64(j)*subClipDur,
				End:      st.Start + float64(j+1)*subClipDur,
				Duration: subClipDur,
			})
		}
	}

	// If we still have a mismatch in count, fill the rest with default equal distribution or stretch last
	if len(finalTimings) < totalImages {
		lastEnd := 0.0
		if len(finalTimings) > 0 {
			lastEnd = finalTimings[len(finalTimings)-1].End
		}
		remaining := totalImages - len(finalTimings)
		gap := (audioDur - lastEnd) / float64(remaining)
		if gap <= 0 {
			gap = 0.1
		}
		for i := 0; i < remaining; i++ {
			finalTimings = append(finalTimings, ImageTiming{
				Index:    len(finalTimings),
				Start:    lastEnd + float64(i)*gap,
				End:      lastEnd + float64(i+1)*gap,
				Duration: gap,
			})
		}
	} else if len(finalTimings) > totalImages {
		finalTimings = finalTimings[:totalImages]
	}

	return finalTimings, nil
}
