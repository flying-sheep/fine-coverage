import sys
from argparse import ArgumentParser
from pathlib import Path
from runpy import run_module

import rich
from rich_argparse import ArgumentDefaultsRichHelpFormatter

from .report import TraceHighlighter
from .tracer import Tracer

HERE = Path(__file__).parent


def run():
    parser = ArgumentParser(formatter_class=ArgumentDefaultsRichHelpFormatter)
    parser.add_argument('module')
    parser.add_argument('--cov', type=str)

    argv = sys.argv
    args, passed_args = parser.parse_known_args(argv[1:])
    sys.argv = [sys.argv[0], *passed_args]
    with Tracer(args.cov) as tracer:
        run_module(args.module, alter_sys=True)
    sys.argv = argv

    highlighter = TraceHighlighter(tracer.events)
    for mod_file in {locs.file for locs in tracer.events if locs.file is not None}:
        rich.print(highlighter.highlight_mod(mod_file))
