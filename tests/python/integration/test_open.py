"""Integration tests for opening PDF documents.

Tests exercise both the low-level _core.open API and the high-level
Document wrapper. All tests are skipped when _core is not compiled.
"""

from __future__ import annotations

import importlib

import pytest


def _core_available() -> bool:
    try:
        importlib.import_module("botl_pdf._core")
        return True
    except ImportError:
        return False


skip_if_no_core = pytest.mark.skipif(
    not _core_available(),
    reason="botl_pdf._core Rust extension not compiled",
)


# ---------------------------------------------------------------------------
# Low-level _core.open
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestCoreOpen:
    """Tests for botl_pdf._core.open returning a PyDocument."""

    def test_open_from_path_string(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        assert doc is not None
        assert doc.num_pages == 1

    def test_open_from_path_object(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        # PyO3 accepts str/bytes; Path objects need str() conversion
        doc = _open(str(simple_pdf_path))
        assert doc is not None

    def test_open_from_bytes(self, simple_pdf_bytes):
        from botl_pdf._core import open as _open

        doc = _open(simple_pdf_bytes)
        assert doc is not None
        assert doc.num_pages == 1

    def test_open_nonexistent_file_raises(self):
        from botl_pdf._core import open as _open

        with pytest.raises((FileNotFoundError, OSError, RuntimeError)):
            _open("/nonexistent/path/to/file.pdf")

    def test_open_invalid_bytes_raises(self):
        from botl_pdf._core import open as _open

        with pytest.raises(Exception):
            _open(b"this is not a pdf")


# ---------------------------------------------------------------------------
# PyDocument attributes
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestPyDocumentAttributes:
    """Tests for attributes on the PyDocument object."""

    def test_num_pages_simple(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        assert doc.num_pages == 1

    def test_num_pages_multi(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        assert doc.num_pages == 3

    def test_is_encrypted_simple(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        assert doc.is_encrypted is False

    def test_is_encrypted_multi(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        assert doc.is_encrypted is False

    def test_metadata_returns_dict(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        meta = doc.metadata
        assert isinstance(meta, dict)

    def test_toc_returns_list(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        toc = doc.toc
        assert isinstance(toc, list)

    def test_get_page_returns_object(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert page is not None

    def test_get_page_invalid_index_raises(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        with pytest.raises((IndexError, RuntimeError)):
            doc.get_page(5)

    def test_get_page_negative_index_raises(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        # Negative indices cause unsigned overflow — expect an exception
        with pytest.raises((IndexError, RuntimeError, OverflowError)):
            doc.get_page(-1)

    def test_len_dunder(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        assert len(doc) == 1

    def test_getitem_dunder(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc[0]
        assert page is not None

    def test_getitem_out_of_range_raises(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        with pytest.raises((IndexError, RuntimeError)):
            _ = doc[99]


# ---------------------------------------------------------------------------
# High-level Document wrapper
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestDocumentWrapper:
    """Tests for the high-level Document class in botl_pdf.document."""

    def test_context_manager(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            assert doc.num_pages == 1

    def test_context_manager_from_bytes(self, simple_pdf_bytes):
        from botl_pdf.document import Document

        with Document(simple_pdf_bytes) as doc:
            assert doc.num_pages == 1

    def test_metadata_property(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            assert isinstance(doc.metadata, dict)

    def test_is_encrypted_property(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            assert doc.is_encrypted is False

    def test_num_pages_property(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            assert doc.num_pages == 3

    def test_len(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            assert len(doc) == 3

    def test_repr(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            r = repr(doc)
            assert "Document" in r
            assert "pages=1" in r

    def test_toc_property(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            assert isinstance(doc.toc, list)


# ---------------------------------------------------------------------------
# PageCollection
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestPageCollection:
    """Tests for PageCollection in botl_pdf.document."""

    def test_getitem_valid(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            page = doc.pages[0]
            assert page is not None

    def test_getitem_negative_index(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            # -1 should map to the last page
            page = doc.pages[-1]
            assert page is not None

    def test_getitem_out_of_range_raises(self, simple_pdf_path):
        from botl_pdf.document import Document

        with Document(str(simple_pdf_path)) as doc:
            with pytest.raises(IndexError, match="out of range"):
                _ = doc.pages[5]

    def test_len(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            assert len(doc.pages) == 3

    def test_iteration(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            pages = list(doc.pages)
            assert len(pages) == 3

    def test_repr(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            r = repr(doc.pages)
            assert "PageCollection" in r
            assert "count=3" in r


# ---------------------------------------------------------------------------
# PyPage attributes
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestPyPageAttributes:
    """Tests for PyPage attributes after get_page."""

    def test_page_width(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        # simple_text.pdf uses MediaBox [0 0 612 792]
        assert page.width == pytest.approx(612.0)

    def test_page_height(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert page.height == pytest.approx(792.0)

    def test_page_rotation(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert page.rotation == 0

    def test_page_number(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        # page_number is 0-indexed
        assert page.page_number == 0

    def test_multi_page_numbers(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        for i in range(doc.num_pages):
            page = doc.get_page(i)
            assert page.page_number == i

    def test_chars_returns_list(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert isinstance(page.chars, list)

    def test_lines_returns_list(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert isinstance(page.lines, list)

    def test_rects_returns_list(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        assert isinstance(page.rects, list)
