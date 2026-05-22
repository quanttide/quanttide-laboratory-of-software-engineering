"""代码重构智能体 — 入口

用法：
    python src/main.py <路径>           # 扫描单个文件
    python src/main.py <目录>           # 扫描整个目录
"""

from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from src.session import RefactoringSession


USAGE = """用法: python src/main.py <文件路径|目录路径>

示例:
    python src/main.py my_code.py          # 扫描单个文件
    python src/main.py my_project/         # 扫描整个目录
"""


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ("--help", "-h"):
        print(USAGE)
        sys.exit(1)
    target = Path(sys.argv[1])
    if not target.exists():
        print(f"路径不存在: {target}\n")
        print(USAGE)
        sys.exit(1)
    session = RefactoringSession(target)
    session.run()


if __name__ == "__main__":
    main()
