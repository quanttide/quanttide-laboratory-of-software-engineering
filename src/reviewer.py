import subprocess
from pathlib import Path
from src.models import ReviewReport, SmellInstance
from src.detectors import scan_file


def _check_compile(file: Path) -> bool:
    try:
        r = subprocess.run(["python", "-m", "py_compile", str(file)], capture_output=True, timeout=10)
        return r.returncode == 0
    except Exception:
        return True


def review_file(file: Path) -> ReviewReport:
    source = file.read_text()
    smells = scan_file(file)
    compile_ok = _check_compile(file)
    return ReviewReport(file=file, source=source, smells=smells, compile_ok=compile_ok)


def review_project(project_root: Path) -> list[ReviewReport]:
    reports = []
    for file in sorted(project_root.rglob("*.py")):
        if file.name.startswith("__") or ".venv" in str(file):
            continue
        reports.append(review_file(file))
    return reports
