"""OCR integration layer."""

from botl_pdf.ocr.base import OCRBackend, OCRResult
from botl_pdf.ocr.registry import get_backend, available_backends

__all__ = ["OCRBackend", "OCRResult", "get_backend", "available_backends"]
