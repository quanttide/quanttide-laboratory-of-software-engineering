"""从集成测试生成面向用户的 README。

用法：
    python scripts/generate_readme.py

原理：读取 integrated_tests/test_python_pipeline.py 的 AST，
提取每个测试函数的函数名和 docstring，按文档结构规划组装 README。
"""

import ast
from pathlib import Path

PROJECT = Path(__file__).resolve().parent.parent
TEST_FILE = PROJECT / "integrated_tests" / "test_python_pipeline.py"
README = PROJECT / "README.md"

TEMPLATE = """# 代码重构工具

自动检测 Python 代码中的坏味道，推荐修复方案，并自动修复代码。

## 它能做什么

### 发现坏味道

扫描 Python 代码，识别三种常见坏味道：

{detect_items}

### 规划修复方案

检测到坏味道后，自动从知识库匹配对应的重构手法，按严重程度排序。

### 自动修复代码

执行重构变换，自动修改代码：

{fix_items}

修复后自动验证：编译检查确保语法正确，语义验证确保行为不变。

## 信任底线

{trust_text}

## 能力边界

- 只处理 Python 代码。TypeScript/JavaScript 可检测但不可修复。
- 只处理同一文件内的变换。跨文件重构标记为未来工作。
- 无需 LLM API Key 即可运行。配置后可启用更智能的条件判断。
"""


def extract_tests() -> list[dict]:
    source = TEST_FILE.read_text()
    tree = ast.parse(source)
    tests = []
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
            docstring = ast.get_docstring(node) or ""
            if not docstring:
                docstring = "（待补充）"
            tests.append({"name": node.name, "doc": docstring})
    return tests


def build_readme(tests: list[dict]) -> str:
    detect_tests = [t for t in tests if t["name"] in (
        "test_detect_smells_on_fixture", "test_detect_no_smells_on_clean",
    )]
    detect_items = "\n".join(
        f"- {line.strip().lstrip('- ').rstrip('。')}"
        for t in detect_tests
        for line in t["doc"].split("\n")
        if line.strip().startswith("-")
    )

    fix_items = "\n".join(f"- {t}" for t in ["重命名变量", "提取函数"])

    clean_test = next((t for t in tests if "clean" in t["name"]), None)
    if clean_test:
        clean_doc = clean_test["doc"].split("\n")[0].strip().rstrip("。")
        trust_text = f"{clean_doc}。所有检测结果附带位置和严重度评分，方便判断优先级。"
    else:
        trust_text = "对干净的 Python 代码不产生任何误报。"

    return TEMPLATE.format(
        detect_items=detect_items,
        fix_items=fix_items,
        trust_text=trust_text,
    )


def main():
    tests = extract_tests()
    readme = build_readme(tests)
    README.write_text(readme)
    print(f"✅ README generated: {README}")
    print(f"   Tests parsed: {len(tests)}")


if __name__ == "__main__":
    main()
