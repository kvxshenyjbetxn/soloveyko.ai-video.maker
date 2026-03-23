package api

import (
	"context"
	"encoding/csv"
	"fmt"
	"io"
	"net/http"
	"os"
	"regexp"
	"sort"
	"strings"
	"sync"
	"time"

	"google.golang.org/api/docs/v1"
	"google.golang.org/api/option"
	"google.golang.org/api/sheets/v4"
)

type GoogleParserRow struct {
	Index   int      `json:"index"`
	Title   string   `json:"title"`
	Columns []string `json:"columns"`
	DocLink string   `json:"docLink"`
	Content string   `json:"content"`
}

type GoogleParserService struct {
	client        *http.Client
	sheetsService *sheets.Service
	docsService   *docs.Service
	mu            sync.Mutex
}

func NewGoogleParserService() *GoogleParserService {
	return &GoogleParserService{
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (s *GoogleParserService) initServices() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.sheetsService != nil && s.docsService != nil {
		return nil
	}

	ctx := context.Background()
	credPath := "credentials.json"
	if _, err := os.Stat(credPath); os.IsNotExist(err) {
		return fmt.Errorf("credentials.json not found")
	}

	if s.sheetsService == nil {
		sheetsSrv, err := sheets.NewService(ctx, option.WithCredentialsFile(credPath))
		if err != nil {
			return fmt.Errorf("failed to init sheets service: %v", err)
		}
		s.sheetsService = sheetsSrv
	}

	if s.docsService == nil {
		docsSrv, err := docs.NewService(ctx, option.WithCredentialsFile(credPath))
		if err != nil {
			return fmt.Errorf("failed to init docs service: %v", err)
		}
		s.docsService = docsSrv
	}

	return nil
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

	if kind == "sheet" {
		gidMatch := regexp.MustCompile(`gid=([0-9]+)`).FindStringSubmatch(url)
		if len(gidMatch) > 1 {
			subId = gidMatch[1]
		} else {
			subId = "0"
		}
	} else {
		tabMatch := regexp.MustCompile(`tab=([a-zA-Z0-9.\-_]+)`).FindStringSubmatch(url)
		if len(tabMatch) > 1 {
			subId = tabMatch[1]
		} else {
			subId = "t.0"
		}
	}

	return id, kind, subId
}

func (s *GoogleParserService) getSheetName(spreadsheetId string, gid string) (string, error) {
	if err := s.initServices(); err != nil {
		return "", err
	}

	ss, err := s.sheetsService.Spreadsheets.Get(spreadsheetId).Do()
	if err != nil {
		return "", err
	}
	for _, sheet := range ss.Sheets {
		if fmt.Sprintf("%d", sheet.Properties.SheetId) == gid {
			return sheet.Properties.Title, nil
		}
	}
	if len(ss.Sheets) > 0 && (gid == "" || gid == "0") {
		return ss.Sheets[0].Properties.Title, nil
	}
	return "", fmt.Errorf("sheet with gid %s not found", gid)
}

func (s *GoogleParserService) FetchSheet(url string) ([][]string, error) {
	id, kind, gid := s.ExtractID(url)
	if id == "" || kind != "sheet" {
		return nil, fmt.Errorf("invalid google sheet url")
	}

	if err := s.initServices(); err != nil {
		return s.fetchSheetUnofficial(id, gid)
	}

	sheetName, err := s.getSheetName(id, gid)
	if err != nil {
		return s.fetchSheetUnofficial(id, gid)
	}

	resp, err := s.sheetsService.Spreadsheets.Values.Get(id, sheetName+"!A1:Z2000").Do()
	if err != nil {
		return s.fetchSheetUnofficial(id, gid)
	}

	var results [][]string
	for _, row := range resp.Values {
		// Завжди робимо рядок довжиною мінімум 26 стовпців (A-Z)
		size := len(row)
		if size < 26 {
			size = 26
		}
		strRow := make([]string, size)
		for i, val := range row {
			strRow[i] = fmt.Sprintf("%v", val)
		}
		results = append(results, strRow)
	}

	return results, nil
}

func (s *GoogleParserService) fetchSheetUnofficial(id string, gid string) ([][]string, error) {
	exportUrl := fmt.Sprintf("https://docs.google.com/spreadsheets/d/%s/export?format=csv&gid=%s", id, gid)
	resp, err := s.client.Get(exportUrl)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP %s", resp.Status)
	}

	reader := csv.NewReader(resp.Body)
	reader.FieldsPerRecord = -1
	return reader.ReadAll()
}

func (s *GoogleParserService) extractTextFromDoc(doc *docs.Document) string {
	var sb strings.Builder
	for _, element := range doc.Body.Content {
		if element.Paragraph != nil {
			for _, run := range element.Paragraph.Elements {
				if run.TextRun != nil {
					sb.WriteString(run.TextRun.Content)
				}
			}
		} else if element.Table != nil {
			for _, row := range element.Table.TableRows {
				for _, cell := range row.TableCells {
					for _, content := range cell.Content {
						if content.Paragraph != nil {
							for _, run := range content.Paragraph.Elements {
								if run.TextRun != nil {
									sb.WriteString(run.TextRun.Content)
								}
							}
						}
					}
					sb.WriteString(" | ")
				}
				sb.WriteString("\n")
			}
		}
	}
	return sb.String()
}

