"""集成测试：在真实 TypeScript 代码上运行完整重构流程"""

import re
import ast
from pathlib import Path
from src.models import SmellInstance, CodeLocation
from src.planner import plan
from src.knowledge import code_smells, correspondences


FIXTURE = Path(__file__).resolve().parent / "fixtures" / "sample.ts"


def _find_functions_ts(source: str) -> list[dict]:
    """
    用正则提取 TypeScript 函数/方法定义及其行号范围。
    覆盖: function foo(), const foo = () =>, const foo = function(),
           method() {{ }}, async/await 等常见模式。
    注意: 大括号匹配不完美但足以做集成测试的告警。
    """
    lines = source.splitlines()
    functions = []

    patterns = [
        re.compile(r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)"),
        re.compile(r"^\s*(?:export\s+)?(?:async\s+)?const\s+(\w+)\s*=\s*(?:async\s+)?(?:\(|[\w,]+)"),
        re.compile(r"^\s*(?:export\s+)?(?:async\s+)?const\s+(\w+)\s*=\s*function"),
        re.compile(r"^\s*(?:public|private|protected|static|async|export)?\s*(\w+)\s*\([^)]*\)\s*\{"),
    ]

    for i, line in enumerate(lines, 1):
        for pat in patterns:
            m = pat.search(line)
            if m:
                func_name = m.group(1)
                start = i
                # 粗略找匹配的结束大括号
                brace_count = 0
                end = start
                for j in range(i - 1, len(lines)):
                    brace_count += lines[j].count("{") - lines[j].count("}")
                    if brace_count <= 0 and j > i - 1:
                        end = j + 1
                        break
                if end > start:
                    functions.append({"name": func_name, "start": start, "end": end})
                break

    return functions


def _find_classes_ts(source: str) -> list[dict]:
    """提取 TypeScript class 定义及其行号范围。"""
    lines = source.splitlines()
    classes = []
    for i, line in enumerate(lines, 1):
        m = re.search(r"^\s*(?:export\s+)?(?:abstract\s+)?class\s+(\w+)", line)
        if m:
            start = i
            brace_count = 0
            end = start
            for j in range(i - 1, len(lines)):
                brace_count += lines[j].count("{") - lines[j].count("}")
                if brace_count <= 0 and j > i - 1:
                    end = j + 1
                    break
            classes.append({"name": m.group(1), "start": start, "end": end})
    return classes


def test_pipeline_detects_smells_in_real_code():
    """集成测试 #1: 端到端检测真实代码的坏味道"""
    source = FIXTURE.read_text()
    assert len(source) > 0

    functions = _find_functions_ts(source)
    classes = _find_classes_ts(source)

    assert len(functions) > 0, "应至少检测到一个函数"
    assert len(classes) > 0, "应至少检测到一个 class"

    # 对每个超长函数创建 SmellInstance
    smells: list[SmellInstance] = []
    for fn in functions:
        line_count = fn["end"] - fn["start"]
        if line_count > 30:
            smells.append(SmellInstance(
                smell_id="long-function",
                location=CodeLocation(FIXTURE, fn["start"], fn["end"]),
                severity=min(1.0, line_count / 80),
                metrics={"line_count": line_count, "function": fn["name"]},
            ))

    for cls in classes:
        line_count = cls["end"] - cls["start"]
        method_count = sum(
            1 for fn in functions
            if fn["start"] >= cls["start"] and fn["end"] <= cls["end"]
        )
        if method_count > 10:
            smells.append(SmellInstance(
                smell_id="large-class",
                location=CodeLocation(FIXTURE, cls["start"], cls["end"]),
                severity=min(1.0, max(method_count, line_count) / 50),
                metrics={"method_count": method_count, "class": cls["name"]},
            ))

    assert len(smells) > 0, "sample.ts 应该有坏味道"


def test_pipeline_plan_from_real_smells():
    """集成测试 #2: 检测 → 规划全流程"""
    source = FIXTURE.read_text()

    # 复用 test_pipeline_detects_smells_in_real_code 的逻辑
    functions = _find_functions_ts(source)
    classes = _find_classes_ts(source)

    smells = []
    for fn in functions:
        lc = fn["end"] - fn["start"]
        if lc > 30:
            smells.append(SmellInstance(
                smell_id="long-function",
                location=CodeLocation(FIXTURE, fn["start"], fn["end"]),
                severity=min(1.0, lc / 80),
                metrics={"line_count": lc, "function": fn["name"]},
            ))
    for cls in classes:
        mc = sum(1 for fn in functions if fn["start"] >= cls["start"] and fn["end"] <= cls["end"])
        if mc > 10:
            smells.append(SmellInstance(
                smell_id="large-class",
                location=CodeLocation(FIXTURE, cls["start"], cls["end"]),
                severity=min(1.0, mc / 50),
                metrics={"method_count": mc, "class": cls["name"]},
            ))

    # plan
    steps = plan(smells)

    assert len(steps) > 0, "应规划出重构步骤"
    for step in steps:
        assert step.method_id in {c.target for c in correspondences}, \
            f"手法 {step.method_id} 应在知识库中"


def test_fixture_has_long_functions():
    """集成测试 #3: 验证 sample.ts 确实包含过长函数"""
    source = FIXTURE.read_text()
    functions = _find_functions_ts(source)
    long_ones = [f for f in functions if f["end"] - f["start"] > 30]
    assert len(long_ones) > 0, "sample.ts 应包含超长函数"
    print(f"\n  超长函数 ({len(long_ones)} 个):")
    for f in sorted(long_ones, key=lambda x: -(x["end"] - x["start"])):
        print(f"    {f['name']}: L{f['start']}-{f['end']} ({f['end'] - f['start']} 行)")


def test_fixture_summary():
    """集成测试 #4: 输出 sample.ts 概览"""
    source = FIXTURE.read_text()
    functions = _find_functions_ts(source)
    classes = _find_classes_ts(source)
    methods = [f for f in functions if any(
        c["start"] <= f["start"] <= c["end"] for c in classes
    )]
    print(f"\n  总行数: {len(source.splitlines())}")
    print(f"  函数/方法: {len(functions)} 个")
    print(f"  Class: {len(classes)} 个")
    print(f"  超长函数 (>30行): {sum(1 for f in functions if f['end'] - f['start'] > 30)} 个")
    assert len(functions) > 0
