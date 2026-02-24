package utils

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"
	"unicode"
)

type ImageTiming struct {
	Index      int     `json:"index"`
	Start      float64 `json:"start"`
	End        float64 `json:"end"`
	Duration   float64 `json:"duration"`
	Confidence float64 `json:"confidence"`
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
	lines := strings.Split(strings.ReplaceAll(content, "\r\n", "\n"), "\n")
	var blocks []SrtBlock
	currentIdx := -1
	timeRegex := regexp.MustCompile(`(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})`)
	blockIdx := 0

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		match := timeRegex.FindStringSubmatch(line)
		if len(match) > 0 {
			blockIdx++
			blocks = append(blocks, SrtBlock{
				Index: blockIdx,
				Start: parseSrtTime(match[1]),
				End:   parseSrtTime(match[2]),
				Text:  "",
			})
			currentIdx = len(blocks) - 1
			continue
		}

		// Check if line is just a number (index)
		isNumeric := true
		for _, r := range line {
			if !unicode.IsDigit(r) {
				isNumeric = false
				break
			}
		}

		if isNumeric {
			currentIdx = -1 // Close current block, as numbers mark the start of a new SRT block
			continue
		}

		if currentIdx != -1 {
			if blocks[currentIdx].Text != "" {
				blocks[currentIdx].Text += " "
			}
			blocks[currentIdx].Text += line
		}
	}
	return blocks
}

// normalizeTextWithMapping returns normalized text and a map of normalized rune index to original rune index
func normalizeTextWithMapping(text string) (string, []int) {
	var normalizedRunes []rune
	var mapping []int

	// Punctuation to remove
	punctuation := `!"#$%&'()*+,./:;<=>?@[\\]^_{|}~。！？、，；：""''【】（）…·؟،؛।॥「」『』〈〉《》〔〕`
	puncSet := make(map[rune]bool)
	for _, r := range punctuation {
		puncSet[r] = true
	}

	text = strings.ReplaceAll(text, "ё", "е")
	text = strings.ReplaceAll(text, "Ё", "Е")

	runes := []rune(text)
	for i, r := range runes {
		c := r
		if r == '-' || r == '—' || puncSet[r] || unicode.IsSpace(r) {
			c = ' '
		}
		c = unicode.ToLower(c)

		if c == ' ' {
			if len(normalizedRunes) > 0 && normalizedRunes[len(normalizedRunes)-1] == ' ' {
				continue
			}
			if len(normalizedRunes) == 0 {
				continue
			}
		}

		normalizedRunes = append(normalizedRunes, c)
		mapping = append(mapping, i)
	}

	// Trim trailing space
	if len(normalizedRunes) > 0 && normalizedRunes[len(normalizedRunes)-1] == ' ' {
		normalizedRunes = normalizedRunes[:len(normalizedRunes)-1]
		mapping = mapping[:len(mapping)-1]
	}

	return string(normalizedRunes), mapping
}

type charToTime struct {
	CharStart int // Rune index
	CharEnd   int // Rune index
	TimeStart float64
	TimeEnd   float64
}

func buildTextStream(blocks []SrtBlock) (string, []charToTime) {
	var streamRunes []rune
	var timeMap []charToTime
	currentChar := 0

	for _, b := range blocks {
		text := strings.TrimSpace(b.Text)
		if text == "" {
			continue
		}

		if len(streamRunes) > 0 {
			streamRunes = append(streamRunes, ' ')
			currentChar++
		}

		bRunes := []rune(text)
		startChar := currentChar
		streamRunes = append(streamRunes, bRunes...)
		currentChar += len(bRunes)
		endChar := currentChar

		timeMap = append(timeMap, charToTime{
			CharStart: startChar,
			CharEnd:   endChar,
			TimeStart: b.Start,
			TimeEnd:   b.End,
		})
	}
	return string(streamRunes), timeMap
}

