#!/usr/bin/env python3
"""契约测试：文档声明的 API ↔ 代码实际导出。

文档 → 代码 + 闭环校验：文档与代码不一致即红灯。
覆盖三种偏差：
1. 代码有文档无（代码演进未同步文档）
2. 文档有代码无（文档声明了不存在的 API）
3. 参数签名不一致
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

import calculator  # noqa: E402

DOC = Path(__file__).parent.parent / "docs" / "api.md"

FUNC_RE = re.compile(r"^### (\w+)\(([^)]*)\)\s*$")


def parse_doc_apis(doc_text: str) -> dict[str, list[str]]:
    """解析文档声明的 API：{函数名: [参数列表]}。"""
    apis = {}
    for line in doc_text.splitlines():
        m = FUNC_RE.match(line.strip())
        if m:
            name = m.group(1)
            params = [p.strip() for p in m.group(2).split(",") if p.strip()]
            apis[name] = params
    return apis


def code_apis(module) -> dict[str, list[str]]:
    """代码实际导出的 API：{函数名: [参数列表]}。"""
    return {
        name: list(getattr(module, name).__code__.co_varnames[:getattr(module, name).__code__.co_argcount])
        for name in dir(module)
        if callable(getattr(module, name)) and not name.startswith("_")
    }


def main() -> int:
    doc_apis = parse_doc_apis(DOC.read_text(encoding="utf-8"))
    actual = code_apis(calculator)
    failures = []

    # 方向 1：代码有文档无
    for name in sorted(set(actual) - set(doc_apis)):
        failures.append(f"代码有文档无：{name}{tuple(actual[name])}")

    # 方向 2：文档有代码无
    for name in sorted(set(doc_apis) - set(actual)):
        failures.append(f"文档有代码无：{name}{tuple(doc_apis[name])}")

    # 方向 3：参数签名不一致
    for name in sorted(set(doc_apis) & set(actual)):
        if doc_apis[name] != actual[name]:
            failures.append(f"签名不一致：{name} 文档{doc_apis[name]} vs 代码{actual[name]}")

    if failures:
        print("契约测试红灯：")
        for f in failures:
            print(f"  ✗ {f}")
        return 1
    print(f"契约测试绿：文档与代码一致（{len(actual)} 个 API）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
