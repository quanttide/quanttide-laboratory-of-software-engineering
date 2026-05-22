import ast
from pathlib import Path
from src.detectors import (
    detect_long_function,
    detect_long_parameter_list,
    detect_large_class,
    scan_file,
)
from src.models import SmellInstance


def test_detect_long_function():
    code = """
def short(): pass

def long():
    x = 1
    x = 2
    x = 3
    x = 4
    x = 5
    x = 6
    x = 7
    x = 8
    x = 9
    x = 10
    x = 11
    x = 12
    x = 13
    x = 14
    x = 15
    x = 16
    x = 17
    x = 18
    x = 19
    x = 20
    x = 21
    x = 22
    x = 23
    x = 24
    x = 25
    x = 26
    x = 27
    x = 28
    x = 29
    x = 30
    x = 31
    x = 32
    return x
"""
    tree = ast.parse(code)
    file = Path("test.py")
    results = detect_long_function(tree, file)

    long_funcs = [r for r in results if r.location.start_line == 4]
    assert len(long_funcs) == 1
    assert long_funcs[0].metrics["line_count"] >= 30

    short_funcs = [r for r in results if r.location.start_line == 2]
    assert len(short_funcs) == 0


def test_detect_long_parameter_list():
    code = """
def ok(a, b, c): pass

def too_many(a, b, c, d, e, f): pass
"""
    tree = ast.parse(code)
    file = Path("test.py")
    results = detect_long_parameter_list(tree, file)

    bad = [r for r in results if r.location.start_line == 4]
    assert len(bad) == 1
    assert bad[0].metrics["param_count"] == 6

    good = [r for r in results if r.location.start_line == 2]
    assert len(good) == 0


def test_detect_large_class():
    code = """
class Small:
    def a(self): pass

class Big:
    def a(self): pass
    def b(self): pass
    def c(self): pass
    def d(self): pass
    def e(self): pass
    def f(self): pass
    def g(self): pass
    def h(self): pass
    def i(self): pass
    def j(self): pass
    def k(self): pass
"""
    tree = ast.parse(code)
    file = Path("test.py")
    results = detect_large_class(tree, file)

    bad = [r for r in results if r.metrics["method_count"] > 10]
    assert len(bad) == 1
    assert bad[0].metrics["method_count"] > 10

    good = [r for r in results if r.location.start_line == 2]
    assert len(good) == 0


def test_scan_file_ignores_short_code():
    code = "x = 1\n"
    file = Path("test.py")
    file.write_text(code)
    results = scan_file(file)
    assert len(results) == 0
