import sys
from pathlib import Path
from runpy import run_module

import rich

from .report import TraceHighlighter
from .tracer import Tracer

HERE = Path(__file__).parent


def run():
    argv = sys.argv
    mod = argv[1]
    sys.argv = [sys.executable, '-m', mod, *argv[2:]]
    with Tracer() as tracer:
        run_module(mod)
    highlighter = TraceHighlighter(tracer.events)
    rich.print(highlighter.highlight_mod(str(HERE / 'ast.py')))
    sys.argv = argv
