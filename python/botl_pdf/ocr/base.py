"""OCR integration layer.

Provides an abstract base for OCR backends. Actual backends
(Tesseract, EasyOCR) are optional dependencies.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class OCRResult:
    """Result from OCR recognition."""
    text: str
    confidence: float
    x0: float
    y0: float
    x1: float
    y1: float


class OCRBackend(ABC):
    """Abstract base class for OCR backends."""

    @abstractmethod
    def recognize(self, image_bytes: bytes, language: str = "eng") -> list[OCRResult]:
        """Perform OCR on an image.

        Args:
            image_bytes: PNG or JPEG image bytes.
            language: Language code (e.g., "eng", "fra", "chi_sim").

        Returns:
            List of OCR results with bounding boxes.
        """
        ...

    @abstractmethod
    def is_available(self) -> bool:
        """Check if this OCR backend is installed and available."""
        ...
