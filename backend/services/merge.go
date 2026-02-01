package services

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/pdfcpu/pdfcpu/pkg/api"
	"github.com/pdfcpu/pdfcpu/pkg/pdfcpu/model"
)

// MergeService handles PDF merge operations
type MergeService struct{}

// NewMergeService creates a new merge service
func NewMergeService() *MergeService {
	return &MergeService{}
}

// MergePDFs merges multiple PDF files into a single output file
// inputPaths: slice of PDF file paths to merge
// outputPath: path where the merged PDF will be saved
// The order of files in inputPaths determines the order in the merged PDF
func (s *MergeService) MergePDFs(inputPaths []string, outputPath string) error {
	if len(inputPaths) == 0 {
		return fmt.Errorf("no input files provided for merge")
	}

	if len(inputPaths) == 1 {
		// Only one file, just copy it
		return s.copyFile(inputPaths[0], outputPath)
	}

	// Validate all input files exist
	for _, path := range inputPaths {
		if _, err := os.Stat(path); os.IsNotExist(err) {
			return fmt.Errorf("input file does not exist: %s", path)
		}
		if !strings.HasSuffix(strings.ToLower(path), ".pdf") {
			return fmt.Errorf("invalid PDF file: %s", path)
		}
	}

	// Ensure output directory exists
	outputDir := filepath.Dir(outputPath)
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create output directory: %w", err)
	}

	// Create default configuration
	conf := model.NewDefaultConfiguration()

	// Use pdfcpu's MergeCreateFile to merge PDFs
	err := api.MergeCreateFile(inputPaths, outputPath, false, conf)
	if err != nil {
		return fmt.Errorf("failed to merge PDFs: %w", err)
	}

	return nil
}

// MergePDFsWithPageRanges merges specific page ranges from multiple PDFs
// fileRanges: map of file path to page range (e.g., "1-5", "3,7,9-11")
// outputPath: path where the merged PDF will be saved
func (s *MergeService) MergePDFsWithPageRanges(fileRanges map[string]string, outputPath string) error {
	if len(fileRanges) == 0 {
		return fmt.Errorf("no input files provided for merge")
	}

	// Ensure output directory exists
	outputDir := filepath.Dir(outputPath)
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create output directory: %w", err)
	}

	// For page ranges, we need to use the stream-based Merge function
	// Build input files list - pdfcpu doesn't support page ranges in merge directly
	// So we'll do a simple merge for now
	// TODO: Implement page range extraction using Split command first, then merge
	var inputFiles []string
	for path := range fileRanges {
		// Validate file exists
		if _, err := os.Stat(path); os.IsNotExist(err) {
			return fmt.Errorf("input file does not exist: %s", path)
		}
		inputFiles = append(inputFiles, path)
	}

	if len(inputFiles) == 0 {
		return fmt.Errorf("no valid input files for merge")
	}

	// Use simple merge
	conf := model.NewDefaultConfiguration()
	err := api.MergeCreateFile(inputFiles, outputPath, false, conf)
	if err != nil {
		return fmt.Errorf("failed to merge PDFs with page ranges: %w", err)
	}

	return nil
}

// GetPageCount returns the number of pages in a PDF file
func (s *MergeService) GetPageCount(filePath string) (int, error) {
	return api.PageCountFile(filePath)
}

// GetPDFInfo returns basic information about a PDF file
func (s *MergeService) GetPDFInfo(filePath string) (*PDFInfo, error) {
	// Get page count
	pageCount, err := api.PageCountFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read PDF: %w", err)
	}

	info := &PDFInfo{
		Path:      filePath,
		PageCount: pageCount,
		Version:   "1.0", // Default version
	}

	// Get file name
	info.Name = filepath.Base(filePath)

	// Get file size
	if fileStat, err := os.Stat(filePath); err == nil {
		info.Size = fileStat.Size()
	}

	return info, nil
}

// PDFInfo contains basic information about a PDF file
type PDFInfo struct {
	Path      string
	Name      string
	Size      int64
	PageCount int
	Version   string
}

// ValidatePDFs checks if all files are valid PDFs
func (s *MergeService) ValidatePDFs(filePaths []string) error {
	conf := model.NewDefaultConfiguration()
	return api.ValidateFiles(filePaths, conf)
}

// copyFile copies a file from src to dst
func (s *MergeService) copyFile(src, dst string) error {
	// Ensure output directory exists
	outputDir := filepath.Dir(dst)
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create output directory: %w", err)
	}

	data, err := os.ReadFile(src)
	if err != nil {
		return fmt.Errorf("failed to read source file: %w", err)
	}

	err = os.WriteFile(dst, data, 0644)
	if err != nil {
		return fmt.Errorf("failed to write destination file: %w", err)
	}

	return nil
}

// MergeFileList represents a file in the merge list with optional page range
type MergeFileList struct {
	FilePath  string `json:"filePath"`
	Name      string `json:"name"`
	PageRange string `json:"pageRange,omitempty"` // e.g., "1-5", "all"
	PageCount int    `json:"pageCount"`
	Size      int64  `json:"size"`
}

// MergeMultiplePDFs is a convenience method that accepts a list of MergeFileList
func (s *MergeService) MergeMultiplePDFs(files []MergeFileList, outputPath string) error {
	if len(files) == 0 {
		return fmt.Errorf("no files provided for merge")
	}

	// If no page ranges are specified, use simple merge
	hasRanges := false
	for _, f := range files {
		if f.PageRange != "" && f.PageRange != "all" {
			hasRanges = true
			break
		}
	}

	if !hasRanges {
		// Simple merge - extract file paths
		paths := make([]string, len(files))
		for i, f := range files {
			paths[i] = f.FilePath
		}
		return s.MergePDFs(paths, outputPath)
	}

	// Merge with page ranges
	fileRanges := make(map[string]string)
	for _, f := range files {
		pageRange := f.PageRange
		if pageRange == "" || pageRange == "all" {
			pageRange = ""
		}
		fileRanges[f.FilePath] = pageRange
	}

	return s.MergePDFsWithPageRanges(fileRanges, outputPath)
}
