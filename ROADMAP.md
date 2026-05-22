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

**理由**：检测逻辑（行数统计、参数个数、方法个数）根本不需要 AST——集成测试已证明。用 CLI wrapper 成本低一个数量级。

---

## 阶段路线图

### Phase 0 — 清理不一致

- 删除 `examples/code_refactor.py`（已稳定无用），数据内联到 `knowledge.py` 或归档

### Phase 2 — Transformers 扩展（Python）

目标：Python 自动修复覆盖主要坏味道类型。**这是路线图上价值最高的阶段**——Python 有 `ast.NodeTransformer` + `ast.unparse()` 的 stdlib 红利，应优先收割。

交付标准放宽：接受同一文件内简单场景，跨文件场景标记为"未来工作"。

- 实现 `extract-class`（同一文件内）：基于 AST 引用分析做字段/方法内聚性聚类
- 实现 `move-function`（同一文件内）

### Phase 3 — 真实验证

目标：处理验证失败的归因问题。

- 处理"验证失败是因为项目本身有问题还是重构引入的"——重构前运行 baseline 检查，重构后对比增量

---

## 用户文档策略

### 目标

面向普通用户（非开发者）写一份产品说明书，回答"这工具能做什么"而非"这代码怎么工作的"。

### 事实源：集成测试

文档不引用源码实现，以集成测试作为唯一事实源。理由：

| 源 | 可靠性 | 问题 |
|----|--------|------|
| 源码注释 | ❌ | 过时、与实现耦合、对用户无意义 |
| README | ❌ | 手动维护，容易和代码脱节 |
| 集成测试 | ✅ | 每次 `pytest` 验证，通不过就红，不存在"过时文档" |

集成测试 vs 用户文档的映射关系：

```
集成测试断言                              → 用户文档承诺
─────────────────────────────────────────  ────────────────────────
detect_long_function 发现 >30 行函数       → 工具能检测过长函数（阈值 30 行）
detect_large_class 发现 >10 方法类         → 工具能检测过大类（阈值 10 方法）
detect_long_parameter_list 发现 >5 参数    → 工具能检测过长参数列表（阈值 5 个）
test_full_pipeline 对 sample.py 成功变换    → 工具能自动修复 Python 代码
test_detect_no_smells_on_clean 返回 0      → 工具对干净代码静默通过
```

**原则**：文档不承诺集成测试没有验证过的行为。新增功能必须先有集成测试，再写文档。

### 写作方式

| 维度 | 要求 |
|------|------|
| 视角 | 产品经理向普通用户介绍能力，非开发者向开发者解释实现 |
| 篇幅 | 一页（README 级），不超过 30 行正文 |
| 结构 | 见下方"文档结构规划" |
| 语言 | 中文，无代码块，无架构图，无实现细节 |
| 输出位置 | `README.md`（替换当前技术导向的内容） |

### 文档结构规划

结构按用户使用流程组织：先知道工具是什么，再看能做什么，然后试用，最后了解边界。

```
1. 身份卡 — 一行说明工具是什么
   └─ 来源：项目定义（ROADMAP 假设第 2 条）

2. 三步走 — 三种功能按使用流程排列
   2.1 发现坏味道
       └─ 来源：test_detect_smells_on_fixture
           "long-function"（超 30 行函数）
           "long-parameter-list"（超 5 个参数）
           "large-class"（超 10 个方法的类）
   2.2 规划修复方案
       └─ 来源：test_plan_from_detection
           检测后自动推荐对应重构手法
   2.3 自动修复代码
       └─ 来源：test_full_pipeline
           重命名变量、提取函数
           修复后自动验证（编译检查 + 语义一致性）

3. 信任底线 — 什么情况下工具不误报
   └─ 来源：test_detect_no_smells_on_clean
       干净代码 → 零误报
       修复后输出仍是合法 Python → ast.parse 验证

4. 能力边界 — 不做的事
   └─ 来源：ROADMAP「不做什么」章节
       不改 TypeScript（只读）
       不修改跨文件代码
       不接 LLM 做决策（经 try/except 降级可空运行）
```

### 各节与集成测试的对应关系

| 文档章节 | 对应集成测试 | 断言依据 |
|---------|-------------|---------|
| 2.1 发现坏味道 | `test_detect_smells_on_fixture` | `"long-function" in smell_ids` 等 |
| 2.1 阈值精确值 | fixtures/sample.py 的构造方式 | `len(smells) >= 3` + smell_id 断言 |
| 2.2 规划修复方案 | `test_plan_from_detection` | `step.method_id in known_methods` |
| 2.3 自动修复代码 | `test_full_pipeline` | `result.status in ("success", "failed")` |
| 3 信任底线 | `test_detect_no_smells_on_clean` | `len(smells) == 0` |
| 3 修复不破坏语法 | `test_full_pipeline` | `ast.parse(work_file.read_text())` |

### 生成方式

文档由脚本从集成测试断言中提取描述生成，而非手写。当集成测试断言变更时，脚本自动更新 README。

### 实施

- 新增 `scripts/generate_readme.py`：读取 `integrated_tests/test_python_pipeline.py`，提取 `test_` 函数的 docstring（或函数名 + fixture 元数据），组装为面向用户的 README
- 新增 CI 步骤：`pytest integrated_tests/ && python scripts/generate_readme.py`，确保 README 始终与集成测试一致
