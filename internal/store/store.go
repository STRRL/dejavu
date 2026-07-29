package store

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"unicode/utf8"

	"github.com/strrl/dejavu/internal/model"
	_ "modernc.org/sqlite"
)

var cjkPattern = regexp.MustCompile(`[\p{Han}\p{Hiragana}\p{Katakana}\p{Hangul}]`)

type Store struct {
	db        *sql.DB
	root      string
	imageRoot string
	dbPath    string
}

func Open(root string) (*Store, error) {
	imageRoot := filepath.Join(root, "images")
	if err := os.MkdirAll(imageRoot, 0o700); err != nil {
		return nil, fmt.Errorf("create image directory: %w", err)
	}

	dbPath := filepath.Join(root, "dejavu.db")
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("open database: %w", err)
	}
	db.SetMaxOpenConns(1)

	store := &Store{
		db:        db,
		root:      root,
		imageRoot: imageRoot,
		dbPath:    dbPath,
	}
	if err := store.initialize(); err != nil {
		_ = db.Close()
		return nil, err
	}
	return store, nil
}

func (s *Store) initialize() error {
	statements := []string{
		"PRAGMA journal_mode = WAL",
		"PRAGMA foreign_keys = ON",
		"PRAGMA busy_timeout = 5000",
		`CREATE TABLE IF NOT EXISTS moments (
			id INTEGER PRIMARY KEY,
			captured_at INTEGER NOT NULL,
			app_name TEXT NOT NULL DEFAULT '',
			bundle_id TEXT NOT NULL DEFAULT '',
			process_id INTEGER NOT NULL DEFAULT 0,
			window_title TEXT NOT NULL DEFAULT '',
			window_x REAL NOT NULL DEFAULT 0,
			window_y REAL NOT NULL DEFAULT 0,
			window_width REAL NOT NULL DEFAULT 0,
			window_height REAL NOT NULL DEFAULT 0,
			accessibility_granted INTEGER NOT NULL DEFAULT 0
		)`,
		`CREATE INDEX IF NOT EXISTS moments_captured_at_idx
			ON moments(captured_at DESC)`,
		`CREATE TABLE IF NOT EXISTS captures (
			id INTEGER PRIMARY KEY,
			moment_id INTEGER NOT NULL REFERENCES moments(id) ON DELETE CASCADE,
			display_index INTEGER NOT NULL,
			width INTEGER NOT NULL,
			height INTEGER NOT NULL,
			image_file TEXT NOT NULL UNIQUE,
			thumbnail_file TEXT NOT NULL UNIQUE,
			image_bytes INTEGER NOT NULL,
			thumbnail_bytes INTEGER NOT NULL,
			change_ratio REAL NOT NULL,
			ocr_status TEXT NOT NULL,
			ocr_error TEXT NOT NULL DEFAULT ''
		)`,
		`CREATE INDEX IF NOT EXISTS captures_moment_idx
			ON captures(moment_id)`,
		`CREATE TABLE IF NOT EXISTS text_blocks (
			id INTEGER PRIMARY KEY,
			capture_id INTEGER NOT NULL REFERENCES captures(id) ON DELETE CASCADE,
			text TEXT NOT NULL,
			confidence REAL NOT NULL,
			x REAL NOT NULL,
			y REAL NOT NULL,
			width REAL NOT NULL,
			height REAL NOT NULL
		)`,
		`CREATE INDEX IF NOT EXISTS text_blocks_capture_idx
			ON text_blocks(capture_id)`,
		`CREATE TABLE IF NOT EXISTS capture_search (
			capture_id INTEGER PRIMARY KEY REFERENCES captures(id) ON DELETE CASCADE,
			window_title TEXT NOT NULL DEFAULT '',
			app_name TEXT NOT NULL DEFAULT '',
			ocr_text TEXT NOT NULL DEFAULT ''
		)`,
		`CREATE VIRTUAL TABLE IF NOT EXISTS capture_search_fts_unicode USING fts5(
			window_title,
			app_name,
			ocr_text,
			content='capture_search',
			content_rowid='capture_id',
			tokenize='unicode61 remove_diacritics 1'
		)`,
		`CREATE VIRTUAL TABLE IF NOT EXISTS capture_search_fts_trigram USING fts5(
			window_title,
			app_name,
			ocr_text,
			content='capture_search',
			content_rowid='capture_id',
			tokenize='trigram'
		)`,
		`CREATE TRIGGER IF NOT EXISTS capture_search_insert
			AFTER INSERT ON capture_search BEGIN
				INSERT INTO capture_search_fts_unicode(rowid, window_title, app_name, ocr_text)
				VALUES (NEW.capture_id, NEW.window_title, NEW.app_name, NEW.ocr_text);
				INSERT INTO capture_search_fts_trigram(rowid, window_title, app_name, ocr_text)
				VALUES (NEW.capture_id, NEW.window_title, NEW.app_name, NEW.ocr_text);
			END`,
		`CREATE TRIGGER IF NOT EXISTS capture_search_update
			AFTER UPDATE ON capture_search BEGIN
				INSERT INTO capture_search_fts_unicode(
					capture_search_fts_unicode,
					rowid,
					window_title,
					app_name,
					ocr_text
				) VALUES (
					'delete',
					OLD.capture_id,
					OLD.window_title,
					OLD.app_name,
					OLD.ocr_text
				);
				INSERT INTO capture_search_fts_unicode(rowid, window_title, app_name, ocr_text)
				VALUES (NEW.capture_id, NEW.window_title, NEW.app_name, NEW.ocr_text);
				INSERT INTO capture_search_fts_trigram(
					capture_search_fts_trigram,
					rowid,
					window_title,
					app_name,
					ocr_text
				) VALUES (
					'delete',
					OLD.capture_id,
					OLD.window_title,
					OLD.app_name,
					OLD.ocr_text
				);
				INSERT INTO capture_search_fts_trigram(rowid, window_title, app_name, ocr_text)
				VALUES (NEW.capture_id, NEW.window_title, NEW.app_name, NEW.ocr_text);
			END`,
		`CREATE TRIGGER IF NOT EXISTS capture_search_delete
			AFTER DELETE ON capture_search BEGIN
				INSERT INTO capture_search_fts_unicode(
					capture_search_fts_unicode,
					rowid,
					window_title,
					app_name,
					ocr_text
				) VALUES (
					'delete',
					OLD.capture_id,
					OLD.window_title,
					OLD.app_name,
					OLD.ocr_text
				);
				INSERT INTO capture_search_fts_trigram(
					capture_search_fts_trigram,
					rowid,
					window_title,
					app_name,
					ocr_text
				) VALUES (
					'delete',
					OLD.capture_id,
					OLD.window_title,
					OLD.app_name,
					OLD.ocr_text
				);
			END`,
		"PRAGMA user_version = 1",
	}

	for _, statement := range statements {
		if _, err := s.db.Exec(statement); err != nil {
			return fmt.Errorf("initialize database: %w", err)
		}
	}
	return nil
}

