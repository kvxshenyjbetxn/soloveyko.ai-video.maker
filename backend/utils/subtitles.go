package utils

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

type subtitleBlock struct {
	start string
	end   string
	text  []string
}

// SrtToAss converts SRT subtitle content to ASS format using provided settings
func SrtToAss(srtContent string, settings *PipelineSettings) (string, error) {
	// Robust SRT parsing
	lines := strings.Split(strings.ReplaceAll(srtContent, "\r\n", "\n"), "\n")

	var blocks []subtitleBlock
	currentIdx := -1

	timeRegex := regexp.MustCompile(`(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})`)

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		match := timeRegex.FindStringSubmatch(line)
		if len(match) > 0 {
			blocks = append(blocks, subtitleBlock{
				start: match[1],
				end:   match[2],
				text:  []string{},
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
			currentIdx = -1
			continue
		}

		if currentIdx != -1 {
			blocks[currentIdx].text = append(blocks[currentIdx].text, line)
		}
	}

	if len(blocks) == 0 {
		return "", fmt.Errorf("no valid SRT blocks found")
	}

	// Styles calculation
	fontName := "Arial"
	fontSize := 24
	primaryColor := "&H00FFFFFF" // White
	outlineColor := "&H00000000" // Black
	backColor := "&H00000000"    // Black for shadow/bg
	outlineWidth := 2.0
	shadowWidth := 1.0
	alignment := 2 // Bottom center
	marginV := 80
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
	sb.WriteString("[Script Info]\nTitle: Soloveyko AI\nScriptType: v4.00+\nWrapStyle: 2\nScaledBorderAndShadow: yes\nPlayResX: 1920\nPlayResY: 1080\n\n")
	sb.WriteString("[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n")

	// Spacing (Kerning)
	spacing := 0.0
	if settings != nil {
		spacing = settings.SubtitleKerning
	}

	sb.WriteString(fmt.Sprintf("Style: Default,%s,%d,%s,&H000000FF,%s,%s,1,0,0,0,100,100,%.1f,0,%d,%.1f,%.1f,%d,60,60,%d,1\n\n",
		fontName, fontSize, primaryColor, outlineColor, backColor, spacing, borderStyle, outlineWidth, shadowWidth, alignment, marginV))

	sb.WriteString("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n")

	for _, b := range blocks {
		start := formatSrtTimeToAss(b.start)
		end := formatSrtTimeToAss(b.end)
		text := cleanSrtText(strings.Join(b.text, " "), settings)

		if text != "" {
			var tags strings.Builder
			tags.WriteString("{")

			// Animations and Positioning
			if settings != nil && settings.SubtitleAnimation == "slide-up" {
				resY := 1080
				yEnd := resY - marginV
				switch alignment {
				case 8: // Top
					yEnd = marginV
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

			sb.WriteString(fmt.Sprintf("Dialogue: 0,%s,%s,Default,,0,0,0,,%s%s\n", start, end, tags.String(), text))
		}
	}

	return sb.String(), nil
}

// splitBlockRecursive, srtTimeToSeconds, secondsToSrtTime, hexToAssColor, formatSrtTimeToAss, cleanSrtText stay mostly the same but cleanSrtText gets Uppercase

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
	if len(result.Words) == 0 {
		err := json.Unmarshal([]byte(jsonContent), &result)
		if err != nil || len(result.Words) == 0 {
			return "", fmt.Errorf("unknown or empty JSON format for subtitles")
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

				highlightColor := "&H0000D7FF"
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
					textBuilder.WriteString(fmt.Sprintf("{\\c%s&\\t(%d,%d,\\c%s&)%s}", primaryColor, relStartMs, relEndMs, highlightColor, scaleTag))
				} else {
					textBuilder.WriteString(fmt.Sprintf("{\\c%s&\\t(%d,%d,\\c%s&)\\t(%d,%d,\\c%s&)%s}",
						primaryColor, relStartMs, relStartMs+1, highlightColor, relEndMs, relEndMs+1, primaryColor, scaleTag))
				}
				textBuilder.WriteString(cleanSrtText(w.Word, settings))
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
	parts := strings.Split(srtTime, ",")
	if len(parts) != 2 {
		return "0:00:00.00"
	}
	hms := parts[0]
	ms := parts[1]
	cs := ms[:2]
	hmsParts := strings.Split(hms, ":")
	if len(hmsParts) == 3 {
		h := strings.TrimLeft(hmsParts[0], "0")
		if h == "" {
			h = "0"
		}
		return fmt.Sprintf("%s:%s:%s.%s", h, hmsParts[1], hmsParts[2], cs)
	}
	return fmt.Sprintf("%s.%s", hms, cs)
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
}
