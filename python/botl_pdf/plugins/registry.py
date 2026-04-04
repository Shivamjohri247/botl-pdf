"""Plugin registry for extensible components."""

from __future__ import annotations

from typing import Any, Callable, Protocol, runtime_checkable

# Global registries
_EXPORTERS: dict[str, Callable] = {}
_TABLE_STRATEGIES: dict[str, Callable] = {}


def register_exporter(name: str, exporter: Callable) -> None:
    """Register a custom exporter plugin."""
    _EXPORTERS[name] = exporter


def register_table_strategy(name: str, strategy: Callable) -> None:
    """Register a custom table detection strategy plugin."""
    _TABLE_STRATEGIES[name] = strategy


def get_exporter(name: str) -> Callable:
    """Get a registered exporter by name."""
    if name not in _EXPORTERS:
        raise KeyError(f"No exporter registered with name '{name}'. Available: {list(_EXPORTERS.keys())}")
    return _EXPORTERS[name]


def get_table_strategy(name: str) -> Callable:
    """Get a registered table strategy by name."""
    if name not in _TABLE_STRATEGIES:
        raise KeyError(f"No table strategy registered with name '{name}'. Available: {list(_TABLE_STRATEGIES.keys())}")
    return _TABLE_STRATEGIES[name]
