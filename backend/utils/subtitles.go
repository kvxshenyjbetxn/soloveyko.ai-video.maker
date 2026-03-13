package utils

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strconv"
	"strings"
)

type subtitleBlock struct {
	start float64
	end   float64
	text  string
}

type AudioSegment struct {
	Start float64 `json:"start"`
	End   float64 `json:"end"`
}

func srtTimeToSeconds(t string) float64 {
	t = strings.ReplaceAll(strings.TrimSpace(t), ",", ".")
	parts := strings.Split(t, ":")
	if len(parts) != 3 {
		return 0
	}
	h, _ := strconv.ParseFloat(parts[0], 64)
	m, _ := strconv.ParseFloat(parts[1], 64)
	s, _ := strconv.ParseFloat(parts[2], 64)
	return h*3600 + m*60 + s
}

func splitSrtRecursive(seg subtitleBlock, maxWords int) []subtitleBlock {
	words := strings.Fields(seg.text)
	if len(words) <= maxWords || maxWords <= 0 {
		return []subtitleBlock{seg}
	}

	part1Words := words[:maxWords]
	part2Words := words[maxWords:]

	totalWords := float64(len(words))
	duration := seg.end - seg.start

	part1Dur := (float64(len(part1Words)) / totalWords) * duration
	splitTime := seg.start + part1Dur

	part1 := subtitleBlock{
		start: seg.start,
		end:   splitTime,
		text:  strings.Join(part1Words, " "),
	}
	part2 := subtitleBlock{
		start: splitTime,
		end:   seg.end,
		text:  strings.Join(part2Words, " "),
	}

	res := []subtitleBlock{part1}
	res = append(res, splitSrtRecursive(part2, maxWords)...)
	return res
}

