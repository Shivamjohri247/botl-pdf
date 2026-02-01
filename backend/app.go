package backend

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"

	"botl-pdf/backend/services"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

// App struct
type App struct {
	ctx            context.Context
	storageService *services.StorageService
	mergeService   *services.MergeService
}

// NewApp creates a new App application struct
func NewApp() *App {
	storageService := services.NewStorageService("")
	storageService.LoadRecentFiles()

	return &App{
		storageService: storageService,
		mergeService:   services.NewMergeService(),
	}
}

// Startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) Startup(ctx context.Context) {
	a.ctx = ctx
}

// Greet returns a greeting for the given name
func (a *App) Greet(name string) string {
	return fmt.Sprintf("Hello %s, It's show time!", name)
}

// OpenFilePicker opens a native file dialog and returns the selected file paths
func (a *App) OpenFilePicker() ([]string, error) {
	if a.ctx == nil {
		return nil, fmt.Errorf("application context not initialized")
	}

	// Open file dialog with PDF filter
	dialog, err := runtime.OpenMultipleFilesDialog(a.ctx, runtime.OpenDialogOptions{
		Title: "Select PDF Files",
		Filters: []runtime.FileFilter{
			{
				DisplayName: "PDF Documents",
				Pattern:     "*.pdf",
			},
			{
				DisplayName: "All Files",
				Pattern:     "*.*",
			},
		},
	})

	if err != nil {
		return nil, fmt.Errorf("failed to open file dialog: %w", err)
	}

	return dialog, nil
}

// SaveFileDialog opens a native save dialog and returns the selected file path
func (a *App) SaveFileDialog(defaultFilename string) (string, error) {
	if a.ctx == nil {
		return "", fmt.Errorf("application context not initialized")
	}

	if defaultFilename == "" {
		defaultFilename = "document.pdf"
	}

	path, err := runtime.SaveFileDialog(a.ctx, runtime.SaveDialogOptions{
		Title:           "Save PDF",
		DefaultFilename: defaultFilename,
		Filters: []runtime.FileFilter{
			{
				DisplayName: "PDF Documents",
				Pattern:     "*.pdf",
			},
		},
	})

	if err != nil {
		return "", fmt.Errorf("failed to open save dialog: %w", err)
	}

	return path, nil
}

// ReadFileAsBase64 reads a file and returns its content as a base64-encoded data URL
func (a *App) ReadFileAsBase64(filePath string) (string, error) {
	if filePath == "" {
		return "", fmt.Errorf("file path is empty")
	}

	// Read the file
	data, err := os.ReadFile(filePath)
	if err != nil {
		return "", fmt.Errorf("failed to read file: %w", err)
	}

	// Encode to base64
	base64Str := base64.StdEncoding.EncodeToString(data)

	// Return as data URL
	return fmt.Sprintf("data:application/pdf;base64,%s", base64Str), nil
}

// MergeFileInfo represents a file for merging with optional page range
type MergeFileInfo struct {
	FilePath  string `json:"filePath"`
	Name      string `json:"name"`
	PageRange string `json:"pageRange,omitempty"`
	PageCount int    `json:"pageCount"`
	Size      int64  `json:"size"`
}

// MergePDFs merges multiple PDF files into one
// Input: slice of file paths and output path
// Returns: error if merge fails
func (a *App) MergePDFs(filePaths []string, outputPath string) error {
	if len(filePaths) == 0 {
		return fmt.Errorf("no files provided for merge")
	}
	if outputPath == "" {
		return fmt.Errorf("output path not provided")
	}

	// Convert file paths to MergeFileList
	files := make([]services.MergeFileList, len(filePaths))
	for i, path := range filePaths {
		info, err := a.mergeService.GetPDFInfo(path)
		if err != nil {
			return fmt.Errorf("failed to get PDF info for %s: %w", path, err)
		}
		files[i] = services.MergeFileList{
			FilePath:  path,
			Name:      info.Name,
			PageCount: info.PageCount,
			Size:      info.Size,
			PageRange: "all",
		}
	}

	return a.mergeService.MergeMultiplePDFs(files, outputPath)
}

// MergePDFsWithInfo merges PDFs with detailed file information
// This version supports page ranges and returns more detailed information
func (a *App) MergePDFsWithInfo(files []MergeFileInfo, outputPath string) error {
	if len(files) == 0 {
		return fmt.Errorf("no files provided for merge")
	}
	if outputPath == "" {
		return fmt.Errorf("output path not provided")
	}

	// Convert to service types
	mergeFiles := make([]services.MergeFileList, len(files))
	for i, f := range files {
		mergeFiles[i] = services.MergeFileList{
			FilePath:  f.FilePath,
			Name:      f.Name,
			PageRange: f.PageRange,
			PageCount: f.PageCount,
			Size:      f.Size,
		}
	}

	return a.mergeService.MergeMultiplePDFs(mergeFiles, outputPath)
}

// GetPDFInfo returns information about a PDF file
func (a *App) GetPDFInfo(filePath string) (*services.PDFInfo, error) {
	return a.mergeService.GetPDFInfo(filePath)
}

// ValidatePDFs checks if all provided files are valid PDFs
func (a *App) ValidatePDFs(filePaths []string) error {
	return a.mergeService.ValidatePDFs(filePaths)
}

// GetRecentFiles returns the list of recently opened files
func (a *App) GetRecentFiles() []services.RecentFile {
	return a.storageService.GetRecentFiles()
}

// AddRecentFile adds a file to the recent files list
func (a *App) AddRecentFile(path string) error {
	info, err := a.storageService.GetFileInfo(path)
	if err != nil {
		return err
	}
	return a.storageService.AddRecentFile(*info)
}
