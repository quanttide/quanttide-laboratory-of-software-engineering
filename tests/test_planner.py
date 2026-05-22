from pathlib import Path
from src.models import SmellInstance, CodeLocation
from src.planner import plan, _calc_priority


def test_plan_empty():
    assert plan([]) == []


def test_plan_single_smell():
    smell = SmellInstance(
        smell_id="long-function",
        location=CodeLocation(Path("test.py"), 1, 30),
        severity=0.5,
        metrics={"line_count": 30},
    )
    steps = plan([smell])
    assert len(steps) >= 1
    assert steps[0].method_id == "extract-function"


def test_plan_no_duplicate_methods():
    smells = [
        SmellInstance("long-function", CodeLocation(Path("a.py"), 1, 30), 0.5, {}),
        SmellInstance("long-function", CodeLocation(Path("b.py"), 1, 30), 0.5, {}),
    ]
    steps = plan(smells)
    method_ids = [s.method_id for s in steps]
    assert len(set(method_ids)) == len(method_ids)


def test_calc_priority_higher_severity_lower_number():
    low = SmellInstance("x", CodeLocation(Path(""), 1, 1), severity=0.2, metrics={})
    high = SmellInstance("x", CodeLocation(Path(""), 1, 1), severity=0.9, metrics={})
    assert _calc_priority(high) < _calc_priority(low)
