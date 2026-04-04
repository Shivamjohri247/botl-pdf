"""High-level Page wrapper."""

from __future__ import annotations

from typing import Optional

from botl_pdf._core import PyPage, PyLayoutParams


class Page:
    """High-level page interface wrapping PyPage."""

    def __init__(self, page: PyPage):
        self._page = page

    @property
    def width(self) -> float:
        return self._page.width

    @property
    def height(self) -> float:
        return self._page.height

    @property
    def rotation(self) -> int:
        return self._page.rotation

    @property
    def page_number(self) -> int:
        return self._page.page_number

    @property
    def label(self) -> str:
        return self._page.label

    def extract_text(
        self,
        layout: bool = False,
        word_margin: float = 2.0,
        line_margin: float = 0.5,
        boxes_flow: float = 0.5,
    ) -> str:
        """Extract text from this page.

        Args:
            layout: If True, preserve spatial whitespace in the output.
            word_margin: Max horizontal gap between chars in same word (× font_size).
            line_margin: Max vertical gap between lines in same block (× line height).
            boxes_flow: Reading-order strictness (0.0=horizontal, 1.0=vertical).
        """
        params = PyLayoutParams(word_margin=word_margin, line_margin=line_margin, boxes_flow=boxes_flow)
        return self._page.extract_text(layout=layout, layout_params=params)

    @property
    def chars(self) -> list:
        return self._page.chars

    @property
    def lines(self) -> list:
        return self._page.lines

    @property
    def rects(self) -> list:
        return self._page.rects

    def __repr__(self) -> str:
        return f"<Page number={self.page_number} width={self.width:.1f} height={self.height:.1f}>"
