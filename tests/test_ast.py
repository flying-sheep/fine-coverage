from fine_coverage.ast import parse, Span, Pos

def test_trinary():
    assert list(parse('a if test else b')) == [Span(Pos(1, 0), Pos(1, 1)), Span(Pos(1, 15), Pos(1, 16))]

def test_nested_trinaries():
    assert list(parse('a if test else (b if test2 else c)')) == [
        Span(Pos(1, 0), Pos(1, 1)),
        Span(Pos(1, 16), Pos(1, 17)),
        Span(Pos(1, 32), Pos(1, 33)),
    ]
