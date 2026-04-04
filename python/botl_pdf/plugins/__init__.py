"""Plugin system for botl-pdf.

Provides the plugin registry and base classes for extending
botl-pdf with custom table strategies, exporters, and OCR backends.
"""

from botl_pdf.plugins.registry import register_exporter, register_table_strategy, get_exporter, get_table_strategy

__all__ = [
    "register_exporter",
    "register_table_strategy",
    "get_exporter",
    "get_table_strategy",
]