func (s *GoogleParserService) FetchDoc(url string) (string, error) {
	id, kind, subId := s.ExtractID(url)
	if id == "" {
		return "", fmt.Errorf("invalid google url")
	}

	if kind == "sheet" {
		rows, err := s.FetchSheet(url)
		if err != nil {
			return "", err
		}
		var sb strings.Builder
		for _, row := range rows {
			sb.WriteString(strings.Join(row, " | "))
			sb.WriteString("\n")
		}
		return sb.String(), nil
	}

	if err := s.initServices(); err != nil {
		return s.fetchDocUnofficial(id, subId)
	}

	doc, err := s.docsService.Documents.Get(id).Do()
	if err != nil {
		return s.fetchDocUnofficial(id, subId)
	}

	return s.extractTextFromDoc(doc), nil
}

func (s *GoogleParserService) fetchDocUnofficial(id string, subId string) (string, error) {
	exportUrl := fmt.Sprintf("https://docs.google.com/document/d/%s/export?format=txt&tab=%s", id, subId)
	resp, err := s.client.Get(exportUrl)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("ERROR %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	content := string(body)
	content = strings.TrimPrefix(content, "\ufeff")
	return s.clipToFirstTab(content), nil
}

func (s *GoogleParserService) clipToFirstTab(content string) string {
	content = strings.ReplaceAll(content, "\r\n", "\n")
	content = strings.TrimPrefix(content, "\ufeff")

	if strings.Contains(content, "\x0c") {
		parts := strings.Split(content, "\x0c")
		for _, p := range parts {
			trimmed := strings.TrimSpace(p)
			if trimmed != "" {
				return trimmed
			}
		}
	}

	reGap := regexp.MustCompile(`\n(\s*\n){9,}`)
	gapLoc := reGap.FindStringIndex(content)
	if gapLoc != nil {
		content = content[:gapLoc[0]]
	}

	reNextTab := regexp.MustCompile(`(?mi)\n.*(Вкладка|Tab|Sheet|Page|Сторінка|Раздел|Section)\s*[23456789].*\n`)
	loc := reNextTab.FindStringIndex(content)
	if loc != nil {
		content = content[:loc[0]]
	}

	reFirstTab := regexp.MustCompile(`(?mi)^.*(Вкладка|Tab|Sheet|Page|Сторінка|Раздел|Section)\s*1.*$`)
	content = reFirstTab.ReplaceAllString(content, "")

	return strings.TrimSpace(content)
}

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

func (s *GoogleParserService) ParseWithFilter(sheetUrl string, filter string, ignoreRows int) ([]GoogleParserRow, error) {
	rows, err := s.FetchSheet(sheetUrl)
	if err != nil {
		return nil, err
	}

	var results []GoogleParserRow

	for i, row := range rows {
		// Пропускаємо перші N рядків
		if i < ignoreRows {
			continue
		}
		matchAll := true
		if filter != "" {
			parts := strings.Split(filter, "&")
			for _, part := range parts {
				part = strings.TrimSpace(part)
				if part == "" {
					continue
				}

				partMatch := false
				targetCol := -1
				searchValue := part

				colMatch := regexp.MustCompile(`^([a-zA-Z]+):(.*)$`).FindStringSubmatch(part)
				if len(colMatch) > 2 {
					targetCol = colLetterToIndex(colMatch[1])
					searchValue = strings.TrimSpace(colMatch[2])
				}

				isNot := false
				if strings.HasPrefix(searchValue, "!") {
					isNot = true
					searchValue = strings.TrimSpace(strings.TrimPrefix(searchValue, "!"))
				}

				isRegex := strings.ContainsAny(searchValue, "\\[]*+?")
				var re *regexp.Regexp
				if isRegex {
					re, _ = regexp.Compile("(?i)" + searchValue)
				}
				valLower := strings.ToLower(searchValue)

				foundMatch := false
				if targetCol != -1 {
					if targetCol < len(row) {
						cellVal := strings.TrimSpace(row[targetCol])
						if isRegex && re != nil {
							if re.MatchString(cellVal) { foundMatch = true }
						} else {
							if strings.Contains(strings.ToLower(cellVal), valLower) { foundMatch = true }
						}
					}
				} else {
					for _, col := range row {
						cellVal := strings.TrimSpace(col)
						if isRegex && re != nil {
							if re.MatchString(cellVal) { foundMatch = true; break }
						} else {
							if strings.Contains(strings.ToLower(cellVal), valLower) { foundMatch = true; break }
						}
					}
				}

				if isNot { partMatch = !foundMatch } else { partMatch = foundMatch }
				if !partMatch { matchAll = false; break }
			}
		}

		if !matchAll { continue }

		isEmptyRow := true
		for _, col := range row {
			if strings.TrimSpace(col) != "" { isEmptyRow = false; break }
		}
		if isEmptyRow { continue }

		var docLink string
		if len(row) > 5 && strings.Contains(row[5], "docs.google.com") {
			docLink = row[5]
		} else if len(row) > 2 && strings.Contains(row[2], "docs.google.com") {
			docLink = row[2]
		} else {
			for _, col := range row {
				if strings.Contains(col, "docs.google.com") {
					docLink = col
					break
				}
			}
		}

		title := ""
		if len(row) > 1 && strings.TrimSpace(row[1]) != "" {
			title = row[1]
		} else {
			for _, col := range row {
				if strings.TrimSpace(col) != "" { title = col; break }
			}
		}

		finalRow := GoogleParserRow{
			Index:   i,
			Title:   title,
			Columns: rows[i], // Використовуємо рядок з FetchSheet (вже з []string)
			DocLink: docLink,
		}
		results = append(results, finalRow)
	}

	sort.Slice(results, func(i, j int) bool {
		return results[i].Index < results[j].Index
	})

	return results, nil
}
