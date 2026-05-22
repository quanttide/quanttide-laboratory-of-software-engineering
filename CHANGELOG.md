# CHANGELOG

## 2026-05-23

### LLM 集成（Phase LLM）

- 新建 `src/llm_client.py` — 基于 `quanttide-agent` 的 LLM 封装，提供 `check_condition`、`suggest_variable_name`、`verify_semantic` 三个接口，失败时静默降级
- `planner.py` `_check_condition` 从恒 `True` 改为读取源码 → LLM 判定重构前置条件
- `transformers.py` 新增 `_llm_suggest_rename`：优先 LLM 建议变量名，失败后硬编码兜底；`apply_step` 双路径调用
- `session.py` `verify()` 在 `py_compile` 通过后追加 LLM 语义一致性验证；`_backup()` 保存原始内容供比对

### 修复

- `_check_condition` / `verify()` 增加 `file.exists() and file.is_absolute()` 守卫，禁止读取相对路径/不存在文件
- 将残留的 `test.py` 从项目根目录移至 `tests/fixtures/`，移除 `.gitignore` 中 `test.py` 条目

### 文档

- 新增 `docs/quanttide.md` — quanttide + quanttide-agent API 用法教程
- 重写 ROADMAP 测试策略章节为「集成测试设计」，含 4 阶段实施计划
- 更新 TODO.md 新增 Phase LLM 小节

## 2026-05-22

### 初始版本

- 实现代码重构智能体 demo，含检测器/变换器/规划器/会话管理和 17 项测试
- 添加集成测试 fixtures（sample.ts）
- 添加集成测试，在真实 TypeScript 代码上验证坏味道检测和重构规划流水线
- 重写架构设计文档和 index.md
- 添加 STATUS、ROADMAP、TODO 文档
