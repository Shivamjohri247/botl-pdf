import { useState, useCallback, useRef, useEffect } from 'react'
import { FileText, GripVertical, X, Plus, ChevronUp, ChevronDown, Download, Check } from 'lucide-react'
import { OpenFilePicker, SaveFileDialog, MergePDFs, GetPDFInfo } from '../../wailsjs/go/backend/App'
import type { PDFFile } from '../App'

export interface MergeFile {
  id: string
  path: string
  name: string
  size: number
  pageCount: number
}

interface MergeDialogProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  existingFiles?: PDFFile[]  // PDFs already loaded in the main app
  onMergeSuccess?: () => void
}

export default function MergeDialog({ isOpen, onOpenChange, existingFiles = [], onMergeSuccess }: MergeDialogProps) {
  const [files, setFiles] = useState<MergeFile[]>([])
  const [isProcessing, setIsProcessing] = useState(false)
  const [error, setError] = useState<string>('')
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)
  const dragStartIndex = useRef<number | null>(null)

  // Initialize file list with existing PDFs when dialog opens
  useEffect(() => {
    if (isOpen) {
      // Convert PDFFile[] to MergeFile[] - use originalPath for files loaded via file picker
      const initialFiles = existingFiles
        .filter(f => f.originalPath)  // Only include files with original file paths
        .map(f => ({
          id: f.id,
          path: f.originalPath!,  // Use original file path for merge operation
          name: f.name,
          size: f.size,
          pageCount: f.pages,
        } as MergeFile))

      setFiles(initialFiles)
      setError('')
    }
  }, [isOpen, existingFiles])

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(0) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  const handleAddFiles = useCallback(async () => {
    try {
      setError('')
      const filePaths = await OpenFilePicker()
      if (!filePaths || filePaths.length === 0) return

      const newFiles: MergeFile[] = []
      for (const path of filePaths) {
        // Check if file already in list
        if (files.some(f => f.path === path)) {
          continue
        }

        try {
          const info = await GetPDFInfo(path)
          newFiles.push({
            id: Date.now().toString() + Math.random(),
            path: path,
            name: info.Name || path.split('/').pop() || path.split('\\').pop() || 'Unknown',
            size: info.Size,
            pageCount: info.PageCount,
          })
        } catch (err) {
          console.error('Failed to get PDF info:', err)
          setError(`Failed to load PDF: ${path}`)
        }
      }

      if (newFiles.length > 0) {
        setFiles(prev => [...prev, ...newFiles])
      }
    } catch (err) {
      console.error('Failed to add files:', err)
      setError('Failed to open file picker')
    }
  }, [files])

  const handleRemoveFile = useCallback((id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id))
  }, [])

  const handleMoveUp = useCallback((index: number) => {
    if (index === 0) return
    setFiles(prev => {
      const newFiles = [...prev]
      ;[newFiles[index - 1], newFiles[index]] = [newFiles[index], newFiles[index - 1]]
      return newFiles
    })
  }, [])

  const handleMoveDown = useCallback((index: number) => {
    if (index === files.length - 1) return
    setFiles(prev => {
      const newFiles = [...prev]
      ;[newFiles[index], newFiles[index + 1]] = [newFiles[index + 1], newFiles[index]]
      return newFiles
    })
  }, [files.length])

  const handleMerge = useCallback(async () => {
    if (files.length < 2) {
      setError('Please add at least 2 PDF files to merge')
      return
    }

    try {
      setIsProcessing(true)
      setError('')

      // Get save path
      const defaultName = 'merged-document.pdf'
      const savePath = await SaveFileDialog(defaultName)

      if (!savePath) {
        setIsProcessing(false)
        return
      }

      // Extract file paths in order
      const filePaths = files.map(f => f.path)

      // Call backend merge
      await MergePDFs(filePaths, savePath)

      // Success - close dialog and notify
      setFiles([])
      onOpenChange(false)
      onMergeSuccess?.()
    } catch (err) {
      console.error('Failed to merge PDFs:', err)
      setError(err instanceof Error ? err.message : 'Failed to merge PDFs')
    } finally {
      setIsProcessing(false)
    }
  }, [files, onOpenChange])

  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    dragStartIndex.current = index
    setDraggedIndex(index)
    e.dataTransfer.effectAllowed = 'move'
    // Set a drag image if supported
    if (e.dataTransfer.setDragImage) {
      const target = e.target as HTMLElement
      e.dataTransfer.setDragImage(target, 0, 0)
    }
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent, index: number) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'

    // Just track which index we're hovering over for visual feedback
    if (draggedIndex !== null && draggedIndex !== index) {
      setDragOverIndex(index)
    }
  }, [draggedIndex])

  const handleDragLeave = useCallback(() => {
    setDragOverIndex(null)
  }, [])

  const handleDrop = useCallback((e: React.DragEvent, dropIndex: number) => {
    e.preventDefault()

    const startIndex = dragStartIndex.current
    if (startIndex === null || startIndex === dropIndex) {
      setDraggedIndex(null)
      setDragOverIndex(null)
      return
    }

    // Reorder the files
    setFiles(prev => {
      const newFiles = [...prev]
      const [removed] = newFiles.splice(startIndex, 1)
      newFiles.splice(dropIndex, 0, removed)
      return newFiles
    })

    // Update the dragged index to the new position
    setDraggedIndex(dropIndex)
    dragStartIndex.current = dropIndex
    setDragOverIndex(null)
  }, [])

  const handleDragEnd = useCallback(() => {
    setDraggedIndex(null)
    setDragOverIndex(null)
    dragStartIndex.current = null
  }, [])

  // Calculate total pages and size
  const totalPages = files.reduce((sum, f) => sum + f.pageCount, 0)
  const totalSize = files.reduce((sum, f) => sum + f.size, 0)

  // Don't render anything if not open
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => onOpenChange(false)}
      />

      {/* Dialog */}
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] flex flex-col mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">Merge PDF Files</h2>
            <p className="text-sm text-gray-500 mt-1">
              Combine multiple PDF files into a single document. Drag to reorder.
            </p>
          </div>
          <button
            onClick={() => onOpenChange(false)}
            disabled={isProcessing}
            className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-6">
          {/* File List */}
          <div className="border border-gray-200 rounded-lg overflow-hidden">
            <div className="bg-gray-50 px-4 py-3 border-b border-gray-200 flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">
                Files to merge ({files.length})
              </span>
              <button
                onClick={handleAddFiles}
                className="flex items-center gap-2 px-3 py-1.5 text-sm bg-emerald-500 text-white rounded-lg hover:bg-emerald-600 transition-colors"
              >
                <Plus className="w-4 h-4" />
                Add Files
              </button>
            </div>

            <div className="max-h-80 overflow-y-auto">
              {files.length === 0 ? (
                <div className="p-8 text-center text-gray-500">
                  <FileText className="w-12 h-12 mx-auto mb-3 text-gray-400" />
                  <p className="text-sm">No files added yet</p>
                  <p className="text-xs mt-1">Click "Add Files" to select PDFs</p>
                </div>
              ) : (
                <div className="divide-y divide-gray-200">
                  {files.map((file, index) => (
                    <div
                      key={file.id}
                      draggable
                      onDragStart={(e) => handleDragStart(e, index)}
                      onDragOver={(e) => handleDragOver(e, index)}
                      onDragLeave={handleDragLeave}
                      onDrop={(e) => handleDrop(e, index)}
                      onDragEnd={handleDragEnd}
                      className={`flex items-center gap-3 px-4 py-3 transition-colors cursor-move ${
                        draggedIndex === index
                          ? 'bg-emerald-100 border-2 border-emerald-500'
                          : dragOverIndex === index
                            ? 'bg-emerald-50 border-2 border-dashed border-emerald-300'
                            : 'hover:bg-gray-50'
                      }`}
                    >
                      {/* Drag handle */}
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-medium text-gray-400 w-4">
                          {index + 1}
                        </span>
                        <GripVertical className="w-5 h-5 text-gray-400 cursor-grab active:cursor-grabbing" />
                      </div>
                      <FileText className="w-5 h-5 text-gray-500 shrink-0" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 truncate">{file.name}</p>
                        <p className="text-xs text-gray-500">
                          {file.pageCount} {file.pageCount === 1 ? 'page' : 'pages'} • {formatFileSize(file.size)}
                        </p>
                      </div>
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => handleMoveUp(index)}
                          disabled={index === 0}
                          className="p-1.5 text-gray-400 hover:text-gray-600 disabled:opacity-30 disabled:cursor-not-allowed rounded hover:bg-gray-100"
                          title="Move up"
                        >
                          <ChevronUp className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleMoveDown(index)}
                          disabled={index === files.length - 1}
                          className="p-1.5 text-gray-400 hover:text-gray-600 disabled:opacity-30 disabled:cursor-not-allowed rounded hover:bg-gray-100"
                          title="Move down"
                        >
                          <ChevronDown className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleRemoveFile(file.id)}
                          className="p-1.5 text-gray-400 hover:text-red-500 rounded hover:bg-gray-100"
                          title="Remove"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Summary */}
            {files.length > 0 && (
              <div className="px-4 py-2 bg-gray-50 border-t border-gray-200 flex justify-between text-sm">
                <span className="text-gray-600">
                  Total: {files.length} {files.length === 1 ? 'file' : 'files'}, {totalPages} {totalPages === 1 ? 'page' : 'pages'}
                </span>
                <span className="text-gray-600">{formatFileSize(totalSize)}</span>
              </div>
            )}
          </div>

          {/* Error Message */}
          {error && (
            <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-200">
          <button
            onClick={() => onOpenChange(false)}
            disabled={isProcessing}
            className="px-4 py-2 text-sm font-medium text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Cancel
          </button>
          <button
            onClick={handleMerge}
            disabled={files.length < 2 || isProcessing}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-emerald-500 rounded-lg hover:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isProcessing ? (
              <>
                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Merging...
              </>
            ) : (
              <>
                <Download className="w-4 h-4" />
                Merge & Save
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
