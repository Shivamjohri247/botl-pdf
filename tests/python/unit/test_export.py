"""Unit tests for the export module.

These tests require the compiled Rust extension because to_markdown and
to_text call into _core.open. They are skipped when _core is unavailable.
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
# to_markdown
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestToMarkdown:
    """Tests for botl_pdf.export.to_markdown."""

    def test_simple_pdf_returns_string(self, simple_pdf_path):
        from botl_pdf.export import to_markdown

        result = to_markdown(str(simple_pdf_path))
        assert isinstance(result, str)

    def test_simple_pdf_contains_hello_world(self, simple_pdf_path):
        from botl_pdf.export import to_markdown

        result = to_markdown(str(simple_pdf_path))
        assert "Hello World" in result

    def test_simple_pdf_from_bytes(self, simple_pdf_bytes):
        from botl_pdf.export import to_markdown

        result = to_markdown(simple_pdf_bytes)
        assert "Hello World" in result

    def test_multi_page_separates_with_horizontal_rule(self, multi_page_pdf_path):
        from botl_pdf.export import to_markdown

        result = to_markdown(str(multi_page_pdf_path))
        # Pages are separated by "\n\n---\n\n"
        assert "---" in result

    def test_multi_page_contains_all_page_text(self, multi_page_pdf_path):
        from botl_pdf.export import to_markdown

        result = to_markdown(str(multi_page_pdf_path))
        assert "Page One" in result
        assert "Page Two" in result
        assert "Page Three" in result

    def test_pages_range_filter(self, multi_page_pdf_path):
        from botl_pdf.export import to_markdown

        # Only page index 0 (first page)
        result = to_markdown(str(multi_page_pdf_path), pages=range(1))
        assert "Page One" in result
        # Pages Two and Three should not be present
        assert "Page Two" not in result
        assert "Page Three" not in result

    def test_compressed_pdf(self, compressed_pdf_path):
        from botl_pdf.export import to_markdown

        result = to_markdown(str(compressed_pdf_path))
        assert "Compressed Text" in result


# ---------------------------------------------------------------------------
# to_text
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestToText:
    """Tests for botl_pdf.export.to_text."""

    def test_simple_pdf_returns_string(self, simple_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(simple_pdf_path))
        assert isinstance(result, str)

    def test_simple_pdf_contains_hello_world(self, simple_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(simple_pdf_path))
        assert "Hello World" in result

    def test_simple_pdf_from_bytes(self, simple_pdf_bytes):
        from botl_pdf.export import to_text

        result = to_text(simple_pdf_bytes)
        assert "Hello World" in result

    def test_multi_page_joins_with_blank_lines(self, multi_page_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(multi_page_pdf_path))
        assert "Page One" in result
        assert "Page Two" in result
        assert "Page Three" in result
        # Pages are joined by "\n\n"
        parts = result.split("\n\n")
        assert len(parts) >= 3

    def test_layout_mode(self, simple_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(simple_pdf_path), layout=True)
        # Layout mode preserves spatial whitespace; verify both words present
        assert "Hello" in result and "World" in result

    def test_compressed_pdf(self, compressed_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(compressed_pdf_path))
        assert "Compressed Text" in result

    def test_metadata_pdf_text(self, metadata_pdf_path):
        from botl_pdf.export import to_text

        result = to_text(str(metadata_pdf_path))
        assert "Test Document" in result
