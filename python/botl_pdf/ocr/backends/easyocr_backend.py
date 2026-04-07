"""EasyOCR backend.

Requires: pip install botlpdf[ocr-easyocr]
"""

from __future__ import annotations

import io
from typing import Union

from botl_pdf.ocr.base import OCRBackend, OCRResult


class Backend(OCRBackend):
    """EasyOCR backend with lazy model loading."""

    # Map Tesseract/ISO 639-2 codes to EasyOCR (ISO 639-1) codes
    _LANG_MAP = {
        "eng": "en",
        "fra": "fr",
        "deu": "de",
        "spa": "es",
        "por": "pt",
        "ita": "it",
        "nld": "nl",
        "pol": "pl",
        "rus": "ru",
        "zho": "ch_sim",
        "jpn": "ja",
        "kor": "ko",
        "ara": "ar",
        "hin": "hi",
        "tur": "tr",
    }

    def __init__(self, gpu: bool = False, **kwargs):
        self._gpu = gpu
        self._reader = None
        self._reader_lang = None
        self._kwargs = kwargs

    def _get_reader(self, language: str = "en"):
        """Lazy-load the EasyOCR reader (models download on first use)."""
        import easyocr

        # Convert Tesseract-style codes to EasyOCR codes
        easyocr_lang = self._LANG_MAP.get(language, language)

        if self._reader is None or self._reader_lang != easyocr_lang:
            lang_list = [easyocr_lang]
            self._reader = easyocr.Reader(lang_list, gpu=self._gpu, **self._kwargs)
            self._reader_lang = easyocr_lang
        return self._reader

    def recognize(
        self, image: Union[bytes, "PIL.Image.Image"], language: str = "en"
    ) -> list[OCRResult]:
        import numpy as np
        from PIL import Image

        if isinstance(image, bytes):
            img = Image.open(io.BytesIO(image))
        else:
            img = image

        # EasyOCR requires numpy arrays, not PIL Images
        img_array = np.array(img)

        reader = self._get_reader(language)
        raw_results = reader.readtext(
            img_array,
            detail=1,  # return bounding boxes
            paragraph=False,
        )

        results = []
        for bbox, text, confidence in raw_results:
            if not text.strip():
                continue
            # bbox is [[x0,y0],[x1,y1],[x2,y2],[x3,y3]] — get min/max
            xs = [p[0] for p in bbox]
            ys = [p[1] for p in bbox]
            results.append(
                OCRResult(
                    text=text,
                    confidence=confidence,
                    x0=min(xs),
                    y0=min(ys),
                    x1=max(xs),
                    y1=max(ys),
                )
            )
        return results

    def is_available(self) -> bool:
        try:
            import easyocr  # noqa: F401

            return True
        except ImportError:
            return False
