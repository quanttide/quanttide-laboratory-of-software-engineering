# reflect × LLM 探索总结

## 工具清单

实验室当前积累了 8 个确定性分析工具：

| 工具 | 输入 | 输出 | 成熟度 |
|------|------|------|--------|
| `backward_slice` | 文件 + 行号 | 影响该行的定义链 | ✅ |
| `forward_slice` | 文件 + 定义行 | 该定义的所有引用位置 | ✅ |
| `flatten_stmts` | 函数节点 | 展平后的语句列表 | ✅ |
| `trace_variable` | 文件 + 行号 + 变量名 | 变量的赋值源头路径 | ⚠️ |
| `build_call_graph` | 源码 | 函数级调用和被调用关系 | ✅ |
| `impact_analysis` | 文件 + 行号 | 变更影响范围（forward + call_graph） | ✅ |
| `code_search` | 源码 + 节点类型 | 匹配的 AST 节点列表 | ✅ |
| `type_info` | 源码 | 所有变量的类型注解 | ✅ |
| `enhance_finding` (LLM) | 代码 + prompt | LLM 分析结果 | ✅ |

## 在真实项目上的发现

在 `qtcloud-code-cli`（v0.2.0）源码上运行了全部工具，覆盖 `main.rs`、`long_function.rs`、`missing_tests.rs`、`long_parameter_list.rs`、`rename.rs`。

### call_graph 发现

| 函数 | 调用数 | 被调用数 | 说明 |
|------|--------|----------|------|
| `run_review` | 39 | 1 | CLI 主流程，调用量最大 |
| `build_symbol_table` | 37 | 2 | 单函数遍历树 + 收集定义 + 收集引用 |
| `check_missing_tests` | 24 | 4 | 文件扫描 + 模式匹配 |

> 注意：调用数包含了第三方库调用（`tree_sitter::Parser::new()`、`.walk()`、`.utf8_text()` 等）。不是精确的项目内函数调用数。

### type_info 发现

- 大多数变量类型标注为 `(推断)`——代码中很少显式写类型注解
- 显式标注的类型集中在**集合类型**和**API 边界**：`Vec<PathBuf>`、`Vec<Finding>`、`Vec<Box<dyn Detector>>`、`Vec<(&str, &str)>`、`Vec<String>`
- 符合 Rust 最佳实践：只在必须标注类型的地方写注解

### code_search 发现

- `main.rs`: 9 个 `return` 表达式（正常范围）
- `missing_tests.rs`: 6 个 `return` 表达式
- 各检测器的 `detect` 函数都有大量 `findings.push()` 调用

### 未解决的精度问题

| 问题 | 影响 | 原因 |
|------|------|------|
| call_graph 统计含库调用 | 调用数虚高 | 无法区分项目函数 vs 第三方库函数 |
| 测试函数"被 0 调用" | 无法判断真实调用关系 | 测试框架通过反射调用 |
| forward_slice 跨函数 | 只追踪同一文件 | 需要模块级符号表 |

## 验证的工具组合模式

### 模式 1：backward_slice + dataflow → 安全分析

输入：`parts[2]` 行的 backward_slice + `qty` 的 dataflow
输出：LLM 发现 `parts.len()` 未检查，越界风险

| 工具 | 产出 |
|------|------|
| backward_slice | L3 trimmed.split → L2 raw.trim |
| dataflow | parts[2] → qty |
| LLM 推理 | split 不保证长度 → 缺少长度检查 → 可能 panic |

### 模式 2：flatten_stmts → 重复模式识别

输入：展平后的 8 条语句
输出：LLM 识别出 3 处 `trim()` 重复、2 处 `parse().map_err()` 重复

### 模式 3：dataflow × N → 一致性检查

输入：4 个变量的 dataflow 路径
输出：LLM 交叉对比后指出"price 和 total 都依赖 parts[1]，单个解析失败会传播到两个输出"

## 工具组合的通用模式

```
reflect 工具链 → 结构化证据 → LLM 推理
   │                   │           │
   │                   │           └─ 发现规则引擎看不到的问题
   │                   │
   │                   └─ 证据链（语句 + 数据流 + 调用关系）
   │
   └─ backward_slice / forward_slice / call_graph / type_info / ...
```

关键原则：**reflect 工具产生 LLM 无法自己生成的证据**（精确的行号、变量链、调用关系），LLM 在证据基础上做**规则引擎做不到的推理**。

## 当前限制

1. call_graph 统计含第三方库调用，需过滤
2. 跨函数分析精度不足（跨文件 forward_slice 没实现）
3. LLM 输出不稳定——同一条 prompt 在不同调用中可能给出不同结论
4. 没有自动化的 prompt 模板——每个场景需要手写 prompt
5. 工具在人工编写的示例代码上验证充分，在真实项目上验证不够
