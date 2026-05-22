from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class CodeLocation:
    file: Path
    start_line: int
    end_line: int


@dataclass
class SmellInstance:
    smell_id: str
    location: CodeLocation
    severity: float
    metrics: dict


@dataclass
class AppliedMethod:
    method_id: str
    target: CodeLocation
    result_diff: str
    status: str  # "success" | "failed" | "reverted"


@dataclass
class ReviewReport:
    file: Path
    source: str
    smells: list[SmellInstance]
    compile_ok: bool


@dataclass
class Reflection:
    action: str  # "refactor" | "accept" | "skip" | "abort"
    method_id: str | None = None
    target: SmellInstance | None = None
    reason: str = ""


@dataclass
class SessionState:
    project_root: Path
    smells: list[SmellInstance] = field(default_factory=list)
    applied: list[AppliedMethod] = field(default_factory=list)
    branch: str | None = None
