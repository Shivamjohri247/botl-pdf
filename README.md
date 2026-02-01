# Botl PDF

> A cross-platform desktop PDF editor built with Wails, Go, and React.

![Go Version](https://img.shields.io/badge/Go-1.24+-00ADD8?style=flat&logo=go)
![React](https://img.shields.io/badge/React-18-22272?style=flat&logo=react)
![Wails](https://img.shields.io/badge/Wails-v2.11-41B883?style=flat&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAABSCAMAAADJr4LRAAAAGXBMVEUAAABUVFRgYGCAgMjIyRkZGSEhIVFRUVFBQUFBQUICAgHBwcICAgISEhIiMjIyKioqLi4uMTExISEhISEhNzc3OTk5PT09QUFBQUFBgYGBgYICAgHx8fIiIiIiIiIiIiHx8fHyIiIiIjIiIiIiIiHx8fHyMjIyMiIyIiIyMjIyP///9WLvRaAAACnUlEQVR42u3BaQqAMAwEwOz9255L1yy61a4TWwQLgZJQwAAQABJREFUeJztwENAAAIBKCA/gA+19O+aIAAAAAASUVORK5CYII=)

## Features

### Current Features ✅

- **PDF Viewing** - View PDF documents with react-pdf
- **Page Navigation** - Jump to any page, previous/next navigation
- **Zoom Controls** - Zoom in/out, fit to page
- **Rotation** - Rotate pages 90° left/right
- **Merge PDFs** - Combine multiple PDFs into one document
  - Drag & drop to reorder files
  - Visual feedback during reordering
  - Auto-populates with loaded PDFs
- **Notification System** - Toast notifications and bell icon for updates
- **Recent Files** - Track recently opened documents

### Planned Features 🚧

- **Split PDF** - Extract pages or split by ranges
- **Delete Pages** - Remove unwanted pages
- **Reorder Pages** - Drag and drop page thumbnails
- **Watermark** - Add text or image watermarks
- **Encrypt/Decrypt** - Password protect PDFs
- **Compress** - Optimize PDF file size
- **PDF to Image** - Export pages as images
- **Text Extraction** - Extract text content

## Screenshots

<div align="center">
  <img src="design-preview.html" alt="Botl PDF Interface" width="800">
</div>

## Tech Stack

| Component | Technology |
|-----------|------------|
| **Desktop Framework** | [Wails v2](https://wails.io/) |
| **Backend** | Go 1.24+ |
| **PDF Processing** | [pdfcpu](https://pdfcpu.io/) |
| **Frontend** | React 18 + TypeScript |
| **Styling** | Tailwind CSS v4 |
| **PDF Viewing** | react-pdf (PDF.js wrapper) |
| **Icons** | Lucide React |

## Installation

### Prerequisites

- **Go** 1.24 or later
- **Node.js** 18 or later
- **Wails CLI** - Install with `go install github.com/wailsapp/wails/v2/cmd/wails@latest`

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Shivamjohri247/botl-pdf.git
cd botl-pdf

# Install Go dependencies
go mod download

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run in development mode
wails dev
```

### Build for Production

```bash
# Build for your current platform
wails build

# Build for specific platforms
wails build -platform darwin/amd64    # macOS Intel
wails build -platform darwin/arm64    # macOS Apple Silicon
wails build -platform windows/amd64   # Windows
wails build -platform linux/amd64      # Linux
```

## Development

```bash
# Start development server with hot reload
wails dev

# The frontend will be served at http://localhost:5173
# The Wails dev server is at http://localhost:34115
```

## Project Structure

```
botl-pdf/
├── frontend/                 # React frontend
│   └── src/
│       ├── components/      # UI components
│       ├── contexts/        # React contexts (notifications)
│       └── App.tsx          # Main app component
├── backend/                 # Go backend package
│   ├── app.go              # Wails app with exposed methods
│   └── services/           # Business logic
│       ├── storage.go      # Recent files management
│       └── merge.go        # PDF merge operations
├── main.go                 # Wails entry point
├── wails.json             # Wails configuration
├── PLAN.md                # Implementation plan
└── README.md              # This file
```

## Usage

### Opening PDFs

1. Click the **"Add PDF"** button in the left sidebar
2. Select one or more PDF files using the native file picker
3. The PDF will appear in the viewer with page thumbnails

### Merging PDFs

1. Open two or more PDF files using "Add PDF"
2. Click the **"Merge"** button in the right panel
3. Reorder files by dragging or using the up/down arrows
4. Click **"Merge & Save"** and choose the output location
5. A success notification will appear when complete

## Roadmap

See [PLAN.md](PLAN.md) for the complete implementation roadmap.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Acknowledgments

Built with:
- [Wails](https://wails.io/) - Cross-platform desktop apps with Go
- [pdfcpu](https://pdfcpu.io/) - PDF processing library
- [react-pdf](https://react-pdf.org/) - PDF viewing in React
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS framework
