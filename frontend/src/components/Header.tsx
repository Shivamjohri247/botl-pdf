import { FileText, Save, Download } from 'lucide-react'
import { NotificationBell } from './Notifications'
import type { PDFFile } from '../App'

interface HeaderProps {
  currentPdf: PDFFile | null
  onSave: () => void
  onExport: () => void
}

export default function Header({ currentPdf, onSave, onExport }: HeaderProps) {
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  return (
    <header className="h-14 bg-white border-b border-border flex items-center justify-between px-4 shrink-0">
      <div className="flex items-center gap-3">
        {/* Logo */}
        <div className="w-8 h-8 bg-accent-green rounded flex items-center justify-center">
          <span className="text-white font-bold text-sm">B</span>
        </div>
        <h1 className="text-lg font-semibold text-text-primary">Botl PDF</h1>

        {/* Current document */}
        {currentPdf && (
          <>
            <span className="text-text-muted ml-2">/</span>
            <div className="flex items-center gap-2 text-text-secondary text-sm">
              <FileText className="w-4 h-4" />
              <span>{currentPdf.name}</span>
              <span className="text-text-muted">({formatFileSize(currentPdf.size)})</span>
            </div>
          </>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        <NotificationBell />
        <button
          onClick={onSave}
          disabled={!currentPdf}
          className="btn-secondary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Save className="w-4 h-4" />
          Save
        </button>
        <button
          onClick={onExport}
          disabled={!currentPdf}
          className="btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Download className="w-4 h-4" />
          Export
        </button>
      </div>
    </header>
  )
}
