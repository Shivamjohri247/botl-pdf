"""Tesseract OCR backend.

Requires: pip install botlpdf[ocr-tesseract]
System: tesseract binary must be installed (apt install tesseract-ocr / brew install tesseract)
"""

from __future__ import annotations

import io
from typing import Union

from botl_pdf.ocr.base import OCRBackend, OCRResult


class Backend(OCRBackend):
    """Tesseract OCR backend via pytesseract."""

    def __init__(self, tesseract_cmd: str | None = None):
        self._tesseract_cmd = tesseract_cmd

    def recognize(
        self, image: Union[bytes, "PIL.Image.Image"], language: str = "eng"
    ) -> list[OCRResult]:
        import pytesseract
        from PIL import Image

        if self._tesseract_cmd:
            pytesseract.pytesseract.tesseract_cmd = self._tesseract_cmd

        if isinstance(image, bytes):
            img = Image.open(io.BytesIO(image))
        else:
            img = image

        # Get detailed results with bounding boxes and confidence
        data = pytesseract.image_to_data(img, lang=language, output_type=pytesseract.Output.DICT)

        results = []
        n = len(data["text"])
        for i in range(n):
            text = data["text"][i].strip()
            conf = float(data["conf"][i])
            if not text or conf < 0:
                continue
            results.append(
                OCRResult(
                    text=text,
                    confidence=conf / 100.0 if conf > 1 else conf,
                    x0=float(data["left"][i]),
                    y0=float(data["top"][i]),
                    x1=float(data["left"][i] + data["width"][i]),
                    y1=float(data["top"][i] + data["height"][i]),
                )
            )
        return results

    def is_available(self) -> bool:
        try:
            import pytesseract

            pytesseract.get_tesseract_version()
            return True
        except Exception:
            return False
