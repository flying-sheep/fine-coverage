from collections.abc import Collection
from dataclasses import dataclass, field
from pathlib import Path

from rich.text import Text

from .ast import Span, parse
from .tracer import CodeLocs


@dataclass
class TraceHighlighter:
    events: Collection[CodeLocs]
    spans: dict[str, list[Span]] = field(init=False)

    def __post_init__(self):
        file_paths: set[str] = {locs.file for locs in self.events if locs is not None}
        self.spans = {file_path: list(parse(file_name=file_path)) for file_path in file_paths}

    def highlight_mod(self, file_path: str) -> Text:
        # TODO: UTF-8
        text = Text(Path(file_path).read_text().splitlines())
        for span in self.spans[file_path]:
            start, end = span.start.col, span.end.col  # TODO: translate line positions to indices
            covered = any(
                loc == span
                for event in self.events
                if event.file == file_path
                for loc in event.locs
            )
            text.stylize('green' if covered else 'red', start, end)
        return text
