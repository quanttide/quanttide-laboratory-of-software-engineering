import ast
import shutil
import pytest
from pathlib import Path
from conftest import PY_FIXTURE, PY_CLEAN_FIXTURE
from src.detectors import scan_file
from src.reviewer import review_file
from src.reflector import reflect
from src.transformers import apply_step
from src.knowledge import correspondences
from src.agent import CodeAgent


@pytest.mark.integration
def test_detect_smells_on_fixture():
    """检测 Python 代码中的三种坏味道。"""
    smells = scan_file(PY_FIXTURE)
    smell_ids = {s.smell_id for s in smells}
    assert "long-function" in smell_ids
    assert "long-parameter-list" in smell_ids
    assert "large-class" in smell_ids
    for s in smells:
        assert s.location.start_line > 0
        assert s.location.end_line > 0
        assert s.metrics is not None
        assert 0.0 <= s.severity <= 1.0


@pytest.mark.integration
def test_detect_no_smells_on_clean():
    """对干净的 Python 代码不产生任何误报。"""
    smells = scan_file(PY_CLEAN_FIXTURE)
    assert len(smells) == 0


@pytest.mark.integration
def test_reflect_recommends_method():
    """检测后 Reflect 推荐对应的重构手法。"""
    report = review_file(PY_FIXTURE)
    assert len(report.smells) >= 1
    r = reflect(report, set())
    known_methods = {c.target for c in correspondences}
    if r.action == "refactor":
        assert r.method_id in known_methods
    elif r.action == "skip":
        assert r.target is not None


@pytest.mark.integration
def test_review_and_refactor(tmp_path):
    """Review 文件 → Reflect 决策 → Refactor 执行。"""
    work_file = tmp_path / "sample.py"
    shutil.copy2(PY_FIXTURE, work_file)
    report = review_file(work_file)
    assert len(report.smells) >= 1
    r = reflect(report, set())
    if r.action == "refactor":
        result = apply_step(r.method_id, work_file, r.target.location)
        assert result.status in ("success", "failed")
        ast.parse(work_file.read_text())


@pytest.mark.integration
def test_agent_run_fix_smells(tmp_path):
    """CodeAgent 完整 run() 循环：Review → Reflect → Refactor。"""
    work_file = tmp_path / "sample.py"
    shutil.copy2(PY_FIXTURE, work_file)
    agent = CodeAgent(work_file)
    agent.run()
    assert len(agent.state.applied) >= 1
    for a in agent.state.applied:
        assert a.status == "success"
    ast.parse(work_file.read_text())


@pytest.mark.integration
def test_agent_run_clean_file(tmp_path):
    """对干净的代码，run() 不执行任何重构。"""
    clean_file = tmp_path / "clean.py"
    shutil.copy2(PY_CLEAN_FIXTURE, clean_file)
    agent = CodeAgent(clean_file)
    agent.run()
    assert len(agent.state.applied) == 0
