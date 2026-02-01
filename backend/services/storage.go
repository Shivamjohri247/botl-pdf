package services

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// RecentFile represents a recently opened PDF file
type RecentFile struct {
	Path         string    `json:"path"`
	Name         string    `json:"name"`
	LastOpened   time.Time `json:"lastOpened"`
	Size         int64     `json:"size"`
	PageCount    int       `json:"pageCount"`
}

// StorageService handles local file storage and recent files management
type StorageService struct {
	configPath string
	recentFile string
	recentList []RecentFile
	mu         sync.RWMutex
}

// NewStorageService creates a new storage service
func NewStorageService(configDir string) *StorageService {
	if configDir == "" {
		// Use default config directory
		homeDir, _ := os.UserHomeDir()
		configDir = filepath.Join(homeDir, ".botl-pdf")
	}

	// Ensure config directory exists
	os.MkdirAll(configDir, 0755)

	return &StorageService{
		configPath: configDir,
		recentFile: filepath.Join(configDir, "recent.json"),
		recentList: make([]RecentFile, 0),
	}
}

// LoadRecentFiles loads the recent files list from disk
func (s *StorageService) LoadRecentFiles() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	data, err := os.ReadFile(s.recentFile)
	if err != nil {
		if os.IsNotExist(err) {
			// File doesn't exist yet, start with empty list
			s.recentList = make([]RecentFile, 0)
			return nil
		}
		return err
	}

	return json.Unmarshal(data, &s.recentList)
}

// SaveRecentFiles saves the recent files list to disk
func (s *StorageService) SaveRecentFiles() error {
	s.mu.RLock()
	defer s.mu.RUnlock()

	data, err := json.MarshalIndent(s.recentList, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(s.recentFile, data, 0644)
}

// AddRecentFile adds a file to the recent files list
func (s *StorageService) AddRecentFile(file RecentFile) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	file.LastOpened = time.Now()

	// Check if file already exists in list
	for i, recent := range s.recentList {
		if recent.Path == file.Path {
			// Move to front
			s.recentList = append([]RecentFile{file}, append(s.recentList[:i], s.recentList[i+1:]...)...)
			return s.SaveRecentFiles()
		}
	}

	// Add to front of list
	s.recentList = append([]RecentFile{file}, s.recentList...)

	// Keep only the most recent 20 files
	if len(s.recentList) > 20 {
		s.recentList = s.recentList[:20]
	}

	return s.SaveRecentFiles()
}

// GetRecentFiles returns the list of recent files
func (s *StorageService) GetRecentFiles() []RecentFile {
	s.mu.RLock()
	defer s.mu.RUnlock()

	// Return a copy to avoid concurrent modification
	result := make([]RecentFile, len(s.recentList))
	copy(result, s.recentList)
	return result
}

// RemoveRecentFile removes a file from the recent files list
func (s *StorageService) RemoveRecentFile(path string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	for i, recent := range s.recentList {
		if recent.Path == path {
			s.recentList = append(s.recentList[:i], s.recentList[i+1:]...)
			return s.SaveRecentFiles()
		}
	}

	return nil
}

// ClearRecentFiles clears all recent files
func (s *StorageService) ClearRecentFiles() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.recentList = make([]RecentFile, 0)
	return s.SaveRecentFiles()
}

// GetFileInfo returns information about a file
func (s *StorageService) GetFileInfo(path string) (*RecentFile, error) {
	info, err := os.Stat(path)
	if err != nil {
		return nil, err
	}

	return &RecentFile{
		Path:       path,
		Name:       filepath.Base(path),
		LastOpened: info.ModTime(),
		Size:       info.Size(),
		PageCount:  0, // Will be determined by PDF service
	}, nil
}

// FileExists checks if a file exists
func (s *StorageService) FileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}
