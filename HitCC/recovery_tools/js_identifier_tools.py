from __future__ import annotations

import bisect
import re
from dataclasses import dataclass


WORD_CHARS = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$")
IDENTIFIER_START = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$")
REGEX_PRECEDERS = {
    None,
    "(",
    "[",
    "{",
    ",",
    ";",
    ":",
    "?",
    "=",
    "==",
    "===",
    "!=",
    "!==",
    "!",
    "~",
    "+",
    "-",
    "*",
    "%",
    "&",
    "|",
    "^",
    "<<",
    ">>",
    ">>>",
    "&&",
    "||",
    "??",
    "=>",
    "return",
    "throw",
    "case",
    "delete",
    "typeof",
    "void",
    "new",
    "in",
    "instanceof",
    "yield",
    "await",
}
OPERATORS = (
    ">>>=",
    "===",
    "!==",
    ">>>",
    "<<=",
    ">>=",
    "&&=",
    "||=",
    "??=",
    "==",
    "!=",
    "<=",
    ">=",
    "&&",
    "||",
    "??",
    "=>",
    "++",
    "--",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "&=",
    "|=",
    "^=",
    "<<",
    ">>",
    "**",
    "?.",
)

FUNCTION_PATTERN = re.compile(r"(?:^|\n)(?P<indent>\s*)(?P<async>async\s+)?function\s+(?P<name>[A-Za-z_$][\w$]*)\s*\(")
ARROW_PATTERN = re.compile(
    r"(?:^|\n)(?P<indent>\s*)(?:var|let|const)\s+(?P<name>[A-Za-z_$][\w$]*)\s*=\s*(?P<async>async\s*)?(?:\([^)]{0,200}\)|[A-Za-z_$][\w$]*)\s*=>",
    re.DOTALL,
)
WRAPPED_ARROW_ASSIGN_PATTERN = re.compile(
    r"(?:^|\n)(?P<indent>\s*)(?P<name>[A-Za-z_$][\w$]*)\s*=\s*[A-Za-z_$][\w$]*\(\s*(?P<async>async\s*)?(?:\([^)]{0,200}\)|[A-Za-z_$][\w$]*)\s*=>",
    re.DOTALL,
)


@dataclass(slots=True)
class SymbolEntry:
    name: str
    line: int
    column: int
    kind: str
    is_async: bool
    preview: str


def rewrite_identifiers(source: str, aliases: dict[str, str]) -> str:
    chars = source
    length = len(chars)
    index = 0
    output: list[str] = []
    last_token: str | None = None

    while index < length:
        ch = chars[index]

        if ch in " \t\r\n":
            output.append(ch)
            index += 1
            continue

        if ch == "/" and _peek(chars, index, 1) == "/":
            end = index + 2
            while end < length and chars[end] != "\n":
                end += 1
            output.append(chars[index:end])
            index = end
            continue

        if ch == "/" and _peek(chars, index, 1) == "*":
            end = index + 2
            while end < length - 1:
                if chars[end] == "*" and chars[end + 1] == "/":
                    end += 2
                    break
                end += 1
            output.append(chars[index:end])
            index = end
            continue

        if ch in ("'", '"', "`"):
            literal, index = _consume_string(chars, index, ch)
            output.append(literal)
            last_token = "literal"
            continue

        if ch == "/" and last_token in REGEX_PRECEDERS:
            literal, index = _consume_regex(chars, index)
            output.append(literal)
            last_token = "literal"
            continue

        if ch in IDENTIFIER_START:
            token, index = _consume_identifier(chars, index)
            output.append(aliases.get(token, token))
            last_token = token
            continue

        if ch.isdigit():
            token, index = _consume_number(chars, index)
            output.append(token)
            last_token = "literal"
            continue

        operator = _consume_operator(chars, index)
        output.append(operator)
        last_token = operator
        index += len(operator)

    return "".join(output)


def extract_symbols(source: str) -> list[SymbolEntry]:
    line_starts = _line_start_offsets(source)
    entries: dict[tuple[str, int, str], SymbolEntry] = {}

    for match in FUNCTION_PATTERN.finditer(source):
        name = match.group("name")
        line, column = _offset_to_line_column(line_starts, match.start("name"))
        preview = source[match.start("name"): source.find("\n", match.start("name"))]
        key = (name, line, "function")
        entries[key] = SymbolEntry(
            name=name,
            line=line,
            column=column,
            kind="function",
            is_async=bool(match.group("async")),
            preview=preview.strip(),
        )

    for match in ARROW_PATTERN.finditer(source):
        name = match.group("name")
        line, column = _offset_to_line_column(line_starts, match.start("name"))
        preview = source[match.start("name"): source.find("\n", match.start("name"))]
        key = (name, line, "arrow")
        entries[key] = SymbolEntry(
            name=name,
            line=line,
            column=column,
            kind="arrow",
            is_async=bool(match.group("async")),
            preview=preview.strip(),
        )

    for match in WRAPPED_ARROW_ASSIGN_PATTERN.finditer(source):
        name = match.group("name")
        line, column = _offset_to_line_column(line_starts, match.start("name"))
        preview = source[match.start("name"): source.find("\n", match.start("name"))]
        key = (name, line, "wrapped_arrow")
        entries[key] = SymbolEntry(
            name=name,
            line=line,
            column=column,
            kind="wrapped_arrow",
            is_async=bool(match.group("async")),
            preview=preview.strip(),
        )

    return sorted(entries.values(), key=lambda item: (item.line, item.column, item.name))


def _peek(text: str, index: int, offset: int) -> str:
    target = index + offset
    if target >= len(text):
        return ""
    return text[target]


def _consume_string(text: str, index: int, quote: str) -> tuple[str, int]:
    start = index
    index += 1
    while index < len(text):
        ch = text[index]
        if ch == "\\":
            index += 2
            continue
        index += 1
        if ch == quote:
            break
    return text[start:index], index


def _consume_regex(text: str, index: int) -> tuple[str, int]:
    start = index
    index += 1
    in_char_class = False
    while index < len(text):
        ch = text[index]
        if ch == "\\":
            index += 2
            continue
        if ch == "[":
            in_char_class = True
        elif ch == "]" and in_char_class:
            in_char_class = False
        elif ch == "/" and not in_char_class:
            index += 1
            while index < len(text) and text[index].isalpha():
                index += 1
            break
        index += 1
    return text[start:index], index


def _consume_identifier(text: str, index: int) -> tuple[str, int]:
    start = index
    index += 1
    while index < len(text) and text[index] in WORD_CHARS:
        index += 1
    return text[start:index], index


def _consume_number(text: str, index: int) -> tuple[str, int]:
    start = index
    index += 1
    while index < len(text) and text[index] in WORD_CHARS.union({".", "x", "X"}):
        index += 1
    return text[start:index], index


def _consume_operator(text: str, index: int) -> str:
    for operator in OPERATORS:
        if text.startswith(operator, index):
            return operator
    return text[index]


def _line_start_offsets(text: str) -> list[int]:
    starts = [0]
    for idx, ch in enumerate(text):
        if ch == "\n":
            starts.append(idx + 1)
    return starts


def _offset_to_line_column(line_starts: list[int], offset: int) -> tuple[int, int]:
    line_index = bisect.bisect_right(line_starts, offset) - 1
    line_start = line_starts[line_index]
    return line_index + 1, offset - line_start + 1
