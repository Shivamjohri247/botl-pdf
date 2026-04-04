"""Shared fixtures for botl-pdf Python test suite."""

from __future__ import annotations

import importlib
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Paths to test fixtures
# ---------------------------------------------------------------------------

FIXTURES_DIR = Path(__file__).resolve().parent.parent / "fixtures"


def _fixture(name: str) -> Path:
    """Return the absolute path to a named fixture file."""
    p = FIXTURES_DIR / name
    assert p.is_file(), f"Missing test fixture: {p}"
    return p


# ---------------------------------------------------------------------------
# Core availability helper
# ---------------------------------------------------------------------------

def _core_available() -> bool:
    """Return True if the compiled Rust extension (_core) can be imported."""
    try:
        importlib.import_module("botl_pdf._core")
        return True
    except ImportError:
        return False


CORE_AVAILABLE = _core_available()

skip_if_no_core = pytest.mark.skipif(
    not CORE_AVAILABLE,
    reason="botl_pdf._core Rust extension not compiled",
)


# ---------------------------------------------------------------------------
# Path fixtures
# ---------------------------------------------------------------------------

@pytest.fixture()
def fixtures_dir() -> Path:
    """Return the directory containing PDF test fixtures."""
    return FIXTURES_DIR


@pytest.fixture()
def simple_pdf_path() -> Path:
    """Path to simple_text.pdf (single page with 'Hello World')."""
    return _fixture("simple_text.pdf")


@pytest.fixture()
def multi_page_pdf_path() -> Path:
    """Path to multi_page.pdf (3 pages: Page One, Page Two, Page Three)."""
    return _fixture("multi_page.pdf")


@pytest.fixture()
def metadata_pdf_path() -> Path:
    """Path to metadata.pdf (Title, Author, Subject, Creator, Producer)."""
    return _fixture("metadata.pdf")


@pytest.fixture()
def compressed_pdf_path() -> Path:
    """Path to flate_compressed.pdf (FlateDecode compressed content stream)."""
    return _fixture("flate_compressed.pdf")


@pytest.fixture()
def simple_pdf_bytes(simple_pdf_path: Path) -> bytes:
    """Raw bytes of simple_text.pdf."""
    return simple_pdf_path.read_bytes()


@pytest.fixture()
def multi_page_pdf_bytes(multi_page_pdf_path: Path) -> bytes:
    """Raw bytes of multi_page.pdf."""
    return multi_page_pdf_path.read_bytes()


@pytest.fixture()
def metadata_pdf_bytes(metadata_pdf_path: Path) -> bytes:
    """Raw bytes of metadata.pdf."""
    return metadata_pdf_path.read_bytes()


@pytest.fixture()
def compressed_pdf_bytes(compressed_pdf_path: Path) -> bytes:
    """Raw bytes of flate_compressed.pdf."""
    return compressed_pdf_path.read_bytes()
