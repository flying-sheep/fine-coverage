from __future__ import annotations
import sys
import inspect
from collections.abc import Iterable

from types import FrameType
from typing import Any, Literal, Self, Protocol


Event = Literal['call', 'line', 'return', 'exception', 'opcode']
class TraceFunction(Protocol):
    def __call__(self, frame: FrameType, event: Event, arg: Any) -> Self | None: ...


def index_source(source: Iterable, start_line: int, end_line: int, start_column: int, end_column: int) -> str:
    if start_line == end_line:
        return source[start_line-1][start_column:end_column]
    else:
        first_line, *lines, last_line = source
        return ''.join([first_line[start_column:], *lines, last_line[:end_column]])


class Tracer:
    old_trace: TraceFunction
    
    def __enter__(self) -> Self:
        self.old_trace = sys.gettrace()
        sys.settrace(self.dispatch)
        return self

    def __exit__(self, typ, value, traceback):
        sys.settrace(self.old_trace)
    
    def dispatch(self, frame: FrameType, event: Literal['call'], arg: Any) -> TraceFunction:
        return self.process
    
    def process(self, frame: FrameType, event: Event, arg: Any) -> TraceFunction | None:
        full_source, _ = inspect.findsource(frame)
        print(event, [index_source(full_source, *positions) for positions in frame.f_code.co_positions()])
        match event:
            case 'call':
                pass
            case 'line':
                pass
            case 'return':
                pass
            case 'exception':
                pass
            case 'opcode':
                pass
        return self.process
