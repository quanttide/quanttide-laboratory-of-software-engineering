from src.models import SmellInstance, Reflection, ReviewReport
from src.knowledge import find_method, correspondences


def _calc_priority(smell: SmellInstance) -> int:
    return int((1 - smell.severity) * 100)


def _map_smell_to_method(smell_id: str) -> str | None:
    for corr in correspondences:
        if corr.source == smell_id:
            return corr.target
    return None


def reflect(report: ReviewReport, tried: set[str]) -> Reflection:
    if not report.smells:
        return Reflection(action="accept", reason="代码干净")

    active = [s for s in report.smells if _smell_key(s) not in tried]
    if not active:
        return Reflection(action="accept", reason="所有坏味道已处理或跳过")

    target = max(active, key=lambda s: (s.severity, s.location.start_line))
    method_id = _map_smell_to_method(target.smell_id)
    if method_id is None:
        return Reflection(action="skip", target=target, reason="无可用修复手法")

    return Reflection(action="refactor", method_id=method_id, target=target,
                      reason=f"severity={target.severity:.2f}")


def _smell_key(smell: SmellInstance) -> str:
    return f"{smell.smell_id}:{smell.location.file}:{smell.location.start_line}"
