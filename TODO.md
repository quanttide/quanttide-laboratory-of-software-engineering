# TODO

> 由 ROADMAP.md 拆解为具体代办。格式：`[ ] <phase> <area> <description>`。

## Phase 0 — CodeAgent（代码重构已完成，测试已通过）

### AST 清理（还未做）

- [ ] `transform_rename_variable` 从 `ast.unparse` 改为字符串操作 + `ruff format`
- [ ] `transform_extract_function` 修 return 处理和缩进偏移 edge case

### 知识库清理（还未做）

- [ ] 删除 `examples/code_refactor.py`，数据内联到 `knowledge.py`
- [ ] 更新 `knowledge.py` 改为内联数据，移除 `sys.path.insert`

---

## Phase 1 — Reflector L2 增强（LLM）

- [ ] 失败归因：LLM 分析失败原因是策略不对还是代码太复杂 → 换策略重试
- [ ] 增量坏味道权衡：LLM 判断新坏味道比原来的更轻，接受还是继续重构
- [ ] 循环终止判断：LLM 判定"当前状态可接受"，主动结束循环
- [ ] 自然语言解释：每次 Reflection 输出人类可读的决策理由

---

## Phase 2 — Transformers 扩展（Python）

> AST 仅用于分析阶段。代码输出统一用字符串操作 + `ruff format`。

### Python extract-class（同一文件内）

- [ ] `transformers.py` 新增 `transform_extract_class(source, location) → str`
- [ ] AST 分析：扫描 `self.<field>` 引用频率 → 构建方法-字段邻接矩阵 → 按共用字段数聚类
- [ ] 字符串输出：生成新 class 定义 + 原 class 委托调用 + `ruff format`
- [ ] 单元测试 `tests/test_transformers.py` 新增 3-5 个测试用例

### Python move-function（同一文件内）

- [ ] `transformers.py` 新增 `transform_move_function(source, target_class) → str`
- [ ] AST 分析：引用上下文（`self`、类字段、其他方法调用）
- [ ] 字符串输出：目标 class 方法 + 原处委托 + `ruff format`
- [ ] 单元测试


