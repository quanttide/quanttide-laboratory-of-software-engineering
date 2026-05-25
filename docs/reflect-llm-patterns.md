# reflect × LLM 探索总结

## 背景

实验室积累了 4 个确定性分析工具：

| 工具 | 输入 | 输出 |
|------|------|------|
| `flatten_stmts` | 函数节点 | 展平后的语句列表 |
| `backward_slice` | 文件 + 行号 | 影响该行的定义链 |
| `dataflow::trace_variable` | 文件 + 行号 + 变量名 | 变量的赋值源头路径 |
| `enhance_finding` | 代码 + prompt | LLM 分析结果 |

这些工具各自有局限——单独使用时只能回答"这个变量从哪来"，不能回答"这样做安全吗"。

## 发现的模式

### 模式 1：backward_slice + dataflow → 安全分析

输入：`parts[2]` 行的 backward_slice + `qty` 的 dataflow
输出：LLM 发现 `parts.len()` 未检查，越界风险

| 工具 | 产出 |
|------|------|
| backward_slice | L3 trimmed.split → L2 raw.trim |
| dataflow | parts[2] → qty |
| LLM 推理 | split 不保证长度 → 缺少长度检查 → 可能 panic |

**价值：** 规则引擎只检查"函数太长"，reflect 工具只追溯"变量从哪来"，两者的结合才能发现"输入验证不足"这类语义漏洞。

### 模式 2：flatten_stmts → 重复模式识别

输入：展平后的 8 条语句
输出：LLM 识别出 3 处 `trim()` 重复、2 处 `parse().map_err()` 重复

| 工具 | 产出 |
|------|------|
| flatten_stmts | 8 条独立语句 |
| LLM 推理 | 第 3/4/5 行都调用了 trim()，第 4/5 行用了相同 parse 模式 |

**价值：** 规则引擎看到"8 条语句，正常"，但 LLM 看到"3 个字段解析用了同样的 3 步模式"。

### 模式 3：dataflow × N → 一致性检查

输入：4 个变量的 dataflow 路径
输出：LLM 交叉对比后指出"price 和 total 都依赖 parts[1]，单个解析失败会传播到两个输出"

**价值：** dataflow 单独只能追踪单个变量，跨变量对比靠 LLM。

## 工具组合的通用模式

```
reflect 工具链 → 结构化证据 → LLM 推理
   │                   │           │
   │                   │           └─ 发现规则引擎看不到的问题
   │                   │
   │                   └─ 证据链（语句 + 数据流）
   │
   └─ backward_slice / dataflow / flatten_stmts
```

关键原则：**reflect 工具产生 LLM 无法自己生成的证据**（精确的行号、变量链），LLM 在证据基础上做**规则引擎做不到的推理**。

## 限制

1. 当前只在单函数内验证，跨函数分析还没有
2. dataflow 的跨函数路径不工作
3. LLM 输出不稳定——同一条 prompt 在不同调用中可能给出不同结论
4. 没有自动化的 prompt 模板 —— 每个场景需要手写 prompt

## 下一步

- 把确认有效的模式（slice + dataflow → LLM 安全分析）做成可复用的 prompt 模板
- 将模板注册到 `llm.rs` 中，作为 `analyse::security` / `analyse::consistency` 等命名分析
