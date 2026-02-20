package utils

import (
	"path/filepath"
	"sort"
	"sync"
)

type GalleryImage struct {
	Name string `json:"name"`
	Path string `json:"path"`
	URL  string `json:"url"` // Used by frontend via assetserver
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

func (m *GalleryManager) AddImage(taskName, templateName, imageName, imgPath string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.tasks[taskName] == nil {
		m.tasks[taskName] = make(map[string][]GalleryImage)
	}

	urlPath := filepath.ToSlash(imgPath)
	image := GalleryImage{
		Name: imageName,
		Path: imgPath,
		URL:  "local/" + urlPath,
	}

	m.tasks[taskName][templateName] = append(m.tasks[taskName][templateName], image)
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

			// sort images by name
			sort.Slice(copies, func(i, j int) bool {
				return copies[i].Name < copies[j].Name
			})

			templates = append(templates, GalleryTemplate{
				Name:   tmpName,
				Images: copies,
			})
		}

		// Sort templates
		sort.Slice(templates, func(i, j int) bool {
			return templates[i].Name < templates[j].Name
		})

		results = append(results, GalleryTask{
			Name:      tName,
			Templates: templates,
		})
	}

	// Sort tasks
	sort.Slice(results, func(i, j int) bool {
		return results[i].Name < results[j].Name
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
