# 代码重构智能体 — 架构设计

## 1. 概述

基于 `code_refactor.py` 中的知识模型，构建一个能自主感知代码坏味道、规划并执行重构的 AI 智能体。

核心思想：用结构化知识（类型 + 实例）驱动智能体的推理，而非硬编码规则。

---

## 2. 数据模型

### 2.1 知识层（引用 `code_refactor.py`）

知识层是只读的元数据，定义了智能体知道的所有概念：

| 模型 | 字段 | 用途 |
|------|------|------|
| `RefactorMethod` | id, name, label, motivation, steps, condition | 重构操作的描述 |
| `CodeSmell` | id, name, label, symptom, characteristic, counter_methods | 坏味道的描述 |
| `Correspondence` | id, name, label, source, target | 问题→方案的映射 |
| `RefactorGoal` | id, name, label, criterion, counter_methods | 质量目标 |
| `RefactorProcess` | id, name, label, steps, inputs, outputs | 流程定义 |
| `SafetyNet` | id, name, label, requirement, tools | 安全约束 |

### 2.2 运行时数据

知识是静态的，运行时需要实例化到具体的代码和会话：

```python
@dataclass
class CodeLocation:
    file: Path
    start_line: int
    end_line: int

@dataclass
class SmellInstance:
    """检测到的坏味道实例 — 将 CodeSmell 知识绑定到具体代码位置"""
    smell_id: str          # 引用 CodeSmell.id，如 "long-function"
    location: CodeLocation
    severity: float        # 0.0 ~ 1.0
    metrics: dict          # 如 {"line_count": 120, "complexity": 25}

@dataclass
class AppliedMethod:
    """已执行的重构操作记录"""
    method_id: str         # 引用 RefactorMethod.id
    target: CodeLocation
    result_diff: str       # git diff 输出
    status: str            # "success" | "failed" | "reverted"

@dataclass
class SessionState:
    """一次重构会话的状态"""
    project_root: Path
    smells: list[SmellInstance]
    plan: list[PlanStep]
    applied: list[AppliedMethod]
    branch: str | None     # git 分支名，None 表示未创建
```

---

## 3. 核心流程

智能体的主循环由四个阶段组成，每个阶段有明确的输入/输出：

```
Scan → Plan → Execute → Verify
 ↑                         │
 └─── Rollback ←── Fail ──┘
```

### 3.1 Scan

**输入**: 源代码目录  
**输出**: `list[SmellInstance]`

```python
def scan(project_root: Path) -> list[SmellInstance]:
    """
    扫描项目代码，检测所有 CodeSmell 实例。
    
    对每个 CodeSmell，用其 characteristic 字段决定检测方式：
    - long-function: 计算函数 AST 节点的行数，超过阈值则记录
    - long-parameter-list: 统计函数参数个数
    - duplicate-code: 对 AST 子树做哈希，找重复
    - large-class: 统计类的方法数和字段数
    - shotgun-surgery: 分析跨文件调用关系
    """
    results = []
    for file in project_root.rglob("*.py"):
        tree = ast.parse(file.read_text())
        for smell_def in code_smells:
            detector = get_detector(smell_def.id)
            instances = detector.detect(tree, file)
            results.extend(instances)
    return results
```

每种坏味道对应一个检测函数，函数签名统一：

```python
Detector = Callable[[ast.AST, Path], list[SmellInstance]]
```

### 3.2 Plan

**输入**: `list[SmellInstance]`  
**输出**: `list[PlanStep]`

```python
@dataclass
class PlanStep:
    method_id: str          # 引用 RefactorMethod.id
    target: SmellInstance
    priority: int           # 排序用，越小越优先
    conditions_met: bool    # 执行前验证

def plan(smells: list[SmellInstance]) -> list[PlanStep]:
    """
    1. 通过 Correspondence 将坏味道映射为候选手法
    2. 检查每个候选手法的 condition 是否满足
    3. 按优先级排序（高严重度优先、无依赖优先）
    4. 检测冲突（同一区域的操作不能并行）
    """
    steps = []

    # Step 1: 映射 — 对每个坏味道，找到对应的手法
    for smell in smells:
        for corr in correspondences:
            if corr.source == smell.smell_id:
                method = find_method(corr.target)
                steps.append(PlanStep(
                    method_id=method.id,
                    target=smell,
                    priority=_calc_priority(smell, method),
                    conditions_met=_check_condition(method.condition, smell),
                ))

    # Step 2: 排序
    steps.sort(key=lambda s: (s.priority, _conflict_count(s, steps)))

    return steps

def _check_condition(condition: str, smell: SmellInstance) -> bool:
    """
    检查 RefactorMethod.condition 是否满足。
    条件是用自然语言写的，需要 LLM 或模式匹配来评估。
    例如 extract-function 的条件是 "提取后的函数名能清晰表达代码意图"，
    需要检查目标代码是否可以给出一个有意义的名称。
    """
    ...
```

### 3.3 Execute

**输入**: `PlanStep`  
**输出**: `AppliedMethod`

