from pathlib import Path
from src.agent import CodeAgent, _smell_key
from src.models import SmellInstance, CodeLocation, ReviewReport, AppliedMethod


def test_agent_review_file(tmp_path):
    code = "x = 1\n"
    file = tmp_path / "test.py"
    file.write_text(code)
    agent = CodeAgent(file)
    report = agent.review()
    assert report.file == file
    assert report.source == code


def test_agent_review_specific_file(tmp_path):
    a = tmp_path / "a.py"
    b = tmp_path / "b.py"
    a.write_text("x = 1")
    b.write_text("y = 2")
    agent = CodeAgent(tmp_path)
    report = agent.review(b)
    assert report.file == b


def test_agent_reflect_delegates(tmp_path):
    file = tmp_path / "test.py"
    file.write_text("x = 1")
    agent = CodeAgent(file)
    report = ReviewReport(file=file, source="", smells=[], compile_ok=True)
    r = agent.reflect(report)
    assert r.action == "accept"


def test_agent_refactor_calls_apply(tmp_path):
    file = tmp_path / "test.py"
    file.write_text("def f():\n    x = 1\n    y = 2\n    z = 3\n")
    smell = SmellInstance(
        smell_id="long-function",
        location=CodeLocation(file, 1, 4),
        severity=0.5,
        metrics={},
    )
    agent = CodeAgent(file)
    from src.reflector import reflect
    report = ReviewReport(file=file, source=file.read_text(), smells=[smell], compile_ok=True)
    r = reflect(report, set())
    result = agent.refactor(r)
    assert result.method_id == "extract-function"


def test_agent_tracks_skip(tmp_path):
    file = tmp_path / "test.py"
    file.write_text("x = 1")
    agent = CodeAgent(file)
    smell = SmellInstance("unknown", CodeLocation(file, 1, 1), 0.5, {})
    agent._tried.add(_smell_key(smell))
    report = ReviewReport(file=file, source="", smells=[smell], compile_ok=True)
    r = agent.reflect(report)
    assert r.action == "accept"


def test_smell_key():
    file = Path("/a/b.py")
    loc = CodeLocation(file, 5, 10)
    s = SmellInstance("long-function", loc, 0.5, {})
    assert _smell_key(s) == "long-function:/a/b.py:5"
