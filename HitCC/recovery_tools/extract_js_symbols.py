from __future__ import annotations

import argparse
import json
from pathlib import Path

from js_identifier_tools import extract_symbols


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Extract function-like symbols from a JS file.")
    parser.add_argument("input", type=Path, help="JS input path")
    parser.add_argument("output", type=Path, help="JSON output path")
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    source = args.input.read_text("utf-8", errors="replace")
    symbols = extract_symbols(source)
    payload = [
        {
            "name": symbol.name,
            "line": symbol.line,
            "column": symbol.column,
            "kind": symbol.kind,
            "is_async": symbol.is_async,
            "preview": symbol.preview,
        }
        for symbol in symbols
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"input={args.input}")
    print(f"output={args.output}")
    print(f"symbols={len(payload)}")


if __name__ == "__main__":
    main()
