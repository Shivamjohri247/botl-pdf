"""Export convenience functions."""

from __future__ import annotations

from typing import Optional


def to_markdown(path_or_bytes: str | bytes, *, pages: Optional[range] = None) -> str:
    """Export a PDF document to Markdown.

    Args:
        path_or_bytes: Path to PDF file or raw bytes.
        pages: Optional range of page indices to include.

    Returns:
        Markdown string.
    """
    from botl_pdf._core import open as _open

    doc = _open(path_or_bytes)
    parts = []

    page_range = range(doc.num_pages) if pages is None else pages
    for i in page_range:
        page = doc.get_page(i)
        text = page.extract_text(layout=False)
        if text.strip():
            parts.append(text)

    return "\n\n---\n\n".join(parts)


def to_text(path_or_bytes: str | bytes, *, layout: bool = False) -> str:
    """Export a PDF document to plain text.

    Args:
        path_or_bytes: Path to PDF file or raw bytes.
        layout: If True, preserve spatial layout with whitespace.

    Returns:
        Plain text string.
    """
    from botl_pdf._core import open as _open

    doc = _open(path_or_bytes)
    parts = []
    for i in range(doc.num_pages):
        page = doc.get_page(i)
        parts.append(page.extract_text(layout=layout))
    return "\n\n".join(parts)
