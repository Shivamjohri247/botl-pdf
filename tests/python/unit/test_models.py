"""Unit tests for Python-level data models.

These tests exercise pure-Python dataclasses and registry functions
that do NOT require the compiled Rust extension module. They use
importlib to load submodules directly, bypassing botl_pdf/__init__.py
which tries to import _core at package level.
"""

from __future__ import annotations

import importlib
import importlib.util
import sys
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Helper: import a submodule without triggering botl_pdf.__init__
# ---------------------------------------------------------------------------

_PYTHON_DIR = Path(__file__).resolve().parent.parent.parent.parent / "python"


def _import_submodule(dotted_name: str, file_path: Path):
    """Import a submodule by its file path, skipping the package __init__."""
    module_name = dotted_name
    if module_name in sys.modules:
        return sys.modules[module_name]
    spec = importlib.util.spec_from_file_location(module_name, str(file_path))
    assert spec is not None, f"Could not create spec for {file_path}"
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


# Pre-load the pure-Python submodules we need.
_tables_mod = _import_submodule("botl_pdf.tables", _PYTHON_DIR / "botl_pdf" / "tables.py")
_ocr_base_mod = _import_submodule("botl_pdf.ocr.base", _PYTHON_DIR / "botl_pdf" / "ocr" / "base.py")
_plugins_registry_mod = _import_submodule(
    "botl_pdf.plugins.registry", _PYTHON_DIR / "botl_pdf" / "plugins" / "registry.py"
)

# Convenience aliases
TableCell = _tables_mod.TableCell
TableRow = _tables_mod.TableRow
Table = _tables_mod.Table
TableDetectionStrategy = _tables_mod.TableDetectionStrategy

OCRResult = _ocr_base_mod.OCRResult
OCRBackend = _ocr_base_mod.OCRBackend

register_exporter = _plugins_registry_mod.register_exporter
register_table_strategy = _plugins_registry_mod.register_table_strategy
get_exporter = _plugins_registry_mod.get_exporter
get_table_strategy = _plugins_registry_mod.get_table_strategy


# ---------------------------------------------------------------------------
# TableCell / TableRow / Table (botl_pdf.tables)
# ---------------------------------------------------------------------------

class TestTableCell:
    """Tests for the TableCell frozen dataclass."""

    def test_create_basic_cell(self):
        cell = TableCell(text="Hello", x0=0.0, y0=0.0, x1=50.0, y1=12.0)
        assert cell.text == "Hello"
        assert cell.x0 == 0.0
        assert cell.y0 == 0.0
        assert cell.x1 == 50.0
        assert cell.y1 == 12.0
        assert cell.row_span == 1
        assert cell.col_span == 1
        assert cell.is_header is False

    def test_create_header_cell(self):
        cell = TableCell(
            text="Name",
            x0=0.0,
            y0=0.0,
            x1=80.0,
            y1=14.0,
            row_span=2,
            col_span=1,
            is_header=True,
        )
        assert cell.is_header is True
        assert cell.row_span == 2
        assert cell.col_span == 1

    def test_frozen_immutable(self):
        cell = TableCell(text="X", x0=0.0, y0=0.0, x1=10.0, y1=10.0)
        with pytest.raises(AttributeError):
            cell.text = "Y"  # type: ignore[misc]

    def test_equality(self):
        a = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        b = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        assert a == b

    def test_inequality(self):
        a = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        b = TableCell(text="B", x0=0, y0=0, x1=10, y1=10)
        assert a != b

    def test_hash_in_set(self):
        a = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        b = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        assert len({a, b}) == 1


class TestTableRow:
    """Tests for the TableRow frozen dataclass."""

    def test_create_row(self):
        c1 = TableCell(text="A", x0=0, y0=0, x1=10, y1=10)
        c2 = TableCell(text="B", x0=10, y0=0, x1=20, y1=10)
        row = TableRow(cells=(c1, c2))
        assert len(row.cells) == 2
        assert row.cells[0].text == "A"
        assert row.cells[1].text == "B"

    def test_empty_row(self):
        row = TableRow(cells=())
        assert len(row.cells) == 0

    def test_frozen(self):
        c = TableCell(text="Z", x0=0, y0=0, x1=5, y1=5)
        row = TableRow(cells=(c,))
        with pytest.raises(AttributeError):
            row.cells = ()  # type: ignore[misc]


class TestTable:
    """Tests for the Table frozen dataclass and its conversion methods."""

    @pytest.fixture()
    def sample_table(self):
        header_row = TableRow(
            cells=(
                TableCell(text="Name", x0=0, y0=0, x1=50, y1=12, is_header=True),
                TableCell(text="Age", x0=50, y0=0, x1=100, y1=12, is_header=True),
            )
        )
        data_row = TableRow(
            cells=(
                TableCell(text="Alice", x0=0, y0=12, x1=50, y1=24),
                TableCell(text="30", x0=50, y0=12, x1=100, y1=24),
            )
        )
        return Table(
            rows=(header_row, data_row),
            x0=0, y0=0, x1=100, y1=24,
        )

    def test_to_list(self, sample_table):
        result = sample_table.to_list()
        assert result == [
            ["Name", "Age"],
            ["Alice", "30"],
        ]

    def test_to_markdown(self, sample_table):
        md = sample_table.to_markdown()
        lines = md.split("\n")
        assert lines[0] == "| Name | Age |"
        assert lines[1] == "| --- | --- |"
        assert lines[2] == "| Alice | 30 |"

    def test_empty_table_to_markdown(self):
        table = Table(rows=(), x0=0, y0=0, x1=0, y1=0)
        assert table.to_markdown() == ""

    def test_empty_table_to_list(self):
        table = Table(rows=(), x0=0, y0=0, x1=0, y1=0)
        assert table.to_list() == []

    def test_single_row_table(self):
        row = TableRow(
            cells=(TableCell(text="Only", x0=0, y0=0, x1=30, y1=12),)
        )
        table = Table(rows=(row,), x0=0, y0=0, x1=30, y1=12)
        assert table.to_list() == [["Only"]]
        md = table.to_markdown()
        assert "Only" in md

    def test_frozen(self, sample_table):
        with pytest.raises(AttributeError):
            sample_table.rows = ()  # type: ignore[misc]

    def test_bounding_box_attributes(self, sample_table):
        assert sample_table.x0 == 0
        assert sample_table.y0 == 0
        assert sample_table.x1 == 100
        assert sample_table.y1 == 24


