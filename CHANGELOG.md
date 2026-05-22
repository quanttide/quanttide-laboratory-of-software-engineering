# CHANGELOG

## 2026-05-23

### 文档重构

- 重写 `README.md` — 从技术导向改为面向用户的工具说明
- 新增 `docs/user-guide.md` — 完整用户指南，描述实际行为
- 删除自动生成脚本 `scripts/generate_readme.py`

### 核心变更

- `main.py` 移除硬编码 `sample.py`，改为接受命令行参数 `<文件路径|目录路径>`
- `session.run()` 去除自动 `_restore_all` 回退，成功变更持久保留
- `RefactoringSession` 目标路径解析为绝对路径，修复相对路径验证失败

### 修复

- 修复 `extract-function` 缩进错误（`base_indent` 包含函数定义行偏移）
- 修复重复嵌套函数检测 — `_get_functions` 重复遍历嵌套 `def`
- 集成测试 fixture 清理：使用 `tmp_path` 避免 fixture 变异
- `verify()` 和 `_check_condition` 增加 `file.exists()` 和 `file.is_absolute()` 守卫
- 将残留的 `test.py` 从项目根目录移至 `tests/fixtures/`，移除 `.gitignore` 中 `test.py` 条目

### 集成测试重构

- 新建 `conftest.py` — 共享 fixture 路径 + pytest mark 注册（`@pytest.mark.integration`、`@pytest.mark.llm`）
- 新建 `test_python_pipeline.py` — 4 个端到端测试（检测完整性、误报、plan 映射、全流水线 + ast.parse 语法验证）
- 新建 `test_fixtures.py` — 3 个 fixture 自检
- 新建 `fixtures/sample.py`（Python 坏味道对照）和 `fixtures/clean.py`（干净对照）
- `test_refactoring_pipeline.py` 删除正则解析函数，改为只做 fixture 自检
- `test_detectors.py` `test_scan_file_ignores_short_code` 改用 `tmp_path` 根治 `test.py` 残留
- 强化断言：增加 `ast.parse`、severity 范围、location 检查
- 删除 `sample.py`（原坏味道 fixture），改用集成测试 fixtures

### 文档维护

- ROADMAP 移除所有 `qtcloud-code` 相关计划，聚焦 `examples/default`
- ROADMAP 新增用户文档策略和结构规划
- TODO 清理已完成的集成测试重构和 LLM 条目
- ROADMAP 增加对 SKILL.md 文档流转规则的引用
- 重写 ROADMAP 测试策略章节为「集成测试设计」

### LLM 集成（Phase LLM）

- 新建 `src/llm_client.py` — 基于 `quanttide-agent` 的 LLM 封装，提供 `check_condition`、`suggest_variable_name`、`verify_semantic` 三个接口，失败时静默降级
- `planner.py` `_check_condition` 从恒 `True` 改为读取源码 → LLM 判定重构前置条件
- `transformers.py` 新增 `_llm_suggest_rename`：优先 LLM 建议变量名，失败后硬编码兜底；`apply_step` 双路径调用
- `session.py` `verify()` 在 `py_compile` 通过后追加 LLM 语义一致性验证；`_backup()` 保存原始内容供比对

### 文档归档

- 新增 `docs/quanttide.md` — quanttide + quanttide-agent API 用法教程
- 删除过时架构文档（`agent-cognition.md`、`architecture.md`、`index.md`）
- `quanttide.md` 移至 `assets/`

## 2026-05-22

### 初始版本

- 实现代码重构智能体 demo，含检测器/变换器/规划器/会话管理和 17 项测试
- 添加集成测试 fixtures（sample.ts）
- 添加集成测试，在真实 TypeScript 代码上验证坏味道检测和重构规划流水线
- 重写架构设计文档和 index.md
- 添加 STATUS、ROADMAP、TODO 文档
