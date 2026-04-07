"""Export convenience functions."""

from __future__ import annotations

from typing import Optional


def to_markdown(
    path_or_bytes: str | bytes,
    *,
    pages: Optional[range] = None,
    ocr: str | bool = False,
    ocr_language: str = "eng",
) -> str:
    """Export a PDF document to Markdown.

    Args:
        path_or_bytes: Path to PDF file or raw bytes.
        pages: Optional range of page indices to include.
        ocr: OCR fallback strategy. False = no OCR. True/"auto" = use OCR if
            page has no native text. "tesseract"/"easyocr" = force specific backend.
        ocr_language: Language code for OCR (default "eng").

    Returns:
        Markdown string.
    """
    from botl_pdf._core import open as _open

    doc = _open(path_or_bytes)
    parts = []

    page_range = range(doc.num_pages) if pages is None else pages
    for i in page_range:
        raw_page = doc.get_page(i)
        if ocr:
            from botl_pdf.page import Page

            page = Page(raw_page)
            text = page.extract_text(
                layout=False,
                ocr=ocr,
                ocr_language=ocr_language,
            )
        else:
            text = raw_page.extract_text(layout=False)
        if text.strip():
            parts.append(text)

    return "\n\n---\n\n".join(parts)


def to_text(
    path_or_bytes: str | bytes,
    *,
    layout: bool = False,
    ocr: str | bool = False,
    ocr_language: str = "eng",
) -> str:
    """Export a PDF document to plain text.

    Args:
        path_or_bytes: Path to PDF file or raw bytes.
        layout: If True, preserve spatial layout with whitespace.
        ocr: OCR fallback strategy. False = no OCR. True/"auto" = use OCR if
            page has no native text. "tesseract"/"easyocr" = force specific backend.
        ocr_language: Language code for OCR (default "eng").

    Returns:
        Plain text string.
    """
    from botl_pdf._core import open as _open

    doc = _open(path_or_bytes)
    parts = []
    for i in range(doc.num_pages):
        raw_page = doc.get_page(i)
        if ocr:
            from botl_pdf.page import Page

            page = Page(raw_page)
            parts.append(page.extract_text(
                layout=layout,
                ocr=ocr,
                ocr_language=ocr_language,
            ))
        else:
            parts.append(raw_page.extract_text(layout=layout))
    return "\n\n".join(parts)
