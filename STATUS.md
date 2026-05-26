# 状态

reflect 工具 + LLM 组合的实验场。验证模式、评估质量、然后决定是否整合到正式 CLI。

## 当前工具

| 工具 | 状态 | 说明 |
|------|------|------|
| `backward_slice` | ✅ 稳定 | 从行号追溯变量定义（已修复 method chain 追踪） |
| `forward_slice` | ✅ 稳定 | 从定义找所有引用 |
| `flatten_stmts` | ✅ 稳定 | 展平函数体 |
| `call_graph` | ✅ 稳定 | 函数级调用关系 |
| `type_info` | ✅ 稳定 | 变量类型注解 |
| `trace_variable` | ⚠️ 基本路径 | 跨函数不工作 |
| `enhance_finding` (LLM) | ✅ 可用 | DeepSeek + Vault |
| `compute_confidence` | ✅ 可用 | 证据锚定率置信度 |

## 已验证结论

### 证据完整性
- A（单行）→ ❌ LLM 跑偏，编造看似合理的分析
- B（完整代码）→ ✅ 发现 bug + 置信度高
- C（追溯链）→ ✅ 发现 bug + 置信度高
- 证据不足时 LLM **不会说"不知道"**，而是编造一个看似合理的分析。

### 证据排序影响发现方向
- 正向链（执行顺序）：LLM 关注**结构问题**（未使用变量、命名）
- 反向链（依赖顺序）：LLM 关注**正确性问题**（负数校验、浮点精度）
- Forward = 代码审查员，Reverse = 调试器
- 两者交集：数组越界 bug 均发现

### 假阳性过滤流水线（3 轮实验）
- 模式化假阳性占驳回 87.5%（骨架文件 50% + 测试函数 37.5%）
- 启发式过滤降低 48% LLM 调用量，剩余 finding 驳回率 0%
- LLM 在边界情况有效（复杂但单一职责的函数能正确判断）
- 推荐架构：两级流水线 规则启发式 → LLM 二次审查

### 其他
- LLM 在单函数 + 精确证据 → 有效
- LLM 跨文件/跨模块 → 不可靠（空洞架构建议）
- 置信度 = 证据锚定率（确定性指标，非 LLM 自评）

## 已落地到生产 CLI

| 实验结论 | 生产落地 | 位置 |
|----------|----------|------|
| 跳过测试函数的 long-function 检测 | `LongFunctionDetector { skip_test_functions }` 默认 true | `apps/qtcloud-code/src/cli/src/detector/long_function.rs` |
| 跳过骨架文件的 missing-tests 检测 | `is_skeleton_file` + `is_declaration_only` | `apps/qtcloud-code/src/cli/src/detector/missing_tests.rs` |
| 可配置化 | `contract.yaml` → `skip_test_functions` / `skip_skeleton_files` | `apps/qtcloud-code/src/cli/src/config.rs` |

## 已知限制

- `trace_variable` 跨函数不工作
- `call_graph` 统计含第三方库调用（虚高）
- `cross_function_slice` 基本废弃
- LLM 输出不稳定：同 prompt 不同调用可能不同结论
- 没有自动化的 prompt 模板