// SrtToAss converts SRT subtitle content to ASS format using provided settings
func SrtToAss(srtContent string, settings *PipelineSettings) (string, error) {
	// 1. Parsing (matching Python's _parse_srt_content)
	content := strings.ReplaceAll(srtContent, "\r\n", "\n")
	content = strings.TrimSpace(content)
	blocks := regexp.MustCompile(`\n\s*\n`).Split(content, -1)

	var segments []subtitleBlock
	for _, block := range blocks {
		lines := strings.Split(block, "\n")
		var cleanLines []string
		for _, l := range lines {
			if strings.TrimSpace(l) != "" {
				cleanLines = append(cleanLines, strings.TrimSpace(l))
			}
		}
		if len(cleanLines) < 2 {
			continue
		}

		timeLineIdx := -1
		for i := 0; i < len(cleanLines) && i < 2; i++ {
			if strings.Contains(cleanLines[i], "-->") {
				timeLineIdx = i
				break
			}
		}

		if timeLineIdx != -1 {
			timeLine := cleanLines[timeLineIdx]
			times := strings.Split(timeLine, "-->")
			if len(times) == 2 {
				start := srtTimeToSeconds(times[0])
				end := srtTimeToSeconds(times[1])
				text := strings.Join(cleanLines[timeLineIdx+1:], " ")
				segments = append(segments, subtitleBlock{
					start: start,
					end:   end,
					text:  text,
				})
			}
		}
	}

	if len(segments) == 0 {
		return "", fmt.Errorf("no valid SRT blocks found")
	}

	// 2. Splitting (matching Python's _split_long_lines)
	maxWords := 10
	if settings != nil && settings.SubtitleMaxWords > 0 {
		maxWords = settings.SubtitleMaxWords
	}
	var processedSegments []subtitleBlock
	for _, seg := range segments {
		processedSegments = append(processedSegments, splitSrtRecursive(seg, maxWords)...)
	}

	// 3. Writing (Restored full Go formatting)
	fontName := "Arial"
	fontSize := 24
	primaryColor := "&H00FFFFFF" // White
	outlineColor := "&H00000000" // Black
	backColor := "&H00000000"    // Black for shadow/bg
	outlineWidth := 2.0
	shadowWidth := 1.0
	alignment := 2 // Bottom center
	marginVVal := 80
	borderStyle := 1 // 1 = outline, 3 = opaque box
	blur := 0.0

	if settings != nil {
		if settings.SubtitleFont != "" {
			fontName = settings.SubtitleFont
		}
		if settings.SubtitleSize > 0 {
			fontSize = settings.SubtitleSize
		}
		if settings.SubtitleColor != "" {
			primaryColor = hexToAssColor(settings.SubtitleColor)
		}
		if settings.SubtitleOutlineColor != "" {
			outlineColor = hexToAssColor(settings.SubtitleOutlineColor)
		}
		if settings.SubtitleOutlineWidth >= 0 {
			outlineWidth = settings.SubtitleOutlineWidth
		}
		if settings.SubtitleShadowColor != "" {
			backColor = hexToAssColor(settings.SubtitleShadowColor)
		}
		if settings.SubtitleShadowWidth >= 0 {
			shadowWidth = settings.SubtitleShadowWidth
		}
		if settings.SubtitleMarginV > 0 {
			marginVVal = settings.SubtitleMarginV
		}
		if settings.SubtitleBlur >= 0 {
			blur = settings.SubtitleBlur
		}
		switch settings.SubtitlePosition {
		case "top":
			alignment = 8
		case "middle":
			alignment = 5
		default:
			alignment = 2
		}
	}

	var sb strings.Builder
	sb.WriteString("[Script Info]\nTitle: Soloveyko AI\nScriptType: v4.00+\nWrapStyle: 2\nScaledBorderAndShadow: yes\nPlayResX: 1920\nPlayResY: 1080\n\n")
	sb.WriteString("[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n")

	// Spacing (Kerning)
	spacing := 0.0
	if settings != nil {
		spacing = settings.SubtitleKerning
	}

	sb.WriteString(fmt.Sprintf("Style: Default,%s,%d,%s,&H000000FF,%s,%s,1,0,0,0,100,100,%.1f,0,%d,%.1f,%.1f,%d,60,60,%d,1\n\n",
		fontName, fontSize, primaryColor, outlineColor, backColor, spacing, borderStyle, outlineWidth, shadowWidth, alignment, marginVVal))

	sb.WriteString("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n")

	for _, seg := range processedSegments {
		startStr := formatSecondsToAss(seg.start)
		endStr := formatSecondsToAss(seg.end)
		text := cleanSrtText(seg.text, settings)

		if text != "" {
			var tags strings.Builder
			tags.WriteString("{")

			// Animations and Positioning
			if settings != nil && settings.SubtitleAnimation == "slide-up" {
				resY := 1080
				yEnd := resY - marginVVal
				switch alignment {
				case 8: // Top
					yEnd = marginVVal
				case 5: // Middle
					yEnd = resY / 2
				}
				yStart := yEnd + 30
				tags.WriteString(fmt.Sprintf("\\move(960,%d,960,%d,0,300)", yStart, yEnd))
			}

			if settings != nil && settings.SubtitleFadeEnabled {
				tags.WriteString(fmt.Sprintf("\\fad(%d,%d)", settings.SubtitleFadeIn, settings.SubtitleFadeOut))
			}

			if blur > 0 {
				tags.WriteString(fmt.Sprintf("\\blur%.1f", blur))
			}
			tags.WriteString("}")

			sb.WriteString(fmt.Sprintf("Dialogue: 0,%s,%s,Default,,0,0,0,,%s%s\n", startStr, endStr, tags.String(), text))
		}
	}

	return sb.String(), nil
}

func cleanSrtText(text string, settings *PipelineSettings) string {
	reTags := regexp.MustCompile(`<[^>]*>`)
	text = reTags.ReplaceAllString(text, "")
	text = strings.ReplaceAll(text, "\r\n", "\\N")
	text = strings.ReplaceAll(text, "\n", "\\N")
	text = strings.TrimSpace(text)
	if settings != nil && settings.SubtitleUppercase {
		text = strings.ToUpper(text)
	}
	for strings.Contains(text, "  ") {
		text = strings.ReplaceAll(text, "  ", " ")
	}
	return text
}

