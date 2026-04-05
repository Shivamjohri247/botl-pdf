# botl-pdf

[![PyPI version](https://img.shields.io/pypi/v/botlpdf.svg)](https://pypi.org/project/botlpdf/) [![Python versions](https://img.shields.io/pypi/pyversions/botlpdf.svg)](https://pypi.org/project/botlpdf/) [![License](https://img.shields.io/pypi/l/botlpdf.svg)](https://pypi.org/project/botlpdf/)

High-performance PDF text extraction library with a custom Rust core and Python bindings. No dependency on poppler, pdfium, or pdfbox — the entire PDF parsing and text extraction pipeline is written from scratch.

## Features

- Fast text extraction with layout analysis
- Character-level output with bounding boxes, fonts, colors, and styles
- Layout-preserving text extraction (spatial whitespace)
- Table of contents (TOC/outline) extraction with page numbers
- Document metadata extraction (title, author, dates, etc.)
- Geometric element extraction (lines, rectangles)
- Configurable layout parameters (word spacing, line grouping, reading order)
- Run-aware de-interleaving for correct reading order on complex PDFs
- Pythonic API with type hints throughout
- CLI for common operations
- Zero external PDF library dependencies

## Install

```bash
pip install botlpdf
```

Build from source (requires Rust toolchain):

```bash
pip install maturin
git clone https://github.com/Shivamjohri247/botl-pdf.git
cd botl-pdf
maturin develop --release
```

---

## Quick Start

```python
import botl_pdf

doc = botl_pdf.open("report.pdf")
text = doc.pages[0].extract_text()
print(text)
```

---

## Opening Documents

### From a file path

```python
import botl_pdf

doc = botl_pdf.open("report.pdf")
print(f"Pages: {doc.num_pages}")
print(f"Encrypted: {doc.is_encrypted}")
```

### From bytes

```python
with open("report.pdf", "rb") as f:
    data = f.read()

doc = botl_pdf.open(data)
print(f"Pages: {doc.num_pages}")
```

### As a context manager

```python
with botl_pdf.open("report.pdf") as doc:
    text = doc.pages[0].extract_text()
```

---

## Text Extraction

### Plain text (default)

Returns clean, readable text. Blocks are separated by double newlines, lines by single newlines, words by spaces.

```python
doc = botl_pdf.open("report.pdf")

# Single page
text = doc.pages[0].extract_text()
print(text)

# All pages
for page in doc.pages:
    print(page.extract_text())

# Subscript access (0-based, supports negative)
text_last = doc.pages[-1].extract_text()
```

### Layout-preserving text

Maintains spatial positioning using proportional spaces between words. Useful when you need to preserve visual alignment of columns, tables, or indented text.

```python
doc = botl_pdf.open("financial_report.pdf")
page = doc.pages[0]

# Layout mode preserves spatial whitespace
layout_text = page.extract_text(layout=True)
print(layout_text)
```

### Tuning extraction parameters

```python
import botl_pdf

doc = botl_pdf.open("two_column.pdf")

# Tighter word grouping (merge chars closer together)
params = botl_pdf.LayoutParams(
    word_margin=1.5,   # max horizontal gap in same word (× font_size), default 2.0
    line_margin=0.5,   # max vertical gap in same block (× line height), default 0.5
    boxes_flow=0.5,    # reading order: 0.0=horizontal, 1.0=vertical, default 0.5
)

text = doc.pages[0].extract_text(layout=True, layout_params=params)
```

### Exporting entire documents

```python
from botl_pdf.export import to_text, to_markdown

# Plain text for all pages
full_text = to_text("report.pdf")

# Layout-preserved text
full_text_layout = to_text("report.pdf", layout=True)

# Markdown (pages separated by horizontal rules)
markdown = to_markdown("report.pdf")

# Specific page range only
markdown_subset = to_markdown("report.pdf", pages=range(0, 5))
```

---

## Character-Level Access

Each page exposes individual characters with full style information: bounding box, font name, font size, bold/italic flags, fill and stroke colors, rotation, and run ID.

### Inspecting individual characters

```python
doc = botl_pdf.open("report.pdf")
page = doc.pages[0]

for char in page.chars[:5]:
    print(f"  char={char.text!r}  "
          f"pos=({char.bbox.x0:.1f}, {char.bbox.y0:.1f})  "
          f"size={char.font_size:.1f}  "
          f"font={char.font_name}")
```

Output:
```
  char='H'  pos=(100.0, 700.0)  size=12.0  font=F1
  char='e'  pos=(108.0, 700.0)  size=12.0  font=F1
  char='l'  pos=(115.0, 700.0)  size=12.0  font=F1
  char='l'  pos=(120.0, 700.0)  size=12.0  font=F1
  char='o'  pos=(125.0, 700.0)  size=12.0  font=F1
```

### Finding text by style

```python
# Find all bold characters on page 0
bold_chars = [c for c in doc.pages[0].chars if c.bold]
bold_text = "".join(c.text for c in bold_chars)

# Find characters in a specific color (e.g., red links)
red_chars = [
    c for c in doc.pages[0].chars
    if c.color and c.color[0] > 0.8 and c.color[1] < 0.2 and c.color[2] < 0.2
]

# Find large decorative initials (font size > 30)
initials = [c for c in doc.pages[0].chars if c.font_size > 30]
for c in initials:
    print(f"Decorative initial: {c.text!r} at size {c.font_size:.0f}")
```

### Extracting text from a region

```python
# Get all text in a specific rectangular area
x0, y0, x1, y1 = 100.0, 600.0, 400.0, 700.0

region_chars = [
    c for c in doc.pages[0].chars
    if c.bbox.x0 >= x0 and c.bbox.x1 <= x1
    and c.bbox.y0 >= y0 and c.bbox.y1 <= y1
]
region_text = "".join(c.text for c in region_chars)
print(region_text)
```

### Run ID tracking

Characters from the same text-showing operation (Tj/TJ) share a `run_id`. This lets you group characters by their PDF text operation — useful for debugging extraction issues or understanding the PDF's internal structure.

```python
from collections import defaultdict

# Group characters by their source text operation
runs = defaultdict(str)
for c in doc.pages[0].chars:
    runs[c.run_id] += c.text

for run_id, text in sorted(runs.items()):
    print(f"  Run {run_id}: {text[:60]!r}")
```

---

## Document Metadata

```python
doc = botl_pdf.open("report.pdf")

meta = doc.metadata
print(f"Title:    {meta.get('title')}")
print(f"Author:   {meta.get('author')}")
print(f"Subject:  {meta.get('subject')}")
print(f"Creator:  {meta.get('creator')}")
print(f"Producer: {meta.get('producer')}")
print(f"Created:  {meta.get('creation_date')}")
print(f"Modified: {meta.get('mod_date')}")
print(f"Version:  {meta.get('version')}")
```

---

## Table of Contents

```python
doc = botl_pdf.open("book.pdf")

toc = doc.toc
for entry in toc:
    indent = "  " * entry.level
    page = entry.page_number
    print(f"{indent}{entry.title}  →  page {page}")
```

Output:
```
Preface  →  page 5
  Acknowledgments  →  page 7
Part I. Foundations  →  page 11
  Chapter 1. Introduction  →  page 13
  Chapter 2. Methods  →  page 27
Part II. Results  →  page 45
  Chapter 3. Analysis  →  page 47
```

### Building a page lookup from TOC

```python
# Map page numbers to their chapter titles
chapters = {}
current_chapter = None
for entry in doc.toc:
    if entry.level == 0 and entry.page_number is not None:
        current_chapter = entry.title
    if current_chapter and entry.page_number is not None:
        chapters[entry.page_number] = current_chapter

# Find which chapter a page belongs to
def chapter_for_page(page_idx):
    page_nums = sorted(chapters.keys())
    for i, p in enumerate(page_nums):
        if page_idx < p:
            return chapters[page_nums[max(0, i - 1)]] if i > 0 else None
    return chapters[page_nums[-1]]

print(f"Page 30 is in: {chapter_for_page(30)}")
```

---

## Geometric Elements

Pages expose geometric lines and rectangles drawn on the PDF canvas — useful for detecting table borders, rules, decorative elements, and form fields.

### Lines

```python
page = doc.pages[0]

for line in page.lines:
    print(f"  Line ({line.x0:.1f},{line.y0:.1f}) → ({line.x1:.1f},{line.y1:.1f})  "
          f"width={line.line_width:.1f}")
```

### Rectangles

```python
for rect in page.rects:
    fill = rect.fill_color
    stroke = rect.stroke_color
    print(f"  Rect ({rect.bbox.x0:.1f},{rect.bbox.y0:.1f})-"
          f"({rect.bbox.x1:.1f},{rect.bbox.y1:.1f})  "
          f"stroke={stroke}  fill={fill}")
```

### Detecting horizontal rules

```python
# Find horizontal lines (useful for detecting separators/tables)
h_rules = [
    line for line in page.lines
    if abs(line.y1 - line.y0) < 1.0 and (line.x1 - line.x0) > 50.0
]

for rule in h_rules:
    print(f"Horizontal rule at y={rule.y0:.1f} from x={rule.x0:.1f} to x={rule.x1:.1f}")
```

---

## Page Properties

```python
doc = botl_pdf.open("report.pdf")

for i, page in enumerate(doc.pages):
    print(f"Page {i}: {page.width:.0f}×{page.height:.0f}pt  "
          f"rotation={page.rotation}°  "
          f"label={page.label!r}")
```

Output:
```
Page 0: 612×792pt  rotation=0°  label='1'
Page 1: 612×792pt  rotation=0°  label='2'
```

Common page sizes:
- Letter: 612 × 792 pt (8.5" × 11")
- A4: 595 × 842 pt (210mm × 297mm)

---

## Visual Debugging

Requires `Pillow`. Draws bounding boxes and geometric elements on a rendered page image — useful for debugging extraction issues or understanding PDF layout.

```bash
pip install botlpdf[debug]
```

```python
from botl_pdf.debug import VisualDebugger
import botl_pdf

doc = botl_pdf.open("report.pdf")
page = doc.pages[0]

debugger = VisualDebugger(page)

# Draw character bounding boxes (red)
img = debugger.draw_chars(resolution=150)
img.save("debug_chars.png")

# Draw geometric lines (blue)
img = debugger.draw_lines(resolution=150)
img.save("debug_lines.png")

# Draw geometric rectangles (green)
img = debugger.draw_rects(resolution=150)
img.save("debug_rects.png")

# All elements layered together
img = debugger.draw_all(resolution=150)
img.save("debug_all.png")
```

---

## CLI

```bash
pip install botlpdf[cli]
```

### Extract text

```bash
# To stdout
botl-pdf text report.pdf

# To file
botl-pdf text report.pdf --output text.txt

# Specific pages
botl-pdf text report.pdf --pages 1-5

# Layout-preserved
botl-pdf text report.pdf --layout
```

### Show metadata

```bash
botl-pdf info report.pdf
```

Output:
```json
{
  "version": "1.4",
  "page_count": 42,
  "encrypted": false,
  "title": "Annual Report 2024",
  "author": "Acme Corp",
  "creator": "LaTeX",
  "producer": "pdfTeX-1.40"
}
```

### Export

```bash
# Markdown
botl-pdf export report.pdf --format markdown --output report.md

# Plain text
botl-pdf export report.pdf --format text --output report.txt
```

---

## API Reference

### `botl_pdf.open(path_or_bytes, *, password=None, lazy=True) -> Document`

Open a PDF from a file path (str) or raw bytes.

### `Document`

| Property / Method | Type | Description |
|---|---|---|
| `.metadata` | `dict` | Metadata fields: title, author, subject, keywords, creator, producer, creation_date, mod_date, version, page_count |
| `.num_pages` | `int` | Number of pages |
| `.is_encrypted` | `bool` | Whether the document is encrypted |
| `.toc` | `list[TOCEntry]` | Table of contents / outline bookmarks |
| `.pages` | `PageCollection` | Iterable, subscriptable page access |
| `doc[i]` | `PyPage` | Shortcut for `doc.pages[i]` (supports negative indices) |
| `len(doc)` | `int` | Same as `.num_pages` |

### `Page` (via `doc.pages[i]`)

| Property / Method | Type | Description |
|---|---|---|
| `.extract_text(layout=False, layout_params=None)` | `str` | Extract text (plain or layout-preserved) |
| `.chars` | `list[Char]` | All characters with full style info |
| `.lines` | `list[GeomLine]` | Geometric lines on the page |
| `.rects` | `list[GeomRect]` | Geometric rectangles on the page |
| `.width` | `float` | Page width in points |
| `.height` | `float` | Page height in points |
| `.rotation` | `int` | Rotation in degrees (0, 90, 180, 270) |
| `.page_number` | `int` | Zero-based page index |
| `.label` | `str` | Page label string (e.g. "iii", "A-1") |

### `Char`

| Property | Type | Description |
|---|---|---|
| `.text` | `str` | Unicode character |
| `.bbox` | `BBox` | Bounding box |
| `.font_name` | `str` | Font resource name (e.g. "F1") |
| `.font_size` | `float` | Font size in points |
| `.bold` | `bool` | Bold flag |
| `.italic` | `bool` | Italic flag |
| `.color` | `tuple[float, float, float] or None` | Fill color (RGB, 0.0-1.0) |
| `.stroking_color` | `tuple[float, float, float] or None` | Stroke color (RGB, 0.0-1.0) |
| `.rotation` | `float` | Rotation in degrees |
| `.run_id` | `int` | Text operation ID (chars from same Tj/TJ share this) |

### `BBox`

| Property / Method | Type | Description |
|---|---|---|
| `.x0`, `.y0` | `float` | Top-left corner |
| `.x1`, `.y1` | `float` | Bottom-right corner |
| `.width` | `float` | Width (x1 - x0) |
| `.height` | `float` | Height (y1 - y0) |
| `.center()` | `(float, float)` | Center point |
| `.area()` | `float` | Area |

### `TOCEntry`

| Property | Type | Description |
|---|---|---|
| `.title` | `str` | Outline entry title |
| `.level` | `int` | Nesting depth (0 = top-level) |
| `.page_number` | `int or None` | 0-indexed destination page (None if unresolvable) |
| `.dest` | `str or None` | Raw destination string |

### `GeomLine`

| Property | Type | Description |
|---|---|---|
| `.x0`, `.y0` | `float` | Start point |
| `.x1`, `.y1` | `float` | End point |
| `.line_width` | `float` | Stroke width |
| `.color` | `tuple or None` | RGB color (0.0-1.0) |

### `GeomRect`

| Property | Type | Description |
|---|---|---|
| `.bbox` | `BBox` | Bounding box |
| `.line_width` | `float` | Stroke width |
| `.stroke_color` | `tuple or None` | Stroke RGB color |
| `.fill_color` | `tuple or None` | Fill RGB color |

### `LayoutParams`

| Parameter | Type | Default | Description |
|---|---|---|---|
| `word_margin` | `float` | `2.0` | Max horizontal gap between chars in same word, as a multiple of font size |
| `line_margin` | `float` | `0.5` | Max vertical gap between lines in same block, as a multiple of line height |
| `boxes_flow` | `float` | `0.5` | Reading-order direction (0.0 = strict horizontal, 1.0 = strict vertical) |

```python
params = botl_pdf.LayoutParams(word_margin=1.5, line_margin=0.3, boxes_flow=0.0)
text = page.extract_text(layout=True, layout_params=params)
```

---

## Architecture

```
PDF bytes
  → Parser (nom tokenizer + recursive-descent objects)
    → Content stream interpreter (Tj/TJ/q/Q/cm operators)
      → Character extraction (CMap, fonts, glyph widths)
        → Layout analysis (chars → words → lines → blocks)
          → Reading order (column detection, run de-interleaving)
            → Text output (plain or layout-preserved)
```

The pipeline is entirely custom Rust — no dependency on poppler, pdfium, pdfbox, or any other PDF library.

**Key design decisions:**

- **Run-aware de-interleaving** — Each Tj/TJ text operation tags characters with a `run_id`. When PDF producers interleave characters from different operations at alternating x-positions, the layout engine detects this and groups by run, preserving correct reading order.
- **Font-band separation** — Within a line, characters are grouped by font size to handle decorative initials and mixed-size text on the same visual line.
- **Lazy extraction** — Page content is decoded on first access and cached. The parsed `Document` is shared across pages via `Arc<Mutex>`, so there's no per-page re-parsing.

---

## Benchmarks

Tested on real-world PDFs (textbooks, novels, academic papers — 17 PDFs, 6,663 pages total).

### Text Extraction Quality

| Type | Pages | Words Extracted |
|---|---|---|
| Novel | 408 | 118,767 |
| Tech book | 558 | 136,669 |
| Study guide | 576 | 89,490 |
| Textbook | 438 | 100,594 |
| Textbook | 565 | 93,691 |
| Tech book | 1,038 | 85,854 |
| Tech book | 341 | 47,769 |
| History book | 293 | 107,411 |
| Programming book | 806 | 203,941 |
| **Total (17 PDFs)** | **6,663** | **1,399,763** |

### Performance

| Type | Pages | Time |
|---|---|---|
| Tech book | 1,038 | 0.56s |
| Tech book | 341 | 0.21s |
| Textbook | 565 | 0.45s |
| Novel | 196 | 0.21s |
| History book | 293 | 0.49s |
| Programming book | 806 | 0.91s |
| **Overall (17 PDFs)** | **6,663** | **6.40s** |

### What changed in v0.2.0

- **~2x faster** than v0.1.x through Arc-based caching, cross-page font cache, zlib-ng backend, and reduced cloning
- **Fixed word boundary detection** for PDFs that encode spaces as position gaps instead of literal space characters

---

## Development

```bash
# Set up environment
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest

# Build Rust extension in release mode
maturin develop --release

# Run Rust tests (198 tests)
cd rust && cargo test

# Run Python tests
pytest tests/python/

# Run benchmarks
pytest tests/python/benchmarks/ --benchmark-only
```

### Project structure

```
botl-pdf/
├── rust/
│   ├── botl-pdf-core/        # Core engine (parser, text, layout, codecs)
│   ├── botl-pdf-python/      # PyO3 bindings → _core native module
│   └── botl-pdf-csys/        # Image codec FFI (JPEG, JPEG2000)
├── python/botl_pdf/          # High-level Python API
│   ├── document.py           # Document, PageCollection
│   ├── page.py               # Page wrapper
│   ├── export.py             # to_text(), to_markdown()
│   ├── debug.py              # VisualDebugger (Pillow overlays)
│   ├── tables.py             # Table/TableCell dataclasses
│   └── cli/main.py           # CLI: text, info, export
├── tests/
│   ├── rust/                 # Integration tests (parser, text, layout, geometry)
│   └── python/               # Unit + integration tests
└── docs/                     # Sphinx docs
```

## License

Apache 2.0
