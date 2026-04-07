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
        ocr: str | bool = False,
        ocr_language: str = "eng",
        ocr_dpi: int = 200,
    ) -> str:
        """Extract text from this page.

        Args:
            layout: If True, preserve spatial whitespace in the output.
            word_margin: Max horizontal gap between chars in same word (× font_size).
            line_margin: Max vertical gap between lines in same block (× line height).
            boxes_flow: Reading-order strictness (0.0=horizontal, 1.0=vertical).
            ocr: OCR fallback strategy. False = no OCR. True/"auto" = use OCR if
                page has no native text. "tesseract"/"easyocr" = force specific backend.
            ocr_language: Language code for OCR (default "eng").
            ocr_dpi: Resolution for page-to-image rendering when OCR is needed.

        Returns:
            Extracted text as a string.
        """
        # Try native extraction first
        params = PyLayoutParams(
            word_margin=word_margin, line_margin=line_margin, boxes_flow=boxes_flow
        )
        text = self._page.extract_text(layout=layout, layout_params=params)

        # Determine if OCR is needed
        needs_ocr = False
        backend_name = None

        if ocr is True or ocr == "auto":
            # Auto mode: only OCR if native extraction produced nothing
            if not text.strip():
                needs_ocr = True
                backend_name = None  # will try first available
        elif isinstance(ocr, str) and ocr not in (False, "auto"):
            # Specific backend requested — force OCR regardless
            backend_name = ocr
            needs_ocr = True

        if not needs_ocr:
            return text

        # Perform OCR
        return self._ocr_fallback(
            backend_name=backend_name,
            language=ocr_language,
            dpi=ocr_dpi,
        )

    def _ocr_fallback(
        self,
        backend_name: str | None = None,
        language: str = "eng",
        dpi: int = 200,
    ) -> str:
        """Run OCR on this page as a fallback for native text extraction."""
        from botl_pdf.ocr.renderer import get_page_image

        # Get page image
        image = get_page_image(self._page, dpi=dpi)

        # Get backend
        if backend_name:
            from botl_pdf.ocr.registry import get_backend

            backend = get_backend(backend_name)
        else:
            from botl_pdf.ocr.registry import available_backends

            backends = available_backends()
            if not backends:
                raise ImportError(
                    "No OCR backend available. Install one with:\n"
                    "  pip install botlpdf[ocr-tesseract]\n"
                    "  pip install botlpdf[ocr-easyocr]"
                )
            from botl_pdf.ocr.registry import get_backend

            backend = get_backend(backends[0])

        return backend.recognize_to_text(image, language=language)

    @property
    def chars(self) -> list:
        return self._page.chars

    @property
    def lines(self) -> list:
        return self._page.lines

    @property
    def rects(self) -> list:
        return self._page.rects

    @property
    def has_text(self) -> bool:
        """Whether this page contains extractable text characters.

        Returns False for image-only/scanned pages that would need OCR.
        """
        return self._page.has_text

    def extract_images(self) -> list:
        """Extract embedded images from this page.

        Returns a list of image objects with width, height, and data (RGB bytes)
        attributes. Useful for OCR on scanned/image-only pages.
        """
        return self._page.extract_images()

    def __repr__(self) -> str:
        return f"<Page number={self.page_number} width={self.width:.1f} height={self.height:.1f}>"
