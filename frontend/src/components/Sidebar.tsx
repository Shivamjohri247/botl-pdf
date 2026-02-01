import { useRef, useEffect } from 'react'
import { Plus, FileText, X } from 'lucide-react'
import type { PDFFile } from '../App'

interface SidebarProps {
  pdfFiles: PDFFile[]
  activePdfId: string | null
  currentPage: number
  totalPages: number
  onOpenFilePicker: () => void
  onRemovePDF: (id: string) => void
  onSelectPDF: (id: string) => void
  onSelectPage: (page: number) => void
}

export default function Sidebar({
  pdfFiles,
  activePdfId,
  currentPage,
  totalPages,
  onOpenFilePicker,
  onRemovePDF,
  onSelectPDF,
  onSelectPage,
}: SidebarProps) {
  const activePdf = pdfFiles.find(f => f.id === activePdfId)

  return (
    <aside className="w-[200px] bg-primary-sidebar border-r border-border flex flex-col shrink-0">
      {/* PDF Files Section */}
      <div className="p-3 border-b border-border shrink-0">
        <button
          onClick={onOpenFilePicker}
          className="w-full btn-primary flex items-center justify-center gap-2"
        >
          <Plus className="w-4 h-4" />
          Add PDF
        </button>
      </div>

      {/* PDF Files List */}
      <div className="flex-1 overflow-y-auto">
        {pdfFiles.length === 0 ? (
          <div className="p-4 text-center text-text-secondary text-sm">
            No PDF files added
          </div>
        ) : (
          <div className="p-2 space-y-1">
            {pdfFiles.map(file => (
              <div
                key={file.id}
                className={`group relative flex items-center gap-2 p-2 rounded cursor-pointer transition-colors ${
                  activePdfId === file.id
                    ? 'bg-accent-green/10 text-accent-green'
                    : 'hover:bg-white/50'
                }`}
                onClick={() => onSelectPDF(file.id)}
              >
                <FileText className="w-4 h-4 shrink-0" />
                <span className="flex-1 text-sm truncate">{file.name}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    onRemovePDF(file.id)
                  }}
                  className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger-red transition-opacity"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Pages Section */}
      {activePdf && (
        <>
          <div className="px-3 py-2 border-t border-border shrink-0">
            <h3 className="text-xs font-medium text-text-secondary uppercase tracking-wide">
              Pages
            </h3>
          </div>

          <div className="flex-1 overflow-y-auto p-2 min-h-0">
            {Array.from({ length: totalPages }, (_, i) => i + 1).map(pageNum => (
              <button
                key={pageNum}
                onClick={() => onSelectPage(pageNum)}
                className={`w-full aspect-[0.707] mb-2 rounded border-2 flex items-center justify-center text-sm transition-all ${
                  currentPage === pageNum
                    ? 'border-accent-green bg-accent-green/10 text-accent-green shadow-md'
                    : 'border-transparent bg-white hover:border-accent-sage'
                }`}
              >
                {pageNum}
              </button>
            ))}
          </div>
        </>
      )}
    </aside>
  )
}
