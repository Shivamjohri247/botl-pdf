import { useState, useCallback, useRef, useEffect } from 'react'
import { Document, Page, pdfjs } from 'react-pdf'
import { FileText, Edit3, Loader2 } from 'lucide-react'
import type { PDFFile } from '../App'

// Set up PDF.js worker
pdfjs.GlobalWorkerOptions.workerSrc = 'pdf.worker.min.mjs'

interface EditableTextItem {
  id: string
  originalText: string
  x: number
  y: number
  width: number
  height: number
  fontSize: number
  fontFamily: string
  color: string
  isEditing: boolean
}

interface PDFViewerProps {
  pdfFile: PDFFile | null
  currentPage: number
  zoom: number
  rotation: number
  isEditMode: boolean
  currentTool: 'select' | 'text' | 'highlight'
  onPageCountChange?: (numPages: number) => void
}

export default function PDFViewer({
  pdfFile,
  currentPage,
  zoom,
  rotation,
  isEditMode,
  currentTool,
  onPageCountChange,
}: PDFViewerProps) {
  const [numPages, setNumPages] = useState<number>(0)
  const [pdfError, setPdfError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [workerReady, setWorkerReady] = useState(false)
  const [pageWidth, setPageWidth] = useState<number>(700)
  const [textItems, setTextItems] = useState<EditableTextItem[]>([])
  const [selectedTextId, setSelectedTextId] = useState<string | null>(null)

  const containerRef = useRef<HTMLDivElement>(null)
  const pdfWrapperRef = useRef<HTMLDivElement>(null)
  const textLayerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    setWorkerReady(true)
  }, [])

  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.clientWidth - 64
        setPageWidth(Math.max(400, Math.min(containerWidth, 900)))
      }
    }
    updateDimensions()
    window.addEventListener('resize', updateDimensions)
    return () => window.removeEventListener('resize', updateDimensions)
  }, [])

  useEffect(() => {
    setNumPages(0)
    setPdfError(null)
    setIsLoading(true)
    setTextItems([])
  }, [pdfFile?.id])

  useEffect(() => {
    if (numPages > 0 && onPageCountChange) {
      onPageCountChange(numPages)
    }
  }, [numPages, onPageCountChange])

  const onDocumentLoadSuccess = useCallback(({ numPages: loadedNumPages }: { numPages: number }) => {
    setNumPages(loadedNumPages)
    setIsLoading(false)
    setPdfError(null)
  }, [])

  const onDocumentLoadError = useCallback((error: Error) => {
    console.error('PDF Load Error:', error)
    setPdfError(error.message || 'Failed to load PDF')
    setIsLoading(false)
  }, [])

  const onPageLoadSuccess = useCallback((page: any) => {
    setIsLoading(false)

    // Extract text items with positions for editing
    if (page && page.textLayer && currentTool === 'text') {
      const textContent = page.textLayer
      const items: EditableTextItem[] = []

      // Get all text items from the text layer
      const textItems = textContent.querySelectorAll('.react-pdf__Page__textLayer span')
      textItems.forEach((item: any, index: number) => {
        const rect = item.getBoundingClientRect()
        const parentRect = textContent.getBoundingClientRect()

        items.push({
          id: `text-${index}-${Date.now()}`,
          originalText: item.textContent || '',
          x: rect.left - parentRect.left,
          y: rect.top - parentRect.top,
          width: rect.width,
          height: rect.height,
          fontSize: parseFloat(window.getComputedStyle(item).fontSize) || 12,
          fontFamily: window.getComputedStyle(item).fontFamily || 'Helvetica',
          color: window.getComputedStyle(item).color || '#000000',
          isEditing: false,
        })
      })

      setTextItems(items)
    }
  }, [currentTool])

  const onPageLoadError = useCallback((error: Error) => {
    console.error('Page Load Error:', error)
    setIsLoading(false)
  }, [])

  const handleTextClick = useCallback((id: string) => {
    if (currentTool === 'text') {
      setTextItems(prev => prev.map(item =>
        item.id === id ? { ...item, isEditing: true } : item
      ))
      setSelectedTextId(id)
    }
  }, [currentTool])

  const handleTextBlur = useCallback((id: string, newText: string) => {
    setTextItems(prev => prev.map(item =>
      item.id === id ? { ...item, isEditing: false, originalText: newText } : item
    ))
  }, [])

  const formatText = useCallback((command: string, value?: string) => {
    document.execCommand(command, false, value)
  }, [])

  if (!pdfFile) {
    return (
      <div className="flex-1 flex items-center justify-center bg-[#F5F7F6]">
        <div className="text-center">
          <FileText className="w-16 h-16 mx-auto text-text-muted mb-4" />
          <h2 className="text-lg font-medium text-text-primary mb-2">No PDF Selected</h2>
          <p className="text-text-secondary text-sm">Add a PDF file to get started</p>
        </div>
      </div>
    )
  }

  if (pdfError) {
    return (
      <div className="flex-1 flex items-center justify-center bg-[#F5F7F6]">
        <div className="text-center">
          <FileText className="w-16 h-16 mx-auto text-danger-red mb-4" />
          <h2 className="text-lg font-medium text-text-primary mb-2">Failed to Load PDF</h2>
          <p className="text-text-secondary text-sm mb-4">{pdfError}</p>
          <p className="text-text-muted text-xs">{pdfFile.name}</p>
        </div>
      </div>
    )
  }

  if (!workerReady) {
    return (
      <div className="flex-1 flex items-center justify-center bg-[#F5F7F6]">
        <div className="flex items-center gap-2 text-accent-green">
          <Loader2 className="w-5 h-5 animate-spin" />
          <span className="text-sm">Initializing...</span>
        </div>
      </div>
    )
  }

  const scale = zoom / 100
  const isEditing = currentTool === 'text'

  return (
    <div className="relative flex-1">
      <div
        className="flex-1 flex items-center justify-center bg-[#F5F7F6] overflow-auto p-8"
        ref={containerRef}
      >
        <div className="flex items-center justify-center min-w-full min-h-full">
          <div
            ref={pdfWrapperRef}
            className={`relative bg-white shadow-xl transition-all ${isEditing ? 'border-2 border-dashed border-accent-green' : ''}`}
            style={{
              width: `${pageWidth}px`,
              transform: `scale(${scale})`,
              transformOrigin: 'top center',
            }}
          >
            {/* Edit Mode Badge */}
            {isEditing && (
              <div className="absolute -top-10 left-0 bg-accent-green text-white px-3 py-1 rounded text-sm font-medium flex items-center gap-1 z-20">
                <Edit3 className="w-3 h-3" />
                Edit Mode - Click text to edit
              </div>
            )}

            {/* Loading overlay */}
            {isLoading && numPages === 0 && (
              <div className="absolute inset-0 flex items-center justify-center bg-white/90 z-10">
                <div className="flex items-center gap-2 text-accent-green">
                  <Loader2 className="w-5 h-5 animate-spin" />
                  <span className="text-sm">Loading...</span>
                </div>
              </div>
            )}

            {/* PDF Document */}
            <Document
              file={pdfFile.path}
              onLoadSuccess={onDocumentLoadSuccess}
              onLoadError={onDocumentLoadError}
              className="block"
            >
              <Page
                pageNumber={currentPage}
                width={pageWidth}
                rotate={rotation}
                renderTextLayer={true}
                renderAnnotationLayer={false}
                onLoadSuccess={onPageLoadSuccess}
                onError={onPageLoadError}
                loading={null}
                error={null}
              />
            </Document>
          </div>
        </div>
      </div>

      {/* Floating Format Toolbar when text selected */}
      {selectedTextId && isEditing && (
        <div className="fixed bottom-20 left-1/2 -translate-x-1/2 bg-gray-900 text-white px-3 py-2 rounded-lg shadow-lg flex items-center gap-2 z-50">
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded font-bold"
            onClick={() => formatText('bold')}
            title="Bold"
          >
            B
          </button>
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded italic"
            onClick={() => formatText('italic')}
            title="Italic"
          >
            I
          </button>
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded underline"
            onClick={() => formatText('underline')}
            title="Underline"
          >
            U
          </button>
          <div className="w-px h-5 bg-white/20" />
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded text-red-500"
            onClick={() => formatText('foreColor', '#ef4444')}
            title="Red"
          >
            A
          </button>
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded text-green-500"
            onClick={() => formatText('foreColor', '#10b981')}
            title="Green"
          >
            A
          </button>
          <button
            className="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded text-white"
            onClick={() => formatText('foreColor', '#000000')}
            title="Black"
          >
            A
          </button>
        </div>
      )}
    </div>
  )
}
