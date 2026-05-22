from pathlib import Path
from src.models import SmellInstance, CodeLocation, ReviewReport
from src.reflector import reflect


def test_reflect_empty():
    report = ReviewReport(file=Path(""), source="", smells=[], compile_ok=True)
    r = reflect(report, set())
    assert r.action == "accept"


def test_reflect_single_smell():
    smell = SmellInstance(
        smell_id="long-function",
        location=CodeLocation(Path("test.py"), 1, 30),
        severity=0.5,
        metrics={"line_count": 30},
    )
    report = ReviewReport(file=Path("test.py"), source="", smells=[smell], compile_ok=True)
    r = reflect(report, set())
    assert r.action == "refactor"
    assert r.method_id == "extract-function"


def test_reflect_skip_unmapped():
    smell = SmellInstance(
        smell_id="unknown-smell",
        location=CodeLocation(Path("test.py"), 1, 10),
        severity=0.5,
        metrics={},
    )
    report = ReviewReport(file=Path("test.py"), source="", smells=[smell], compile_ok=True)
    r = reflect(report, set())
    assert r.action == "skip"


def test_reflect_accept_all_tried():
    smell = SmellInstance(
        smell_id="long-function",
        location=CodeLocation(Path("test.py"), 1, 30),
        severity=0.5,
        metrics={},
    )
    report = ReviewReport(file=Path("test.py"), source="", smells=[smell], compile_ok=True)
    key = f"{smell.smell_id}:{smell.location.file}:{smell.location.start_line}"
    r = reflect(report, {key})
    assert r.action == "accept"


def test_reflect_prioritizes_higher_severity():
    low = SmellInstance("long-function", CodeLocation(Path("a.py"), 1, 30), 0.3, {})
    high = SmellInstance("long-function", CodeLocation(Path("b.py"), 1, 30), 0.9, {})
    report = ReviewReport(file=Path(""), source="", smells=[low, high], compile_ok=True)
    r = reflect(report, set())
    assert r.target is high
