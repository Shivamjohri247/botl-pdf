import {
  FileText,
  Scissors,
  Download,
  Trash2,
  Lock,
  Unlock,
  Type,
  Minimize2,
} from 'lucide-react'
import type { PDFFile } from '../App'

interface RightPanelProps {
  pdfFile: PDFFile | null
  isEditMode: boolean
  onMerge: () => void
  onSplit: () => void
  onExtract: () => void
  onDelete: () => void
  onProtect: () => void
  onUnlock: () => void
  onWatermark: () => void
  onCompress: () => void
}

export default function RightPanel({
  pdfFile,
  onMerge,
  onSplit,
  onExtract,
  onDelete,
  onProtect,
  onUnlock,
  onWatermark,
  onCompress,
}: RightPanelProps) {
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(0) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  const handleToolAction = (actionName: string, action: () => void) => {
    if (!pdfFile) {
      alert(`Please open a PDF file first to use the ${actionName} tool.`)
      return
    }
    action()
  }

  const tools = [
    { icon: FileText, label: 'Merge', action: () => handleToolAction('Merge', onMerge) },
    { icon: Scissors, label: 'Split', action: () => handleToolAction('Split', onSplit) },
    { icon: Download, label: 'Extract', action: () => handleToolAction('Extract', onExtract) },
    { icon: Trash2, label: 'Delete', action: () => handleToolAction('Delete', onDelete) },
    { icon: Lock, label: 'Protect', action: () => handleToolAction('Protect', onProtect) },
    { icon: Unlock, label: 'Unlock', action: () => handleToolAction('Unlock', onUnlock) },
    { icon: Type, label: 'Watermark', action: () => handleToolAction('Watermark', onWatermark) },
    { icon: Minimize2, label: 'Compress', action: () => handleToolAction('Compress', onCompress) },
  ]

  return (
    <aside className="w-[280px] bg-white border-l border-border flex flex-col shrink-0 min-w-[280px]">
      {/* Text Formatting Section (shown in edit mode) */}
      <div className="p-4 border-b border-border">
        <h3 className="text-xs font-medium text-text-secondary uppercase tracking-wide mb-3">
          Text Formatting
        </h3>
        <div className="flex gap-2">
          <button className="btn-toolbar w-9 h-9 font-bold">B</button>
          <button className="btn-toolbar w-9 h-9 italic">I</button>
          <button className="btn-toolbar w-9 h-9 underline">U</button>
        </div>
      </div>

      {/* Document Actions Section */}
      <div className="p-4 flex-1 overflow-y-auto min-h-0">
        <h3 className="text-xs font-medium text-text-secondary uppercase tracking-wide mb-3">
          Document Actions
        </h3>
        <div className="grid grid-cols-4 gap-2">
          {tools.map((tool) => (
            <button
              key={tool.label}
              onClick={tool.action}
              disabled={!pdfFile}
              className="flex flex-col items-center gap-1 p-3 rounded border border-border hover:border-accent-green hover:bg-accent-green/5 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              title={tool.label}
            >
              <tool.icon className="w-5 h-5 text-text-secondary shrink-0" />
              <span className="text-[10px] text-text-secondary text-center leading-tight">{tool.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* File Info Section */}
      {pdfFile && (
        <div className="p-4 bg-primary-sidebar border-t border-border shrink-0">
          <div className="flex items-center gap-2 mb-2">
            <FileText className="w-4 h-4 text-text-muted shrink-0" />
            <span className="text-sm font-medium text-text-primary truncate">
              {pdfFile.name}
            </span>
          </div>
          <p className="text-xs text-text-secondary">
            {pdfFile.pages} {pdfFile.pages === 1 ? 'page' : 'pages'} • {formatFileSize(pdfFile.size)}
          </p>
        </div>
      )}
    </aside>
  )
}