// JsonToAss converts WhisperX JSON to an ASS subtitle format with optional karaoke tags
func JsonToAss(jsonContent string, settings *PipelineSettings, karaokeEffect bool) (string, error) {
	var result WhisperXResult
	var assemblyRes AssemblyAIResult

	// Try AssemblyAI format first (by checking if 'words' is present and structured like AssemblyAI)
	var raw map[string]interface{}
	_ = json.Unmarshal([]byte(jsonContent), &raw)

	if _, ok := raw["words"]; ok {
		err := json.Unmarshal([]byte(jsonContent), &assemblyRes)
		if err == nil && len(assemblyRes.Words) > 0 {
			// Convert AssemblyAI to WhisperX format
			result.Language = "auto"
			for _, aw := range assemblyRes.Words {
				result.Words = append(result.Words, WhisperXWord{
					Word:  aw.Text,
					Start: float64(aw.Start) / 1000.0, // ms to s
					End:   float64(aw.End) / 1000.0,   // ms to s
					Score: 1.0,
				})
			}
		}
	}

	// If result.Words is still empty, it might be WhisperX format
	// If result.Words is still empty, it might be standard WhisperX format (segments -> words)
	if len(result.Words) == 0 {
		err := json.Unmarshal([]byte(jsonContent), &result)
		if err == nil {
			// Extract words from segments if the root 'words' array is empty
			if len(result.Words) == 0 && len(result.Segments) > 0 {
				for _, seg := range result.Segments {
					result.Words = append(result.Words, seg.Words...)
				}
			}
		}

		if len(result.Words) == 0 {
			return "", fmt.Errorf("unknown or empty JSON format for subtitles (no words found in root or segments)")
		}
	}

	// Styles calculation
	fontName := "Arial"
	fontSize := 24
	primaryColor := "&H00FFFFFF"
	outlineColor := "&H00000000"
	backColor := "&H00000000"
	outlineWidth := 2.0
	shadowWidth := 1.0
	alignment := 2
	marginV := 80
	borderStyle := 1
	blur := 0.0

	if settings != nil {
		if settings.SubtitleFont != "" {
			fontName = settings.SubtitleFont
		}
		if settings.SubtitleSize > 0 {
			fontSize = settings.SubtitleSize
		}
		if settings.SubtitleColor != "" {
			primaryColor = hexToAssColor(settings.SubtitleColor)
		}
		if settings.SubtitleOutlineColor != "" {
			outlineColor = hexToAssColor(settings.SubtitleOutlineColor)
		}
		if settings.SubtitleOutlineWidth >= 0 {
			outlineWidth = settings.SubtitleOutlineWidth
		}
		if settings.SubtitleShadowColor != "" {
			backColor = hexToAssColor(settings.SubtitleShadowColor)
		}
		if settings.SubtitleShadowWidth >= 0 {
			shadowWidth = settings.SubtitleShadowWidth
		}
		if settings.SubtitleMarginV > 0 {
			marginV = settings.SubtitleMarginV
		}
		if settings.SubtitleBlur >= 0 {
			blur = settings.SubtitleBlur
		}
		switch settings.SubtitlePosition {
		case "top":
			alignment = 8
		case "middle":
			alignment = 5
		default:
			alignment = 2
		}
	}

	var sb strings.Builder
	sb.WriteString("[Script Info]\nTitle: Soloveyko AI (WhisperX)\nScriptType: v4.00+\nWrapStyle: 2\nScaledBorderAndShadow: yes\nPlayResX: 1920\nPlayResY: 1080\n\n")
	sb.WriteString("[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n")
	// Spacing (Kerning)
	spacing := 0.0
	if settings != nil {
		spacing = settings.SubtitleKerning
	}

	sb.WriteString(fmt.Sprintf("Style: Default,%s,%d,%s,&H000000FF,%s,%s,1,0,0,0,100,100,%.1f,0,%d,%.1f,%.1f,%d,60,60,%d,1\n\n",
		fontName, fontSize, primaryColor, outlineColor, backColor, spacing, borderStyle, outlineWidth, shadowWidth, alignment, marginV))

	sb.WriteString("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n")

	maxWords := 10
	if settings != nil && settings.SubtitleMaxWords > 0 {
		maxWords = settings.SubtitleMaxWords
	}

	var chunks [][]WhisperXWord
	var currentChunk []WhisperXWord
	for i, w := range result.Words {
		currentChunk = append(currentChunk, w)
		isEnd := len(currentChunk) >= maxWords || strings.ContainsAny(w.Word, ".!?")
		if !isEnd && i+1 < len(result.Words) && result.Words[i+1].Start-w.End > 1.0 && result.Words[i+1].Start != 0 {
			isEnd = true
		}
		if isEnd {
			chunks = append(chunks, currentChunk)
			currentChunk = nil
		}
	}
	if len(currentChunk) > 0 {
		chunks = append(chunks, currentChunk)
	}

	var lastDisplayEnd float64 = 0.0
	for cIdx, chunk := range chunks {
		if len(chunk) == 0 {
			continue
		}
		blockStart := chunk[0].Start
		blockEnd := chunk[len(chunk)-1].End

		// Unaligned fix
		if chunk[0].Start == 0 && chunk[0].End == 0 {
			blockStart = lastDisplayEnd
			blockEnd = blockStart + float64(len(chunk))*0.3
		}

		displayStart := blockStart
		if karaokeEffect {
			displayStart = blockStart - 1.0
			if displayStart < lastDisplayEnd {
				displayStart = lastDisplayEnd
			}
			if displayStart < 0 || cIdx == 0 {
				displayStart = 0
			}
		}

		strStart := formatSecondsToAss(displayStart)
		strEnd := formatSecondsToAss(blockEnd)

		var textBuilder strings.Builder
		textBuilder.WriteString("{")

		// Animations and Positioning
		if settings != nil && settings.SubtitleAnimation == "slide-up" {
			resY := 1080
			yEnd := resY - marginV
			switch alignment {
			case 8:
				yEnd = marginV
			case 5:
				yEnd = resY / 2
			}
			yStart := yEnd + 30
			textBuilder.WriteString(fmt.Sprintf("\\move(960,%d,960,%d,0,300)", yStart, yEnd))
		}

		if settings != nil && settings.SubtitleFadeEnabled {
			textBuilder.WriteString(fmt.Sprintf("\\fad(%d,%d)", settings.SubtitleFadeIn, settings.SubtitleFadeOut))
		}
		if blur > 0 {
			textBuilder.WriteString(fmt.Sprintf("\\blur%.1f", blur))
		}
		textBuilder.WriteString("}")

		currentPos := displayStart
		for i, w := range chunk {
			wStart, wEnd := w.Start, w.End
			if wStart == 0 && wEnd == 0 {
				wStart, wEnd = currentPos, currentPos+0.3
			}
			if wStart < currentPos {
				wStart = currentPos
			}
			if wEnd < wStart {
				wEnd = wStart + 0.1
			}

			if i > 0 {
				textBuilder.WriteString(" ")
			}

			if karaokeEffect {
				relStartMs := int((wStart - displayStart) * 1000)
				relEndMs := int((wEnd - displayStart) * 1000)
				durationCs := int((wEnd - wStart) * 100) // ASS karaoke uses centiseconds

				highlightColor := "&H0000FFFF" // Yellow default
				if settings != nil && settings.SubtitleKaraokeColor != "" {
					highlightColor = hexToAssColor(settings.SubtitleKaraokeColor)
				}

				karaokeMode := "highlight"
				if settings != nil && settings.SubtitleKaraokeMode != "" {
					karaokeMode = settings.SubtitleKaraokeMode
				}

				scale := 1.0
				if settings != nil && settings.SubtitleKaraokeScale > 0 {
					scale = settings.SubtitleKaraokeScale
				}

				scaleTag := ""
				if scale > 1.0 {
					speed := 100
					if settings != nil && settings.SubtitleKaraokeSpeed > 0 {
						speed = settings.SubtitleKaraokeSpeed
					}
					// Scale from 100% to Target% and back
					scaleTag = fmt.Sprintf("\\t(%d,%d,\\fscx%d\\fscy%d\\fsp-1)\\t(%d,%d,\\fscx100\\fscy100\\fsp0)",
						relStartMs, relStartMs+speed, int(scale*100), int(scale*100),
						relEndMs, relEndMs+speed)
				}

				if karaokeMode == "fill" {
					// Use \k for filling effect (standard karaoke)
					// Duration is in centiseconds
					textBuilder.WriteString(fmt.Sprintf("{\\k%d%s}%s", durationCs, scaleTag, cleanSrtText(w.Word, settings)))
				} else {
					// Traditional highlight with \t colors
					textBuilder.WriteString(fmt.Sprintf("{\\c%s&\\t(%d,%d,\\c%s&)\\t(%d,%d,\\c%s&)%s}%s",
						primaryColor, relStartMs, relStartMs+1, highlightColor, relEndMs, relEndMs+1, primaryColor, scaleTag, cleanSrtText(w.Word, settings)))
				}
				
				// Explicit reset after word to prevent scale/spacing from leaking
				if scale > 1.0 {
					textBuilder.WriteString("{\\fscx100\\fscy100\\fsp0}")
				}
				currentPos = wEnd
			} else {
				textBuilder.WriteString(cleanSrtText(w.Word, settings))
			}
		}

		sb.WriteString(fmt.Sprintf("Dialogue: 0,%s,%s,Default,,0,0,0,,%s\n", strStart, strEnd, textBuilder.String()))
		lastDisplayEnd = blockEnd
	}

	return sb.String(), nil
}

