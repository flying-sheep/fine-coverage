from __future__ import annotations

from collections.abc import Callable, Collection, Generator, Iterable
from dataclasses import dataclass, field
from pathlib import Path
from typing import NamedTuple, TypeVar

from rich.text import Text

from .ast import Span, parse
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


class LineSpan(NamedTuple):
    start: int
    end: int

    def __len__(self) -> int:
        return self.end - self.start


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
            # Get AST spans for this line. Assumes non-overlapping spans
            spans = sorted(
                [
                    LineSpan(
                        0 if span.start.line < l else span.start.col,
                        len(line) if span.end.line > l else span.end.col,
                    )
                    for span in self.spans[file_path]
                    if span.start.line <= l and span.end.line >= l
                ]
            )
            # Add unstyled spans for parts we don’t measure coverage for.
            # This could be done using `text.stylize` if we had char indices instead of UTF-8.
            span_styles = intersperse(
                [(span, 'green' if self.covered(file_path, span) else 'red') for span in spans],
                lambda prev, next_: (
                    LineSpan(
                        0 if prev is None else prev[0].end,
                        len(line) if next_ is None else next_[0].start,
                    ),
                    None,
                ),
            )
            for span, style in span_styles:
                if len(span) == 0:
                    # `intersperse` leaves 0-sized spans at start and/or end.
                    continue
                text.append(line[span.start : span.end].decode('utf-8'), style)
            text.append('\n')
        return text

    def covered(self, file_path: str, span: Span) -> bool:
        return any(
            loc == span for event in self.events if event.file == file_path for loc in event.locs
        )
