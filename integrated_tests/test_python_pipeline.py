import pytest
from pathlib import Path
from conftest import PY_FIXTURE, PY_CLEAN_FIXTURE
from src.detectors import scan_file, scan_project
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
    assert len(smells) >= 3


@pytest.mark.integration
def test_detect_no_smells_on_clean():
    smells = scan_file(PY_CLEAN_FIXTURE)
    assert len(smells) == 0


@pytest.mark.integration
def test_scan_project_scans_directory():
    smells = scan_project(Path(__file__).resolve().parent / "fixtures")
    assert len(smells) > 0


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
def test_full_pipeline():
    smells = scan_file(PY_FIXTURE)
    assert len(smells) >= 3
    steps = plan(smells)
    assert len(steps) >= 1
    for step in steps:
        result = apply_step(
            method_id=step.method_id,
            file=step.target.location.file,
            location=step.target.location,
        )
        assert result.status in ("success", "failed")