func (s *Store) Close() error {
	return s.db.Close()
}

func (s *Store) ImageRoot() string {
	return s.imageRoot
}

func (s *Store) SaveMoment(ctx context.Context, input model.MomentInput) error {
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("start save: %w", err)
	}
	defer func() {
		_ = tx.Rollback()
	}()

	result, err := tx.ExecContext(
		ctx,
		`INSERT INTO moments (
			captured_at,
			app_name,
			bundle_id,
			process_id,
			window_title,
			window_x,
			window_y,
			window_width,
			window_height,
			accessibility_granted
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		input.CapturedAt,
		input.Window.AppName,
		input.Window.BundleID,
		input.Window.ProcessID,
		input.Window.WindowTitle,
		input.Window.WindowX,
		input.Window.WindowY,
		input.Window.WindowWidth,
		input.Window.WindowHeight,
		input.Window.AccessibilityGranted,
	)
	if err != nil {
		return fmt.Errorf("save moment: %w", err)
	}
	momentID, err := result.LastInsertId()
	if err != nil {
		return fmt.Errorf("read moment id: %w", err)
	}

	for _, capture := range input.Captures {
		captureResult, err := tx.ExecContext(
			ctx,
			`INSERT INTO captures (
				moment_id,
				display_index,
				width,
				height,
				image_file,
				thumbnail_file,
				image_bytes,
				thumbnail_bytes,
				change_ratio,
				ocr_status,
				ocr_error
			) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
			momentID,
			capture.DisplayIndex,
			capture.Width,
			capture.Height,
			capture.ImageFile,
			capture.ThumbnailFile,
			capture.ImageBytes,
			capture.ThumbnailBytes,
			capture.ChangeRatio,
			capture.OCRStatus,
			capture.OCRError,
		)
		if err != nil {
			return fmt.Errorf("save capture: %w", err)
		}
		captureID, err := captureResult.LastInsertId()
		if err != nil {
			return fmt.Errorf("read capture id: %w", err)
		}

		texts := make([]string, 0, len(capture.Blocks))
		for _, block := range capture.Blocks {
			texts = append(texts, block.Text)
			if _, err := tx.ExecContext(
				ctx,
				`INSERT INTO text_blocks (
					capture_id,
					text,
					confidence,
					x,
					y,
					width,
					height
				) VALUES (?, ?, ?, ?, ?, ?, ?)`,
				captureID,
				block.Text,
				block.Confidence,
				block.X,
				block.Y,
				block.Width,
				block.Height,
			); err != nil {
				return fmt.Errorf("save text block: %w", err)
			}
		}

		if _, err := tx.ExecContext(
			ctx,
			`INSERT INTO capture_search (
				capture_id,
				window_title,
				app_name,
				ocr_text
			) VALUES (?, ?, ?, ?)`,
			captureID,
			input.Window.WindowTitle,
			input.Window.AppName,
			strings.Join(texts, "\n"),
		); err != nil {
			return fmt.Errorf("save search text: %w", err)
		}
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit save: %w", err)
	}
	return nil
}

