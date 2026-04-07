"""OCR integration layer.

Provides an abstract base for OCR backends. Actual backends
(Tesseract, EasyOCR) are optional dependencies.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Union


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
    def recognize(
        self, image: Union[bytes, "PIL.Image.Image"], language: str = "eng"
    ) -> list[OCRResult]:
        """Perform OCR on an image.

        Args:
            image: PIL Image or PNG/JPEG bytes.
            language: Language code (e.g., "eng", "fra", "chi_sim").

        Returns:
            List of OCR results with bounding boxes.
        """
        ...

    @abstractmethod
    def is_available(self) -> bool:
        """Check if this OCR backend is installed and available."""
        ...

    def recognize_to_text(
        self, image: Union[bytes, "PIL.Image.Image"], language: str = "eng"
    ) -> str:
        """Perform OCR and return concatenated text only."""
        results = self.recognize(image, language)
        return "\n".join(r.text for r in results if r.text.strip())
