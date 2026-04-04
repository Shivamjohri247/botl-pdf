"""Basic performance benchmarks for botl-pdf.

Uses pytest-benchmark if available; otherwise falls back to simple
time-based measurements via the `time` module. All benchmarks are
skipped when _core is not compiled.
"""

from __future__ import annotations

import importlib
import time
from functools import partial

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

# Try to import pytest-benchmark; mark tests accordingly.
try:
    import pytest_benchmark  # noqa: F401

    HAS_BENCHMARK = True
except ImportError:
    HAS_BENCHMARK = False

requires_benchmark = pytest.mark.skipif(
    not HAS_BENCHMARK,
    reason="pytest-benchmark not installed (pip install pytest-benchmark)",
)


# ---------------------------------------------------------------------------
# Benchmark: Document opening
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestBenchDocumentOpen:
    """Benchmarks for opening PDF documents."""

    @requires_benchmark
    def test_bench_open_simple_pdf(self, benchmark, simple_pdf_path):
        from botl_pdf._core import open as _open

        path = str(simple_pdf_path)
        benchmark(_open, path)

    @requires_benchmark
    def test_bench_open_simple_pdf_from_bytes(self, benchmark, simple_pdf_bytes):
        from botl_pdf._core import open as _open

        benchmark(_open, simple_pdf_bytes)

    @requires_benchmark
    def test_bench_open_multi_page_pdf(self, benchmark, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        path = str(multi_page_pdf_path)
        benchmark(_open, path)

    @requires_benchmark
    def test_bench_open_compressed_pdf(self, benchmark, compressed_pdf_path):
        from botl_pdf._core import open as _open

        path = str(compressed_pdf_path)
        benchmark(_open, path)

    def test_open_simple_pdf_timing(self, simple_pdf_path):
        """Simple timing benchmark when pytest-benchmark is unavailable."""
        from botl_pdf._core import open as _open

        path = str(simple_pdf_path)
        iterations = 50
        start = time.perf_counter()
        for _ in range(iterations):
            _open(path)
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        # Just ensure it completes and reports a positive time
        assert avg_ms > 0, "Benchmark completed successfully"

    def test_open_multi_page_timing(self, multi_page_pdf_path):
        """Simple timing benchmark for multi-page document."""
        from botl_pdf._core import open as _open

        path = str(multi_page_pdf_path)
        iterations = 50
        start = time.perf_counter()
        for _ in range(iterations):
            _open(path)
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        assert avg_ms > 0, "Benchmark completed successfully"


# ---------------------------------------------------------------------------
# Benchmark: Text extraction
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestBenchTextExtraction:
    """Benchmarks for text extraction."""

    @requires_benchmark
    def test_bench_extract_text_simple(self, benchmark, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        benchmark(page.extract_text)

    @requires_benchmark
    def test_bench_extract_text_layout(self, benchmark, simple_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        benchmark(page.extract_text, layout=True)

    @requires_benchmark
    def test_bench_extract_text_multi_page(self, benchmark, multi_page_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        pages = [doc.get_page(i) for i in range(doc.num_pages)]

        def extract_all():
            return [p.extract_text() for p in pages]

        benchmark(extract_all)

    @requires_benchmark
    def test_bench_extract_text_compressed(self, benchmark, compressed_pdf_path):
        from botl_pdf._core import open as _open

        doc = _open(str(compressed_pdf_path))
        page = doc.get_page(0)
        benchmark(page.extract_text)

    def test_extract_text_simple_timing(self, simple_pdf_path):
        """Simple timing benchmark for text extraction."""
        from botl_pdf._core import open as _open

        doc = _open(str(simple_pdf_path))
        page = doc.get_page(0)
        iterations = 100
        start = time.perf_counter()
        for _ in range(iterations):
            page.extract_text()
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        assert avg_ms > 0, "Benchmark completed successfully"

    def test_extract_text_multi_page_timing(self, multi_page_pdf_path):
        """Simple timing benchmark for multi-page text extraction."""
        from botl_pdf._core import open as _open

        doc = _open(str(multi_page_pdf_path))
        iterations = 50
        start = time.perf_counter()
        for _ in range(iterations):
            for i in range(doc.num_pages):
                doc.get_page(i).extract_text()
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        assert avg_ms > 0, "Benchmark completed successfully"


# ---------------------------------------------------------------------------
# Benchmark: Export functions
# ---------------------------------------------------------------------------

@skip_if_no_core
class TestBenchExport:
    """Benchmarks for the export convenience functions."""

    @requires_benchmark
    def test_bench_to_markdown_simple(self, benchmark, simple_pdf_path):
        from botl_pdf.export import to_markdown

        benchmark(to_markdown, str(simple_pdf_path))

    @requires_benchmark
    def test_bench_to_text_simple(self, benchmark, simple_pdf_path):
        from botl_pdf.export import to_text

        benchmark(to_text, str(simple_pdf_path))

    @requires_benchmark
    def test_bench_to_markdown_multi_page(self, benchmark, multi_page_pdf_path):
        from botl_pdf.export import to_markdown

        benchmark(to_markdown, str(multi_page_pdf_path))

    @requires_benchmark
    def test_bench_to_text_multi_page(self, benchmark, multi_page_pdf_path):
        from botl_pdf.export import to_text

        benchmark(to_text, str(multi_page_pdf_path))

    def test_to_markdown_timing(self, multi_page_pdf_path):
        """Simple timing benchmark for to_markdown."""
        from botl_pdf.export import to_markdown

        path = str(multi_page_pdf_path)
        iterations = 50
        start = time.perf_counter()
        for _ in range(iterations):
            to_markdown(path)
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        assert avg_ms > 0, "Benchmark completed successfully"

    def test_to_text_timing(self, multi_page_pdf_path):
        """Simple timing benchmark for to_text."""
        from botl_pdf.export import to_text

        path = str(multi_page_pdf_path)
        iterations = 50
        start = time.perf_counter()
        for _ in range(iterations):
            to_text(path)
        elapsed = time.perf_counter() - start
        avg_ms = (elapsed / iterations) * 1000
        assert avg_ms > 0, "Benchmark completed successfully"
