"""High-level Document wrapper."""

from __future__ import annotations

from typing import Optional

from botl_pdf._core import PyDocument, PyPage


class Document:
    """High-level PDF document interface.

    Wraps the native PyDocument with Pythonic conveniences like
    context manager support and page iteration.
    """

    def __init__(self, path_or_bytes: str | bytes, *, password: Optional[str] = None, lazy: bool = True):
        from botl_pdf._core import open as _open
        self._doc: PyDocument = _open(path_or_bytes, password=password, lazy=lazy)

    def __enter__(self) -> Document:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        pass

    @property
    def metadata(self) -> dict[str, Optional[str]]:
        return self._doc.metadata

    @property
    def num_pages(self) -> int:
        return self._doc.num_pages

    @property
    def is_encrypted(self) -> bool:
        return self._doc.is_encrypted

    @property
    def toc(self) -> list:
        return self._doc.toc

    @property
    def pages(self) -> PageCollection:
        return PageCollection(self._doc)

    def __len__(self) -> int:
        return self._doc.num_pages

    def __repr__(self) -> str:
        return f"<Document pages={self.num_pages} encrypted={self.is_encrypted}>"


class PageCollection:
    """Lazy, subscriptable, iterable page collection."""

    def __init__(self, doc: PyDocument):
        self._doc = doc

    def __getitem__(self, index: int) -> PyPage:
        if index < 0:
            index += len(self)
        if index < 0 or index >= len(self):
            raise IndexError(f"Page index {index} out of range (document has {len(self)} pages)")
        return self._doc.get_page(index)

    def __len__(self) -> int:
        return self._doc.num_pages

    def __iter__(self):
        for i in range(len(self)):
            yield self._doc.get_page(i)

    def __repr__(self) -> str:
        return f"<PageCollection count={len(self)}>"