func formatSrtTimeToAss(srtTime string) string {
	sec := srtTimeToSeconds(srtTime)
	return formatSecondsToAss(sec)
}

func hexToAssColor(hex string) string {
	hex = strings.TrimPrefix(hex, "#")
	if len(hex) == 6 {
		r, g, b := hex[0:2], hex[2:4], hex[4:6]
		return fmt.Sprintf("&H00%s%s%s", b, g, r)
	} else if len(hex) == 8 {
		r, g, b, a := hex[0:2], hex[2:4], hex[4:6], hex[6:8]
		return fmt.Sprintf("&H%s%s%s%s", a, b, g, r)
	}
	return "&H00FFFFFF"
}

type WhisperXWord struct {
	Word  string  `json:"word"`
	Start float64 `json:"start"`
	End   float64 `json:"end"`
	Score float64 `json:"score"`
}

type AssemblyAIWord struct {
	Text  string `json:"text"`
	Start int    `json:"start"`
	End   int    `json:"end"`
}

type AssemblyAIResult struct {
	Text  string           `json:"text"`
	Words []AssemblyAIWord `json:"words"`
}

func formatSecondsToAss(totalSec float64) string {
	h := int(totalSec / 3600)
	m := int((totalSec - float64(h*3600)) / 60)
	s := int(totalSec - float64(h*3600) - float64(m*60))
	cs := int((totalSec - float64(int(totalSec))) * 100)
	return fmt.Sprintf("%d:%02d:%02d.%02d", h, m, s, cs)
}

