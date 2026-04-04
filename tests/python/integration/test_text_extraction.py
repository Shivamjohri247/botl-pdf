"""Integration tests for text extraction.

Tests exercise both the low-level PyPage.extract_text and the high-level
Page wrapper. All tests are skipped when _core is not compiled.
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
# Low-level text extraction via PyPage
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestPyPageExtractText:
    """Tests for text extraction directly on PyPage objects."""

    def test_simple_pdf_hello_world(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Hello World" in text

    def test_simple_pdf_returns_non_empty(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert len(text) > 0

    def test_multi_page_page_one(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Page One" in text

    def test_multi_page_page_two(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        page = doc.get_page(1)
        text = page.extract_text()
        assert "Page Two" in text

    def test_multi_page_page_three(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        page = doc.get_page(2)
        text = page.extract_text()
        assert "Page Three" in text

    def test_compressed_pdf_text(self, compressed_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(compressed_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Compressed Text" in text

    def test_metadata_pdf_text(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Test Document" in text

    def test_layout_mode_returns_string(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text(layout=True)
        assert isinstance(text, str)
        # Layout mode preserves spatial whitespace; verify both words present
        assert "Hello" in text and "World" in text

    def test_layout_mode_multi_page(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text(layout=True)
        assert "Page" in text and "One" in text

    def test_bytes_input_simple(self, simple_pdf_bytes):
        from botl_pdf._core import open as _open

        doc = _open(simple_pdf_bytes)
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Hello World" in text


# ---------------------------------------------------------------------------
# High-level text extraction via Page wrapper
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestPageWrapperExtractText:
    """Tests for text extraction via the high-level Page wrapper."""

    def test_simple_pdf(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text()
        assert "Hello" in text and "World" in text

    def test_with_custom_word_margin(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text(word_margin=0.2)
        assert "Hello" in text and "World" in text

    def test_with_custom_line_margin(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text(line_margin=0.3)
        assert "Hello" in text and "World" in text

    def test_with_custom_boxes_flow(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text(boxes_flow=0.0)
        assert "Hello" in text and "World" in text

    def test_layout_mode(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text(layout=True)
        assert "Hello" in text and "World" in text

    def test_all_params_combined(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        text = page.extract_text(
            layout=True,
            word_margin=0.15,
            line_margin=0.4,
            boxes_flow=0.5,
        )
        assert "Hello" in text and "World" in text

    def test_page_properties(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        assert page.width == pytest.approx(612.0)
        assert page.height == pytest.approx(792.0)
        assert page.rotation == 0
        # page_number is 0-indexed
        assert page.page_number == 0

    def test_page_repr(self, simple_pdf_path):
        from botl_pdf._core import open as _open
        from botl_pdf.page import Page

        doc = _open(str(simple_pdf_path))
        page = Page(doc.get_page(0))
        r = repr(page)
        assert "Page" in r
        assert "number=0" in r


# ---------------------------------------------------------------------------
# Multi-page content verification
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestMultiPageTextExtraction:
    """Verify text extraction across all pages of a multi-page document."""

    def test_all_pages_have_content(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        texts = [doc.get_page(i).extract_text() for i in range(doc.num_pages)]
        assert len(texts) == 3
        assert all(t.strip() for t in texts), "Each page should have non-empty text"

    def test_pages_contain_distinct_text(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        texts = [doc.get_page(i).extract_text() for i in range(doc.num_pages)]
        assert "Page One" in texts[0]
        assert "Page Two" in texts[1]
        assert "Page Three" in texts[2]

    def test_page_content_does_not_bleed_across_pages(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        page0_text = doc.get_page(0).extract_text()
        # Page 0 should contain "Page One" but not "Page Two" or "Page Three"
        assert "Page One" in page0_text
        assert "Page Two" not in page0_text
        assert "Page Three" not in page0_text

    def test_iteration_via_document_wrapper(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        expected = ["Page One", "Page Two", "Page Three"]
        with Document(str(multi_page_pdf_path)) as doc:
            for i, page in enumerate(doc.pages):
                text = page.extract_text()
                assert expected[i] in text

    def test_compressed_content_extraction(self, compressed_pdf_path):
        """FlateDecode compressed streams should be transparently decompressed."""
        from botl_pdf._core import open as _open

        doc = _open(str(compressed_pdf_path))
        page = doc.get_page(0)
        text = page.extract_text()
        assert "Compressed Text" in text
