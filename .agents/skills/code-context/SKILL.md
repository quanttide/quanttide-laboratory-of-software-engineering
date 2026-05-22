---
name: code-context
description: >-
  Use when the user asks about project structure, file boundaries, maintenance
  rules for ROADMAP/TODO/CHANGELOG, module responsibilities, integration test
  conventions, the LLM degradation strategy, or how special files relate to
  each other. Read this skill first when entering the project to understand
  file conventions before making changes.
---

# Code Context: 文件边界与维护方式

## 顶层文档

| 文件 | 用途 | 维护者 | 更新时机 |
|------|------|--------|---------|
| `ROADMAP.md` | 战略方向、取舍决策、阶段路线图 | 项目负责人 | 决策变更或阶段完成时 |
| `TODO.md` | ROADMAP 拆解的具体待办，`[ ]` / `[x]` 标记 | 开发者 | 开始/完成一项工作时 |
| `CHANGELOG.md` | 已完成变更的记录，按日期倒排 | 开发者 | 每次提交或阶段性完成时 |
| `STATUS.md` | 当前实现状态的快照（模块×状态矩阵） | 项目负责人 | 架构变更后 |

### 文档流转规则

```
ROADMAP (战略) → TODO (执行) → 完成 → CHANGELOG (归档)
```

- ROADMAP 的每个阶段拆解为 TODO 的具体条目。TODO 不发明 ROADMAP 没有的任务。
- TODO 的 `[x]` 项在完成一批后批量移入 CHANGELOG，同时从 TODO 中删除。
- ROADMAP 只写"将要做什么"和"不做什么"，不写历史。已完成的项目从 ROADMAP 中移除。

## 代码目录

| 目录 | 用途 | 测试对应 |
|------|------|---------|
| `src/` | 核心引擎 | `tests/`（单元测试） |
| `integrated_tests/` | 端到端流水线测试 | 自身 |
| `tests/` | 单元测试 | `src/` |
| `assets/` | 文档、教程 | — |
| `examples/` | 知识库元数据 | — |

### 依赖方向

```
tests/ → src/
integrated_tests/ → src/ (仅公共 API)
```

- `integrated_tests/` 只调用 `src/` 的公共 API（`scan_file`、`scan_project`、`plan`、`apply_step`），不访问 `_` 开头的内部函数。
- `tests/` 可以测试内部函数，但使用 `from src.transformers import _find_params` 显式导入。
- `integrated_tests/` 不重复检测逻辑：不使用正则/AST 自建检测器。

## 模块职责

| 模块 | 负责 | 不负责 |
|------|------|--------|
| `detectors.py` | 检测代码坏味道 | 规划、变换、验证 |
| `planner.py` | 坏味道→重构步骤映射 | 检测、执行变换 |
| `transformers.py` | AST 变换执行 | 检测、规划、验证 |
| `session.py` | Scan→Plan→Execute→Verify 编排 | 具体检测/变换/验证逻辑 |
| `llm_client.py` | LLM 调用封装，失败静默降级 | 业务逻辑、重构策略决策 |
| `knowledge.py` | 知识库查询 | 业务执行 |
| `models.py` | 数据类定义 | 方法逻辑 |

## 集成测试结构

```
integrated_tests/
├── conftest.py              # 共享 fixture 路径 + pytest mark 注册
├── test_python_pipeline.py  # Python 端到端（检测→规划→变换）
├── test_fixtures.py         # fixture 可读性自检
├── test_refactoring_pipeline.py  # 已弃用，仅 fixture 自检
└── fixtures/
    ├── sample.py            # Python：1 large-class + 1 long-params + 1 long-function
    ├── clean.py             # Python：无坏味道
    └── sample.ts            # TypeScript：2157 行
```

### 测试标记

| 标记 | 适用场景 | CI 行为 |
|------|---------|--------|
| `@pytest.mark.integration` | 端到端流水线测试 | 始终运行 |
| `@pytest.mark.llm` | 需要 LLM API Key | 有 `LLM_API_KEY` 时运行 |

## LLM 降级策略

`llm_client.py` 所有调用均 try/except 兜底。未配置 `LLM_API_KEY` 时：

| 函数 | 降级值 |
|------|--------|
| `check_condition` | `True`（不阻塞重构） |
| `suggest_variable_name` | `None`（使用硬编码兜底） |
| `verify_semantic` | `(True, "LLM unavailable")` |
