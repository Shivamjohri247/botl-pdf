# botl-pdf

High-performance PDF text extraction library with a custom Rust core and Python bindings.

## Features

- Fast PDF text extraction with layout analysis
- Character-level output with bounding boxes, fonts, and styles
- Layout-preserving text extraction (spatial whitespace)
- Table of contents (TOC/outline) extraction with page numbers
- Document metadata extraction (title, author, dates, etc.)
- Geometric element extraction (lines, rectangles)
- Configurable layout parameters (word spacing, line grouping, reading order)
- Pythonic API with type hints throughout
- CLI for common operations
- Zero external PDF library dependencies (custom parser)

## Install

```bash
pip install botl-pdf
```

Or build from source (requires Rust toolchain):

```bash
pip install maturin
git clone https://github.com/botl-pdf/botl-pdf.git
cd botl-pdf
maturin develop --release
```

## Quick Start

```python
import botl_pdf

# Open a document
with botl_pdf.open("report.pdf") as doc:
    # Extract plain text from a page
    text = doc.pages[0].extract_text()
    print(text)

    # Layout-preserving extraction (maintains spatial positioning)
    text = doc.pages[0].extract_text(layout=True)

    # Character-level access with bounding boxes
    for char in doc.pages[0].chars:
        print(f"{char.text!r} at ({char.bbox.x0:.1f}, {char.bbox.y0:.1f}) "
              f"font={char.font_name} size={char.font_size:.1f}")

    # Document metadata
    print(doc.metadata)
    print(f"Pages: {doc.num_pages}")

    # Table of contents
    for entry in doc.toc:
        print(f"  {'  ' * entry.level}{entry.title} (p.{entry.page_number})")
```

## API Reference

### `botl_pdf.open(path_or_bytes, *, password=None, lazy=True) -> Document`

Open a PDF from a file path or bytes.

### `Document`

| Property / Method | Description |
|---|---|
| `.metadata` | Dict with title, author, subject, keywords, etc. |
| `.num_pages` | Number of pages |
| `.is_encrypted` | Whether the document is encrypted |
| `.toc` | List of `TOCEntry` objects (outline / bookmarks) |
| `.pages` | `PageCollection` (iterable, subscriptable) |
| `doc[i]` | Shortcut for `doc.pages[i]` |

### `Page` (accessed via `doc.pages[i]`)

| Property / Method | Description |
|---|---|
| `.extract_text(layout=False, layout_params=None)` | Extract text (plain or layout-preserved) |
| `.chars` | List of `Char` objects with full style info |
| `.lines` | List of geometric `GeomLine` objects |
| `.rects` | List of geometric `GeomRect` objects |
| `.width` / `.height` | Page dimensions in points |
| `.rotation` | Rotation in degrees (0, 90, 180, 270) |
| `.page_number` | Zero-based page index |
| `.label` | Page label string (e.g. "iii", "A-1") |

### `LayoutParams(word_margin=2.0, line_margin=0.5, boxes_flow=0.5)`

| Parameter | Default | Description |
|---|---|---|
| `word_margin` | 2.0 | Max horizontal gap between chars in same word, as a multiple of font size |
| `line_margin` | 0.5 | Max vertical gap between lines in same block, as a multiple of line height |
| `boxes_flow` | 0.5 | Reading-order strictness (0.0 = horizontal, 1.0 = vertical) |

```python
import botl_pdf

doc = botl_pdf.open("report.pdf")
params = botl_pdf.LayoutParams(word_margin=2.0, line_margin=0.5)
text = doc[0].extract_text(layout=True, layout_params=params)
```

## Architecture

```
PDF bytes
  → Parser (nom tokenizer + recursive-descent objects)
    → Content stream interpreter (Tj/TJ/q/Q/cm operators)
      → Character extraction (CMap, fonts, glyph widths)
        → Layout analysis (chars → words → lines → blocks)
          → Reading order (column detection, de-interleaving)
            → Text output (plain or layout-preserved)
```

The entire PDF parsing and text extraction pipeline is written in Rust — no dependency on poppler, pdfium, or pdfbox. Python bindings are generated via PyO3/maturin.

## Benchmarks

Tested against PyMuPDF on a set of real-world PDFs (textbooks, novels, academic papers):

| PDF | botl-pdf words | PyMuPDF words | botl-pdf time | PyMuPDF time |
|---|---|---|---|---|
| 100-page textbook | ~35,400 | ~34,700 | ~240ms | ~174ms |
| 293-page history book | ~101,000 | ~99,900 | ~590ms | ~380ms |
| 560-page programming book | ~200,000 | ~197,000 | ~1260ms | ~870ms |

Word counts match within ~1% of PyMuPDF across all tested documents.

## CLI

```bash
botl-pdf text report.pdf          # Extract text to stdout
botl-pdf info report.pdf          # Show document metadata
botl-pdf export report.pdf        # Export (markdown/text)
```

## Development

```bash
# Create virtual environment
python -m venv .venv && source .venv/bin/activate

# Install dev dependencies
pip install maturin pytest fitz

# Build and install in editable mode
maturin develop --release

# Run tests
cd rust && cargo test
pytest tests/python/
```

## License

Apache 2.0
