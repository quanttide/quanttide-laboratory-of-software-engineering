import ast
from pathlib import Path
from src.models import CodeLocation, SmellInstance


def detect_long_function(tree: ast.AST, file: Path) -> list[SmellInstance]:
    results = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            line_count = node.end_lineno - node.lineno
            if line_count > 30:
                results.append(SmellInstance(
                    smell_id="long-function",
                    location=CodeLocation(file, node.lineno, node.end_lineno),
                    severity=min(1.0, line_count / 80),
                    metrics={"line_count": line_count},
                ))
    return results


def detect_long_parameter_list(tree: ast.AST, file: Path) -> list[SmellInstance]:
    results = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            param_count = len(node.args.args)
            if param_count > 5:
                results.append(SmellInstance(
                    smell_id="long-parameter-list",
                    location=CodeLocation(file, node.lineno, node.end_lineno),
                    severity=min(1.0, (param_count - 5) / 5),
                    metrics={"param_count": param_count},
                ))
    return results


def detect_large_class(tree: ast.AST, file: Path) -> list[SmellInstance]:
    results = []
    for node in ast.walk(tree):
        if isinstance(node, ast.ClassDef):
            method_count = sum(1 for n in node.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)))
            field_count = sum(1 for n in node.body if isinstance(n, ast.Assign))
            if method_count > 10 or field_count > 10:
                results.append(SmellInstance(
                    smell_id="large-class",
                    location=CodeLocation(file, node.lineno, node.end_lineno),
                    severity=min(1.0, max(method_count, field_count) / 20),
                    metrics={"method_count": method_count, "field_count": field_count},
                ))
    return results


DETECTORS: dict[str, callable] = {
    "long-function": detect_long_function,
    "long-parameter-list": detect_long_parameter_list,
    "large-class": detect_large_class,
}


def scan_file(file: Path) -> list[SmellInstance]:
    try:
        tree = ast.parse(file.read_text())
    except SyntaxError:
        return []
    results = []
    for smell_id, detector in DETECTORS.items():
        results.extend(detector(tree, file))
    return results


def scan_project(project_root: Path) -> list[SmellInstance]:
    results = []
    for file in project_root.rglob("*.py"):
        if file.name.startswith("__") or ".venv" in str(file):
            continue
        results.extend(scan_file(file))
    return results
