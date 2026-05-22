"""加载知识模型实例"""

from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from examples.code_refactor import (
    RefactorMethod, CodeSmell, Correspondence, RefactorGoal,
    refactoring_techniques, code_smells, correspondences, refactor_goals,
)


def find_method(method_id: str) -> RefactorMethod | None:
    for m in refactoring_techniques:
        if m.id == method_id:
            return m
    return None


def find_smell(smell_id: str) -> CodeSmell | None:
    for s in code_smells:
        if s.id == smell_id:
            return s
    return None
