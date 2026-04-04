"""Integration tests for PDF metadata extraction.

All tests are skipped when _core is not compiled.
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


@skip_if_no_core
class TestMetadata:
    """Tests for metadata extraction from PDF documents."""

    def test_metadata_is_dict(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert isinstance(meta, dict)

    def test_metadata_has_title(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert "title" in meta
        assert meta["title"] == "Test PDF Title"

    def test_metadata_has_author(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert "author" in meta
        assert meta["author"] == "Test Author"

    def test_metadata_has_subject(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert "subject" in meta
        assert meta["subject"] == "Test Subject"

    def test_metadata_has_creator(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert "creator" in meta
        assert meta["creator"] == "Test Creator"

    def test_metadata_has_producer(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        assert "producer" in meta
        assert meta["producer"] == "botl-pdf test"

    def test_metadata_all_expected_keys(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        meta = doc.metadata
        expected_keys = {"title", "author", "subject", "creator", "producer"}
        assert expected_keys.issubset(meta.keys()), (
            f"Missing keys: {expected_keys - meta.keys()}"
        )

    def test_metadata_from_bytes(self, metadata_pdf_bytes):
        from botl_pdf._core import open as _open

        doc = _open(metadata_pdf_bytes)
        meta = doc.metadata
        assert meta["title"] == "Test PDF Title"
        assert meta["author"] == "Test Author"

    def test_simple_pdf_metadata_is_dict(self, simple_pdf_path):
        """Even a PDF with no Info dict should return a dict (possibly empty)."""
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        meta = doc.metadata
        assert isinstance(meta, dict)

    def test_multi_page_pdf_metadata(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        meta = doc.metadata
        assert isinstance(meta, dict)


@skip_if_no_core
class TestMetadataViaDocumentWrapper:
    """Tests for metadata via the high-level Document wrapper."""

    def test_document_metadata_property(self, metadata_pdf_path):
        from botl_pdf.document import Document

        with Document(str(metadata_pdf_path)) as doc:
            meta = doc.metadata
            assert isinstance(meta, dict)
            assert meta.get("title") == "Test PDF Title"
            assert meta.get("author") == "Test Author"


@skip_if_no_core
class TestPageCountMatchesMetadata:
    """Verify that page count from the document matches expectations."""

    def test_simple_pdf_page_count(self, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        assert doc.num_pages == 1

    def test_multi_page_pdf_page_count(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        assert doc.num_pages == 3

    def test_metadata_pdf_page_count(self, metadata_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(metadata_pdf_path))
        assert doc.num_pages == 1

    def test_compressed_pdf_page_count(self, compressed_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(compressed_pdf_path))
        assert doc.num_pages == 1

    def test_len_matches_num_pages(self, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        assert len(doc) == doc.num_pages == 3

    def test_document_wrapper_len_matches(self, multi_page_pdf_path):
        from botl_pdf.document import Document

        with Document(str(multi_page_pdf_path)) as doc:
            assert len(doc) == doc.num_pages == 3
