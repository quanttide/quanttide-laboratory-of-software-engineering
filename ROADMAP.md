# ROADMAP

## 假设

1. `examples/default/` 是研究原型，探索"知识驱动的自主重构智能体"——设计有野心，但实现只有 Python 且不完整。
2. 主力开发语言是 Python，需要读各种代码（TS/JS 等）。因此多语言**检测**有价值，多语言**变换**不需要。

---

## 取舍建议

### 核心判断：不做"大而全的智能体"，做"小而精的审计+修复流水线"

当前智能体架构（6 元认知模型、推理链）设计过度，工程实现跟不上。与其维持一个残缺的"AI 智能体"，不如把资源集中在**可交付的工程价值**上。

### 要做什么

| 优先级 | 项目 | 原因 |
|--------|------|------|
| P0 | 明确语言策略 | 决定 multi-language 的技术路线（tree-sitter vs CLI wrapper），这会阻塞后续所有阶段 |
| P1 | Transformers 扩展（Python） | **主力写 Python，自动修复直接提升效率。Python 有 `ast.NodeTransformer` + `unparse` 的语言红利，应优先收割** |
| P2 | 多语言检测（只读） | 需要读各种代码，CLI wrapper 检测报告辅助理解 |

### 不做什么

| 放弃 | 原因 |
|------|------|
| 6 元认知模型的运行时推理 | 当前的 planner 只用了 Correspondence，其他 4 个模型只是文档。不应强行在代码中体现全部认知模型 |
| 回滚机制的持久化改进 | 内存备份对 demo 够用，不用过度工程 |
| tree-sitter AST 全量解析 | 成本被严重低估（检测器全部重写 + transformers 适配 + C 扩展 CI），改用 CLI wrapper 策略 |
| TS 变换（extract-class / move-function 等） | 主力写 Python，TS 只读不改。TS 变换无可靠工程方案（`unparse` 等价物缺失），不值得投入 |

---

## 技术路线选择：多语言检测策略

两种路线，选择 **B**：

| 方案 | 做法 | 成本 |
|------|------|------|
| **A: tree-sitter AST** | 替换 `ast.parse`，统一多语言 AST 接口 | 3-5 周，检测器全量重写，CI 需 C 编译 toolchain |
| **B: CLI wrapper ← 选此** | `tsc --noEmit` + `eslint` 做 TS 检测；Python 侧保持 `ast` 不变 | 1 周，新增 2 个 CLI wrapper 函数，无需重写现有代码 |

**理由**：检测逻辑（行数统计、参数个数、方法个数）根本不需要 AST——集成测试的正则已经证明了。用 CLI wrapper 成本低一个数量级，且在 Phase 5 合并前可以和生产 CLI 共用同一套调用模式（`subprocess.run`，`audit.py` 已在用）。

---

## 阶段路线图

### Phase 0 — 清理不一致

目标：让现有代码说同一套话。

- 统一检测阈值（>30 行 vs >50 行，选择 >30）
- 统一检测入口：`integrated_tests` 不再自建正则，改为调用公共 detector
- 删除 `examples/code_refactor.py`（已稳定无用），数据内联到 `knowledge.py` 或归档

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。**这是路线图上价值最高的阶段**——Python 有 `ast.NodeTransformer` + `ast.unparse()` 的 stdlib 红利，应优先收割。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

- 实现 `extract-class`（同一文件内）：基于 AST 引用分析做字段/方法内聚性聚类
- 实现 `move-function`（同一文件内）

### Phase 3 — 真实验证

目标：处理验证失败的归因问题。

- 处理"验证失败是因为项目本身有问题还是重构引入的"——重构前运行 baseline 检查，重构后对比增量

### Phase 4 — 多语言检测（只读）

目标：TS/JS 代码能走通检测→报告流水线。只读不改，辅助理解代码质量。

策略：**两阶段检测**，不引入自定义 eslint rule。
- 阶段一：`tsc --noEmit` 捕获类型错误
- 阶段二：正则解析（方法同集成测试已验证的方案）做结构性检测：长函数（行数）、长参数列表（参数个数）、大类型（方法数）
- 不创建自定义 eslint rule——避免向用户项目注入侵入性配置

- 实现 `TypeScriptDetector`：`tsc --noEmit` + 正则结构性检测
- 实现 `PythonDetector`：保持当前 `ast` 实现不变
- `scan_project` 按文件扩展名路由到对应 Detector
- 删除 `integrated_tests/_find_functions_ts` 手写正则，改为调用 `TypeScriptDetector`

---

## 集成测试设计

### 现状问题