type WhisperXResult struct {
	Language string         `json:"language"`
	Audio    string         `json:"audio"`
	Words    []WhisperXWord `json:"words"`
	Segments []struct {
		Words []WhisperXWord `json:"words"`
	} `json:"segments"`
}
func (s WhisperXResult) ToSrt() string {
	var sb strings.Builder
	for i, w := range s.Words {
		sb.WriteString(fmt.Sprintf("%d\n", i+1))
		sb.WriteString(fmt.Sprintf("%s --> %s\n", formatSecondsToSrt(w.Start), formatSecondsToSrt(w.End)))
		sb.WriteString(fmt.Sprintf("%s\n\n", w.Word))
	}
	return sb.String()
}

func formatSecondsToSrt(totalSec float64) string {
	h := int(totalSec / 3600)
	m := int((totalSec - float64(h*3600)) / 60)
	s := int(totalSec - float64(h*3600) - float64(m*60))
	ms := int((totalSec - float64(int(totalSec))) * 1000)
	return fmt.Sprintf("%02d:%02d:%02d,%03d", h, m, s, ms)
}

// TrimSrt filters and offsets SRT content based on provided audio segments
func TrimSrt(srtContent string, segments []AudioSegment) string {
	if len(segments) == 0 {
		return srtContent
	}

	content := strings.ReplaceAll(srtContent, "\r\n", "\n")
	blocks := regexp.MustCompile(`\n\s*\n`).Split(content, -1)

	var originalSegments []subtitleBlock
	for _, block := range blocks {
		lines := strings.Split(strings.TrimSpace(block), "\n")
		if len(lines) < 2 {
			continue
		}
		timeLineIdx := -1
		for i := 0; i < len(lines) && i < 2; i++ {
			if strings.Contains(lines[i], "-->") {
				timeLineIdx = i
				break
			}
		}
		if timeLineIdx != -1 {
			times := strings.Split(lines[timeLineIdx], "-->")
			if len(times) == 2 {
				start := srtTimeToSeconds(times[0])
				end := srtTimeToSeconds(times[1])
				text := strings.Join(lines[timeLineIdx+1:], "\n")
				originalSegments = append(originalSegments, subtitleBlock{start: start, end: end, text: text})
			}
		}
	}

	var trimmedBlocks []subtitleBlock
	currentTimelineOffset := 0.0

	for _, seg := range segments {
		segDur := seg.End - seg.Start
		for _, sub := range originalSegments {
			// Check overlap between subtitle and current segment
			overlapStart := mathMax(sub.start, seg.Start)
			overlapEnd := mathMin(sub.end, seg.End)

			if overlapStart < overlapEnd {
				// This subtitle (or part of it) is in the segment
				trimmedBlocks = append(trimmedBlocks, subtitleBlock{
					start: currentTimelineOffset + (overlapStart - seg.Start),
					end:   currentTimelineOffset + (overlapEnd - seg.Start),
					text:  sub.text,
				})
			}
		}
		currentTimelineOffset += segDur
	}

	var sb strings.Builder
	for i, b := range trimmedBlocks {
		sb.WriteString(fmt.Sprintf("%d\n", i+1))
		sb.WriteString(fmt.Sprintf("%s --> %s\n", formatSecondsToSrt(b.start), formatSecondsToSrt(b.end)))
		sb.WriteString(fmt.Sprintf("%s\n\n", b.text))
	}
	return sb.String()
}

