from __future__ import annotations
import sys
import inspect

from types import FrameType
from typing import Any, Literal, Self, Protocol


Event = Literal['call', 'line', 'return', 'exception', 'opcode']
class TraceFunction(Protocol):
    def __call__(self, frame: FrameType, event: Event, arg: Any) -> Self | None: ...


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
        print(event)
        print(inspect.getsource(frame.f_code))
        list(frame.f_code.co_positions())
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
