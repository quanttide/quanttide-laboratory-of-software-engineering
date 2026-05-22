# ROADMAP

> 项目约定、文件边界和维护规则见 `.agents/skills/code-context/SKILL.md`。
> ROADMAP 遵循 SKILL.md 的文档流转规则：已完成项目移入 CHANGELOG，仅保留待实施内容。

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

**理由**：检测逻辑（行数统计、参数个数、方法个数）根本不需要 AST——集成测试已证明。用 CLI wrapper 成本低一个数量级。

---

## 阶段路线图

### Phase 0 — 清理不一致

- 删除 `examples/code_refactor.py`（已稳定无用），数据内联到 `knowledge.py` 或归档
- `.agents/skills/code-context/SKILL.md` 中的文档流转规则（ROADMAP→TODO→CHANGELOG）已建立并遵循，无需额外工作

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。**这是路线图上价值最高的阶段**——Python 有 `ast.NodeTransformer` + `ast.unparse()` 的 stdlib 红利，应优先收割。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

- 实现 `extract-class`（同一文件内）：基于 AST 引用分析做字段/方法内聚性聚类
- 实现 `move-function`（同一文件内）

### Phase 3 — 真实验证

目标：处理验证失败的归因问题。

- 处理"验证失败是因为项目本身有问题还是重构引入的"——重构前运行 baseline 检查，重构后对比增量

---

## 用户文档

面向普通用户的产品说明书，以集成测试为唯一事实源。详细规划见 `docs/user-guide.md`。

- 写作原则：文档不承诺集成测试没有验证过的行为
- 生成方式：手写。不采用自动生成——用户文档需要产品经理视角的叙述逻辑，无法由测试断言拼接
- 输出位置：`README.md`（替换当前技术导向的内容）
