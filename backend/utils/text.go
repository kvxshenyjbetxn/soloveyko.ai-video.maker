package utils

import (
	"strings"
	"unicode"
)

// SplitTextByChunks нарізає текст на частини, намагаючись зберігати цілісність речень та абзаців
func SplitTextByChunks(text string, limit int) []string {
	if len([]rune(text)) <= limit {
		return []string{text}
	}

	var chunks []string
	runes := []rune(text)

	for len(runes) > 0 {
		if len(runes) <= limit {
			chunks = append(chunks, string(runes))
			break
		}

		// Шукаємо найкраще місце для розриву в межах ліміту
		splitIdx := limit

		// 1. Пробуємо знайти кінець абзацу
		idx := LastIndexAny(runes[:limit], "\n\r")
		if idx != -1 && idx > limit/2 {
			splitIdx = idx + 1
		} else {
			// 2. Пробуємо знайти кінець речення
			idx = LastIndexAny(runes[:limit], ".!?")
			if idx != -1 && idx > limit/2 {
				splitIdx = idx + 1
			} else {
				// 3. Пробуємо знайти пробіл
				idx = LastIndexAny(runes[:limit], " \t")
				if idx != -1 && idx > limit/2 {
					splitIdx = idx + 1
				}
				// Якщо нічого не знайшли, просто ріжемо по ліміту
			}
		}

		chunks = append(chunks, strings.TrimSpace(string(runes[:splitIdx])))
		runes = runes[splitIdx:]

		// Пропускаємо початкові пробіли для наступного чанку
		for len(runes) > 0 && unicode.IsSpace(runes[0]) {
			runes = runes[1:]
		}
	}

	return chunks
}

// LastIndexAny повертає останній індекс будь-якого символу з chars у runes
func LastIndexAny(runes []rune, chars string) int {
	charRunes := []rune(chars)
	for i := len(runes) - 1; i >= 0; i-- {
		for _, c := range charRunes {
			if runes[i] == c {
				return i
			}
		}
	}
	return -1
}
