from pathlib import Path
from src.reviewer import review_file, review_project, _check_compile
from src.models import ReviewReport


def test_review_file_returns_report(tmp_path):
    code = "x = 1\n"
    file = tmp_path / "test.py"
    file.write_text(code)
    report = review_file(file)
    assert isinstance(report, ReviewReport)
    assert report.file == file
    assert report.source == code
    assert report.compile_ok is True


def test_review_file_parse_error_is_empty(tmp_path):
    code = "not valid python {{{"
    file = tmp_path / "bad.py"
    file.write_text(code)
    report = review_file(file)
    assert len(report.smells) == 0


def test_review_project_finds_py_files(tmp_path):
    (tmp_path / "sub").mkdir()
    a = tmp_path / "a.py"
    b = tmp_path / "sub" / "b.py"
    a.write_text("x = 1")
    b.write_text("y = 2")
    reports = review_project(tmp_path)
    paths = {r.file.name for r in reports}
    assert paths == {"a.py", "b.py"}


def test_review_project_skips_init(tmp_path):
    (tmp_path / "__init__.py").write_text("")
    a = tmp_path / "a.py"
    a.write_text("x = 1")
    reports = review_project(tmp_path)
    assert len(reports) == 1
    assert reports[0].file.name == "a.py"


def test_check_compile_valid(tmp_path):
    file = tmp_path / "ok.py"
    file.write_text("x = 1")
    assert _check_compile(file) is True


def test_check_compile_invalid(tmp_path):
    file = tmp_path / "bad.py"
    file.write_text("not valid python")
    assert _check_compile(file) is False
