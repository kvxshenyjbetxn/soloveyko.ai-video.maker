package utils

import (
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
)

type FileLoader struct {
	http.Handler
}

func NewFileLoader() *FileLoader {
	return &FileLoader{}
}

func (h *FileLoader) ServeHTTP(res http.ResponseWriter, req *http.Request) {
	requestedFilename := strings.TrimPrefix(req.URL.Path, "/")
	requestedFilename = strings.TrimPrefix(requestedFilename, "local/")

	// Unescape the path to handle spaces and other special characters
	if path, err := url.PathUnescape(requestedFilename); err == nil {
		requestedFilename = path
	}

	// Security: simple check to stay within relative allowlist if needed,
	// but here we just serve what's requested as "local/".

	if info, err := os.Stat(requestedFilename); err == nil && !info.IsDir() {
		// Check if thumbnail is requested
		if req.URL.Query().Get("thumb") == "1" {
			ext := filepath.Ext(requestedFilename)
			// Only images can have thumbnails
			lowExt := strings.ToLower(ext)
			if lowExt == ".jpg" || lowExt == ".jpeg" || lowExt == ".png" || lowExt == ".webp" {
				thumbPath := filepath.Join(filepath.Dir(requestedFilename), ".thumbs", filepath.Base(requestedFilename))

				// Ensure .thumbs dir exists
				_ = os.MkdirAll(filepath.Dir(thumbPath), 0755)

				// Check if thumb already exists and is newer than original
				origInfo, _ := os.Stat(requestedFilename)
				thumbInfo, err := os.Stat(thumbPath)

				if err != nil || thumbInfo.ModTime().Before(origInfo.ModTime()) {
					// Generate thumbnail
					err := CreateThumbnail(requestedFilename, thumbPath, 400)
					if err != nil {
						// Fallback to original if failed
						http.ServeFile(res, req, requestedFilename)
						return
					}
				}
				http.ServeFile(res, req, thumbPath)
				return
			}
		}

		http.ServeFile(res, req, requestedFilename)
		return
	}
	res.WriteHeader(http.StatusNotFound)
}