func (s *Store) Search(ctx context.Context, request model.SearchRequest) (model.SearchPage, error) {
	request.Query = strings.TrimSpace(request.Query)
	if request.Offset < 0 {
		request.Offset = 0
	}
	if request.Limit <= 0 {
		request.Limit = 40
	}
	if request.Limit > 100 {
		request.Limit = 100
	}

	target := request.Offset + request.Limit + 1
	var ids []int64
	var err error
	if request.Query == "" {
		ids, err = s.recentCaptureIDs(ctx, target)
	} else if needsLikeFallback(request.Query) {
		ids, err = s.likeCaptureIDs(ctx, request.Query, target)
	} else {
		ids, err = s.ftsCaptureIDs(ctx, request.Query, target)
	}
	if err != nil {
		return model.SearchPage{}, err
	}

	if request.Offset >= len(ids) {
		return model.SearchPage{Results: []model.SearchResult{}}, nil
	}
	end := request.Offset + request.Limit
	if end > len(ids) {
		end = len(ids)
	}
	selected := ids[request.Offset:end]
	results := make([]model.SearchResult, 0, len(selected))
	for _, id := range selected {
		result, err := s.loadSearchResult(ctx, id, request.Query)
		if err != nil {
			return model.SearchPage{}, err
		}
		results = append(results, result)
	}

	nextOffset := request.Offset + len(results)
	return model.SearchPage{
		Results:    results,
		NextOffset: nextOffset,
		HasMore:    end < len(ids),
	}, nil
}

func (s *Store) recentCaptureIDs(ctx context.Context, limit int) ([]int64, error) {
	rows, err := s.db.QueryContext(
		ctx,
		`SELECT captures.id
		FROM captures
		JOIN moments ON moments.id = captures.moment_id
		ORDER BY moments.captured_at DESC, captures.id DESC
		LIMIT ?`,
		limit,
	)
	if err != nil {
		return nil, fmt.Errorf("load recent captures: %w", err)
	}
	defer rows.Close()
	return scanIDs(rows)
}

func (s *Store) ftsCaptureIDs(ctx context.Context, query string, limit int) ([]int64, error) {
	table := "capture_search_fts_unicode"
	if cjkPattern.MatchString(query) {
		table = "capture_search_fts_trigram"
	}

	plans := buildSearchPlans(query, table == "capture_search_fts_unicode")
	seen := make(map[int64]bool)
	ids := make([]int64, 0, limit)
	for _, plan := range plans {
		statement := fmt.Sprintf(
			`SELECT capture_search.capture_id
			FROM %s
			JOIN capture_search ON capture_search.capture_id = %s.rowid
			JOIN captures ON captures.id = capture_search.capture_id
			JOIN moments ON moments.id = captures.moment_id
			WHERE %s MATCH ?
			ORDER BY bm25(%s, 5.0, 3.0, 1.0), moments.captured_at DESC
			LIMIT ?`,
			table,
			table,
			table,
			table,
		)
		rows, err := s.db.QueryContext(ctx, statement, plan, limit)
		if err != nil {
			return nil, fmt.Errorf("search captures: %w", err)
		}
		group, scanErr := scanIDs(rows)
		closeErr := rows.Close()
		if scanErr != nil {
			return nil, scanErr
		}
		if closeErr != nil {
			return nil, fmt.Errorf("close search rows: %w", closeErr)
		}
		for _, id := range group {
			if seen[id] {
				continue
			}
			seen[id] = true
			ids = append(ids, id)
			if len(ids) >= limit {
				return ids, nil
			}
		}
	}
	return ids, nil
}

