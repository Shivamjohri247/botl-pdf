import {
  MousePointer2,
  Type,
  Highlighter,
  ChevronLeft,
  ChevronRight,
  Minus,
  Plus,
  RotateCcw,
  RotateCw,
  Maximize2,
} from 'lucide-react'

interface ToolbarProps {
  currentTool: 'select' | 'text' | 'highlight'
  currentPage: number
  totalPages: number
  zoom: number
  onToolChange: (tool: 'select' | 'text' | 'highlight') => void
  onPagePrev: () => void
  onPageNext: () => void
  onPageJump: (page: number) => void
  onZoomIn: () => void
  onZoomOut: () => void
  onZoomFit: () => void
  onRotateLeft: () => void
  onRotateRight: () => void
}

export default function Toolbar({
  currentTool,
  currentPage,
  totalPages,
  zoom,
  onToolChange,
  onPagePrev,
  onPageNext,
  onPageJump,
  onZoomIn,
  onZoomOut,
  onZoomFit,
  onRotateLeft,
  onRotateRight,
}: ToolbarProps) {
  const tools = [
    { id: 'select' as const, icon: MousePointer2, label: 'Select' },
    { id: 'text' as const, icon: Type, label: 'Text' },
    { id: 'highlight' as const, icon: Highlighter, label: 'Highlight' },
  ]

  const handlePageInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const page = parseInt(e.target.value)
    if (!isNaN(page) && page >= 1 && page <= totalPages) {
      onPageJump(page)
    }
  }

  // Common button class for all toolbar buttons
  const buttonClass = `
    btn-toolbar
    min-w-[2rem]
    h-8
    sm:h-9
    flex
    items-center
    justify-center
    transition-all
    duration-150
  `

  return (
    <div className="h-12 sm:h-14 bg-white border-t border-border flex items-center justify-between px-2 sm:px-4 shrink-0 flex-shrink-0">
      {/* Tools Section */}
      <div className="flex items-center gap-0.5 sm:gap-1">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => onToolChange(tool.id)}
            className={`${buttonClass} ${currentTool === tool.id ? 'active' : ''}`}
            title={tool.label}
            aria-label={tool.label}
            aria-pressed={currentTool === tool.id}
          >
            <tool.icon className="w-4 h-4" />
          </button>
        ))}
      </div>

      {/* Page Navigation - hide on very small screens, show on sm+ */}
      <div className="flex items-center gap-0.5 sm:gap-1">
        <button
          onClick={onPagePrev}
          disabled={currentPage <= 1}
          className={`${buttonClass} disabled:opacity-50 disabled:cursor-not-allowed`}
          title="Previous page"
          aria-label="Previous page"
        >
          <ChevronLeft className="w-4 h-4" />
        </button>
        <div className="flex items-center gap-1 px-1 sm:px-3">
          <input
            type="number"
            min={1}
            max={totalPages}
            value={currentPage}
            onChange={handlePageInput}
            disabled={totalPages === 0}
            className="w-10 sm:w-12 text-center bg-transparent border-none outline-none text-sm"
            aria-label="Current page"
          />
          <span className="text-text-secondary text-sm">/ {totalPages}</span>
        </div>
        <button
          onClick={onPageNext}
          disabled={currentPage >= totalPages || totalPages === 0}
          className={`${buttonClass} disabled:opacity-50 disabled:cursor-not-allowed`}
          title="Next page"
          aria-label="Next page"
        >
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>

      {/* Zoom Controls */}
      <div className="flex items-center gap-0.5 sm:gap-1">
        <button
          onClick={onZoomOut}
          className={buttonClass}
          title="Zoom out"
          aria-label="Zoom out"
        >
          <Minus className="w-4 h-4" />
        </button>
        <span className="w-10 sm:w-12 text-center text-sm text-text-secondary">{zoom}%</span>
        <button
          onClick={onZoomIn}
          className={buttonClass}
          title="Zoom in"
          aria-label="Zoom in"
        >
          <Plus className="w-4 h-4" />
        </button>
        <button
          onClick={onZoomFit}
          className={buttonClass}
          title="Fit to page"
          aria-label="Fit to page"
        >
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>

      {/* Rotate Controls */}
      <div className="flex items-center gap-0.5 sm:gap-1">
        <button
          onClick={onRotateLeft}
          className={buttonClass}
          title="Rotate left"
          aria-label="Rotate left"
        >
          <RotateCcw className="w-4 h-4" />
        </button>
        <button
          onClick={onRotateRight}
          className={buttonClass}
          title="Rotate right"
          aria-label="Rotate right"
        >
          <RotateCw className="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
