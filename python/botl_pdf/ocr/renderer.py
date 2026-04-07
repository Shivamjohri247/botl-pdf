"""Page-to-image conversion for OCR.

Provides utilities to get a page image suitable for OCR processing.
Tries embedded image extraction first (fast, no extra deps),
falls back to a basic Pillow-based rendering using char/line/rect positions.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Union

if TYPE_CHECKING:
    from PIL import Image


def get_page_image(
    page,
    dpi: int = 200,
) -> "Image.Image":
    """Get a page image suitable for OCR.

    Args:
        page: A botl_pdf Page or PyPage object.
        dpi: Resolution for rendering (default 200, good balance of speed/accuracy).

    Returns:
        PIL Image in RGB mode.
    """
    from PIL import Image

    # Try to extract an embedded full-page image first (common in scanned PDFs)
    embedded = _try_extract_embedded_image(page)
    if embedded is not None:
        return embedded

    # Fall back to rendering from extracted elements
    return _render_from_elements(page, dpi)


def _try_extract_embedded_image(page) -> "Image.Image | None":
    """Try to find and extract a full-page embedded image (scan)."""
    from PIL import Image

    # Check if page has extract_images (Rust core method)
    extract_fn = getattr(page, "extract_images", None)
    if extract_fn is None:
        extract_fn = getattr(page, "_page", None)
        if extract_fn:
            extract_fn = getattr(extract_fn, "extract_images", None)
    if extract_fn is None:
        return None

    try:
        images = extract_fn()
    except Exception:
        return None

    if not images:
        return None

    page_w = getattr(page, "width", 0) or 612.0
    page_h = getattr(page, "height", 0) or 792.0
    page_area = page_w * page_h

    # Find the largest image (by pixel area)
    best = None
    best_pixel_area = 0
    for img_info in images:
        w = getattr(img_info, "width", 0)
        h = getattr(img_info, "height", 0)
        pixel_area = w * h
        if pixel_area > best_pixel_area:
            best_pixel_area = pixel_area
            best = img_info

    if best is None:
        return None

    w = getattr(best, "width", 0)
    h = getattr(best, "height", 0)
    if w <= 0 or h <= 0:
        return None

    # Only use embedded image if it's large enough to be a meaningful page scan.
    # A page scan at even 150 DPI has pixel_area >> page_area (in points²).
    # e.g. letter at 150 DPI: 1275x1650 = 2.1M vs 612*792 = 485K ≈ 4.3x
    if best_pixel_area < page_area * 2.0:
        return None

    try:
        return Image.frombytes("RGB", (w, h), best.data)
    except Exception:
        return None


def _render_from_elements(page, dpi: int) -> "Image.Image":
    """Render a page to an image using extracted text/line/rect positions.

    This creates a white canvas and draws colored rectangles for text,
    lines, and rectangles. Not pixel-perfect but sufficient for OCR engines.
    """
    from PIL import Image, ImageDraw

    # Get page dimensions (in points, 72 dpi)
    pw = getattr(page, "width", None)
    ph = getattr(page, "height", None)
    if pw is None:
        pw = getattr(page, "_page", None)
        if pw:
            pw = pw.width
            ph = ph.height if hasattr(ph, "height") else page._page.height
        else:
            pw, ph = 612.0, 792.0

    if ph is None:
        ph = 792.0

    scale = dpi / 72.0
    img_w = int(pw * scale)
    img_h = int(ph * scale)

    img = Image.new("RGB", (img_w, img_h), "white")
    draw = ImageDraw.Draw(img)

    # Draw rects
    rects = getattr(page, "rects", None)
    if rects is None:
        rects = getattr(page, "_page", None)
        if rects:
            rects = rects.rects
    if rects:
        try:
            for r in rects:
                bbox = getattr(r, "bbox", r)
                x0 = getattr(bbox, "x0", 0) * scale
                y0 = getattr(bbox, "y0", 0) * scale
                x1 = getattr(bbox, "x1", 0) * scale
                y1 = getattr(bbox, "y1", 0) * scale
                draw.rectangle([x0, y0, x1, y1], fill="black")
        except Exception:
            pass

    # Draw chars as black rectangles at their positions
    chars = getattr(page, "chars", None)
    if chars is None:
        chars = getattr(page, "_page", None)
        if chars:
            chars = chars.chars
    if chars:
        try:
            for c in chars:
                bbox = c.bbox
                x0 = bbox.x0 * scale
                y0 = bbox.y0 * scale
                x1 = bbox.x1 * scale
                y1 = bbox.y1 * scale
                draw.rectangle([x0, y0, x1, y1], fill="black")
        except Exception:
            pass

    return img
