package utils

import (
	"path/filepath"
	"sort"
	"strings"
	"sync"
)

type GalleryImage struct {
	Name     string  `json:"name"`
	Path     string  `json:"path"`
	URL      string  `json:"url"` // Used by frontend via assetserver
	Prompt   string  `json:"prompt"`
	Duration float64 `json:"duration"`
}

type GalleryTemplate struct {
	Name   string         `json:"name"`
	Images []GalleryImage `json:"images"`
}

type GalleryTask struct {
	Name      string            `json:"name"`
	Templates []GalleryTemplate `json:"templates"`
}

type GalleryManager struct {
	mu    sync.RWMutex
	tasks map[string]map[string][]GalleryImage
}

func NewGalleryManager() *GalleryManager {
	return &GalleryManager{
		tasks: make(map[string]map[string][]GalleryImage),
	}
}

func (m *GalleryManager) AddImage(taskName, templateName, imageName, imgPath, prompt string, duration float64) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.tasks[taskName] == nil {
		m.tasks[taskName] = make(map[string][]GalleryImage)
	}

	// Remove any existing entries for this path within the template to avoid duplicates
	existingImages := m.tasks[taskName][templateName]
	filteredImages := make([]GalleryImage, 0, len(existingImages))
	for _, img := range existingImages {
		if img.Path != imgPath {
			filteredImages = append(filteredImages, img)
		}
	}
	m.tasks[taskName][templateName] = filteredImages

	url := "local/" + filepath.ToSlash(imgPath)
	if strings.HasPrefix(imgPath, "http://") || strings.HasPrefix(imgPath, "https://") {
		url = imgPath
	}

	image := GalleryImage{
		Name:     imageName,
		Path:     imgPath, // For remote, path is the URL too
		URL:      url,
		Prompt:   prompt,
		Duration: duration,
	}

	m.tasks[taskName][templateName] = append(m.tasks[taskName][templateName], image)
}

// NaturalLess compares two strings using natural sort order (e.g. "2.jpg" < "10.jpg")
func NaturalLess(s1, s2 string) bool {
	i, j := 0, 0
	for i < len(s1) && j < len(s2) {
		c1, c2 := s1[i], s2[j]
		if (c1 >= '0' && c1 <= '9') && (c2 >= '0' && c2 <= '9') {
			// Extract numbers
			n1 := ""
			for i < len(s1) && s1[i] >= '0' && s1[i] <= '9' {
				n1 += string(s1[i])
				i++
			}
			n2 := ""
			for j < len(s2) && s2[j] >= '0' && s2[j] <= '9' {
				n2 += string(s2[j])
				j++
			}
			// Compare numbers by length first, then by value
			if len(n1) != len(n2) {
				return len(n1) < len(n2)
			}
			if n1 != n2 {
				return n1 < n2
			}
		} else {
			if c1 != c2 {
				return c1 < c2
			}
			i++
			j++
		}
	}
	return len(s1) < len(s2)
}

func (m *GalleryManager) GetGalleryData() []GalleryTask {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var results []GalleryTask
	for tName, tmpMap := range m.tasks {
		var templates []GalleryTemplate
		for tmpName, imgs := range tmpMap {
			// deep copy images
			var copies []GalleryImage
			copies = append(copies, imgs...)

			// sort images by name naturally
			sort.Slice(copies, func(i, j int) bool {
				return NaturalLess(copies[i].Name, copies[j].Name)
			})

			templates = append(templates, GalleryTemplate{
				Name:   tmpName,
				Images: copies,
			})
		}

		// Sort templates naturally
		sort.Slice(templates, func(i, j int) bool {
			return NaturalLess(templates[i].Name, templates[j].Name)
		})

		results = append(results, GalleryTask{
			Name:      tName,
			Templates: templates,
		})
	}

	// Sort tasks naturally
	sort.Slice(results, func(i, j int) bool {
		return NaturalLess(results[i].Name, results[j].Name)
	})

	return results
}
func (m *GalleryManager) RemoveImage(imgPath string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	for taskName, tmpMap := range m.tasks {
		for tmpName, imgs := range tmpMap {
			for i, img := range imgs {
				if img.Path == imgPath {
					m.tasks[taskName][tmpName] = append(imgs[:i], imgs[i+1:]...)
					if len(m.tasks[taskName][tmpName]) == 0 {
						delete(m.tasks[taskName], tmpName)
					}
					if len(m.tasks[taskName]) == 0 {
						delete(m.tasks, taskName)
					}
					return
				}
			}
		}
	}
}
// ReplaceImage replaces an existing gallery entry in-place, preserving its position.
func (m *GalleryManager) ReplaceImage(oldPath, newName, newPath, prompt string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	url := "local/" + filepath.ToSlash(newPath)
	if strings.HasPrefix(newPath, "http://") || strings.HasPrefix(newPath, "https://") {
		url = newPath
	}

	for taskName, tmpMap := range m.tasks {
		for tmpName, imgs := range tmpMap {
			for i, img := range imgs {
				if img.Path == oldPath {
					m.tasks[taskName][tmpName][i] = GalleryImage{
						Name:   newName,
						Path:   newPath,
						URL:    url,
						Prompt: prompt,
					}
					return
				}
			}
		}
	}
}

func (m *GalleryManager) GetImagePrompt(imgPath string) string {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, tmpMap := range m.tasks {
		for _, imgs := range tmpMap {
			for _, img := range imgs {
				if img.Path == imgPath {
					return img.Prompt
				}
			}
		}
	}
	return ""
}

func (m *GalleryManager) Clear() {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.tasks = make(map[string]map[string][]GalleryImage)
}
