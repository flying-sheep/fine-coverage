from __future__ import annotations

from collections.abc import Callable, Collection, Generator, Iterable
from dataclasses import dataclass, field
from pathlib import Path
from typing import TypeVar

from rich.text import Text

from .ast import Pos, Span, parse
from .tracer import CodeLocs

T = TypeVar('T')


def intersperse(
    items: Iterable[T], fn: Callable[[T | None, T | None], T]
) -> Generator[T, None, None]:
    it = iter(items)
    prev = None
    next_ = None
    while True:
        try:
            next_ = next(it)
        except StopIteration:
            yield fn(next_, None)
            break
        else:
            yield fn(prev, next_)
            yield next_
        prev = next_


@dataclass
class TraceHighlighter:
    events: Collection[CodeLocs]
    spans: dict[str, list[Span]] = field(init=False)

    def __post_init__(self):
        file_paths = {locs.file for locs in self.events if locs.file is not None}
        self.spans = {file_path: list(parse(file_name=file_path)) for file_path in file_paths}

    def highlight_mod(self, file_path: str) -> Text:
        lines = Path(file_path).read_bytes().splitlines()
        text = Text()
        for l, line in enumerate(lines, 1):
            spans = sorted(
                [
                    span
                    for span in self.spans[file_path]
                    if span.start.line <= l and span.end.line >= l
                ],
                key=lambda span: span.start,
            )
            span_styles = intersperse(
                [(span, 'green' if self.covered(file_path, span) else 'red') for span in spans],
                lambda prev, next_: (
                    Span(
                        Pos(l, 0 if prev is None else prev[0].end.col),
                        Pos(l, len(line) if next_ is None else next_[0].start.col),
                    ),
                    None,
                ),
            )
            for span, style in span_styles:
                start = 0 if span.start.line < l else span.start.col
                end = len(line) if span.end.line > l else span.end.col
                text.append(line[start:end].decode('utf-8'), style)
            text.append('\n')
        return text

    def covered(self, file_path: str, span: Span) -> bool:
        return any(
            loc == span for event in self.events if event.file == file_path for loc in event.locs
        )
