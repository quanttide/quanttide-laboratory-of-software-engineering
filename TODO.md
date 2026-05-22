# TODO

> 由 ROADMAP.md 拆解为具体代办。格式：`[ ] <phase> <area> <description>`。

## Phase 0 — 清理不一致

### 知识库工件

- [ ] 删除 `examples/default/examples/` 目录（`code_refactor.py` 已无用）
- [ ] 更新 `knowledge.py` 改为内联数据或指向归档位置

---

## Phase 2 — Transformers 扩展（Python）

### Python extract-class（同一文件内）

- [ ] `transformers.py` 新增 `transform_extract_class(source, location) → str`
- [ ] 聚类策略：扫描 `self.<field>` 引用频率 → 构建方法-字段邻接矩阵 → 按共用字段数聚类
- [ ] 生成新 class 定义 + 原 class 委托调用
- [ ] 单元测试 `tests/test_transformers.py` 新增 3-5 个测试用例

### Python move-function（同一文件内）

- [ ] `transformers.py` 新增 `transform_move_function(source, target_class) → str`
- [ ] 分析源函数的引用上下文（`self`、类字段、其他方法调用）
- [ ] 生成目标 class 方法 + 原处委托
- [ ] 单元测试

---

## Phase 3 — 真实验证

### 条件检查

- [ ] 处理失败来源判断：重构前运行 baseline 检查，重构后对比增量

### 验证分级（L2：未来）

- [ ] 设计中预留 L2 扩展点（运行项目测试）
- [ ] L2 默认关闭，用户通过 `--verify-level 2` 显式启用

---

## Phase 4 — 多语言检测（只读）

### TypeScriptDetector

- [ ] 新建 `integrated_tests/fixtures/sample_small.ts`（精简 fixture，替代 2157 行）
- [ ] 新建 `integrated_tests/test_typescript_detection.py`
- [ ] 阶段一（类型检查）：`tsc --noEmit` 调用，解析输出为结构性问题
- [ ] 阶段二（结构性检测）：正则扫描函数定义、参数列表、class 方法数

---

## 用户文档

- [x] `docs/user-guide.md` — 文档结构规划
- [x] `README.md` — 手写面向用户的 README，替换占位内容
