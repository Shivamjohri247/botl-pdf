"""CLI entry point for botl-pdf."""

from __future__ import annotations

import json
import sys
from typing import Optional


def app() -> None:
    """Main CLI entry point using typer (if available) or argparse fallback."""
    try:
        import typer

        _typer_app = typer.Typer(
            name="botl-pdf",
            help="High-performance PDF processing toolkit.",
            no_args_is_help=True,
        )

        @_typer_app.command()
        def text(
            path: str,
            output: Optional[str] = typer.Option(None, "-o", "--output", help="Output file path"),
            pages: Optional[str] = typer.Option(None, "--pages", help="Page range (e.g., 1-5)"),
            layout: bool = typer.Option(False, "--layout", help="Preserve spatial layout"),
        ) -> None:
            """Extract text from a PDF file."""
            from botl_pdf._core import open as _open

            doc = _open(path)
            page_indices = _parse_page_range(pages, doc.num_pages)
            parts = []
            for i in page_indices:
                page = doc.get_page(i)
                parts.append(page.extract_text(layout=layout))

            result = "\n\n".join(parts)
            if output:
                with open(output, "w", encoding="utf-8") as f:
                    f.write(result)
            else:
                sys.stdout.write(result)
                sys.stdout.write("\n")

        @_typer_app.command()
        def info(path: str) -> None:
            """Show PDF metadata and page count."""
            from botl_pdf._core import open as _open

            doc = _open(path)
            meta = doc.metadata
            result = {
                "version": meta.get("version"),
                "page_count": doc.num_pages,
                "encrypted": doc.is_encrypted,
                "title": meta.get("title"),
                "author": meta.get("author"),
                "subject": meta.get("subject"),
                "creator": meta.get("creator"),
                "producer": meta.get("producer"),
            }
            print(json.dumps(result, indent=2, ensure_ascii=False))

        @_typer_app.command()
        def export(
            path: str,
            format: str = typer.Option("markdown", "--format", help="Output format: markdown, text"),
            output: Optional[str] = typer.Option(None, "-o", "--output", help="Output file path"),
        ) -> None:
            """Export PDF to various formats."""
            from botl_pdf.export import to_markdown, to_text

            if format == "markdown":
                result = to_markdown(path)
            elif format == "text":
                result = to_text(path)
            else:
                print(f"Unknown format: {format}", file=sys.stderr)
                sys.exit(1)

            if output:
                with open(output, "w", encoding="utf-8") as f:
                    f.write(result)
            else:
                sys.stdout.write(result)
                sys.stdout.write("\n")

        _typer_app()

    except ImportError:
        # Fallback to argparse if typer is not installed
        _run_with_argparse()


def _parse_page_range(pages_str: Optional[str], total: int) -> range:
    """Parse a page range string like '1-5' into a range of 0-based indices."""
    if pages_str is None:
        return range(total)

    result_indices = []
    for part in pages_str.split(","):
        part = part.strip()
        if "-" in part:
            start, end = part.split("-", 1)
            s = max(0, int(start) - 1)
            e = min(total, int(end))
            result_indices.extend(range(s, e))
        else:
            idx = int(part) - 1
            if 0 <= idx < total:
                result_indices.append(idx)

    if not result_indices:
        return range(total)
    return range(min(result_indices), max(result_indices) + 1)


def _run_with_argparse() -> None:
    """Fallback CLI using argparse when typer is not installed."""
    import argparse

    parser = argparse.ArgumentParser(prog="botl-pdf", description="High-performance PDF processing toolkit.")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # text command
    text_parser = subparsers.add_parser("text", help="Extract text from a PDF")
    text_parser.add_argument("path", help="Path to PDF file")
    text_parser.add_argument("-o", "--output", help="Output file path")
    text_parser.add_argument("--pages", help="Page range (e.g., 1-5)")
    text_parser.add_argument("--layout", action="store_true", help="Preserve spatial layout")

    # info command
    info_parser = subparsers.add_parser("info", help="Show PDF metadata")
    info_parser.add_argument("path", help="Path to PDF file")

    # export command
    export_parser = subparsers.add_parser("export", help="Export PDF to various formats")
    export_parser.add_argument("path", help="Path to PDF file")
    export_parser.add_argument("--format", default="markdown", help="Output format: markdown, text")
    export_parser.add_argument("-o", "--output", help="Output file path")

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        sys.exit(1)

    from botl_pdf._core import open as _open

    if args.command == "text":
        doc = _open(args.path)
        page_indices = _parse_page_range(args.pages, doc.num_pages)
        parts = [doc.get_page(i).extract_text(layout=args.layout) for i in page_indices]
        result = "\n\n".join(parts)
        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                f.write(result)
        else:
            sys.stdout.write(result)
            sys.stdout.write("\n")

    elif args.command == "info":
        doc = _open(args.path)
        meta = doc.metadata
        print(json.dumps({
            "version": meta.get("version"),
            "page_count": doc.num_pages,
            "encrypted": doc.is_encrypted,
            "title": meta.get("title"),
            "author": meta.get("author"),
        }, indent=2, ensure_ascii=False))

    elif args.command == "export":
        from botl_pdf.export import to_markdown, to_text

        if args.format == "markdown":
            result = to_markdown(args.path)
        elif args.format == "text":
            result = to_text(args.path)
        else:
            print(f"Unknown format: {args.format}", file=sys.stderr)
            sys.exit(1)

        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                f.write(result)
        else:
            sys.stdout.write(result)
            sys.stdout.write("\n")