```python
def execute_step(step: PlanStep, state: SessionState) -> AppliedMethod:
    """
    执行一个 PlanStep：
    1. 读取目标文件
    2. 解析为 AST
    3. 根据 method_id 调用对应的 AST 变换
    4. 写回文件
    5. 记录 diff
    """
    file = step.target.location.file
    source = file.read_text()
    tree = ast.parse(source)

    transformer = _get_transformer(step.method_id)
    new_tree = transformer(tree, step.target.location)

    new_source = ast.unparse(new_tree)
    file.write_text(new_source)

    diff = _git_diff(file)
    return AppliedMethod(
        method_id=step.method_id,
        target=step.target.location,
        result_diff=diff,
        status="success",
    )

def _get_transformer(method_id: str) -> Callable:
    """
    method_id → AST 变换函数。
    每个 RefactorMethod 对应一个变换函数：
    - extract-function: 将指定行范围的语句序列提取为 FunctionDef + Call
    - rename-variable: 替换 AST 中所有 Name 节点
    - extract-class: 将字段/方法分组为新 ClassDef
    当前实现：调用 LLM 生成变换后的代码。
    """
    TRANSFORMERS = {
        "extract-function": _transform_extract_function,
        "rename-variable": _transform_rename_variable,
        "extract-class": _transform_extract_class,
        "move-function": _transform_move_function,
    }
    return TRANSFORMERS[method_id]
```

### 3.4 Verify

**输入**: `AppliedMethod`  
**输出**: `bool`（通过/失败）

```python
def verify(result: AppliedMethod, state: SessionState) -> bool:
    """
    执行安全验证：
    1. 运行现有测试（pytest ...）
    2. 运行类型检查（mypy ...）
    3. 检查 diff 是否只涉及目标区域
    """
    # 运行测试
    test_result = subprocess.run(
        ["pytest", "-x", str(state.project_root)],
        capture_output=True, timeout=60,
    )
    if test_result.returncode != 0:
        return False

    # 运行类型检查
    type_result = subprocess.run(
        ["mypy", str(state.project_root)],
        capture_output=True, timeout=60,
    )
    if type_result.returncode != 0:
        return False

    return True
```

### 3.5 Rollback

```python
def rollback_step(result: AppliedMethod):
    """通过 git revert 回退单步操作"""
    subprocess.run(["git", "revert", "--no-commit", "HEAD"])
```

---

## 4. 检测器实现方案

每个 `CodeSmell` 对应一个检测器，检测器由 characteristic 驱动：

| CodeSmell | characteristic（检测依据） | 检测算法 |
|-----------|--------------------------|----------|
| `long-function` | "函数超过一屏、包含多个层级的缩进" | 函数体 AST 行数 > 50，或圈复杂度 > 10 |
| `duplicate-code` | "复制粘贴的代码块" | AST 子树序列哈希，相似度 > 0.9 |
| `large-class` | "字段超过10个、方法超过20个" | 类节点下字段数 > 10 或方法数 > 20 |
| `long-parameter-list` | "参数超过5个" | 函数参数列表长度 > 5 |
| `shotgun-surgery` | "修改一个功能需要在3个以上的文件中做改动" | git log 中同一功能跨文件提交的频率 |

```python
def detect_long_function(tree: ast.AST, file: Path) -> list[SmellInstance]:
    results = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            line_count = node.end_lineno - node.lineno
            if line_count > 50:
                results.append(SmellInstance(
                    smell_id="long-function",
                    location=CodeLocation(file, node.lineno, node.end_lineno),
                    severity=min(1.0, line_count / 100),
                    metrics={"line_count": line_count},
                ))
    return results
```

---

## 5. 变换器实现方案

每个 `RefactorMethod` 对应一个 AST 变换函数：

### 5.1 ExtractFunction

```
输入: 源文件 + 目标行范围
输出: 修改后的源文件

算法:
1. 解析源文件为 AST
2. 找到目标行范围所在的函数体
3. 将目标语句序列提取到新的 FunctionDef 节点
4. 确定需要传递的局部变量（从剩余代码中回溯引用）
5. 在原位置替换为函数调用表达式
6. 返回 unparse 后的代码
```

### 5.2 RenameVariable

```
输入: 源文件 + 旧变量名 + 新变量名
输出: 修改后的源文件

算法:
1. 解析为 AST
2. 遍历所有 Name 和 arg 节点
3. 替换 id/arg 为新的变量名
4. 返回 unparse 后的代码
```

---

## 6. 会话管理

```python
class RefactoringSession:
    def __init__(self, project_root: Path):
        self.state = SessionState(
            project_root=project_root,
            smells=[],
            plan=[],
            applied=[],
            branch=None,
        )

    def run(self):
        """主循环：Scan → Plan → Execute(loop) → Verify"""
        # Scan
        self.state.smells = scan(self.state.project_root)
        print(f"发现 {len(self.state.smells)} 个坏味道")

        # Plan
        self.state.plan = plan(self.state.smells)
        print(f"规划了 {len(self.state.plan)} 步重构")

        # Execute loop
        for step in self.state.plan:
            if not step.conditions_met:
                print(f"跳过 {step.method_id}：条件不满足")
                continue

            # 执行
            result = execute_step(step, self.state)

            # 验证
            ok = verify(result, self.state)
            if ok:
                self.state.applied.append(result)
                print(f"✓ {step.method_id}")
            else:
                rollback_step(result)
                print(f"✗ {step.method_id}，已回退")
```

---

## 7. 与知识模型的关系

```
code_refactor.py（知识）             本架构（智能体）
─────────────────                    ──────────────────
CodeSmell.characteristic         →   检测器算法的选择依据
Correspondence.source/target     →   Plan 阶段的映射逻辑
RefactorMethod.steps             →   变换器的执行步骤
RefactorMethod.condition         →   Plan 阶段的可行性验证
RefactorProcess                  →   Session 主循环的流程模板
SafetyNet.requirement/tools      →   Verify 阶段的检查项
RefactorGoal.criterion           →   排序阶段的优先级依据
```
