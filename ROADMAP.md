# ROADMAP

> 项目约定、文件边界和维护规则见 `.agents/skills/code-context/SKILL.md`。
> ROADMAP 遵循 SKILL.md 的文档流转规则：已完成项目移入 CHANGELOG，仅保留待实施内容。

## 假设

1. `examples/default/` 是研究原型，探索"知识驱动的自主重构智能体"——设计有野心，但实现只有 Python 且不完整。
2. 主力开发语言是 Python，需要读各种代码（TS/JS 等）。因此多语言**检测**有价值，多语言**变换**不需要。
3. LLM 是智能体的核心推理引擎，但不是硬依赖——无 LLM 时降级为纯规则模式，功能子集可用。

---

## 取舍建议

### 核心判断：做"反思智能体（Reflection Agent）"，不做"大而全的认知智能体"

上一版说"不做智能体，做流水线"——这是对 6 元认知模型过度设计的矫枉过正。实际问题是旧架构（Scan→Plan→Execute→Verify）没有反馈回路，失败后不会换策略，不会反思。

智能体的价值不在"认知模型多完整"，而在"能否根据执行结果调整下一步"。采用 Reflection 架构：Review 观察 → Reflect 决策 → Refactor 执行 → 循环。

### 要做什么

| 优先级 | 项目 | 原因 |
|--------|------|------|
| P0 | **Reflection Agent 架构重构**：Review→Reflect→Refactor 循环 | 旧架构无反馈回路，失败不换策略。这是所有后续能力的基础 |
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

---

## 技术路线选择：多语言检测策略

两种路线，选择 **B**：

| 方案 | 做法 | 成本 |
|------|------|------|
| **A: tree-sitter AST** | 替换 `ast.parse`，统一多语言 AST 接口 | 3-5 周，检测器全量重写，CI 需 C 编译 toolchain |
| **B: CLI wrapper ← 选此** | `tsc --noEmit` + `eslint` 做 TS 检测；Python 侧保持 `ast` 不变 | 1 周，新增 2 个 CLI wrapper 函数，无需重写现有代码 |

**理由**：检测逻辑（行数统计、参数个数、方法个数）根本不需要 AST——集成测试已证明。用 CLI wrapper 成本低一个数量级。

---

## 阶段路线图

### Phase 0 — Reflection Agent 架构

目标：从线性流水线（Scan→Plan→Execute→Verify）重构为反思循环（Review→Reflect→Refactor）。

- 新建 `reviewer.py` — 合并 scan + verify，支持无 baseline 的全量检查和有 baseline 的增量坏味道检测
- 新建 `reflector.py` — L1（纯规则）保证循环不卡死+L2（LLM 增强）智能决策策略重试和失败归因
- 重写 `session.py` — while 循环替代线性执行
- 删除 `planner.py`（功能并入 reflector）、`llm_client.py`（功能并入 reflector）
- 删除 `examples/code_refactor.py`，数据内联到 `knowledge.py` 或归档

### Phase 1 — Reflector L2 增强（LLM）

目标：让 Reflector 在有 LLM 时能做更聪明的判断。

- 失败归因："策略不对还是代码太复杂？"→ 调整策略重试
- 增量坏味道权衡："新坏味道比原来的更轻，接受还是继续重构？"
- 循环终止判断：LLM 判定"当前状态可接受"，主动结束循环
- 自然语言解释：每次 Reflection 输出人类可读的决策理由

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

- 实现 `extract-class`（同一文件内）：基于 AST 引用分析做字段/方法内聚性聚类
- 实现 `move-function`（同一文件内）

---

## 用户文档

面向普通用户的产品说明书，以集成测试为唯一事实源。详细规划见 `docs/user-guide.md`。

- 写作原则：文档不承诺集成测试没有验证过的行为
- 生成方式：手写。不采用自动生成——用户文档需要产品经理视角的叙述逻辑，无法由测试断言拼接
- 输出位置：`README.md`（替换当前技术导向的内容）