// TrimJsonResult offsets and filters word-level timings based on provided audio segments
func TrimJsonResult(jsonContent string, segments []AudioSegment) (string, error) {
	if len(segments) == 0 {
		return jsonContent, nil
	}

	var result WhisperXResult
	var raw map[string]interface{}
	err := json.Unmarshal([]byte(jsonContent), &raw)
	if err != nil {
		return "", err
	}

	// Basic format support: root words or segments
	if _, ok := raw["words"]; ok {
		_ = json.Unmarshal([]byte(jsonContent), &result)
	} else if _, ok := raw["segments"]; ok {
		_ = json.Unmarshal([]byte(jsonContent), &result)
		if len(result.Words) == 0 {
			for _, s := range result.Segments {
				result.Words = append(result.Words, s.Words...)
			}
		}
	}

	if len(result.Words) == 0 {
		return jsonContent, nil // Nothing to trim
	}

	var trimmedWords []WhisperXWord
	currentTimelineOffset := 0.0

	for _, seg := range segments {
		segDur := seg.End - seg.Start
		for _, w := range result.Words {
			// Check overlap
			overlapStart := mathMax(w.Start, seg.Start)
			overlapEnd := mathMin(w.End, seg.End)

			if overlapStart < overlapEnd {
				// We keep the word if it overlap with the segment
				// We offset it relative to the new timeline
				trimmedWords = append(trimmedWords, WhisperXWord{
					Word:  w.Word,
					Start: currentTimelineOffset + (overlapStart - seg.Start),
					End:   currentTimelineOffset + (overlapEnd - seg.Start),
					Score: w.Score,
				})
			}
		}
		currentTimelineOffset += segDur
	}

	// Update result and return
	result.Words = trimmedWords
	result.Segments = nil // Clear segments to avoid confusion with the flattened trimmed list
	
	newJson, _ := json.Marshal(result)
	return string(newJson), nil
}

func mathMax(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}

func mathMin(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}
