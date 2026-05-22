from pathlib import Path
from src.models import ReviewReport, Reflection, AppliedMethod, SessionState
from src.reviewer import review_file, review_project
from src.reflector import reflect
from src.transformers import apply_step


class CodeAgent:

    def __init__(self, target: Path):
        self.target = target.resolve()
        self.state = SessionState(project_root=self.target if self.target.is_dir() else self.target.parent)
        self._tried: set[str] = set()
        self._backups: dict[str, str] = {}

    def review(self, file: Path | None = None) -> ReviewReport:
        target = file or self.target
        if target.is_dir():
            return ReviewReport(file=target, source="", smells=[], compile_ok=True)
        return review_file(target)

    def reflect(self, report: ReviewReport) -> Reflection:
        return reflect(report, self._tried)

    def refactor(self, reflection: Reflection) -> AppliedMethod:
        file = reflection.target.location.file
        self._backup(file)
        return apply_step(
            method_id=reflection.method_id,
            file=file,
            location=reflection.target.location,
        )

    def run(self):
        if self.target.is_dir():
            print(f"=== Review: {self.target} ===")
            reports = review_project(self.target)
            all_smells = [s for r in reports for s in r.smells]
            print(f"发现 {len(all_smells)} 个坏味道\n")
            if not all_smells:
                print("没有需要重构的代码")
                return
            for s in all_smells:
                print(f"  [{s.smell_id}] {s.location.file.name}:{s.location.start_line}-{s.location.end_line}  severity={s.severity:.2f}")
            print()
            report = ReviewReport(file=self.target, source="", smells=all_smells, compile_ok=True)
        else:
            report = self.review()
            print(f"=== Review: {self.target} ===")
            for s in report.smells:
                print(f"  [{s.smell_id}] {s.location.file.name}:{s.location.start_line}-{s.location.end_line}  severity={s.severity:.2f}")
            print()

        while True:
            reflection = self.reflect(report)

            if reflection.action == "accept":
                print(f"接受：{reflection.reason}")
                break
            if reflection.action == "abort":
                print(f"终止：{reflection.reason}")
                break
            if reflection.action == "skip":
                print(f"跳过 [{reflection.target.smell_id}]：{reflection.reason}")
                self._tried.add(_smell_key(reflection.target))
                continue

            print(f"重构 [{reflection.target.smell_id}] → {reflection.method_id}  ({reflection.reason})")
            result = self.refactor(reflection)

            if result.status == "success":
                self.state.applied.append(result)
                self._tried.add(_smell_key(reflection.target))
                print(f"  OK {reflection.method_id} 已应用")
                new_report = self.review(reflection.target.location.file)
                new_smells = [s for s in new_report.smells if _smell_key(s) not in {_smell_key(x) for x in report.smells}]
                if new_smells:
                    print(f"  注意：引入 {len(new_smells)} 个新坏味道")
                    for s in new_smells:
                        print(f"    [{s.smell_id}] {s.location.file.name}:{s.location.start_line}")
                report = new_report
            else:
                self._rollback(result)
                self._tried.add(_smell_key(reflection.target))
                print(f"  FAIL {reflection.method_id} 失败，已回退")

        print(f"\n=== Done ===")
        print(f"成功: {sum(1 for a in self.state.applied if a.status == 'success')} 步")
        print(f"失败: {sum(1 for a in self.state.applied if a.status == 'failed')} 步")
        for a in self.state.applied:
            if a.result_diff:
                print(f"\n--- diff: {a.method_id} ---")
                print(a.result_diff[:500])

    def _backup(self, file: Path):
        self._backups[str(file)] = file.read_text()

    def _rollback(self, result: AppliedMethod):
        path = str(result.target.file)
        if path in self._backups:
            Path(path).write_text(self._backups[path])


def _smell_key(smell) -> str:
    return f"{smell.smell_id}:{smell.location.file}:{smell.location.start_line}"