class TestTableDetectionStrategy:
    """Test that the TableDetectionStrategy protocol is runtime-checkable."""

    def test_conforming_class(self):
        class MyStrategy:
            def detect(self, chars, lines, rects):
                return []

            def is_available(self):
                return True

        assert isinstance(MyStrategy(), TableDetectionStrategy)

    def test_non_conforming_class(self):
        class BadStrategy:
            pass

        assert not isinstance(BadStrategy(), TableDetectionStrategy)


# ---------------------------------------------------------------------------
# OCRResult (botl_pdf.ocr.base)
# ---------------------------------------------------------------------------

class TestOCRResult:
    """Tests for the OCRResult frozen dataclass."""

    def test_create_result(self):
        result = OCRResult(
            text="hello",
            confidence=0.95,
            x0=10.0,
            y0=20.0,
            x1=100.0,
            y1=40.0,
        )
        assert result.text == "hello"
        assert result.confidence == pytest.approx(0.95)
        assert result.x0 == 10.0
        assert result.y1 == 40.0

    def test_frozen(self):
        result = OCRResult(text="x", confidence=1.0, x0=0, y0=0, x1=10, y1=10)
        with pytest.raises(AttributeError):
            result.text = "y"  # type: ignore[misc]

    def test_equality(self):
        a = OCRResult(text="a", confidence=0.5, x0=0, y0=0, x1=10, y1=10)
        b = OCRResult(text="a", confidence=0.5, x0=0, y0=0, x1=10, y1=10)
        assert a == b

    def test_hash(self):
        a = OCRResult(text="a", confidence=0.5, x0=0, y0=0, x1=10, y1=10)
        b = OCRResult(text="a", confidence=0.5, x0=0, y0=0, x1=10, y1=10)
        assert hash(a) == hash(b)
        assert len({a, b}) == 1


class TestOCRBackend:
    """Tests for the OCRBackend ABC."""

    def test_cannot_instantiate_directly(self):
        with pytest.raises(TypeError):
            OCRBackend()  # type: ignore[abstract]

    def test_subclass_must_implement_methods(self):
        class Partial(OCRBackend):
            pass

        with pytest.raises(TypeError):
            Partial()  # type: ignore[abstract]

    def test_valid_subclass(self):
        class DummyBackend(OCRBackend):
            def recognize(self, image_bytes, language="eng"):
                return [OCRResult(text="hi", confidence=1.0, x0=0, y0=0, x1=10, y1=10)]

            def is_available(self):
                return True

        backend = DummyBackend()
        results = backend.recognize(b"\x00")
        assert len(results) == 1
        assert results[0].text == "hi"
        assert backend.is_available()


# ---------------------------------------------------------------------------
# Plugin registry (botl_pdf.plugins.registry)
# ---------------------------------------------------------------------------

class TestPluginRegistry:
    """Tests for the plugin registry functions."""

    def setup_method(self):
        """Reset registries before each test."""
        _plugins_registry_mod._EXPORTERS.clear()
        _plugins_registry_mod._TABLE_STRATEGIES.clear()

    def test_register_and_get_exporter(self):
        def my_exporter(doc):
            return "exported"

        register_exporter("csv", my_exporter)
        assert get_exporter("csv") is my_exporter

    def test_get_unknown_exporter_raises(self):
        with pytest.raises(KeyError, match="No exporter registered"):
            get_exporter("nonexistent")

    def test_register_and_get_table_strategy(self):
        def my_strategy(chars, lines, rects):
            return []

        register_table_strategy("custom", my_strategy)
        assert get_table_strategy("custom") is my_strategy

    def test_get_unknown_table_strategy_raises(self):
        with pytest.raises(KeyError, match="No table strategy registered"):
            get_table_strategy("nonexistent")

    def test_overwrite_exporter(self):
        def v1(doc):
            return "v1"

        def v2(doc):
            return "v2"

        register_exporter("json", v1)
        register_exporter("json", v2)
        assert get_exporter("json") is v2

    def test_overwrite_table_strategy(self):
        def old(chars, lines, rects):
            return []

        def new(chars, lines, rects):
            return []

        register_table_strategy("lattice", old)
        register_table_strategy("lattice", new)
        assert get_table_strategy("lattice") is new


class TestPluginPackageExports:
    """Verify the public API exported from botl_pdf.plugins."""

    def test_all_exports_available(self):
        assert callable(register_exporter)
        assert callable(register_table_strategy)
        assert callable(get_exporter)
        assert callable(get_table_strategy)
