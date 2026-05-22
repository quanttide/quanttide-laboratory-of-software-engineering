import ast
import subprocess
from pathlib import Path
from src.models import CodeLocation, AppliedMethod
from src import llm_client


def transform_rename_variable(source: str, old_name: str, new_name: str) -> str:
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return source

    class Renamer(ast.NodeTransformer):
        def visit_Name(self, node):
            if node.id == old_name:
                node.id = new_name
            return node

        def visit_arg(self, node):
            if node.arg == old_name:
                node.arg = new_name
            return node

    new_tree = Renamer().visit(tree)
    ast.fix_missing_locations(new_tree)
    return ast.unparse(new_tree)


def _find_params(source: str, start_line: int, end_line: int) -> list[str]:
    lines = source.splitlines()
    target_lines = lines[start_line - 1 : end_line]
    base_indent = len(target_lines[0]) - len(target_lines[0].lstrip()) if target_lines else 0
    dedented = [line[base_indent:] for line in target_lines]
    target_text = "\n".join(dedented)
    try:
        target_tree = ast.parse(target_text)
    except SyntaxError:
        return []
    used = set()
    defined = set()
    for node in ast.walk(target_tree):
        if isinstance(node, ast.Name):
            if isinstance(node.ctx, ast.Load):
                used.add(node.id)
            elif isinstance(node.ctx, ast.Store):
                defined.add(node.id)
    builtins = {
        "print", "len", "range", "int", "str", "float", "list", "dict",
        "set", "tuple", "type", "isinstance", "hasattr", "open", "super",
        "True", "False", "None", "sum", "min", "max", "abs", "sorted",
        "map", "filter", "enumerate", "zip", "reversed", "any", "all",
    }
    return sorted(used - defined - builtins)


def transform_extract_function(source: str, location: CodeLocation, func_name: str) -> str:
    try:
        ast.parse(source)
    except SyntaxError:
        return source
    lines = source.splitlines(keepends=True)
    first_line = lines[location.start_line - 1]
    if first_line.lstrip().startswith("def ") or first_line.lstrip().startswith("async def "):
        body_start = location.start_line + 1
    else:
        body_start = location.start_line
    target_lines = lines[body_start - 1 : location.end_line]
    base_indent = len(target_lines[0]) - len(target_lines[0].lstrip())

    params = _find_params(source, body_start, location.end_line)
    param_str = ", ".join(params)

    new_func_lines = [
        f"{' ' * base_indent}def {func_name}({param_str}):\n",
        *[f"{' ' * (base_indent + 4)}{line.lstrip()}" for line in target_lines],
    ]
    call_line = f"{' ' * base_indent}{func_name}({param_str})\n"

    new_lines = (
        lines[: body_start - 1]
        + new_func_lines
        + [call_line]
        + lines[location.end_line:]
    )
    return "".join(new_lines)


def _infer_rename_target(source: str) -> tuple[str, str] | None:
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return None
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            for n in ast.walk(node):
                if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
                    if len(n.id) == 1:
                        return n.id, _suggest_name(n.id, node)
    return None


def _suggest_name(old: str, context: ast.FunctionDef) -> str:
    suggestions = {"x": "value", "i": "index", "n": "count", "s": "text", "a": "items"}
    return suggestions.get(old, "renamed")


def _llm_suggest_rename(source: str) -> tuple[str, str] | None:
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return None
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            for n in ast.walk(node):
                if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
                    if len(n.id) == 1:
                        new_name = llm_client.suggest_variable_name(source, n.id)
                        if new_name:
                            return n.id, new_name
    return None


def apply_step(method_id: str, file: Path, location: CodeLocation) -> AppliedMethod:
    source = file.read_text()
    new_source = None

    if method_id == "rename-variable":
        pair = _llm_suggest_rename(source) or _infer_rename_target(source)
        if pair is None:
            return AppliedMethod(method_id=method_id, target=location, result_diff="", status="failed")
        old_name, new_name = pair
        new_source = transform_rename_variable(source, old_name, new_name)
    elif method_id == "extract-function":
        new_source = transform_extract_function(source, location, func_name="extracted_func")
    else:
        return AppliedMethod(method_id=method_id, target=location, result_diff="", status="failed")

    file.write_text(new_source)
    try:
        diff = subprocess.run(
            ["git", "diff", str(file)],
            capture_output=True, text=True, timeout=10,
        ).stdout
    except Exception:
        diff = ""

    return AppliedMethod(method_id=method_id, target=location, result_diff=diff, status="success")
