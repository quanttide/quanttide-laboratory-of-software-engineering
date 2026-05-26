# reflect 工具 + LLM 使用指南

## 架构概览

```
规则引擎 → finding（精确行号/级别/规则ID）
     +
reflect 工具 → 结构化证据（切片/数据流/展平）
     +
LLM → 洞察（安全分析/重复识别/一致性检查）
```

三层是**并行管道**，不是串行层级。证据和推理互补。

## Reflect 工具

### 程序切片（`backward_slice`）

给定一个 finding 位置，反向追溯所有影响该点的语句。

```
输入: 文件 + 行号
输出: 影响该行的定义链

示例: 从 L8 "total = price * qty" 追溯:
  L6 let price = ...     ← price 的定义
  L7 let qty = ...       ← qty 的定义
  L8 let total = ...     ← 目标行
```

### 数据流分析（`trace_variable`）

追踪单个变量从源头到使用点的完整路径。

```
输入: 文件 + 行号 + 变量名
输出: 赋值链路

示例: qty 的数据流
  parts[2] → qty
  trimmed.split(',') → parts
  raw.trim() → trimmed
```

### 展平语句（`flatten_stmts`）

将函数体内的所有可执行语句展开为线性列表，忽略嵌套结构。

```
输入: 函数节点
输出: 语句列表

示例 process_order:
  1. let trimmed = raw.trim()
  2. let parts = trimmed.split(',').collect()
  3. let name = parts[0].trim()
  4. let price = parts[1].trim().parse()...
  5. let qty = parts[2].trim().parse()...
  6. let total = price * qty as f64
  7. Ok(format!(...))
```

## 工具组合模式

### 模式 1：安全分析

```
backward_slice（从数组访问处追溯）+
dataflow（追踪索引变量）
  → LLM 发现：缺少长度检查，可能越界
```

验证结果：`parts[2]` 的行 → backward_slice 追溯 `parts` 来源 → dataflow 追踪 `qty` 源头 → LLM 发现 `parts.len()` 未检查。

### 模式 2：重复模式识别

```
flatten_stmts（展平所有语句）
  → LLM 发现：3 次 trim()、2 次 parse().map_err() 重复
```

验证结果：8 条展平语句 → LLM 识别出 price 解析和 qty 解析用了完全相同的 try-parse 模式。

### 模式 3：一致性检查

```
dataflow × N（追踪多个变量路径）
  → LLM 交叉对比：price 和 total 都依赖 parts[1]，错误传播范围过大
```

验证结果：4 个变量的 dataflow 路径 → LLM 发现 qty 和 price 的解析错误都会让 total 出错。

## 当前状态

| 工具 | 成熟度 | 测试 |
|------|--------|------|
| `backward_slice` | ✅ 稳定 | 已测试 |
| `flatten_stmts` | ✅ 稳定 | 已测试 |
| `trace_variable` | ⚠️ 基本路径工作 | 已测试 |
| `cross_function_slice` | ❌ 废弃 | 无测试 |
| LLM prompt 模板 | ❌ 未注册 | 无 |

## 下一步

- 将验证有效的 prompt 模板注册到 `llm.rs`
- 模板化：`analyse::security`、`analyse::consistency`、`analyse::duplicate`
