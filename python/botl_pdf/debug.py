"""Visual debugging overlays.

Requires Pillow for image rendering. Install with:
    pip install botl-pdf[debug]
"""

from __future__ import annotations

from typing import Optional


class VisualDebugger:
    """Draw debug overlays on PDF page images.

    Usage::

        from botl_pdf._core import open as _open
        doc = _open("report.pdf")
        page = doc.get_page(0)

        debugger = VisualDebugger(page)
        img = debugger.draw_chars()
        img.save("debug_chars.png")
    """

    def __init__(self, page):
        self._page = page
        self._width = page.width
        self._height = page.height

    def _create_canvas(self, resolution: int = 150):
        """Create a blank PIL Image for drawing."""
        try:
            from PIL import Image, ImageDraw
        except ImportError:
            raise ImportError(
                "Visual debugging requires Pillow. "
                "Install with: pip install botl-pdf[debug]"
            )

        scale = resolution / 72.0
        w = int(self._width * scale)
        h = int(self._height * scale)
        img = Image.new("RGB", (w, h), "white")
        draw = ImageDraw.Draw(img)
        return img, draw, scale

    def draw_chars(self, resolution: int = 150, color: str = "red"):
        """Draw character bounding boxes."""
        img, draw, scale = self._create_canvas(resolution)
        chars = self._page.chars
        for ch in chars:
            bbox = ch.bbox
            x0 = int(bbox.x0 * scale)
            y0 = int(bbox.y0 * scale)
            x1 = int(bbox.x1 * scale)
            y1 = int(bbox.y1 * scale)
            draw.rectangle([x0, y0, x1, y1], outline=color, width=1)
        return img

    def draw_lines(self, resolution: int = 150, color: str = "blue"):
        """Draw geometric lines."""
        img, draw, scale = self._create_canvas(resolution)
        for line in self._page.lines:
            x0 = int(line.x0 * scale)
            y0 = int(line.y0 * scale)
            x1 = int(line.x1 * scale)
            y1 = int(line.y1 * scale)
            draw.line([x0, y0, x1, y1], fill=color, width=max(1, int(line.line_width * scale)))
        return img

    def draw_rects(self, resolution: int = 150, stroke: str = "green", fill: Optional[str] = None):
        """Draw geometric rectangles."""
        img, draw, scale = self._create_canvas(resolution)
        for rect in self._page.rects:
            bbox = rect.bbox
            x0 = int(bbox.x0 * scale)
            y0 = int(bbox.y0 * scale)
            x1 = int(bbox.x1 * scale)
            y1 = int(bbox.y1 * scale)
            draw.rectangle([x0, y0, x1, y1], outline=stroke, fill=fill, width=max(1, int(rect.line_width * scale)))
        return img

    def draw_all(self, resolution: int = 150):
        """Draw all elements: chars (red), lines (blue), rects (green)."""
        img, draw, scale = self._create_canvas(resolution)

        # Draw rects first (background)
        for rect in self._page.rects:
            bbox = rect.bbox
            draw.rectangle(
                [int(bbox.x0 * scale), int(bbox.y0 * scale), int(bbox.x1 * scale), int(bbox.y1 * scale)],
                outline="green",
                width=1,
            )

        # Draw lines
        for line in self._page.lines:
            draw.line(
                [int(line.x0 * scale), int(line.y0 * scale), int(line.x1 * scale), int(line.y1 * scale)],
                fill="blue",
                width=max(1, int(line.line_width * scale)),
            )

        # Draw char bboxes on top
        for ch in self._page.chars:
            bbox = ch.bbox
            draw.rectangle(
                [int(bbox.x0 * scale), int(bbox.y0 * scale), int(bbox.x1 * scale), int(bbox.y1 * scale)],
                outline="red",
                width=1,
            )

        return img
