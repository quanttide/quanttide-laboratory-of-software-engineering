# TODO

> 由 ROADMAP.md 拆解为具体代办。格式：`[ ] <phase> <area> <description>`。

---

## Phase 0 — 清理不一致

### 阈值统一

- [ ] `detectors.py` 将长函数阈值从 >30 改为 >30（确认即可，无需改代码）
- [ ] `architecture.md` 更新 >50 为 >30，与代码一致
- [ ] 确认长参数列表阈值 >5、大类型阈值 >10 方法/字段在两个文档中一致

### 检测入口统一

- [ ] `integrated_tests/test_refactoring_pipeline.py` 删除 `_find_functions_ts` 和 `_find_classes_ts` 正则函数
- [ ] 新增 `TypeScriptDetector` stub（Phase 2 实现），集成测试改为调用它
- [ ] 添加 pytest mark 标记集成测试（`@pytest.mark.integration`），与单元测试分离

### 知识库工件

- [x] 确认 `code_refactor.py` 已稳定，可以直接删除
- [ ] 删除 `examples/default/examples/` 目录（`code_refactor.py` 已无用）
- [ ] 更新 `knowledge.py` 改为内联数据或指向归档位置

---

## Phase 1 — 数据模型统一

### 共享数据模型

- [ ] 新建 `apps/qtcloud-code/src/cli/app/quality/models.py`
- [ ] 定义 `SmellReport` 数据类（最小公共接口），字段：
  - `source: Path` / `language: Literal["python", "typescript"]` / `detector: str`
  - `location: CodeLocation` / `severity: float` / `detail: str` / `raw_output: str`
- [ ] 定义 `DetectResult` 数据类（检测结果，含 `SmellReport[]` + `passed`）
- [ ] 定义 `PlanStep` 数据类（引用自 `examples/default/src/models.py`）
- [ ] 定义 `AppliedMethod` 数据类（引用自 `examples/default/src/models.py`）

### 审计适配

- [ ] `audit.py` `AuditResult` 新增 `to_smell_reports()` 转换方法
- [ ] `audit.py` `run()` 返回 `DetectResult` 而非仅 `AuditResult`
- [ ] `cli.py` `audit` 命令适配新返回类型

### 检测器适配

- [ ] `detectors.py` `scan_file` / `scan_project` 返回 `list[SmellReport]` 而非 `list[SmellInstance]`
- [ ] 更新 `tests/test_detectors.py` 适配新返回类型

### Planner 适配

- [ ] `planner.py` `plan()` 消费 `list[SmellReport]`
- [ ] 更新 `tests/test_planner.py`

---

## Phase 2 — Transformers 扩展（Python）

### Python extract-class（同一文件内）

交付标准：同一文件内的简单场景，跨文件标记为"未来工作"。

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

- [ ] `planner.py` 定义 `ConditionCheck` 枚举：`LONG_FUNCTION`、`LONG_PARAMS`、`LARGE_CLASS`
- [ ] `_check_condition` 改为匹配枚举 + 执行对应检查逻辑
- [ ] `code_refactor.py` `RefactorMethod.condition` 标记为 `@deprecated`（保留作为知识文档）

### Verify 阶段（L1：编译/类型检查）

- [ ] `session.py` `verify()` 扩展：Python 文件调用 `python -m py_compile`
- [ ] 处理失败来源判断：重构前运行 baseline 检查，重构后对比增量
- [ ] 单元测试 `tests/test_verifier.py`

### 验证分级（L2：未来）

- [ ] 设计中预留 L2 扩展点（运行项目测试）
- [ ] L2 默认关闭，用户通过 `--verify-level 2` 显式启用

---

## Phase 4 — 多语言检测（只读）

### TypeScriptDetector

策略：两阶段检测，不创建自定义 eslint rule。只读不改。

- [ ] 新建 `apps/qtcloud-code/src/cli/app/quality/detectors_ts.py`
- [ ] 阶段一（类型检查）：实现 `run_tsc_check(source_path: str) → list[SmellReport]`：调用 `tsc --noEmit` 解析输出
- [ ] 阶段二（结构性检测，正则）：
  - [ ] `detect_long_function_ts(source: str) → list[SmellReport]`：正则扫描函数定义 + 大括号匹配算行数
  - [ ] `detect_long_parameter_list_ts(source: str) → list[SmellReport]`：正则匹配函数签名，统计参数个数
  - [ ] `detect_large_class_ts(source: str) → list[SmellReport]`：正则扫描 class 定义，统计方法数
- [ ] 单元测试 `tests/test_detectors_ts.py`（mock subprocess 调用）

### 检测路由

- [ ] `apps/qtcloud-code/src/cli/app/quality/detectors.py` 新增 `scan_project` 统一入口
- [ ] 按文件后缀路由：`.py` → `PythonDetector`，`.ts/.tsx` → `TypeScriptDetector`
- [ ] 集成测试改为调用 `scan_project` 公共入口
- [ ] 删除 `integrated_tests/_find_functions_ts` 和 `_find_classes_ts`