func (s *Store) likeCaptureIDs(ctx context.Context, query string, limit int) ([]int64, error) {
	terms := naturalTerms(query)
	if len(terms) == 0 {
		return []int64{}, nil
	}

	conditions := make([]string, 0, len(terms))
	params := make([]any, 0, len(terms)*3+1)
	for _, term := range terms {
		conditions = append(
			conditions,
			`(
				capture_search.window_title LIKE ? ESCAPE '\'
				OR capture_search.app_name LIKE ? ESCAPE '\'
				OR capture_search.ocr_text LIKE ? ESCAPE '\'
			)`,
		)
		pattern := "%" + escapeLike(term) + "%"
		params = append(params, pattern, pattern, pattern)
	}
	params = append(params, limit)

	rows, err := s.db.QueryContext(
		ctx,
		`SELECT capture_search.capture_id
		FROM capture_search
		JOIN captures ON captures.id = capture_search.capture_id
		JOIN moments ON moments.id = captures.moment_id
		WHERE `+strings.Join(conditions, " AND ")+`
		ORDER BY
			CASE WHEN capture_search.window_title LIKE ? THEN 0 ELSE 1 END,
			moments.captured_at DESC
		LIMIT ?`,
		append(params[:len(params)-1], "%"+escapeLike(query)+"%", limit)...,
	)
	if err != nil {
		return nil, fmt.Errorf("search short query: %w", err)
	}
	defer rows.Close()
	return scanIDs(rows)
}

func (s *Store) loadSearchResult(
	ctx context.Context,
	id int64,
	query string,
) (model.SearchResult, error) {
	var result model.SearchResult
	var text string
	err := s.db.QueryRowContext(
		ctx,
		`SELECT
			captures.id,
			moments.captured_at,
			captures.display_index,
			captures.width,
			captures.height,
			moments.app_name,
			moments.bundle_id,
			moments.window_title,
			capture_search.ocr_text,
			captures.ocr_status
		FROM captures
		JOIN moments ON moments.id = captures.moment_id
		JOIN capture_search ON capture_search.capture_id = captures.id
		WHERE captures.id = ?`,
		id,
	).Scan(
		&result.ID,
		&result.CapturedAt,
		&result.DisplayIndex,
		&result.Width,
		&result.Height,
		&result.AppName,
		&result.BundleID,
		&result.WindowTitle,
		&text,
		&result.OCRStatus,
	)
	if err != nil {
		return model.SearchResult{}, fmt.Errorf("load search result: %w", err)
	}

	terms := naturalTerms(query)
	result.Snippet, result.Highlights = buildSnippet(text, terms)
	blocks, err := s.loadBlocks(ctx, id)
	if err != nil {
		return model.SearchResult{}, err
	}
	result.MatchingBoxes = matchingBlocks(blocks, terms)
	return result, nil
}

func (s *Store) GetCapture(ctx context.Context, id int64) (model.CaptureDetail, error) {
	var detail model.CaptureDetail
	err := s.db.QueryRowContext(
		ctx,
		`SELECT
			captures.id,
			moments.captured_at,
			captures.display_index,
			captures.width,
			captures.height,
			moments.app_name,
			moments.bundle_id,
			moments.process_id,
			moments.window_title,
			moments.window_x,
			moments.window_y,
			moments.window_width,
			moments.window_height,
			captures.ocr_status,
			captures.ocr_error
		FROM captures
		JOIN moments ON moments.id = captures.moment_id
		WHERE captures.id = ?`,
		id,
	).Scan(
		&detail.ID,
		&detail.CapturedAt,
		&detail.DisplayIndex,
		&detail.Width,
		&detail.Height,
		&detail.AppName,
		&detail.BundleID,
		&detail.ProcessID,
		&detail.WindowTitle,
		&detail.WindowX,
		&detail.WindowY,
		&detail.WindowWidth,
		&detail.WindowHeight,
		&detail.OCRStatus,
		&detail.OCRError,
	)
	if errors.Is(err, sql.ErrNoRows) {
		return model.CaptureDetail{}, os.ErrNotExist
	}
	if err != nil {
		return model.CaptureDetail{}, fmt.Errorf("load capture: %w", err)
	}
	detail.Blocks, err = s.loadBlocks(ctx, id)
	if err != nil {
		return model.CaptureDetail{}, err
	}
	return detail, nil
}

