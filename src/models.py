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
class PlanStep:
    method_id: str
    target: SmellInstance
    priority: int
    conditions_met: bool


@dataclass
class AppliedMethod:
    method_id: str
    target: CodeLocation
    result_diff: str
    status: str  # "success" | "failed" | "reverted"


@dataclass
class SessionState:
    project_root: Path
    smells: list[SmellInstance] = field(default_factory=list)
    plan: list[PlanStep] = field(default_factory=list)
    applied: list[AppliedMethod] = field(default_factory=list)
    branch: str | None = None
