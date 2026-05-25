# 3R 架构使用指南

## 概述

3R 是三层递进的代码审查模型：**Review → Reflect → Refactor**。

```
review → reflect → refactor
  检测      理解      修复
```

每层可以在 CLI 中独立启用，下层不依赖上层。

```
qtcloud-code review <path>               # review（默认）
qtcloud-code review <path> --reflect     # review + reflect
qtcloud-code review <path> --refactor    # review + refactor
qtcloud-code review <path> --reflect --refactor  # 全部
```

---

## Review — 检测层

**规则引擎扫描，确定性，无 LLM。**

### 当前规则

| 规则 ID | 级别 | 说明 |
|---------|------|------|
| `long-function` | MAY/SHOULD/MUST | 函数超过 30/50/80 行 |
| `long-parameter-list` | MAY/SHOULD/MUST | 参数超过 4/6/9 个 |
| `rust-wide-unsafe` | MAY/SHOULD/MUST | unsafe 块超过 3/5/8 条 |
| `unused-variable` | SHOULD | 未使用变量（cargo check） |
| `missing-tests` | MUST | 源文件缺少对应测试 |

### 支持的语言

Rust / Python / Go / Dart / TypeScript (TSX)

### 命令

```sh
qtcloud-code review .
qtcloud-code review . --format json
qtcloud-code review . --rules long-function,missing-tests
qtcloud-code review . --status
```

---

## Reflect — 侦探层

**对 finding 做根因追溯。机械侦探（确定性）+ LLM 解释（可选）。**

### 程序切片

给定一个 finding 的位置，反向追溯所有影响该点的语句。

```
输入: finding (file.rs:L53, "函数过长")
过程: 从 L53 开始反向追溯变量定义链
输出: 证据链 (SliceEntry 列表)

示例:
  1. process:L8  let mut sum = 0;
  2. process:L11 let v = helper(*item);
  3. process:L12 sum += v;
```

**单函数切片**：在当前函数内追溯变量定义。**跨函数切片**：遇到函数调用时追溯被调用函数的 return。

### 数据流分析

追踪变量的完整赋值路径：从使用点追溯到源头。

```
输入: 变量名 + 使用行号
过程: 追溯 let 声明链
输出: FlowEntry 列表 (var → from → line)

示例:
  layout ← alloc::Layout::new::<T>()
  ptr    ← alloc(layout)
  value  ← input
```

### 依赖图分析

扫描项目源码中的 `mod`、`use`、`pub use` 声明，构建模块依赖图。

```
输入: 项目目录
过程: 扫描 src/**/*.rs → 提取 mod/use/pub use
输出: 依赖图 (节点 + 边)

反向切片: 哪些模块依赖了目标模块
正向切片: 目标模块影响了哪些下游

示例:
  api/handler.rs → service/processor.rs → data/pointer.rs
```

### 命令

```sh
qtcloud-code review . --reflect
```

---

## Refactor — 修复层

**代码变换。机械变换（确定性）+ LLM 策略选择（可选）。**

### 死代码检测

扫描函数定义和调用，找出未被调用的函数。

```
输入: 源码
过程: 收集所有函数 → 标记被调用的 → 输出未调用的
输出: DeadFunc (name, line)

注意事项:
  - 测试框架调用的函数会被标记为"死代码"（静态分析限制）
  - 特征方法会被标记为"死代码"（虚拟分派限制）
```

### 函数提取（实验性）

给定一个行号，识别可提取的最小完整语句。

### 符号表

扫描函数定义和调用点，构建定义→引用映射。用于重命名操作。

### 安全机制

| 功能 | 状态 |
|------|------|
| Patch 结构 | ✅ |
| dry-run（只显示 diff） | ✅ |
| apply 写文件 | ✅ |
| rollback 回滚 | ✅ |

### 命令

```sh
qtcloud-code review . --refactor
```

---

## 人机协作模型

```
AI   = 海量初级程序员（LLM + 规则引擎）
人类 = 高级程序员（定策略、审结果、做判断）
```

| 层 | AI 角色 | 人类角色 |
|----|---------|----------|
| review | 规则引擎批量扫描 | 配置规则、排除误报 |
| reflect | 切片/数据流/依赖图（机械）+ LLM 因果解释（可选） | 阅读证据链，判断优先级 |
| refactor | 死代码检测 + patch 生成（dry-run 默认） | 审核 patch、确认 apply |

## 模块结构

```
src/
├── main.rs              CLI 入口
├── lib.rs               模块声明
├── config.rs            配置加载 (.quanttide/code/contract.yaml)
├── lang/                语言解析器
│   └── {rust,python,go,dart,typescript}.rs
├── detect/              检测器
│   ├── long_function.rs
│   ├── long_parameter_list.rs
│   ├── unsafe_block.rs
│   ├── unused_variable.rs
│   └── missing_tests.rs
├── reflect/             侦探引擎
│   ├── mod.rs           EvidenceChain 类型
│   ├── slice.rs         程序切片（单函数/跨函数）
│   ├── dataflow.rs      数据流分析
│   └── depgraph.rs      依赖图分析
├── refactor/            变换引擎
│   ├── mod.rs
│   ├── transform.rs     死代码检测 + 边界识别
│   ├── rename.rs        符号表 + 重命名
│   └── safety.rs        Patch/dry-run/apply/rollback
└── report/              输出格式
    └── mod.rs           JSON / Terminal / STATUS.md
```