func (s *Store) loadBlocks(ctx context.Context, captureID int64) ([]model.TextBlock, error) {
	rows, err := s.db.QueryContext(
		ctx,
		`SELECT id, capture_id, text, confidence, x, y, width, height
		FROM text_blocks
		WHERE capture_id = ?
		ORDER BY id`,
		captureID,
	)
	if err != nil {
		return nil, fmt.Errorf("load text blocks: %w", err)
	}
	defer rows.Close()

	blocks := make([]model.TextBlock, 0)
	for rows.Next() {
		var block model.TextBlock
		if err := rows.Scan(
			&block.ID,
			&block.CaptureID,
			&block.Text,
			&block.Confidence,
			&block.X,
			&block.Y,
			&block.Width,
			&block.Height,
		); err != nil {
			return nil, fmt.Errorf("read text block: %w", err)
		}
		blocks = append(blocks, block)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("read text blocks: %w", err)
	}
	return blocks, nil
}

func (s *Store) StorageStats(ctx context.Context) (model.StorageStats, error) {
	var stats model.StorageStats
	if err := s.db.QueryRowContext(
		ctx,
		`SELECT
			COALESCE(SUM(image_bytes + thumbnail_bytes), 0),
			COUNT(*)
		FROM captures`,
	).Scan(&stats.ImageBytes, &stats.CaptureCount); err != nil {
		return model.StorageStats{}, fmt.Errorf("load storage totals: %w", err)
	}

	rows, err := s.db.QueryContext(
		ctx,
		`SELECT
			date(moments.captured_at, 'unixepoch', 'localtime') AS day,
			SUM(captures.image_bytes + captures.thumbnail_bytes),
			COUNT(*)
		FROM captures
		JOIN moments ON moments.id = captures.moment_id
		GROUP BY day
		ORDER BY day DESC
		LIMIT 30`,
	)
	if err != nil {
		return model.StorageStats{}, fmt.Errorf("load daily storage: %w", err)
	}
	defer rows.Close()
	for rows.Next() {
		var daily model.DailyStorage
		if err := rows.Scan(&daily.Date, &daily.ImageBytes, &daily.Count); err != nil {
			return model.StorageStats{}, fmt.Errorf("read daily storage: %w", err)
		}
		stats.Daily = append(stats.Daily, daily)
	}
	if err := rows.Err(); err != nil {
		return model.StorageStats{}, fmt.Errorf("read daily storage: %w", err)
	}

	for _, path := range []string{s.dbPath, s.dbPath + "-wal", s.dbPath + "-shm"} {
		info, err := os.Stat(path)
		if err == nil {
			stats.DatabaseBytes += info.Size()
			continue
		}
		if !errors.Is(err, os.ErrNotExist) {
			return model.StorageStats{}, fmt.Errorf("read database size: %w", err)
		}
	}
	return stats, nil
}

func (s *Store) CaptureCount(ctx context.Context) (int64, error) {
	var count int64
	if err := s.db.QueryRowContext(ctx, "SELECT COUNT(*) FROM captures").Scan(&count); err != nil {
		return 0, fmt.Errorf("load capture count: %w", err)
	}
	return count, nil
}

func (s *Store) ImagePath(ctx context.Context, id int64, thumbnail bool) (string, error) {
	column := "image_file"
	if thumbnail {
		column = "thumbnail_file"
	}
	var relative string
	query := fmt.Sprintf("SELECT %s FROM captures WHERE id = ?", column)
	if err := s.db.QueryRowContext(ctx, query, id).Scan(&relative); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return "", os.ErrNotExist
		}
		return "", fmt.Errorf("load image path: %w", err)
	}

	path := filepath.Join(s.imageRoot, filepath.Clean(relative))
	relativeToRoot, err := filepath.Rel(s.imageRoot, path)
	if err != nil || relativeToRoot == ".." || strings.HasPrefix(relativeToRoot, ".."+string(os.PathSeparator)) {
		return "", errors.New("invalid image path")
	}
	return path, nil
}

