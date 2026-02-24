package api

import (
	"encoding/csv"
	"fmt"
	"io"
	"net/http"
	"regexp"
	"strings"
	"time"
)

type GoogleParserRow struct {
	Index   int      `json:"index"`
	Title   string   `json:"title"`
	Columns []string `json:"columns"`
	DocLink string   `json:"docLink"`
	Content string   `json:"content"`
}

type GoogleParserService struct {
	client *http.Client
}

func NewGoogleParserService() *GoogleParserService {
	return &GoogleParserService{
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// ExtractID витягує ID документа та ID конкретної вкладки (gid або tab)
func (s *GoogleParserService) ExtractID(url string) (id string, kind string, subId string) {
	url = strings.TrimSpace(url)
	url = strings.Trim(url, "\"")
	url = strings.Trim(url, "'")

	kind = "doc"
	if strings.Contains(url, "spreadsheets") {
		kind = "sheet"
	}

	// 1. Витягуємо основний ID документа
	dMatch := regexp.MustCompile(`/d/([a-zA-Z0-9-_]+)`).FindStringSubmatch(url)
	if len(dMatch) > 1 {
		id = dMatch[1]
	} else if strings.Contains(url, "/d/e/") {
		eMatch := regexp.MustCompile(`/d/e/([a-zA-Z0-9-_]+)`).FindStringSubmatch(url)
		if len(eMatch) > 1 {
			id = eMatch[1]
		}
	}

	if id == "" {
		return "", "", ""
	}

	// 2. Витягуємо суб-ID (вкладку/лист)
	if kind == "sheet" {
		gidMatch := regexp.MustCompile(`gid=([0-9]+)`).FindStringSubmatch(url)
		if len(gidMatch) > 1 {
			subId = gidMatch[1]
		} else {
			subId = "0"
		}
	} else {
		// Для Docs вкладка може бути в параметрі tab або в хеші #tab
		tabMatch := regexp.MustCompile(`tab=([a-zA-Z0-9.\-_]+)`).FindStringSubmatch(url)
		if len(tabMatch) > 1 {
			subId = tabMatch[1]
		} else {
			// Якщо вкладку не вказано, за замовчуванням це t.0 (перша вкладка)
			subId = "t.0"
		}
	}

	return id, kind, subId
}

// FetchSheet завантажує таблицю у форматі CSV (конкретну вкладку)
func (s *GoogleParserService) FetchSheet(url string) ([][]string, error) {
	id, kind, gid := s.ExtractID(url)
	if id == "" || kind != "sheet" {
		return nil, fmt.Errorf("invalid google sheet url")
	}

	exportUrl := fmt.Sprintf("https://docs.google.com/spreadsheets/d/%s/export?format=csv&gid=%s", id, gid)

	resp, err := s.client.Get(exportUrl)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusUnauthorized || resp.StatusCode == http.StatusForbidden {
		return nil, fmt.Errorf("Access Denied: Is the sheet shared?")
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP %s", resp.Status)
	}

	reader := csv.NewReader(resp.Body)
	reader.FieldsPerRecord = -1
	return reader.ReadAll()
}

// FetchDoc завантажує конкретну вкладку документа як текст
func (s *GoogleParserService) FetchDoc(url string) (string, error) {
	id, kind, subId := s.ExtractID(url)
	if id == "" {
		return "", fmt.Errorf("invalid google url")
	}

	var exportUrl string
	if kind == "sheet" {
		exportUrl = fmt.Sprintf("https://docs.google.com/spreadsheets/d/%s/export?format=csv&gid=%s", id, subId)
	} else {
		// ВИКОРИСТОВУЄМО ТЕХНІКУ ІЗОЛЯЦІЇ ВКЛАДКИ
		// Google Docs підтримує параметр &tab= для експорту конкретної вкладки
		exportUrl = fmt.Sprintf("https://docs.google.com/document/d/%s/export?format=txt&tab=%s", id, subId)
	}

	resp, err := s.client.Get(exportUrl)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusUnauthorized || resp.StatusCode == http.StatusForbidden {
		return "", fmt.Errorf("ACCESS_DENIED: Share the doc")
	}

	if resp.StatusCode != http.StatusOK {
		// Якщо з &tab= помилка, пробуємо без нього як фолбек
		if kind == "doc" {
			exportUrl = fmt.Sprintf("https://docs.google.com/document/d/%s/export?format=txt", id)
			resp2, err2 := s.client.Get(exportUrl)
			if err2 == nil {
				defer resp2.Body.Close()
				if resp2.StatusCode == http.StatusOK {
					body, _ := io.ReadAll(resp2.Body)
					return s.clipToFirstTab(string(body)), nil
				}
			}
		}
		return "", fmt.Errorf("ERROR %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	content := string(body)
	content = strings.TrimPrefix(content, "\ufeff")

	// Якщо ми завантажили документ цілком (без ізоляції вкладки сервером),
	// обрізаємо його вручну до першої вкладки.
	return s.clipToFirstTab(content), nil
}

// clipToFirstTab обрізає текст, залишаючи лише вміст першої вкладки
func (s *GoogleParserService) clipToFirstTab(content string) string {
	// 0. Нормалізуємо переноси рядків відразу для стабільної роботи регулярних виразів
	content = strings.ReplaceAll(content, "\r\n", "\n")
	content = strings.TrimPrefix(content, "\ufeff")

	// 1. Пошук символу Form Feed (\x0c, ^L) - це стандартний роздільник сторінок/вкладок
	if strings.Contains(content, "\x0c") {
		parts := strings.Split(content, "\x0c")
		for _, p := range parts {
			trimmed := strings.TrimSpace(p)
			if trimmed != "" {
				return trimmed
			}
		}
	}

	// 2. АГРЕСИВНА ОБРІЗКА ЗА ПУСТИМИ РЯДКАМИ
	// Якщо ми бачимо 3 або більше переноів рядка підряд (можливо з пробілами),
	// це майже напевно кінець першої вкладки і початок технічного сміття Google.
	reGap := regexp.MustCompile(`\n(\s*\n){2,}`)
	gapLoc := reGap.FindStringIndex(content)
	if gapLoc != nil {
		// Обрізаємо все, що після першого великого пропуску
		content = content[:gapLoc[0]]
	}

	// 3. ДОДАТКОВА ОБРІЗКА ЗА ЗАГОЛОВКАМИ
	// Шукаємо в тексті будь-які згадки нових вкладок/сторінок/розділів (крім першої)
	reNextTab := regexp.MustCompile(`(?mi)\n.*(Вкладка|Tab|Sheet|Page|Сторінка|Раздел|Section)\s*[23456789].*\n`)
	loc := reNextTab.FindStringIndex(content)
	if loc != nil {
		content = content[:loc[0]]
	}

	// 4. ОЧИЩЕННЯ ПОЧАТКУ
	// Видаляємо заголовок самої першої вкладки ("Вкладка 1") з самого верху, якщо він там є
	reFirstTab := regexp.MustCompile(`(?mi)^.*(Вкладка|Tab|Sheet|Page|Сторінка|Раздел|Section)\s*1.*$`)
	content = reFirstTab.ReplaceAllString(content, "")

	return strings.TrimSpace(content)
}

// colLetterToIndex перетворює букву колонки (A, B, C...) в індекс (0, 1, 2...)
func colLetterToIndex(letter string) int {
	letter = strings.ToUpper(letter)
	index := 0
	for i := 0; i < len(letter); i++ {
		if letter[i] < 'A' || letter[i] > 'Z' {
			return -1
		}
		index = index*26 + int(letter[i]-'A'+1)
	}
	return index - 1
}

// ParseWithFilter виконує повний цикл: парсинг таблиці -> фільтрація -> завантаження контенту з посилань
func (s *GoogleParserService) ParseWithFilter(sheetUrl string, filter string) ([]GoogleParserRow, error) {
	rows, err := s.FetchSheet(sheetUrl)
	if err != nil {
		return nil, err
	}

	var results []GoogleParserRow

	for i, row := range rows {
		// Розширене фільтрування: підтримує декілька умов через '&',
		// конкретні колонки (A:значення) та регулярні вирази
		matchAll := true
		if filter != "" {
			parts := strings.Split(filter, "&")
			for _, part := range parts {
				part = strings.TrimSpace(part)
				if part == "" {
					continue
				}

				partMatch := false
				targetCol := -1 // -1 означає пошук по всіх колонках
				searchValue := part

				// Перевіряємо формат "Буква:Значення" (наприклад A:Done)
				colMatch := regexp.MustCompile(`^([a-zA-Z]+):(.*)$`).FindStringSubmatch(part)
				if len(colMatch) > 2 {
					targetCol = colLetterToIndex(colMatch[1])
					searchValue = strings.TrimSpace(colMatch[2])
				}

				// Перевіряємо, чи є searchValue регулярним виразом
				isRegex := strings.ContainsAny(searchValue, "\\[]*+?")
				var re *regexp.Regexp
				if isRegex {
					re, _ = regexp.Compile("(?i)" + searchValue)
				}
				valLower := strings.ToLower(searchValue)

				if targetCol != -1 {
					// Пошук у конкретній колонці
					if targetCol < len(row) {
						cellVal := row[targetCol]
						if isRegex && re != nil {
							if re.MatchString(cellVal) {
								partMatch = true
							}
						} else {
							if strings.Contains(strings.ToLower(cellVal), valLower) {
								partMatch = true
							}
						}
					}
				} else {
					// Пошук по всіх колонках
					for _, col := range row {
						if isRegex && re != nil {
							if re.MatchString(col) {
								partMatch = true
								break
							}
						} else {
							if strings.Contains(strings.ToLower(col), valLower) {
								partMatch = true
								break
							}
						}
					}
				}

				if !partMatch {
					matchAll = false
					break
				}
			}
		}

		if !matchAll {
			continue
		}

		// Шукаємо посилання на Google Docs/Sheets у рядку
		var docLink string

		// Пріоритет: стовпчик F (індекс 5), потім C (індекс 2)
		if len(row) > 5 && strings.Contains(row[5], "docs.google.com") {
			docLink = row[5]
		} else if len(row) > 2 && strings.Contains(row[2], "docs.google.com") {
			docLink = row[2]
		} else {
			// Якщо в пріоритетних пусто — шукаємо в будь-якому іншому
			for _, col := range row {
				if strings.Contains(col, "docs.google.com") {
					docLink = col
					break
				}
			}
		}

		if docLink != "" {
			// Завантажуємо вміст документу
			content, err := s.FetchDoc(docLink)
			if err != nil {
				content = fmt.Sprintf("Error: %v", err)
			}

			// Назва знаходиться у колонці B (індекс 1)
			title := ""
			if len(row) > 1 {
				title = row[1]
			}

			results = append(results, GoogleParserRow{
				Index:   i,
				Title:   title,
				Columns: row,
				DocLink: docLink,
				Content: content,
			})
		}
	}

	return results, nil
}
