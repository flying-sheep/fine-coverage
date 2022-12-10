from __future__ import annotations
import sys
import inspect
import linecache
from dataclasses import dataclass, field, KW_ONLY
from collections.abc import Iterable, Collection, Generator

from types import FrameType
from typing import Any, Literal, Self, Protocol, NamedTuple


Event = Literal['call', 'line', 'return', 'exception', 'opcode']
class TraceFunction(Protocol):
    def __call__(self, frame: FrameType, event: Event, arg: Any) -> Self | None: ...


class CodeLocs(NamedTuple):
    file: str | None
    locs: Collection[tuple[int, int, int, int]]

    def sources(self) -> Generator[str, None, None]:
        if self.file is None:
            return
        # TODO: positions are utf-8 indices, not string indices
        source = linecache.getlines(self.file)
        for start_line, end_line, start_column, end_column in self.locs: 
            if start_line == end_line:
                yield source[start_line-1][start_column:end_column]
            else:
                first_line, *lines, last_line = source
                yield ''.join([first_line[start_column:], *lines, last_line[:end_column]])


@dataclass
class Tracer:
    _: KW_ONLY
    events: list[CodeLocs] = field(default_factory=list)
    old_trace: TraceFunction | None = None
    
    def __enter__(self) -> Self:
        self.old_trace = sys.gettrace()
        sys.settrace(self.dispatch)
        return self

    def __exit__(self, typ, value, traceback):
        sys.settrace(self.old_trace)
    
    def dispatch(self, frame: FrameType, event: Literal['call'], arg: Any) -> TraceFunction:
        return self.process
    
    def process(self, frame: FrameType, event: Event, arg: Any) -> TraceFunction | None:
        file_name = inspect.getsourcefile(frame)
        match event:
            case 'call':
                pass
            case 'line':
                self.events.append(CodeLocs(file_name, list(frame.f_code.co_positions())))
            case 'return':
                pass
            case 'exception':
                pass
            case 'opcode':
                pass
        return self.process
