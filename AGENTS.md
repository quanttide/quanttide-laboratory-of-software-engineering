# AGENTS — qtcloud-code-cli

## 架构原则

- 纯规则引擎，不依赖 LLM
- 面向「可检测、可复现、可自动化」
- P0 阶段只做 Rust，再扩展多语言
- 做检测，不做自动修复

## 测试

```sh
cargo test
```

## 模块结构

```
src/
├── main.rs        # CLI 入口 (clap)
├── lib.rs         # 公开模块
├── lang/          # 语言解析器抽象与实现
├── detect/        # 检测器
└── report/        # 输出格式
```
