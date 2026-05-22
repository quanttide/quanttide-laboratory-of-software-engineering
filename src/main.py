"""代码重构智能体 — Demo 入口"""

from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src.session import RefactoringSession


def main():
    target = Path(__file__).resolve().parent / "sample.py"
    session = RefactoringSession(target)
    session.run()


if __name__ == "__main__":
    main()
