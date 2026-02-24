package utils

import (
	"net/url"
	"strings"
	"time"
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

// UrlEncode encodes a string for use in a URL path
func UrlEncode(s string) string {
	return url.PathEscape(s)
}

// MaskKey masks an API key, leaving only the first and last characters visible
func MaskKey(key string) string {
	if len(key) <= 8 {
		return "****"
	}
	return key[:4] + "...." + key[len(key)-4:]
}

// LevenshteinDistance calculates the minimum number of single-character edits required
// to change one word into the other.
func LevenshteinDistance(s, t string) int {
	s1 := []rune(s)
	t1 := []rune(t)
	n := len(s1)
	m := len(t1)

	if n == 0 {
		return m
	}
	if m == 0 {
		return n
	}

	// Create a 2D slice to store distances
	d := make([][]int, n+1)
	for i := range d {
		d[i] = make([]int, m+1)
		d[i][0] = i
	}
	for j := 0; j <= m; j++ {
		d[0][j] = j
	}

	for i := 1; i <= n; i++ {
		for j := 1; j <= m; j++ {
			cost := 1
			if s1[i-1] == t1[j-1] {
				cost = 0
			}

			// Delete, Insert, Substitute
			d[i][j] = min(
				d[i-1][j]+1,
				min(d[i][j-1]+1, d[i-1][j-1]+cost),
			)
		}
	}
	return d[n][m]
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// IsWordSimilar checks if two words are phonetically/visually similar based on Levenshtein distance.
// threshold: maximum allowed edits relative to word length (e.g. 0.3 means 30% of length).
func IsWordSimilar(s1, s2 string, threshold float64) bool {
	if s1 == s2 {
		return true
	}
	dist := LevenshteinDistance(s1, s2)
	maxLen := len([]rune(s1))
	if len([]rune(s2)) > maxLen {
		maxLen = len([]rune(s2))
	}
	if maxLen == 0 {
		return true
	}
	return float64(dist)/float64(maxLen) <= threshold
}

// RandomString generates a random string of a given length
func RandomString(n int) string {
	var letters = []rune("abcdefghijklmnopqrstuvwxyz0123456789")
	b := make([]rune, n)
	for i := range b {
		b[i] = letters[time.Now().UnixNano()%int64(len(letters))]
		time.Sleep(1 * time.Nanosecond) // Slight delay to ensure different nano values
	}
	return string(b)
}
