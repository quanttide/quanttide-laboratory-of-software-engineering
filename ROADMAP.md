# ROADMAP

> 项目约定、文件边界和维护规则见 `.agents/skills/code-context/SKILL.md`。
> ROADMAP 遵循 SKILL.md 的文档流转规则：已完成项目移入 CHANGELOG，仅保留待实施内容。

## 假设

1. `examples/default/` 是研究原型，探索"知识驱动的自主重构智能体"——设计有野心，但实现只有 Python 且不完整。
2. 主力开发语言是 Python，需要读各种代码（TS/JS 等）。因此多语言**检测**有价值，多语言**变换**不需要。
3. LLM 是智能体的核心推理引擎，但不是硬依赖——无 LLM 时降级为纯规则模式，功能子集可用。

---

## 取舍建议

### 核心判断：做"CodeAgent"，不做"大而全的认知智能体"

上一版说"不做智能体，做流水线"——这是对 6 元认知模型过度设计的矫枉过正。实际问题是旧架构（Scan→Plan→Execute→Verify）没有反馈回路，失败后不会换策略，不会反思。

CodeAgent 采用 Review→Reflect→Refactor 循环：review 观察代码状态 → reflect 决定怎么做 → refactor 执行修改，然后再次 review 检查结果。

### 要做什么

| 优先级 | 项目 | 原因 |
|--------|------|------|
| P0 | **CodeAgent**：agent.py 提供 review/reflect/refactor/run 四个方法 | 旧架构无反馈回路，失败不换策略。这是所有后续能力的基础 |
| P1 | Reflector L1（无 LLM） | 确定性规则保障离线/CI 下循环可运行，不卡死 |
| P2 | Reflector L2（LLM 增强） | LLM 驱动策略重试、失败归因、自然语言解释 |
| P3 | Transformers 扩展（Python） | 新增 extract-class、move-function |

### 不做什么

| 放弃 | 原因 |
|------|------|
| 6 元认知模型的运行时推理 | 认知模型在策划阶段有用，运行时不需要。Reflection 架构已经提供了足够的推理结构 |
| 回滚机制的持久化改进 | 内存备份对 demo 够用，不用过度工程 |
| tree-sitter AST 全量解析 | 成本被严重低估，改用 CLI wrapper 策略 |
| TS 变换（extract-class / move-function 等） | 主力写 Python，TS 只读不改。TS 变换无可靠工程方案 |
| `ast.unparse` 做代码生成 | 丢失注释/空行/格式。用字符串操作 + `ruff format` 替代 |

---

## 技术路线选择 A：AST 角色定位 — 分析用 AST，修改不用

| 环节 | 做法 | 理由 |
|------|------|------|
| 检测/分析 | `ast.parse` + `ast.walk`（只读） | AST 天然适合理解代码结构，只读无副作用 |
| 代码输出 | 字符串操作 + `ruff format` 兜底 | `ast.unparse` 丢失注释/空行/括号风格，不可用于生产级修改 |
| 变量引用分析 | AST `_find_params` | 纯分析，符合 AST 定位 |
| 类/方法边界识别 | AST 遍历找 `ClassDef`/`FunctionDef` | 纯分析，符合 AST 定位 |

**结论**：AST 停在分析层。所有修改操作的"输出"阶段用字符串操作 + 格式化工具，不用 `NodeTransformer` + `unparse`。

当前 `rename-variable` 在用 `ast.unparse` 输出代码，需要切换到字符串操作 + `ruff format`。

## 技术路线选择 B：多语言检测策略

两种路线，选择 **B**：

| 方案 | 做法 | 成本 |
|------|------|------|
| **A: tree-sitter AST** | 替换 `ast.parse`，统一多语言 AST 接口 | 3-5 周，检测器全量重写，CI 需 C 编译 toolchain |
| **B: CLI wrapper ← 选此** | `tsc --noEmit` + `eslint` 做 TS 检测；Python 侧保持 `ast` 不变 | 1 周，新增 2 个 CLI wrapper 函数，无需重写现有代码 |

**理由**：检测逻辑（行数统计、参数个数、方法个数）根本不需要 AST——集成测试已证明。用 CLI wrapper 成本低一个数量级。

---

## 阶段路线图

### Phase 0 — CodeAgent + AST 清理

目标：从线性流水线重构为 CodeAgent，同时修复 AST 滥用问题。

- 新建 `agent.py` — CodeAgent 类，提供 `review()`、`reflect()`、`refactor()`、`run()` 四个方法
- 新建 `reviewer.py` — 合并 scan + verify，支持无 baseline 的全量检查和有 baseline 的增量坏味道检测
- 新建 `reflector.py` — L1（纯规则）保证循环不卡死 + L2（LLM 增强）智能决策
- 重写 `session.py` — 删除，功能并入 agent.py
- 删除 `planner.py`、`llm_client.py`（功能并入 reflector 和 agent）
- 删除 `examples/code_refactor.py`，数据内联到 `knowledge.py` 或归档
- `transform_rename_variable` 从 `ast.unparse` 改为字符串操作 + `ruff format` 兜底

### Phase 1 — Reflector L2 增强（LLM）

目标：让 Reflector 在有 LLM 时能做更聪明的判断。

- 失败归因："策略不对还是代码太复杂？"→ 调整策略重试
- 增量坏味道权衡："新坏味道比原来的更轻，接受还是继续重构？"
- 循环终止判断：LLM 判定"当前状态可接受"，主动结束循环
- 自然语言解释：每次 Reflection 输出人类可读的决策理由

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

**AST 角色**：AST 仅用于分析阶段（聚类、引用推断）。代码输出统一用字符串操作 + `ruff format`。

- 实现 `extract-class`（同一文件内）：AST 分析字段/方法引用矩阵，字符串操作输出新 class + 委托
- 实现 `move-function`（同一文件内）：AST 分析引用上下文，字符串操作输出目标 class 方法 + 原处委托

---

## 用户文档

面向普通用户的产品说明书，以集成测试为唯一事实源。详细规划见 `docs/user-guide.md`。

- 写作原则：文档不承诺集成测试没有验证过的行为
- 生成方式：手写。不采用自动生成——用户文档需要产品经理视角的叙述逻辑，无法由测试断言拼接
- 输出位置：`README.md`（替换当前技术导向的内容）
