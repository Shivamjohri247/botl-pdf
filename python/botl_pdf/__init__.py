"""botl-pdf: High-performance PDF processing with a Rust + C core.

Usage::

    import botl_pdf

    with botl_pdf.open("report.pdf") as doc:
        text = doc.pages[0].extract_text()
        for char in doc.pages[0].chars:
            print(f"{char.text} at ({char.bbox.x0}, {char.bbox.y0})")
"""

from botl_pdf._core import (  # noqa: F401
    open,
    PyDocument,
    PyPage,
    PyLayoutParams,
    PyBBox,
    PyChar,
    PyWord,
    PyTextLine,
    PyTextBlock,
    PyTOCEntry,
    PyGeomLine,
    PyGeomRect,
    PyWriter,
)

__version__ = "0.1.0"
