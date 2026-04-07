"""OCR backend registry.

Lazy-loads backends only when requested, so importing botl_pdf
never requires OCR dependencies.
"""

from __future__ import annotations

from typing import Optional

from botl_pdf.ocr.base import OCRBackend

_REGISTRY: dict[str, str] = {
    "tesseract": "botl_pdf.ocr.backends.tesseract",
    "easyocr": "botl_pdf.ocr.backends.easyocr_backend",
}

_INSTANCES: dict[str, OCRBackend] = {}


def get_backend(name: str, **kwargs) -> OCRBackend:
    """Get an OCR backend by name.

    Backends are lazy-loaded and cached. Raises ImportError with
    install instructions if the backend is not available.

    Args:
        name: Backend name ("tesseract" or "easyocr").
        **kwargs: Extra arguments passed to the backend constructor.

    Returns:
        An OCRBackend instance.
    """
    key = name.lower()

    if key in _INSTANCES:
        return _INSTANCES[key]

    module_path = _REGISTRY.get(key)
    if module_path is None:
        available = list(_REGISTRY.keys())
        raise ValueError(
            f"Unknown OCR backend '{name}'. Available: {available}"
        )

    import importlib

    try:
        mod = importlib.import_module(module_path)
    except ImportError as e:
        if key == "tesseract":
            raise ImportError(
                "Tesseract backend not available. "
                "Install with: pip install botlpdf[ocr-tesseract]"
            ) from e
        elif key == "easyocr":
            raise ImportError(
                "EasyOCR backend not available. "
                "Install with: pip install botlpdf[ocr-easyocr]"
            ) from e
        raise

    # Each backend module exposes a ``Backend`` class
    backend_cls = getattr(mod, "Backend")
    instance = backend_cls(**kwargs)

    if not instance.is_available():
        raise RuntimeError(
            f"OCR backend '{name}' is not properly installed or available."
        )

    _INSTANCES[key] = instance
    return instance


def available_backends() -> list[str]:
    """Return list of backend names that are importable and available."""
    result = []
    for name in _REGISTRY:
        try:
            b = get_backend(name)
            if b.is_available():
                result.append(name)
        except (ImportError, RuntimeError):
            pass
    return result


def reset_cache() -> None:
    """Clear cached backend instances (useful for testing)."""
    _INSTANCES.clear()
