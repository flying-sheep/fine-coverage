from __future__ import annotations

import ast
import os
from collections.abc import Generator
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, NamedTuple, Self


class Pos(NamedTuple):
    line: int
    col: int


class Span(NamedTuple):
    start: Pos
    end: Pos

    @classmethod
    def from_tuple(cls, span: tuple[int, int, int, int]) -> Self:
        assert span[0] is not None
        assert span[2] is not None
        return cls(Pos(span[0], span[2]), Pos(span[1], span[3]))

    @classmethod
    def from_ast(cls, node: ast.AST) -> Self:
        assert node.lineno is not None
        assert node.col_offset is not None
        return cls(
            Pos(node.lineno, node.col_offset),
            Pos(
                node.lineno if node.end_lineno is None else node.end_lineno,
                node.col_offset if node.end_col_offset is None else node.end_col_offset,
            ),
        )

    def __lt__(self, other: object) -> bool:
        if not isinstance(other, Span) or self == other:
            return False
        return self.start >= other.start and self.end <= other.end

    def __gt__(self, other: object) -> bool:
        if not isinstance(other, Span) or self == other:
            return False
        return self.start <= other.start and self.end >= other.end


@dataclass
class Visitor(ast.NodeVisitor):
    pot_branches: dict[Span, ast.expr] = field(default_factory=dict)

    def branches(self) -> Generator[Span, None, None]:
        for span in self.pot_branches:
            if self.is_leaf(span):
                yield span

    def is_leaf(self, span: Span) -> bool:
        superspan_of = {
            other_span: span > other_span for other_span in self.pot_branches.keys() - span
        }
        return not any(superspan_of.values())

    def add_pot_branch(self, node: ast.expr):
        pos = Span.from_ast(node)
        self.pot_branches[pos] = node

    def visit_BoolOp(self, node: ast.BoolOp) -> Any:
        self.add_pot_branch(node.values[0])
        self.add_pot_branch(node.values[1])
        self.generic_visit(node)
        return node

    def visit_IfExp(self, node: ast.IfExp) -> Any:
        self.add_pot_branch(node.body)
        self.add_pot_branch(node.orelse)
        self.generic_visit(node)
        return node


def parse(
    source: str | bytes | None = None, file_name: str | bytes | Path = '<unknown>'
) -> Generator[Span, None, None]:
    if source is None:
        if file_name == '<unknown>':
            raise ValueError('Specify source and/or file_name')
        source = Path(os.fsdecode(file_name)).read_bytes()
    mod = ast.parse(source, str(file_name))

    visitor = Visitor()
    visitor.generic_visit(mod)
    return visitor.branches()
