from src.models import SmellInstance, PlanStep
from src.knowledge import find_method, correspondences


def _calc_priority(smell: SmellInstance) -> int:
    return int((1 - smell.severity) * 100)


def plan(smells: list[SmellInstance]) -> list[PlanStep]:
    steps = []
    seen = set()

    for smell in sorted(smells, key=lambda s: -s.severity):
        for corr in correspondences:
            if corr.source == smell.smell_id and corr.target not in seen:
                method = find_method(corr.target)
                if method is None:
                    continue
                steps.append(PlanStep(
                    method_id=method.id,
                    target=smell,
                    priority=_calc_priority(smell),
                    conditions_met=_check_condition(method.condition, smell),
                ))
                seen.add(corr.target)

    steps.sort(key=lambda s: (s.priority, s.target.location.start_line))
    return steps


def _check_condition(condition: str, smell: SmellInstance) -> bool:
    return True
