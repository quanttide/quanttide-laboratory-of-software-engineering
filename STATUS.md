# STATUS: examples/default — 代码重构智能体 Demo

## 项目定位

基于知识模型（`code_refactor.py`）的代码重构智能体 Demo，实现 Scan → Plan → Execute → Verify 闭环。

## 目录结构

```
examples/
  code_refactor.py          知识定义（6 类认知模型 + 实例数据，只读元数据）

src/                        核心逻辑
  detectors.py              Python AST 坏味道检测（长函数/长参数/大类型）
  planner.py                通过 Correspondence 将坏味道 → 重构手法映射并排序
  transformers.py           AST 变换（重命名变量、提取函数）
  session.py                RefactoringSession 主循环编排
  models.py                 运行时数据模型（CodeLocation, SmellInstance 等）
  knowledge.py              知识库查询接口（find_method, find_smell）
  main.py                   demo 入口

tests/                      单元测试（17 项）
integrated_tests/           集成测试（4 项，在真实 TS 代码上运行）
  fixtures/sample.ts        2157 行 TypeScript fixture
docs/                       设计文档
```

## 实现状态

| 模块 | 状态 | 说明 |
|------|------|------|
| 知识模型 | ✅ 完整 | 6 类模型 + 5 种坏味道 + 5 种手法 + 5 条映射 |
| Python 检测 | ✅ 实现 | 3 种检测器，基于 `ast.parse` |
| TS 检测 | ❌ 未实现 | `detectors.py` 不支持；集成测试用正则替代（不严谨） |
| 规划器 | ✅ 实现 | 映射 + 排序，但 `_check_condition` 恒返回 True |
| 变换器 | ⚠️ 部分 | 仅实现 `rename-variable` 和 `extract-function`，且仅支持 Python |
| 会话管理 | ✅ 实现 | Scan→Plan→Execute→Verify→Rollback 闭环 |
| 验证 | ⚠️ 简陋 | 仅 `py_compile` 编译检查，无 pytest/mypy |
| 回滚 | ✅ 实现 | 全量文件内存备份 |
| 集成测试 | ✅ 实现 | 4 个测试，基于正则解析 TS |

## 已知设计问题

1. **语言支持不一致** — `detectors.py` / `transformers.py` 只支持 Python（`ast`），但集成测试在测 `.ts`，不得不自建正则解析，核心逻辑与测试各自维护一套检测。
2. **Transformers 无法处理 TS** — `apply_step` 用 Python AST，对 TS 文件静默写坏。
3. **无统一质量模型** — `audit.py` 的 `AuditResult` 与 `SmellInstance` 互不兼容，审计→修复链路不通。
4. **条件检查形同虚设** — `_check_condition` 恒返回 `True`，无真正前置验证。
5. **回滚脆弱** — 全量内存备份，进程崩溃即丢失。
6. **展示逻辑散落** — `AuditResult.summary()` Markdown、`session.py` print、测试 print，无统一 report 层。
7. **知识库引用脆弱** — `knowledge.py` 通过 `sys.path.insert(0, ...)` + `from examples.code_refactor import ...` 引用同级目录模块。
8. **阈值不统一** — `architecture.md` 写函数 >50 行触发，`detectors.py` 实际用 >30 行，`sample.py` 用 >30 行。

## 与 `apps/qtcloud-code/src/cli/app/audit.py` 的关系

两者是互补的独立工具：
- `audit.py` — 只读审计，依赖 ruff/lizard 外部工具，输出 Markdown 报告
- `examples/default/src` — 读写重构引擎，AST 自实现，可自动修改代码

当前无代码共享，数据模型也不兼容。