func charToTimeAt(pos int, timeMap []charToTime, blocks []SrtBlock) float64 {
	if len(timeMap) == 0 {
		return 0
	}
	for _, entry := range timeMap {
		if pos >= entry.CharStart && pos < entry.CharEnd {
			segmentLen := entry.CharEnd - entry.CharStart
			segmentDur := entry.TimeEnd - entry.TimeStart
			if segmentLen > 0 {
				ratio := float64(pos-entry.CharStart) / float64(segmentLen)
				return entry.TimeStart + ratio*segmentDur
			}
			return entry.TimeStart
		}
	}
	if pos >= timeMap[len(timeMap)-1].CharEnd {
		return timeMap[len(timeMap)-1].TimeEnd
	}
	return timeMap[0].TimeStart
}

func findSegmentInStream(segment string, stream string, startFrom int) (int, int, float64) {
	if segment == "" {
		return -1, -1, 0
	}

	streamRunes := []rune(stream)
	if startFrom >= len(streamRunes) {
		return -1, -1, 0
	}

	targetWords := strings.Fields(segment)
	if len(targetWords) == 0 {
		return -1, -1, 0
	}

	type wordPos struct {
		text  string
		start int
		end   int
	}
	var streamWords []wordPos

	currentWord := strings.Builder{}
	wordStart := -1

	for i := startFrom; i < len(streamRunes); i++ {
		r := streamRunes[i]
		if !unicode.IsSpace(r) {
			if wordStart == -1 {
				wordStart = i
			}
			currentWord.WriteRune(r)
		} else {
			if wordStart != -1 {
				streamWords = append(streamWords, wordPos{
					text:  currentWord.String(),
					start: wordStart,
					end:   i,
				})
				currentWord.Reset()
				wordStart = -1
			}
		}
	}
	if wordStart != -1 {
		streamWords = append(streamWords, wordPos{
			text:  currentWord.String(),
			start: wordStart,
			end:   len(streamRunes),
		})
	}

	if len(streamWords) < len(targetWords) {
		idx := strings.Index(string(streamRunes[startFrom:]), segment)
		if idx != -1 {
			return startFrom + idx, startFrom + idx + len([]rune(segment)), 1.0
		}
		return -1, -1, 0
	}

	threshold := 0.60
	if len(targetWords) <= 2 {
		threshold = 1.0
	}

	bestMatchStart := -1
	bestMatchEnd := -1
	maxConfidence := 0.0

	for i := 0; i <= len(streamWords)-len(targetWords); i++ {
		matchCount := 0
		lastWordIdx := -1
		currentIdx := i

		for _, tw := range targetWords {
			lookahead := 6
			limit := currentIdx + lookahead
			if limit > len(streamWords) {
				limit = len(streamWords)
			}

			foundWord := false
			for j := currentIdx; j < limit; j++ {
				if IsWordSimilar(streamWords[j].text, tw, 0.4) {
					matchCount++
					lastWordIdx = j
					currentIdx = j + 1
					foundWord = true
					break
				}
			}
			if currentIdx >= len(streamWords) && !foundWord {
				break
			}
		}

		confidence := float64(matchCount) / float64(len(targetWords))
		if confidence >= threshold && confidence > maxConfidence {
			maxConfidence = confidence
			bestMatchStart = streamWords[i].start
			if lastWordIdx != -1 {
				bestMatchEnd = streamWords[lastWordIdx].end
			} else {
				bestMatchEnd = streamWords[i].end
			}
			if confidence >= 0.9 {
				break
			}
		}
	}

	if maxConfidence >= threshold {
		return bestMatchStart, bestMatchEnd, maxConfidence
	}

	return -1, -1, 0
}

