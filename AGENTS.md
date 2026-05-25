# AGENTS — qtcloud-code-cli（实验室）

## 架构

```
规则引擎 → finding（确定、可复现）
     +
reflect 工具 → 结构化证据（切片/数据流/展平语句）
     +
LLM → 洞察（安全分析、重复识别、一致性检查）
```

三层是**并行管道**，不是串行层级。

## 模块结构（实验室）

```
src/
├── lib.rs            模块入口
├── reflect/          确定性分析工具
│   ├── mod.rs        SliceEntry, FlowEntry 类型
│   ├── slice.rs      backward_slice, flatten_stmts, cross_function_slice
│   └── dataflow.rs   trace_variable
├── llm.rs            DeepSeek LLM 客户端（从 Vault 读取密钥）
└── bin/llm_exp.rs    reflect × LLM 组合实验
```

## 已验证的工具组合模式

### 模式 1：backward_slice + dataflow → 安全分析
从数组访问处追溯变量来源 + dataflow → LLM 发现越界风险。

### 模式 2：flatten_stmts → 重复模式识别
展平语句 → LLM 识别重复的解析/验证模式。

### 模式 3：dataflow × N → 一致性检查
追踪多个变量路径 → LLM 交叉对比发现错误传播问题。

## LLM 集成

- 客户端：`llm.rs`，使用 `reqwest` 调用 DeepSeek API
- 密钥：从 Vault 读取（`VAULT_TOKEN` 环境变量），路径 `secret/data/deepseek`，字段 `apiKey`
- 当前为实验阶段，prompt 模板未注册

## 发现分级

| 级别 | 含义 |
|------|------|
| **MUST** | 可能引入 bug |
| **SHOULD** | 维护负担 |
| **MAY** | 风格建议 |

## 测试

```sh
cargo test        # 11 个单元测试
cargo llvm-cov    # 覆盖率
```

## 跨语言检测注意事项

不同语言 tree-sitter 节点结构差异大（当前实验仅验证 Rust）：
- **Rust** `function_item` → `parameters` → `parameter`
- **Python** `function_definition` → `parameters`
- **Go** `function_declaration` → `parameters` → `parameter_declaration`
- **Dart** `function_declaration` → `function_signature` → `identifier`
- **TypeScript** 同 Go/Dart
