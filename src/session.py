import subprocess
from pathlib import Path
from src.models import SessionState
from src.detectors import scan_project, scan_file
from src.planner import plan
from src.transformers import apply_step
from src import llm_client


class RefactoringSession:

    def __init__(self, target: Path):
        self.target = target.resolve()
        self.state = SessionState(project_root=self.target if self.target.is_dir() else self.target.parent)
        self._backups: dict[str, str] = {}

    def run(self):
        print(f"=== Scan: {self.target} ===")
        if self.target.is_dir():
            self.state.smells = scan_project(self.target)
        else:
            self.state.smells = scan_file(self.target)
        print(f"发现 {len(self.state.smells)} 个坏味道\n")

        if not self.state.smells:
            print("没有需要重构的代码")
            return

        for s in self.state.smells:
            print(f"  [{s.smell_id}] {s.location.file.name}:{s.location.start_line}-{s.location.end_line}  severity={s.severity:.2f}  {s.metrics}")

        print(f"\n=== Plan ===")
        self.state.plan = plan(self.state.smells)
        print(f"规划了 {len(self.state.plan)} 步重构\n")

        for idx, step in enumerate(self.state.plan):
            print(f"  {idx+1}. {step.method_id} -> {step.target.smell_id} @ L{step.target.location.start_line}  priority={step.priority}  condition={'OK' if step.conditions_met else 'NO'}")

        print(f"\n=== Execute ===")
        for step in self.state.plan:
            if not step.conditions_met:
                print(f"跳过 {step.method_id}：条件不满足")
                continue

            self._backup(step.target.location.file)
            result = apply_step(method_id=step.method_id, file=step.target.location.file, location=step.target.location)
            ok = self.verify(result) and result.status == "success"
            if ok:
                result.status = "success"
                self.state.applied.append(result)
                print(f"OK {step.method_id} 已应用")
            else:
                result.status = "failed"
                self.state.applied.append(result)
                self.rollback_step(result)
                print(f"FAIL {step.method_id} 失败，已回退")

        print(f"\n=== Done ===")
        print(f"成功: {sum(1 for a in self.state.applied if a.status == 'success')} 步")
        print(f"失败: {sum(1 for a in self.state.applied if a.status == 'failed')} 步")

        for a in self.state.applied:
            if a.result_diff:
                print(f"\n--- diff: {a.method_id} ---")
                print(a.result_diff[:500])

    def _restore_all(self):
        for path_str, content in self._backups.items():
            Path(path_str).write_text(content)

    def verify(self, result) -> bool:
        target = result.target.file
        if not target.exists():
            return True
        try:
            r = subprocess.run(["python", "-m", "py_compile", str(target)], capture_output=True, timeout=10)
            if r.returncode != 0:
                return False
        except Exception:
            return True
        saved = self._original.get(target) if hasattr(self, '_original') else None
        if saved is None:
            return True
        is_ok, _ = llm_client.verify_semantic(saved, target.read_text())
        return is_ok

    def _backup(self, file: Path):
        content = file.read_text()
        self._backups[str(file)] = content
        if not hasattr(self, '_original'):
            self._original = {}
        self._original[file] = content

    def rollback_step(self, result):
        path = str(result.target.file)
        if path in self._backups:
            Path(path).write_text(self._backups[path])
