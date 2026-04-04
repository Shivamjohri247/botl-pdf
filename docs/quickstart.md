# Quickstart

## Installation

```bash
pip install botl-pdf
```

## Opening a PDF

```python
import botl_pdf

doc = botl_pdf.open("document.pdf")
print(f"Pages: {doc.num_pages}")
```

## Extracting Text

```python
# Extract all text from a page
page = doc.get_page(0)
text = page.extract_text()
print(text)

# Extract with layout preservation
text = page.extract_text(layout=True)
print(text)
```

## Working with Pages

```python
# Iterate over all pages
for page in doc.pages:
    print(page.extract_text())

# Access a specific page
page = doc.pages[2]  # third page (0-indexed)
print(f"Size: {page.width} x {page.height}")
print(f"Rotation: {page.rotation}")
```

## Metadata

```python
doc = botl_pdf.open("document.pdf")
meta = doc.metadata
print(f"Title: {meta.get('title')}")
print(f"Author: {meta.get('author')}")
print(f"Pages: {doc.num_pages}")
print(f"Encrypted: {doc.is_encrypted}")
```

## Character-Level Data

```python
page = doc.get_page(0)
for char in page.chars:
    print(f"'{char.text}' at ({char.bbox.x0:.1f}, {char.bbox.y0:.1f}) "
          f"font={char.font_name} size={char.font_size:.1f}")
```

## Export

```python
from botl_pdf.export import to_markdown, to_text

# Export entire document
markdown = to_markdown("document.pdf")
text = to_text("document.pdf")

# Write to file
with open("output.md", "w") as f:
    f.write(markdown)
```

## CLI Usage

```bash
# Extract text to stdout
botl-pdf text document.pdf

# Extract to file with layout preservation
botl-pdf text document.pdf -o output.txt --layout

# Extract specific pages
botl-pdf text document.pdf --pages 1-5

# Show PDF metadata
botl-pdf info document.pdf

# Export to markdown
botl-pdf export document.pdf --format markdown -o output.md
```

## Context Manager

```python
with botl_pdf.open("document.pdf") as doc:
    print(doc.pages[0].extract_text())
```

## Layout Parameters

Fine-tune text extraction with layout parameters:

```python
from botl_pdf.page import Page

page = doc.get_page(0)
text = page.extract_text(
    layout=True,
    word_margin=0.1,   # gap threshold for word grouping (fraction of font size)
    line_margin=0.5,   # gap threshold for line grouping (fraction of avg height)
    boxes_flow=0.5,    # reading order weight (0=top-down, 1=left-right)
)
```