func GetImageTimings(finalDir string, audioDur float64, totalImages int, visualFiles []string, taskLabel string) ([]ImageTiming, error) {
	segmentsPath := filepath.Join(finalDir, "segments.json")
	srtPath := filepath.Join(finalDir, "subtitle.srt")

	// Fallback function
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
		return defaultTimings(), fmt.Errorf("segments.json not found: %w", err)
	}
	var segments []string
	if err := json.Unmarshal(segmentsData, &segments); err != nil {
		return defaultTimings(), fmt.Errorf("failed to parse segments.json: %w", err)
	}
	if len(segments) == 0 {
		return defaultTimings(), fmt.Errorf("segments.json is empty")
	}

	srtData, err := os.ReadFile(srtPath)
	if err != nil {
		return defaultTimings(), fmt.Errorf("subtitle.srt not found: %w", err)
	}
	blocks := ParseSrt(string(srtData))
	if len(blocks) == 0 {
		return defaultTimings(), fmt.Errorf("no subtitle blocks found in SRT")
	}

	stream, timeMap := buildTextStream(blocks)
	streamNorm, streamMapping := normalizeTextWithMapping(stream)

	matches := make([]*ImageTiming, len(segments))
	lastSearchStart := 0 // Rune index
	anchorThreshold := 0.65

	for i, segment := range segments {
		segNorm, _ := normalizeTextWithMapping(segment)
		if segNorm == "" {
			continue
		}

		startChar, endChar, confidence := findSegmentInStream(segNorm, streamNorm, lastSearchStart)
		if startChar != -1 && confidence >= anchorThreshold {
			// Map back (indices are rune indices)
			if startChar >= len(streamMapping) || endChar-1 >= len(streamMapping) {
				continue
			}
			origStart := streamMapping[startChar]
			origEnd := streamMapping[endChar-1] + 1

			startTime := charToTimeAt(origStart, timeMap, blocks)
			endTime := charToTimeAt(origEnd, timeMap, blocks)

			if endTime <= startTime {
				endTime = startTime + 0.5
			}

			matches[i] = &ImageTiming{
				Index:      i,
				Start:      startTime,
				End:        endTime,
				Duration:   endTime - startTime,
				Confidence: confidence,
			}
			lastSearchStart = endChar
		}
	}

	// Interpolation
	var finalTimings []ImageTiming
	prevValidEnd := 0.0

	i := 0
	for i < len(segments) {
		if matches[i] != nil {
			current := *matches[i]
			if current.Start < prevValidEnd {
				current.Start = prevValidEnd
				if current.End <= current.Start {
					current.End = current.Start + 0.5
				}
				current.Duration = current.End - current.Start
			}
			finalTimings = append(finalTimings, current)
			prevValidEnd = current.End
			i++
		} else {
			// Gap
			gapStartIdx := i
			gapEndIdx := i
			for gapEndIdx < len(segments) && matches[gapEndIdx] == nil {
				gapEndIdx++
			}

			nextValidStart := audioDur
			if gapEndIdx < len(segments) {
				nextValidStart = matches[gapEndIdx].Start
			}

			timeBudget := math.Max(0, nextValidStart-prevValidEnd)
			numMissing := gapEndIdx - gapStartIdx

			// Distribute budget
			var gapTotalTextLen int
			for k := gapStartIdx; k < gapEndIdx; k++ {
				n, _ := normalizeTextWithMapping(segments[k])
				gapTotalTextLen += len(n)
			}
			if gapTotalTextLen == 0 {
				gapTotalTextLen = 1
			}

			currentCursor := prevValidEnd
			for k := gapStartIdx; k < gapEndIdx; k++ {
				n, _ := normalizeTextWithMapping(segments[k])
				weight := float64(len(n)) / float64(gapTotalTextLen)
				if len(n) == 0 {
					weight = 1.0 / float64(numMissing)
				}
				dur := timeBudget * weight
				finalTimings = append(finalTimings, ImageTiming{
					Index:    k,
					Start:    currentCursor,
					End:      currentCursor + dur,
					Duration: dur,
				})
				currentCursor += dur
			}
			prevValidEnd = currentCursor
			i = gapEndIdx
		}
	}

	// Refine to ensure exact match with audioDur
	if len(finalTimings) > 0 {
		finalTimings[len(finalTimings)-1].End = audioDur
		finalTimings[len(finalTimings)-1].Duration = finalTimings[len(finalTimings)-1].End - finalTimings[len(finalTimings)-1].Start
	}

	// Handle multiple images per segment (imageCount > 1)
	imageCount := 1
	if len(segments) > 0 {
		imageCount = totalImages / len(segments)
	}
	if imageCount < 1 {
		imageCount = 1
	}

	var results []ImageTiming
	for _, st := range finalTimings {
		subDur := st.Duration / float64(imageCount)
		for j := 0; j < imageCount; j++ {
			results = append(results, ImageTiming{
				Index:    len(results),
				Start:    st.Start + float64(j)*subDur,
				End:      st.Start + float64(j+1)*subDur,
				Duration: subDur,
			})
		}
	}

	// Final cap/pad
	if len(results) < totalImages {
		lastEnd := 0.0
		if len(results) > 0 {
			lastEnd = results[len(results)-1].End
		}
		rem := totalImages - len(results)
		dur := (audioDur - lastEnd) / float64(rem)
		if dur < 0.1 {
			dur = 0.1
		}
		for k := 0; k < rem; k++ {
			results = append(results, ImageTiming{
				Index:    len(results),
				Start:    lastEnd + float64(k)*dur,
				End:      lastEnd + float64(k+1)*dur,
				Duration: dur,
			})
		}
	} else if len(results) > totalImages {
		results = results[:totalImages]
	}

	// Generate human-readable sync debug report
	var syncLog strings.Builder
	formatSyncTime := func(seconds float64) string {
		m := int(seconds / 60)
		s := int(math.Floor(seconds)) % 60
		cs := int(math.Round((seconds - math.Floor(seconds)) * 100))
		if cs == 100 {
			cs = 0
			s++
			if s == 60 {
				s = 0
				m++
			}
		}
		return fmt.Sprintf("%02d:%02d.%02d", m, s, cs)
	}

	totalConf := 0.0
	confCount := 0
	for _, m := range matches {
		if m != nil {
			totalConf += m.Confidence
			confCount++
		}
	}
	avgConf := 0
	if confCount > 0 {
		avgConf = int((totalConf / float64(confCount)) * 100)
	}

	syncLog.WriteString("====================================================================================================\n")
	syncLog.WriteString("SYNCHRONIZATION DEBUG REPORT\n")
	syncLog.WriteString(fmt.Sprintf("Generated: %s\n", time.Now().Format("2006-01-02 15:04:05")))
	syncLog.WriteString(fmt.Sprintf("Task: %s\n", taskLabel))
	syncLog.WriteString("====================================================================================================\n\n")

	syncLog.WriteString("SUMMARY\n")
	syncLog.WriteString("--------------------------------------------------\n")
	syncLog.WriteString(fmt.Sprintf("Total Segments: %d\n", len(segments)))
	syncLog.WriteString(fmt.Sprintf("Final Visuals:  %d\n", totalImages))
	syncLog.WriteString(fmt.Sprintf("Total Duration: %s (%.2fs)\n", formatSyncTime(audioDur), audioDur))
	syncLog.WriteString(fmt.Sprintf("Avg Confidence: %d%%\n\n", avgConf))

	syncLog.WriteString("DETAILED SYNCHRONIZATION TABLE\n")
	syncLog.WriteString("====================================================================================================\n")
	syncLog.WriteString(fmt.Sprintf("%-5s%-21s%-21s%-21s%-9s%-s\n", "#", "Image", "Display Time", "Subtitle Match", "Conf", "Text Segment"))
	syncLog.WriteString("----------------------------------------------------------------------------------------------------\n")

	for i, r := range results {
		// Find which segment this image belongs to
		segIdx := i / imageCount
		if segIdx >= len(segments) {
			segIdx = len(segments) - 1
		}

		imgName := "n/a"
		if i < len(visualFiles) {
			imgName = filepath.Base(visualFiles[i])
		}

		displayTime := fmt.Sprintf("%s - %s", formatSyncTime(r.Start), formatSyncTime(r.End))

		subMatch := "EST"
		confStr := "EST"
		if segIdx < len(matches) && matches[segIdx] != nil {
			m := matches[segIdx]
			subMatch = fmt.Sprintf("%s - %s", formatSyncTime(m.Start), formatSyncTime(m.End))
			confStr = fmt.Sprintf("%d%%", int(m.Confidence*100))
		}

		text := strings.ReplaceAll(segments[segIdx], "\n", " ")

		syncLog.WriteString(fmt.Sprintf("%-5d%-21s%-21s%-21s%-9s%-s\n",
			i+1, imgName, displayTime, subMatch, confStr, text))
	}

	debugPath := filepath.Join(finalDir, "sync_debug.txt")
	_ = os.WriteFile(debugPath, []byte(syncLog.String()), 0644)

	return results, nil
}
