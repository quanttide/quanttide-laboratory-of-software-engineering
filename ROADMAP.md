# ROADMAP

## 假设

1. `apps/qtcloud-code/` (qtcloud-code-cli) 是生产 CLI 工具，用户实际运行它。
2. `examples/default/` 是研究原型，探索"知识驱动的自主重构智能体"——设计有野心，但实现只有 Python 且不完整。
3. 两边的共同知识基座是 `examples/code_refactor.py`（6 类认知模型 + 实例数据），但它嵌在 git 子模块内，无法被 `apps/` 直接引用。
4. 最终目标是将智能体能力反哺到生产 CLI，使 `qtcloud-code audit` 不仅能报告问题，还能自动修复。
5. 主力开发语言是 Python，需要读各种代码（TS/JS 等）。因此多语言**检测**有价值，多语言**变换**不需要。

---

## 取舍建议

### 核心判断：不做"大而全的智能体"，做"小而精的审计+修复流水线"

当前智能体架构（6 元认知模型、推理链）设计过度，工程实现跟不上。与其维持一个残缺的"AI 智能体"，不如把资源集中在**可交付的工程价值**上。

### 要做什么

| 优先级 | 项目 | 原因 |
|--------|------|------|
| P0 | 统一数据模型 | `AuditResult` 与 `SmellInstance` 能互转，审计→修复链路打通 |
| P0 | 明确语言策略 | 决定 multi-language 的技术路线（tree-sitter vs CLI wrapper），这会阻塞后续所有阶段 |
| P1 | Transformers 扩展（Python） | **主力写 Python，自动修复直接提升效率。Python 有 `ast.NodeTransformer` + `unparse` 的语言红利，应优先收割** |
| P1 | 真实验证机制 | `_check_condition` 不再恒 True，替换为编译/类型检查/测试运行 |
| P2 | 多语言检测（只读） | 需要读各种代码，CLI wrapper 检测报告辅助理解 |
| P2 | CLI 集成 | `qtcloud-code fix` 命令：audit + 推荐 + 用户确认 + 自动修复 |

### 不做什么

| 放弃 | 原因 |
|------|------|
| 6 元认知模型的运行时推理 | 当前的 planner 只用了 Correspondence，其他 4 个模型只是文档。不应强行在代码中体现全部认知模型 |
| LLM 集成 | 当前硬编码规则已经够用，引入 LLM 增加不确定性和成本，且无清晰的评估标准。`condition` 字段标记为"知识文档"而非"执行代码" |
| 合并到 `apps/` | `examples/default` 是最终归宿，不做跨仓库合并 |
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

### Phase 0 — 清理不一致（在 `examples/default` 内完成）

目标：让现有代码说同一套话。

- 统一检测阈值（>30 行 vs >50 行，选择 >30）
- 统一检测入口：`integrated_tests` 不再自建正则，改为调用公共 detector
- 删除 `examples/default/examples/code_refactor.py`（已稳定无用），数据内联到 `knowledge.py` 或归档

> 开发策略：全部在 `examples/default` 内完成，`apps/` 不纳入路线图。

### Phase 1 — 数据模型统一

目标：审计→修复链路首次打通。

- 定义共享的 `SmellReport` 数据类，合并 `AuditResult` + `SmellInstance` 的能力
- `audit.py` 输出 `SmellReport` 而非专有 `AuditResult`
- `planner.py` 消费 `SmellReport` 生成 `PlanStep`
- 将 `examples/default/src/models.py` 提升到共享包

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。**这是路线图上价值最高的阶段**——Python 有 `ast.NodeTransformer` + `ast.unparse()` 的 stdlib 红利，应优先收割。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

- 实现 `extract-class`（同一文件内）：基于 AST 引用分析做字段/方法内聚性聚类
- 实现 `move-function`（同一文件内）

### Phase 3 — 真实验证

目标：自动修复不再盲目执行。

验证分级（避免给用户虚假的"验证通过"信号）：
- **L1（Phase 3 覆盖）**：编译/类型检查 —— `py_compile`（Python）
- **L2（未来）**：运行项目测试 —— 可选，需要用户显式启用

- `_check_condition` 改为枚举检查，弃用字符串 `condition` 字段的运行时角色
- 处理"验证失败是因为项目本身有问题还是重构引入的"——重构前运行 baseline 检查，重构后对比增量
- `condition` 字段保留在 code_refactor.py 中作为知识文档，但不参与执行逻辑

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

~~Phase 5 — CLI 集成~~（已删除）

---

## 补充事项

### 测试策略

| Phase | 测试动作 |
|-------|---------|
| P0-P1 | 保持现有 17 个单元测试 + 4 个集成测试不变 |
| P2 | 新增 Python transformers 测试，与旧测试并行 |
| P3 | 新增 verifier 测试 |
| P4 | 新增 TS 检测器测试（mock subprocess），与 Python 测试并行 |

### CI/CD 策略

- Phase 0-3：`pip install` + `pytest`，无额外构建依赖
- Phase 4：新增 `npm install`（eslint + typescript），无需 C 编译 toolchain
- 明确不引入 tree-sitter，避免构建复杂度膨胀
