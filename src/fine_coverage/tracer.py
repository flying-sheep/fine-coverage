from __future__ import annotations
import sys
import inspect
import linecache
from dataclasses import dataclass, field, KW_ONLY
from collections.abc import Iterable, Collection, Generator
from types import FrameType
from typing import cast, Any, Literal, Self, Protocol, NamedTuple

from .ast import parse, Span


Event = Literal['call', 'line', 'return', 'exception', 'opcode']


class TraceFunction(Protocol):
    def __call__(self, frame: FrameType, event: Event, arg: Any) -> Self | None:
        ...


class CodeLocs(NamedTuple):
    file: str | None
    locs: Collection[Span] = ()

    @classmethod
    def from_tuples(
        cls,
        file: str | None,
        locs: Iterable[tuple[int | None, int | None, int | None, int | None]],
    ) -> Self:
        # Skip incomplete or missing spans
        return cls(
            file,
            [
                Span.from_tuple(cast(tuple[int, int, int, int], ls))
                for ls in locs
                if all(l is not None for l in ls)
            ],
        )

    def sources(self) -> Generator[str, None, None]:
        if self.file is None:
            return
        # TODO: positions are utf-8 indices, not string indices
        source = linecache.getlines(self.file)
        for start, end in self.locs:
            if start.line == end.line:
                yield source[start.line - 1][start.col : end.col]
            else:
                first_line, *lines, last_line = source
                yield ''.join([first_line[start.col :], *lines, last_line[: end.col]])


@dataclass
class Tracer:
    _: KW_ONLY
    events: list[CodeLocs] = field(default_factory=list)
    old_trace: TraceFunction | None = None

    def __enter__(self) -> Self:
        self.old_trace = cast(TraceFunction, sys.gettrace())
        sys.settrace(self.dispatch)  # type: ignore
        return self

    def __exit__(self, typ, value, traceback):
        sys.settrace(self.old_trace)  # type: ignore

    def dispatch(self, frame: FrameType, event: Literal['call'], arg: Any) -> TraceFunction:
        return self.process

    def process(self, frame: FrameType, event: Event, arg: Any) -> TraceFunction | None:
        file_name = inspect.getsourcefile(frame)
        match event:
            case 'call':
                pass
            case 'line':
                self.events.append(
                    CodeLocs.from_tuples(
                        file_name, [Span.from_tuple(pos) for pos in frame.f_code.co_positions()]
                    )
                )
            case 'return':
                pass
            case 'exception':
                pass
            case 'opcode':
                pass
        return self.process
