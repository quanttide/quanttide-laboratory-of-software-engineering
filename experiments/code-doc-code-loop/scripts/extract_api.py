#!/usr/bin/env python3
"""提取器：从代码（AST）提取 API 结构 → 生成 docs/api.md。

代码 → 文档环节：文档是代码的镜像（自动生成，永不过时）。
"""
import ast
import sys
from pathlib import Path

SRC = Path(__file__).parent.parent / "src" / "calculator.py"
DOC = Path(__file__).parent.parent / "docs" / "api.md"

TEMPLATE = """# Calculator API 文档

> 由 scripts/extract_api.py 从代码自动生成——不要手改，运行提取器更新。
> 生成时间：{time}

## 函数

{functions}
"""


def extract_functions(source: str) -> list[dict]:
    """AST 提取顶层函数：名称/参数/文档字符串首行。"""
    tree = ast.parse(source)
    funcs = []
    for node in tree.body:
        if isinstance(node, ast.FunctionDef):
            params = [a.arg for a in node.args.args]
            doc = ast.get_docstring(node) or ""
            funcs.append({
                "name": node.name,
                "params": params,
                "summary": doc.split("\n")[0] if doc else "",
            })
    return funcs


def render(funcs: list[dict]) -> str:
    """渲染文档。"""
    lines = []
    for f in funcs:
        lines.append(f"### {f['name']}({', '.join(f['params'])})")
        if f["summary"]:
            lines.append(f"\n{f['summary']}")
        lines.append("")
    return TEMPLATE.format(time="(提取器生成)", functions="\n".join(lines))


def main() -> int:
    source = SRC.read_text(encoding="utf-8")
    funcs = extract_functions(source)
    DOC.parent.mkdir(parents=True, exist_ok=True)
    DOC.write_text(render(funcs), encoding="utf-8")
    print(f"已生成 {DOC}（{len(funcs)} 个函数）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