| 问题 | 影响 |
|------|------|
| 集成测试自建正则解析 TS（`_find_functions_ts` / `_find_classes_ts`），与 `detectors.py` 维护两套检测逻辑 | 核心检测器改完后集成测试不一致 |
| `plan()` 现在调用 `_check_condition` → `llm_client.check_condition`，prompt 写死 ````python`，但 fixture 是 TypeScript | prompt 与语言不匹配，LLM 结果不可靠 |
| 当前 4 个测试实际是"手动构造 SmellInstance 后调 plan"，不是真正的端到端流水线 | 未覆盖 `scan_file` → `apply_step` → `verify` 链路 |
| 无 Python fixture，测试只能用内联代码段或 TS | 缺少 Python 端到端测试的载体 |

### 设计原则

1. **管道即接口**：集成测试只调用公共 API（`scan_file`、`scan_project`、`plan`、`apply_step`），不访问内部函数。
2. **语言分离**：Python 流水线和 TS 只读检测各成独立测试文件，fixture 语言明确对立。
3. **LLM 可选**：含 LLM 调用的测试标记 `@pytest.mark.llm`，CI 中默认跳过；无 API Key 时全部测试通过（降级路径）。
4. **零检测逻辑重复**：集成测试不写正则/AST，检测全部委托给 `detectors.py` / `TypeScriptDetector`。
5. **fixture 最小化**：每个 fixture 只包含目标坏味道，不混入无关代码。

### 测试架构

```
integrated_tests/
├── conftest.py                          # 共享 fixture
│   ├── PY_FIXTURE / TS_FIXTURE          # fixture 路径常量
│   └── PY_CLEAN_FIXTURE                 # 无坏味道对照
├── test_python_pipeline.py              # Python 端到端（无 LLM）
│   ├── test_detect_on_fixture           # scan_file → smells
│   ├── test_plan_from_detection         # scan → plan
│   └── test_full_pipeline               # scan → plan → apply → verify
├── test_typescript_detection.py         # TS 只读检测（Phase 4）
│   ├── test_structural_detection        # TypeScriptDetector
│   ├── test_tsc_integration             # tsc --noEmit
│   └── test_scan_project_routing        # 按后缀路由
├── test_llm_checks.py                   # LLM 路径（@pytest.mark.llm）
│   ├── test_condition_met               # LLM 判定条件满足
│   ├── test_condition_not_met           # LLM 判定条件不满足
│   └── test_semantic_verify             # LLM 语义验证
├── test_fixtures.py                     # fixture 自检
│   ├── test_python_fixture_has_smells   # 确保 fixture 一直有味道
│   └── test_ts_fixture_has_smells       # 确保 TS fixture 一直有味道
└── fixtures/
    ├── sample.py                        # Python：1 个大类 + 1 个长参 + 1 个长函数
    ├── sample.ts                        # TS 现有 2157 行 fixture（不变）
    └── clean.py                         # Python：无坏味道的干净代码
```

### 关键场景

#### test_detect_on_fixture

```python
def test_detect_on_fixture():
    smells = scan_file(PY_FIXTURE)
    smell_ids = {s.smell_id for s in smells}
    assert "long-function" in smell_ids
    assert "long-parameter-list" in smell_ids
    assert "large-class" in smell_ids
    assert len(smells) == 3
```

#### test_full_pipeline

```python
def test_full_pipeline():
    smells = scan_file(PY_FIXTURE)        # ① 检测
    assert len(smells) == 3
    steps = plan(smells)                   # ② 规划
    assert len(steps) >= 1
    for step in steps:
        assert step.conditions_met is not None  # True（降级）或 False（LLM判定）
    # ③ 执行（不验证结果正确性，只验证不崩溃）
    for step in steps:
        result = apply_step(step.method_id, step.target.location.file, step.target.location)
        assert result.status in ("success", "failed")
```

#### test_llm_condition_met (`@pytest.mark.llm`)

```python
@pytest.mark.llm
def test_condition_extract_function_on_long_function():
    code = PY_FIXTURE.read_text()
    assert check_condition("函数体包含至少 2 个语句，且不是单行 def", code, "{'line_count': 34}")
```

### 实施阶段

| Phase | 动作 | 依赖 |
|-------|------|------|
| 当前 | 删 `_find_functions_ts` / `_find_classes_ts`；建 `test_python_pipeline.py` + `fixtures/sample.py` + `test_fixtures.py`；`test_refactoring_pipeline.py` 改为只做 fixture 自检 | 无 |
| Phase 0 | 集成 `test_python_pipeline.py` 入 CI，标记旧测试文件 deprecated | 无 |
| Phase 4 | 建 `test_typescript_detection.py`；新建 `TypeScriptDetector` | `npm install typescript` |
| LLM | 建 `test_llm_checks.py`；CI 中配置 `LLM_API_KEY` 时自动运行 | `LLM_API_KEY` 环境变量 |

### CI/CD 策略

- 基础 CI：`pip install` + `pytest --ignore=integrated_tests/test_llm_checks.py -m "not llm"`，无额外依赖
- Phase 4：`npm install typescript` 但不引入 C 编译 toolchain
- LLM 测试：CI 检测到 `LLM_API_KEY` 时自动运行 `pytest -m llm`，无 Key 时跳过
- 明确不引入 tree-sitter，避免构建复杂度膨胀
