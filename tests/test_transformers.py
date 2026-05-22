import ast
from pathlib import Path
from src.transformers import (
    transform_rename_variable,
    transform_extract_function,
    _find_params,
    _infer_rename_target,
)
from src.models import CodeLocation


def test_rename_variable():
    source = """
def calc():
    x = 42
    return x * 2
"""
    result = transform_rename_variable(source, old_name="x", new_name="value")
    assert "value = 42" in result
    assert "return value * 2" in result
    assert "x =" not in result


def test_rename_variable_in_args():
    source = """
def add(x, y):
    return x + y
"""
    result = transform_rename_variable(source, old_name="x", new_name="a")
    assert "def add(a, y):" in result
    assert "return a + y" in result


def test_rename_variable_no_change_when_not_found():
    source = "z = 1"
    result = transform_rename_variable(source, old_name="x", new_name="y")
    assert result == source


def test_extract_function_simple():
    source = """
def main():
    x = 1
    y = 2
    z = x + y
    return z
"""
    location = CodeLocation(file=Path(""), start_line=3, end_line=5)
    result = transform_extract_function(source, location, func_name="add")
    assert "def add():" in result
    assert "x = 1" in result
    assert "y = 2" in result
    assert "z = x + y" in result
    assert "add()" in result


def test_extract_function_with_params():
    source = """
def main():
    a = 10
    b = a + 5
    return b
"""
    location = CodeLocation(file=Path(""), start_line=4, end_line=4)
    result = transform_extract_function(source, location, func_name="calc")
    assert "def calc(a):" in result
    assert "calc(a)" in result


def test_extract_function_unparseable():
    result = transform_extract_function("not valid python", CodeLocation(Path(""), 1, 1), "f")
    assert result == "not valid python"


def test_find_params():
    source = """
def main():
    a = 10
    b = a + 5
    c = b * 2
    return c
"""
    params = _find_params(source, start_line=4, end_line=4)
    assert "a" in params  # "b = a + 5" loads a but doesn't define it


def test_infer_rename_target():
    source = """
def calc():
    x = 42
    return x * 2
"""
    pair = _infer_rename_target(source)
    assert pair is not None
    old, new = pair
    assert old == "x"
    assert new == "value"


def test_infer_rename_target_no_single_letter():
    source = """
def calc():
    total = 42
    return total * 2
"""
    pair = _infer_rename_target(source)
    assert pair is None
