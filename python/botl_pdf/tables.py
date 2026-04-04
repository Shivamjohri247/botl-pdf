"""Table extraction helpers.

Table extraction will be implemented in Phase 2 (v0.2.0).
This module provides the data types and protocol definitions
so that downstream code can be written against the expected API.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Protocol, runtime_checkable


@dataclass(frozen=True, slots=True)
class TableCell:
    """A single table cell."""
    text: str
    x0: float
    y0: float
    x1: float
    y1: float
    row_span: int = 1
    col_span: int = 1
    is_header: bool = False


@dataclass(frozen=True, slots=True)
class TableRow:
    """A row of table cells."""
    cells: tuple[TableCell, ...]


@dataclass(frozen=True, slots=True)
class Table:
    """A detected table on a page."""
    rows: tuple[TableRow, ...]
    x0: float
    y0: float
    x1: float
    y1: float

    def to_list(self) -> list[list[str]]:
        """Convert table to a list of lists of strings."""
        return [[cell.text for cell in row.cells] for row in self.rows]

    def to_markdown(self) -> str:
        """Convert table to Markdown format."""
        if not self.rows:
            return ""
        data = self.to_list()
        if not data:
            return ""

        lines = []
        # Header
        lines.append("| " + " | ".join(data[0]) + " |")
        lines.append("| " + " | ".join("---" for _ in data[0]) + " |")
        for row in data[1:]:
            lines.append("| " + " | ".join(row) + " |")
        return "\n".join(lines)


@runtime_checkable
class TableDetectionStrategy(Protocol):
    """Protocol for table detection strategies."""

    def detect(self, chars: list, lines: list, rects: list) -> list[Table]:
        """Detect tables in the given page elements."""
        ...
