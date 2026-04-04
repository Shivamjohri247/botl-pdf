# Architecture

## Overview

botl-pdf is a hybrid Rust + Python library for high-performance PDF processing.
The core parsing and text extraction engine is written in Rust, with Python
bindings provided via PyO3 and distributed as wheels through maturin.

```
┌─────────────────────────────────────────┐
│             Python API Layer            │
│  (botl_pdf package, CLI, plugins, OCR)  │
├─────────────────────────────────────────┤
│           PyO3 Bindings (_core)         │
│     (rust/botl-pdf-python crate)        │
├─────────────────────────────────────────┤
│            Rust Core Engine             │
│  (rust/botl-pdf-core crate)            │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │  Parser   │ │  Codecs  │ │  Text   │ │
│  │ (lexer,   │ │ (flate,  │ │ (cmap,  │ │
│  │  objects, │ │  lzw,    │ │  fonts, │ │
│  │  xref,    │ │  ascii,  │ │  oper-  │ │
│  │  doc)     │ │  ...)    │ │  ators) │ │
│  └──────────┘ └──────────┘ └─────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │  Layout   │ │ Geometry │ │  Error  │ │
│  │ (group-  │ │ (bbox,   │ │  types  │ │
│  │  ing,    │ │  matrix, │ │         │ │
│  │  order)  │ │  spatial)│ │         │ │
│  └──────────┘ └──────────┘ └─────────┘ │
├─────────────────────────────────────────┤
│          C Codec Layer (optional)       │
│  (rust/botl-pdf-csys crate)            │
│  JPEG (DCTDecode), JPEG2000 (JPXDecode)│
└─────────────────────────────────────────┘
```

## Crate Structure

### botl-pdf-core

The main Rust crate implementing all PDF processing logic:

- **parser** — PDF lexical analysis, object model, cross-reference table parsing,
  document structure traversal, and incremental update handling
- **codecs** — Stream decompression: FlateDecode, ASCII85Decode, ASCIIHexDecode,
  LZWDecode, RunLengthDecode
- **text** — CMap parsing, font metrics extraction, content stream interpretation,
  Unicode mapping
- **layout** — Character-to-word-to-line-to-block grouping, reading order
  detection, spatial text reconstruction
- **geometry** — Bounding boxes, affine transformation matrices, R-tree spatial
  indexing
- **error** — Unified error type for all PDF operations

### botl-pdf-csys

C FFI layer for image codecs that don't have pure Rust equivalents:

- JPEG decompression (DCTDecode) via libjpeg-turbo
- JPEG2000 decompression (JPXDecode) via OpenJPEG

This crate is optional and gated behind the `c-codecs` feature flag.

### botl-pdf-python

PyO3 bindings exposing the Rust core to Python:

- `PyDocument` — PDF document wrapper
- `PyPage` — Page with text extraction and character data
- `PyChar`, `PyWord`, `PyTextLine`, `PyTextBlock` — Layout elements
- `PyBBox`, `PyGeomLine`, `PyGeomRect` — Geometry types
- `PyTOCEntry` — Table of contents entries
- All CPU-intensive methods release the Python GIL

## Python Package Structure

```
botl_pdf/
├── __init__.py       # Re-exports from _core, open() wrapper
├── _core.pyi         # Type stubs for Rust extension
├── document.py       # High-level Document and PageCollection wrappers
├── page.py           # High-level Page wrapper with layout params
├── export.py         # to_markdown() and to_text() convenience functions
├── tables.py         # Table detection data models
├── debug.py          # Visual debugging with Pillow
├── cli/              # CLI (typer with argparse fallback)
├── ocr/              # OCR backend abstraction
└── plugins/          # Plugin registry for exporters and table strategies
```

## Text Extraction Pipeline

```
PDF File
  → Parser (lexer → objects → xref → page tree)
    → Content Stream (decompress if needed)
      → Content Stream Interpreter (operator parsing, graphics state machine)
        → Positioned Characters (Char with BBox, font info)
          → Layout Analysis
            → chars_to_words (gap < word_margin × font_size)
            → words_to_lines (vertical overlap > 50%)
            → lines_to_blocks (gap < line_margin × avg_height)
            → Reading order sort (boxes_flow parameter)
          → Text output (plain or layout-preserving)
```

## Build System

- **maturin** builds the Rust extension as a native Python module (`_core`)
- Uses abi3-py310 for forward-compatible wheels (one wheel per platform, works
  with all Python 3.10+)
- Cross-platform wheels via maturin-action: manylinux, macOS (x86_64 + arm64),
  Windows

## Performance Considerations

- **Lazy object resolution** — PDF objects are parsed on first access and cached
- **GIL release** — All CPU-intensive Rust operations release the Python GIL
- **Arena allocation** — bumpalo for temporary parser allocations
- **Hash maps** — hashbrown for faster lookups than std HashMap
- **Spatial indexing** — R-tree via rstar for layout queries
- **mmap I/O** — Memory-mapped file reading for large PDFs (when using file paths)
