import ast
import shutil
import pytest
from pathlib import Path
from conftest import PY_FIXTURE, PY_CLEAN_FIXTURE
from src.detectors import scan_file
from src.planner import plan
from src.transformers import apply_step
from src.knowledge import correspondences


@pytest.mark.integration
def test_detect_smells_on_fixture():
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
    smells = scan_file(PY_CLEAN_FIXTURE)
    assert len(smells) == 0


@pytest.mark.integration
def test_plan_from_detection():
    smells = scan_file(PY_FIXTURE)
    steps = plan(smells)
    assert len(steps) >= 1
    known_methods = {c.target for c in correspondences}
    for step in steps:
        assert step.method_id in known_methods
        assert step.conditions_met is not None


@pytest.mark.integration
def test_full_pipeline(tmp_path):
    work_file = tmp_path / "sample.py"
    shutil.copy2(PY_FIXTURE, work_file)
    smells = scan_file(work_file)
    assert len(smells) >= 3
    steps = plan(smells)
    assert len(steps) >= 1
    step = steps[0]
    result = apply_step(step.method_id, work_file, step.target.location)
    assert result.status in ("success", "failed")
    ast.parse(work_file.read_text())