func scanIDs(rows *sql.Rows) ([]int64, error) {
	ids := make([]int64, 0)
	for rows.Next() {
		var id int64
		if err := rows.Scan(&id); err != nil {
			return nil, fmt.Errorf("read capture id: %w", err)
		}
		ids = append(ids, id)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("read capture ids: %w", err)
	}
	return ids, nil
}

func naturalTerms(query string) []string {
	return strings.Fields(strings.TrimSpace(query))
}

func needsLikeFallback(query string) bool {
	if !cjkPattern.MatchString(query) {
		return false
	}
	for _, term := range naturalTerms(query) {
		if utf8.RuneCountInString(term) < 3 {
			return true
		}
	}
	return false
}

func buildSearchPlans(query string, prefix bool) []string {
	terms := naturalTerms(query)
	if len(terms) == 0 {
		return nil
	}
	quoted := func(value string) string {
		value = strings.ReplaceAll(value, `"`, `""`)
		suffix := ""
		if prefix {
			suffix = "*"
		}
		return `"` + value + `"` + suffix
	}
	if len(terms) == 1 {
		return []string{quoted(terms[0])}
	}

	phrase := `"` + strings.ReplaceAll(strings.Join(terms, " "), `"`, `""`) + `"`
	allTerms := make([]string, 0, len(terms))
	for _, term := range terms {
		allTerms = append(allTerms, quoted(term))
	}
	return []string{phrase, strings.Join(allTerms, " AND ")}
}

func escapeLike(value string) string {
	value = strings.ReplaceAll(value, `\`, `\\`)
	value = strings.ReplaceAll(value, `%`, `\%`)
	return strings.ReplaceAll(value, `_`, `\_`)
}

func buildSnippet(text string, terms []string) (string, []model.HighlightRange) {
	text = strings.TrimSpace(text)
	if text == "" {
		return "", []model.HighlightRange{}
	}
	runes := []rune(text)
	lower := []rune(strings.ToLower(text))
	first := 0
	found := false
	for _, term := range terms {
		index := runeIndex(lower, []rune(strings.ToLower(term)))
		if index >= 0 && (!found || index < first) {
			first = index
			found = true
		}
	}

	start := first - 60
	if start < 0 {
		start = 0
	}
	end := first + 140
	if end > len(runes) {
		end = len(runes)
	}
	snippetRunes := runes[start:end]
	snippet := string(snippetRunes)
	snippetLower := []rune(strings.ToLower(snippet))
	ranges := make([]model.HighlightRange, 0)
	for _, term := range terms {
		termRunes := []rune(strings.ToLower(term))
		searchFrom := 0
		for searchFrom < len(snippetLower) {
			index := runeIndex(snippetLower[searchFrom:], termRunes)
			if index < 0 {
				break
			}
			index += searchFrom
			ranges = append(ranges, model.HighlightRange{
				Start: index,
				End:   index + len(termRunes),
			})
			searchFrom = index + len(termRunes)
		}
	}
	sort.Slice(ranges, func(i, j int) bool {
		return ranges[i].Start < ranges[j].Start
	})
	return snippet, ranges
}

func runeIndex(value []rune, part []rune) int {
	if len(part) == 0 || len(part) > len(value) {
		return -1
	}
	for i := 0; i <= len(value)-len(part); i++ {
		match := true
		for j := range part {
			if value[i+j] != part[j] {
				match = false
				break
			}
		}
		if match {
			return i
		}
	}
	return -1
}

func matchingBlocks(blocks []model.TextBlock, terms []string) []model.TextBlock {
	if len(terms) == 0 {
		return []model.TextBlock{}
	}
	matches := make([]model.TextBlock, 0)
	for _, block := range blocks {
		lower := strings.ToLower(block.Text)
		for _, term := range terms {
			if strings.Contains(lower, strings.ToLower(term)) {
				matches = append(matches, block)
				break
			}
		}
	}
	return matches
}
