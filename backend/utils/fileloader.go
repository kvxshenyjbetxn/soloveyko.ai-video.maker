package utils

import (
	"net/http"
	"os"
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

	if info, err := os.Stat(requestedFilename); err == nil && !info.IsDir() {
		http.ServeFile(res, req, requestedFilename)
		return
	}
	res.WriteHeader(http.StatusNotFound)
}
