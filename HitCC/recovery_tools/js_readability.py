from __future__ import annotations

from dataclasses import dataclass


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

WORD_CHARS = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$")


@dataclass(slots=True)
class FormatterConfig:
    indent: str = "  "
    line_wrap: int = 120


class JsReadableFormatter:
    def __init__(self, config: FormatterConfig | None = None) -> None:
        self.config = config or FormatterConfig()

    def format(self, source: str) -> str:
        text = source.replace("\r\n", "\n").replace("\r", "\n")
        self._chars = text
        self._length = len(text)
        self._index = 0
        self._indent_level = 0
        self._line_start = True
        self._pending_space = False
        self._output: list[str] = []
        self._last_token: str | None = None
        self._paren_stack: list[str] = []
        self._bracket_depth = 0
        self._for_header_depth = 0
        self._current_line_length = 0

        while self._index < self._length:
            ch = self._chars[self._index]

            if ch in " \t\n":
                self._consume_whitespace()
                continue

            if ch == "/" and self._peek(1) == "/":
                self._emit_line_comment()
                continue

            if ch == "/" and self._peek(1) == "*":
                self._emit_block_comment()
                continue

            if ch in ("'", '"', "`"):
                literal = self._consume_string(ch)
                self._emit_token(literal, "literal")
                continue

            if ch == "/" and self._is_regex_start():
                literal = self._consume_regex()
                self._emit_token(literal, "literal")
                continue

            if ch.isalpha() or ch in "_$":
                word = self._consume_word()
                self._emit_token(word, word)
                continue

            if ch.isdigit():
                number = self._consume_number()
                self._emit_token(number, "literal")
                continue

            self._emit_punctuation(ch)
            self._index += 1

        return "".join(self._output).rstrip() + "\n"

    def _peek(self, offset: int) -> str:
        pos = self._index + offset
        if pos >= self._length:
            return ""
        return self._chars[pos]

    def _consume_whitespace(self) -> None:
        saw_newline = False
        while self._index < self._length and self._chars[self._index] in " \t\n":
            if self._chars[self._index] == "\n":
                saw_newline = True
            self._index += 1
        if saw_newline:
            self._newline()
        else:
            self._pending_space = True

    def _consume_word(self) -> str:
        start = self._index
        self._index += 1
        while self._index < self._length and self._chars[self._index] in WORD_CHARS:
            self._index += 1
        return self._chars[start:self._index]

    def _consume_number(self) -> str:
        start = self._index
        self._index += 1
        while self._index < self._length and self._chars[self._index] in WORD_CHARS.union({".", "x", "X"}):
            self._index += 1
        return self._chars[start:self._index]

    def _consume_string(self, quote: str) -> str:
        start = self._index
        self._index += 1
        while self._index < self._length:
            ch = self._chars[self._index]
            if ch == "\\":
                self._index += 2
                continue
            self._index += 1
            if ch == quote:
                break
        return self._chars[start:self._index]

    def _consume_regex(self) -> str:
        start = self._index
        self._index += 1
        in_char_class = False
        while self._index < self._length:
            ch = self._chars[self._index]
            if ch == "\\":
                self._index += 2
                continue
            if ch == "[":
                in_char_class = True
            elif ch == "]" and in_char_class:
                in_char_class = False
            elif ch == "/" and not in_char_class:
                self._index += 1
                while self._index < self._length and self._chars[self._index].isalpha():
                    self._index += 1
                break
            self._index += 1
        return self._chars[start:self._index]

    def _emit_line_comment(self) -> None:
        start = self._index
        self._index += 2
        while self._index < self._length and self._chars[self._index] != "\n":
            self._index += 1
        self._emit_raw(self._chars[start:self._index])
        self._newline()

    def _emit_block_comment(self) -> None:
        start = self._index
        self._index += 2
        while self._index < self._length - 1:
            if self._chars[self._index] == "*" and self._chars[self._index + 1] == "/":
                self._index += 2
                break
            self._index += 1
        self._emit_raw(self._chars[start:self._index])
        self._pending_space = True

    def _emit_token(self, token: str, token_kind: str) -> None:
        if self._needs_space_before(token):
            self._emit_raw(" ")
        self._emit_raw(token)
        self._last_token = token_kind

    def _emit_punctuation(self, ch: str) -> None:
        if ch == "{":
            self._emit_compact("{")
            self._indent_level += 1
            self._newline()
            self._last_token = "{"
            return

        if ch == "}":
            self._indent_level = max(self._indent_level - 1, 0)
            self._newline(force=True)
            self._emit_raw("}")
            self._newline()
            self._last_token = "}"
            return

        if ch == ";":
            self._emit_compact(";")
            if self._for_header_depth == 0:
                self._newline()
            self._last_token = ";"
            return

        if ch == ",":
            self._emit_compact(",")
            self._pending_space = True
            if self._nesting_depth() > 0 and self._current_line_length >= self.config.line_wrap:
                self._newline()
            self._last_token = ","
            return

        if ch == "(":
            self._emit_compact("(")
            if self._last_token == "for":
                self._paren_stack.append("for")
                self._for_header_depth += 1
            else:
                self._paren_stack.append("(")
            self._last_token = "("
            return

        if ch == ")":
            self._emit_compact(")")
            if self._paren_stack:
                scope = self._paren_stack.pop()
                if scope == "for":
                    self._for_header_depth = max(self._for_header_depth - 1, 0)
            self._last_token = ")"
            return

        if ch == "[":
            self._emit_compact("[")
            self._bracket_depth += 1
            self._last_token = "["
            return

        if ch == "]":
            self._bracket_depth = max(self._bracket_depth - 1, 0)
            self._emit_compact("]")
            self._last_token = "]"
            return

        self._emit_compact(ch)
        self._last_token = ch

    def _emit_compact(self, text: str) -> None:
        if self._pending_space and not self._line_start and text not in "),.;]}":
            self._emit_raw(" ")
        self._pending_space = False
        self._emit_raw(text)

    def _emit_raw(self, text: str) -> None:
        if self._line_start and text != "\n":
            indent_text = self.config.indent * self._indent_level
            self._output.append(indent_text)
            self._current_line_length = len(indent_text)
            self._line_start = False
        self._output.append(text)
        if text == "\n":
            self._current_line_length = 0
        else:
            self._current_line_length += len(text)

    def _newline(self, force: bool = False) -> None:
        self._pending_space = False
        if self._line_start and not force:
            return
        if self._output and self._output[-1] == "\n" and not force:
            self._line_start = True
            return
        self._output.append("\n")
        self._line_start = True
        self._current_line_length = 0

    def _needs_space_before(self, token: str) -> bool:
        if self._line_start:
            return False
        if not self._pending_space:
            return False
        if self._last_token in {"(", "[", "{", ".", "!", "~"}:
            return False
        if token in {")", "]", "}", ".", ",", ";", ":"}:
            return False
        return True

    def _is_regex_start(self) -> bool:
        return self._last_token in REGEX_PRECEDERS

    def _nesting_depth(self) -> int:
        return self._indent_level + self._bracket_depth + len(self._paren_stack)
