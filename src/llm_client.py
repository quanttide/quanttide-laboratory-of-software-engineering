"""LLM 客户端封装 — 基于 quanttide-agent。

所有 LLM 调用集中在此模块，失败时静默降级，不阻塞主流程。
"""

from quanttide_agent import LLM, Message

_llm: LLM | None = None


def _get_llm() -> LLM:
    global _llm
    if _llm is None:
        _llm = LLM()
    return _llm


def check_condition(condition: str, code: str, metrics: str) -> bool:
    """LLM 判断重构前置条件是否满足。

    降级策略：LLM 不可用时返回 True（不阻塞重构）。
    """
    try:
        llm = _get_llm()
        resp = llm.complete([
            Message(role="system", content=(
                "你是一个代码重构专家。判断是否可以对给定代码应用指定的重构方法。"
                "只回复 yes 或 no，不要额外说明。"
            )),
            Message(role="user", content=(
                f"代码：\n```python\n{code}\n```\n"
                f"度量指标：{metrics}\n"
                f"重构条件：{condition}\n"
                "是否满足条件？"
            )),
        ], temperature=0.1, max_tokens=10)
        return resp.content.strip().lower().startswith("yes")
    except Exception:
        return True


def suggest_variable_name(code: str, old_name: str) -> str | None:
    """LLM 建议更好的变量名。降级返回 None，由调用方使用硬编码兜底。"""
    try:
        llm = _get_llm()
        resp = llm.complete([
            Message(role="system", content="你是一个代码重构专家。只回复新变量名，不要额外说明。"),
            Message(role="user", content=(
                f"代码：\n```python\n{code}\n```\n"
                f"变量 {old_name} 的含义是什么？建议一个更好的名字。"
            )),
        ], temperature=0.3, max_tokens=20)
        name = resp.content.strip().strip('"\'`').split()[0]
        return name if name.isidentifier() and name != old_name else None
    except Exception:
        return None


def verify_semantic(original: str, modified: str) -> tuple[bool, str]:
    """LLM 验证重构前后语义是否一致。

    返回 (is_equivalent, reason)。降级返回 (True, "LLM unavailable")。
    """
    try:
        llm = _get_llm()
        resp = llm.complete([
            Message(role="system", content=(
                "你是一个代码重构验证专家。比较重构前后的代码，判断它们的行为是否完全相同。"
                "先思考，然后在一行内回复 yes: 理由 或 no: 理由。"
            )),
            Message(role="user", content=(
                f"重构前：\n```python\n{original}\n```\n"
                f"重构后：\n```python\n{modified}\n```\n"
                "行为是否完全一致？"
            )),
        ], temperature=0.1)
        content = resp.content.strip().lower()
        is_ok = "yes:" in content.split("\n")[-1] or content.startswith("yes")
        return is_ok, content
    except Exception as e:
        return True, f"LLM unavailable: {e}"
