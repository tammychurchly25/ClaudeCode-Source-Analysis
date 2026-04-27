from __future__ import annotations

import argparse
from pathlib import Path

from js_readability import JsReadableFormatter


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate a readability-oriented JS copy without modifying the original bundle."
    )
    parser.add_argument("input", type=Path, help="Source JS bundle path")
    parser.add_argument("output", type=Path, help="Readable output path")
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    source = args.input.read_text("utf-8", errors="replace")
    formatter = JsReadableFormatter()
    formatted = formatter.format(source)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formatted, encoding="utf-8")

    print(f"input={args.input}")
    print(f"output={args.output}")
    print(f"input_bytes={args.input.stat().st_size}")
    print(f"output_bytes={args.output.stat().st_size}")
    print(f"input_lines={source.count(chr(10)) + 1}")
    print(f"output_lines={formatted.count(chr(10))}")


if __name__ == "__main__":
    main()
