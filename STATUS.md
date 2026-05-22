# STATUS: examples/default — 代码重构工具

## 项目定位

基于 Python AST 的代码重构工具，实现 Scan → Plan → Execute → Verify 闭环。
知识模型定义在 `examples/code_refactor.py`（待内联），通过 `knowledge.py` 查询。

## 目录结构

```
examples/
  code_refactor.py          知识定义（6 类模型 + 5 种坏味道 + 5 种手法 + 5 条映射，只读元数据）

src/                        核心逻辑
  __init__.py
  detectors.py              Python AST 坏味道检测（长函数/长参数/大类型）
  planner.py                通过 Correspondence 将坏味道 → 重构手法映射并排序
  transformers.py           AST 变换（`rename-variable`、`extract-function`）
  session.py                RefactoringSession 主循环编排
  models.py                 运行时数据模型（CodeLocation, SmellInstance 等）
  knowledge.py              知识库查询接口（find_method, find_smell）
  llm_client.py             LLM 调用封装（quanttide-agent），失败静默降级
  main.py                   接受文件/目录路径参数

tests/                      单元测试（17 项）
  test_detectors.py         3 项
  test_planner.py           4 项
  test_transformers.py      10 项

integrated_tests/           集成测试（9 项，Python-only）
  conftest.py               共享 fixture 路径 + pytest mark 注册
  test_python_pipeline.py   4 项端到端流水线测试
  test_fixtures.py          3 项 fixture 自检
  test_refactoring_pipeline.py 已弃用，2 项 fixture 可读性自检
  fixtures/
    sample.py               Python 坏味道对照（长函数/长参数/大类型）
    clean.py                干净对照（无坏味道）
    sample.ts               遗留 fixture（仅用于已弃用测试的自检）

docs/

assets/
  quanttide.md              quanttide + quanttide-agent API 用法教程

scripts/                    空
```

## 实现状态

| 模块 | 状态 | 说明 |
|------|------|------|
| 知识模型 | ✅ 完整 | 6 类模型 + 5 种坏味道 + 5 种手法 + 5 条映射 |
| Python 检测 | ✅ 实现 | 3 种检测器，基于 `ast.parse` |
| TS 检测 | ❌ 未实现 | `detectors.py` 不支持；已弃用测试仅做 fixture 自检 |
| 规划器 | ✅ 实现 | 映射 + 排序；`_check_condition` 通过 LLM 判断前置条件 |
| 变换器 | ⚠️ 部分 | 仅实现 `rename-variable` 和 `extract-function`，且仅支持 Python |
| 会话管理 | ✅ 实现 | Scan→Plan→Execute→Verify→Rollback 闭环 |
| 验证 | ⚠️ 基础 | `py_compile` 编译检查 + LLM 语义一致性验证；无 pytest/mypy |
| 回滚 | ✅ 实现 | 每步内存备份；成功变更不回退（自 commit c2c011a） |
| 集成测试 | ✅ 实现 | 9 个测试，Python 流水线 + fixture 自检 |

## 已知设计问题

1. **语言支持不一致** — `detectors.py` / `transformers.py` 只支持 Python（`ast`），TS 代码无法检测或变换
2. **变换器覆盖不足** — 仅 `rename-variable` 和 `extract-function`；`extract-class`、`move-function` 等未实现
3. **无统一质量模型** — `audit.py` 的 `AuditResult` 与 `SmellInstance` 互不兼容，审计→修复链路不通
4. **回滚脆弱** — 内存备份，进程崩溃即丢失
5. **展示逻辑散落** — `session.py` print、测试 print，无统一 report 层
6. **知识库引用脆弱** — `knowledge.py` 通过 `sys.path.insert(0, ...)` + `from examples.code_refactor import ...` 引用同级目录模块
7. **阈值不统一** — `detectors.py` 函数 >30 行触发，但无中央配置；`clean.py`/`sample.py` 与之对齐
