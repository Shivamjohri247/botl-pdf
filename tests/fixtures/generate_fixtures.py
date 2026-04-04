#!/usr/bin/env python3
"""Generate minimal test PDF fixtures programmatically.

Each fixture is a hand-crafted minimal valid PDF.
No external dependencies required.
"""

from __future__ import annotations

import os

HERE = os.path.dirname(os.path.abspath(__file__))


def _build_simple_text_pdf() -> bytes:
    """Build a minimal single-page PDF with 'Hello World' text."""
    objects: list[bytes] = []

    # Object 1: Catalog
    objects.append(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n")

    # Object 2: Pages
    objects.append(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n")

    # Object 3: Page
    objects.append(
        b"3 0 obj\n"
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n"
        b"   /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\n"
        b"endobj\n"
    )

    # Object 4: Content stream - text operators
    content = (
        b"BT\n"
        b"/F1 12 Tf\n"
        b"100 700 Td\n"
        b"(Hello World) Tj\n"
        b"ET\n"
    )
    objects.append(
        b"4 0 obj\n"
        b"<< /Length " + str(len(content)).encode() + b" >>\n"
        b"stream\n" + content + b"endstream\n"
        b"endobj\n"
    )

    # Object 5: Font
    objects.append(
        b"5 0 obj\n"
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n"
        b"endobj\n"
    )

    # Build PDF
    header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets: list[int] = []
    body = b""
    xref_offset: int = 0

    pos = len(header)
    for obj in objects:
        offsets.append(pos)
        body += obj
        pos += len(obj)

    xref_offset = len(header) + len(body)

    xref = (
        b"xref\n"
        b"0 6\n"
        b"0000000000 65535 f \n"
        + b"\n".join(f"{off:010d} 00000 n ".encode() for off in offsets)
        + b"\n"
    )

    trailer = (
        b"trailer\n"
        b"<< /Size 6 /Root 1 0 R >>\n"
        b"startxref\n"
        + str(xref_offset).encode() + b"\n"
        b"%%EOF\n"
    )

    return header + body + xref + trailer


def _build_multi_page_pdf() -> bytes:
    """Build a 3-page PDF with different text on each page."""
    # Use contiguous object numbering:
    # 1: Catalog, 2: Pages, 3: Font
    # 4: Page1, 5: Content1, 6: Page2, 7: Content2, 8: Page3, 9: Content3
    objects: dict[int, bytes] = {}

    # Object 1: Catalog
    objects[1] = b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"

    # Object 2: Pages (references to 4, 6, 8)
    objects[2] = b"2 0 obj\n<< /Type /Pages /Kids [4 0 R 6 0 R 8 0 R] /Count 3 >>\nendobj\n"

    # Object 3: Font (shared)
    objects[3] = b"3 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"

    pages_text = [
        ("Page One", 700),
        ("Page Two", 700),
        ("Page Three", 700),
    ]

    for i, (text, y) in enumerate(pages_text):
        page_num = 4 + i * 2    # 4, 6, 8
        content_num = 5 + i * 2  # 5, 7, 9

        # Content stream
        content = (
            b"BT\n/F1 12 Tf\n100 " + str(y).encode() + b" Td\n"
            + b"(" + text.encode() + b") Tj\nET\n"
        )
        objects[content_num] = (
            str(content_num).encode() + b" 0 obj\n"
            + b"<< /Length " + str(len(content)).encode() + b" >>\n"
            + b"stream\n" + content + b"endstream\nendobj\n"
        )

        # Page object
        objects[page_num] = (
            str(page_num).encode() + b" 0 obj\n"
            + b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n"
            + b"   /Contents " + str(content_num).encode() + b" 0 R "
            + b"/Resources << /Font << /F1 3 0 R >> >> >>\n"
            + b"endobj\n"
        )

    # Build PDF with contiguous xref (objects 0..9)
    max_obj = max(objects.keys())
    header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"

    # Write objects in order, track offsets for each object number
    offsets: dict[int, int] = {}
    body = b""
    pos = len(header)

    for obj_num in range(1, max_obj + 1):
        if obj_num in objects:
            offsets[obj_num] = pos
            body += objects[obj_num]
            pos += len(objects[obj_num])

    xref_offset = len(header) + len(body)

    xref_entries = []
    for obj_num in range(max_obj + 1):
        if obj_num == 0:
            xref_entries.append(b"0000000000 65535 f ")
        elif obj_num in offsets:
            xref_entries.append(f"{offsets[obj_num]:010d} 00000 n ".encode())
        else:
            xref_entries.append(b"0000000000 00000 f ")

    xref = (
        b"xref\n"
        + b"0 " + str(max_obj + 1).encode() + b"\n"
        + b"\n".join(xref_entries)
        + b"\n"
    )

    trailer = (
        b"trailer\n"
        b"<< /Size " + str(max_obj + 1).encode() + b" /Root 1 0 R >>\n"
        b"startxref\n"
        + str(xref_offset).encode() + b"\n"
        b"%%EOF\n"
    )

    return header + body + xref + trailer


def _build_metadata_pdf() -> bytes:
    """Build a PDF with metadata (title, author, etc.)."""
    objects: list[bytes] = []

    # Object 1: Catalog with metadata
    objects.append(
        b"1 0 obj\n"
        b"<< /Type /Catalog /Pages 2 0 R /Info 6 0 R >>\n"
        b"endobj\n"
    )

    # Object 2: Pages
    objects.append(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n")

    # Object 3: Page
    objects.append(
        b"3 0 obj\n"
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n"
        b"   /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\n"
        b"endobj\n"
    )

    content = b"BT\n/F1 12 Tf\n100 700 Td\n(Test Document) Tj\nET\n"
    objects.append(
        b"4 0 obj\n<< /Length " + str(len(content)).encode() + b" >>\n"
        b"stream\n" + content + b"endstream\nendobj\n"
    )

    objects.append(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
    )

    # Object 6: Info dict
    objects.append(
        b"6 0 obj\n"
        b"<< /Title (Test PDF Title) /Author (Test Author)\n"
        b"   /Subject (Test Subject) /Creator (Test Creator)\n"
        b"   /Producer (botl-pdf test) /CreationDate (D:20240101120000Z) >>\n"
        b"endobj\n"
    )

    header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets: list[int] = []
    body = b""

    pos = len(header)
    for obj in objects:
        offsets.append(pos)
        body += obj
        pos += len(obj)

    num_objects = len(objects) + 1
    xref_offset = len(header) + len(body)

    xref = (
        b"xref\n"
        + b"0 " + str(num_objects).encode() + b"\n"
        + b"0000000000 65535 f \n"
        + b"\n".join(f"{off:010d} 00000 n ".encode() for off in offsets)
        + b"\n"
    )

    trailer = (
        b"trailer\n"
        b"<< /Size " + str(num_objects).encode() + b" /Root 1 0 R /Info 6 0 R >>\n"
        b"startxref\n"
        + str(xref_offset).encode() + b"\n"
        b"%%EOF\n"
    )

    return header + body + xref + trailer


def _build_flate_compressed_pdf() -> bytes:
    """Build a PDF with FlateDecode compressed content stream."""
    import zlib

    objects: list[bytes] = []

    objects.append(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n")
    objects.append(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n")
    objects.append(
        b"3 0 obj\n"
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n"
        b"   /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\n"
        b"endobj\n"
    )

    content = b"BT\n/F1 12 Tf\n100 700 Td\n(Compressed Text) Tj\nET\n"
    compressed = zlib.compress(content)

    objects.append(
        b"4 0 obj\n"
        b"<< /Filter /FlateDecode /Length " + str(len(compressed)).encode() + b" >>\n"
        b"stream\n" + compressed + b"\nendstream\nendobj\n"
    )

    objects.append(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
    )

    header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets: list[int] = []
    body = b""

    pos = len(header)
    for obj in objects:
        offsets.append(pos)
        body += obj
        pos += len(obj)

    num_objects = len(objects) + 1
    xref_offset = len(header) + len(body)

    xref = (
        b"xref\n"
        + b"0 " + str(num_objects).encode() + b"\n"
        + b"0000000000 65535 f \n"
        + b"\n".join(f"{off:010d} 00000 n ".encode() for off in offsets)
        + b"\n"
    )

    trailer = (
        b"trailer\n"
        b"<< /Size " + str(num_objects).encode() + b" /Root 1 0 R >>\n"
        b"startxref\n"
        + str(xref_offset).encode() + b"\n"
        b"%%EOF\n"
    )

    return header + body + xref + trailer


def main() -> None:
    fixtures = {
        "simple_text.pdf": _build_simple_text_pdf,
        "multi_page.pdf": _build_multi_page_pdf,
        "metadata.pdf": _build_metadata_pdf,
        "flate_compressed.pdf": _build_flate_compressed_pdf,
    }

    for name, builder in fixtures.items():
        path = os.path.join(HERE, name)
        data = builder()
        with open(path, "wb") as f:
            f.write(data)
        print(f"Generated {name} ({len(data)} bytes)")


if __name__ == "__main__":
    main()
