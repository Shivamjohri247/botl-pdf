"""Unit tests for CLI argument parsing.

The _parse_page_range function is pure Python and does not require
the compiled Rust extension. We load the module file directly to
avoid triggering botl_pdf.__init__ which imports _core.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Import _parse_page_range directly from file, bypassing __init__.py
# ---------------------------------------------------------------------------

_CLI_MAIN_PATH = Path(__file__).resolve().parent.parent.parent.parent / "python" / "botl_pdf" / "cli" / "main.py"


def _load_cli_main():
    """Load botl_pdf.cli.main without triggering package __init__."""
    mod_name = "botl_pdf.cli.main"
    if mod_name in sys.modules:
        return sys.modules[mod_name]
    spec = importlib.util.spec_from_file_location(mod_name, str(_CLI_MAIN_PATH))
    assert spec is not None
    mod = importlib.util.module_from_spec(spec)
    sys.modules[mod_name] = mod
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


_cli_mod = _load_cli_main()
_parse_page_range = _cli_mod._parse_page_range


class TestParsePageRange:
    """Tests for _parse_page_range(page_str, total)."""

    # ------------------------------------------------------------------
    # None / default -> all pages
    # ------------------------------------------------------------------

    def test_none_returns_all_pages(self):
        result = _parse_page_range(None, 10)
        assert list(result) == list(range(10))

    def test_none_with_single_page_doc(self):
        result = _parse_page_range(None, 1)
        assert list(result) == [0]

    def test_none_with_empty_doc(self):
        result = _parse_page_range(None, 0)
        assert list(result) == []

    # ------------------------------------------------------------------
    # Single page
    # ------------------------------------------------------------------

    def test_single_page_number(self):
        result = _parse_page_range("1", 10)
        assert list(result) == [0]

    def test_single_last_page(self):
        result = _parse_page_range("10", 10)
        assert list(result) == [9]

    def test_single_page_in_middle(self):
        result = _parse_page_range("5", 10)
        assert list(result) == [4]

    # ------------------------------------------------------------------
    # Range: "start-end"
    # ------------------------------------------------------------------

    def test_range_1_to_5(self):
        result = _parse_page_range("1-5", 10)
        assert list(result) == [0, 1, 2, 3, 4]

    def test_range_full_document(self):
        result = _parse_page_range("1-10", 10)
        assert list(result) == list(range(10))

    def test_range_single_value(self):
        # "3-3" means pages 3 through 3 (0-based: [2])
        result = _parse_page_range("3-3", 10)
        assert list(result) == [2]

    def test_range_clamps_to_total(self):
        # If range extends beyond total, it is clamped
        result = _parse_page_range("1-20", 5)
        assert list(result) == [0, 1, 2, 3, 4]

    # ------------------------------------------------------------------
    # Comma-separated: "1,3,5"
    # ------------------------------------------------------------------

    def test_comma_separated_pages(self):
        result = _parse_page_range("1,3,5", 10)
        # The function returns range(min, max+1), so pages 0..4 inclusive
        assert list(result) == [0, 1, 2, 3, 4]

    def test_comma_separated_out_of_order(self):
        # "5,2,7" -> indices 4, 1, 6 -> range(1, 7) = [1,2,3,4,5,6]
        result = _parse_page_range("5,2,7", 10)
        assert list(result) == [1, 2, 3, 4, 5, 6]

    # ------------------------------------------------------------------
    # Mixed: "1-3,7"
    # ------------------------------------------------------------------

    def test_mixed_range_and_single(self):
        result = _parse_page_range("1-3,7", 10)
        # indices: 0,1,2 from range + 6 from single -> range(0,7) = [0..6]
        assert list(result) == [0, 1, 2, 3, 4, 5, 6]

    def test_mixed_range_and_single_reversed(self):
        # "7,1-3" -> indices 6, 0,1,2 -> range(0, 7) = [0..6]
        result = _parse_page_range("7,1-3", 10)
        assert list(result) == [0, 1, 2, 3, 4, 5, 6]

    # ------------------------------------------------------------------
    # Edge cases
    # ------------------------------------------------------------------

    def test_page_exceeding_total_is_skipped(self):
        # Single page number beyond total: "15" in a 10-page doc
        # The idx (14) fails the 0 <= idx < total check, so nothing is added.
        # Empty result_indices means return range(total).
        result = _parse_page_range("15", 10)
        assert list(result) == list(range(10))

    def test_whitespace_around_parts(self):
        result = _parse_page_range(" 1 , 3 ", 10)
        assert list(result) == [0, 1, 2]  # range(0, 3)

    def test_empty_string_raises(self):
        # Empty string splits to [""] which is not a valid int.
        # int("") raises ValueError.
        with pytest.raises(ValueError):
            _parse_page_range("", 10)

    def test_page_zero_returns_all(self):
        # "0" -> idx = -1, which is < 0 so it's skipped.
        # Empty result_indices means return range(total).
        result = _parse_page_range("0", 10)
        assert list(result) == list(range(10))
