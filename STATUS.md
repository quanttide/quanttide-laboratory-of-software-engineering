# STATUS: examples/default — CodeAgent

## 项目定位

基于 Python AST 的代码重构智能体，实现 Review → Reflect → Refactor 循环。
知识模型定义在 `examples/code_refactor.py`（待内联），通过 `knowledge.py` 查询。

## 目录结构

```
examples/
  code_refactor.py          知识定义（6 类模型 + 5 种坏味道 + 5 种手法 + 5 条映射，只读元数据）

src/                        核心逻辑
  __init__.py
  agent.py                  CodeAgent 主循环（review → reflect → refactor）
  reviewer.py               代码审查（检测 + 编译检查 → ReviewReport）
  reflector.py              决策（L1 规则 → Reflection）
  detectors.py              Python AST 坏味道检测（长函数/长参数/大类型）
  transformers.py           AST 变换（`rename-variable`、`extract-function`）
  models.py                 运行时数据模型（ReviewReport, Reflection, SmellInstance 等）
  knowledge.py              知识库查询接口（find_method, find_smell）
  main.py                   入口（创建 CodeAgent → run）

tests/                      单元测试（18 项）
  test_detectors.py         4 项
  test_reflector.py         5 项
  test_transformers.py      9 项

integrated_tests/           集成测试（9 项，Python-only）
  conftest.py               共享 fixture 路径 + pytest mark 注册
  test_python_pipeline.py   4 端到端测试（detect + reflect + review & refactor）
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
| Reviewer | ✅ 实现 | 检测 + 编译检查 → ReviewReport |
| Reflector (L1) | ✅ 实现 | 规则决策：refactor / skip / accept |
| Reflector (L2) | ❌ 未实现 | LLM 增强：失败归因、策略重试、主动终止 |
| Python 检测 | ✅ 实现 | 3 种检测器，基于 `ast.parse` |
| 变换器 | ⚠️ 部分 | 仅 `rename-variable` 和 `extract-function`，且仅支持 Python |
| CodeAgent | ✅ 实现 | Review→Reflect→Refactor 循环 + 回滚 |
| 回滚 | ✅ 实现 | 每步内存备份 |
| 集成测试 | ✅ 实现 | 9 个测试 |

## 已知设计问题

1. **语言支持不一致** — 仅 Python；TS 代码无法检测或变换
2. **变换器覆盖不足** — 仅 `rename-variable` 和 `extract-function`
3. **Reflector L2 未实现** — 无 LLM 增强，只能用 L1 确定性规则
4. **回滚脆弱** — 内存备份，进程崩溃即丢失
5. **展示逻辑散落** — `agent.py` print、测试 print，无统一 report 层
6. **知识库引用脆弱** — `knowledge.py` 通过 `sys.path.insert` + `from examples.code_refactor` 引用
7. **阈值不统一** — `detectors.py` 函数 >30 行触发，无中央配置
