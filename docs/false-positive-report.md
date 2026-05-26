# 假阳性过滤实验报告

## 统计

| 指标 | 值 |
|------|---|
| 总 finding 数 | 7 |
| LLM 确认 (CONFIRM) | 6 |
| LLM 驳回 (DISMISS) | 1 |
| 驳回率 | 14.3% |

## 驳回详情

- `/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli/src/refactor/mod.rs:1` [missing-tests] 该 finding 指出 `src/refactor/mod.rs` 缺少对应测试，但代码仅包含一行 `pub mod rename;`，即模块声明。模块本身没有可测试的逻辑，测试应针对 `rename` 模块内部的具体函数或功能，而非模块声明文件。因此，该 finding 不是真问题。
