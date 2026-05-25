# ROADMAP — qtcloud-code-cli（实验室）

## 定位

3R 代码审查 CLI：**review → reflect → refactor**。

- **review**: 规则引擎扫描（确定性，无 LLM）
- **reflect**: 程序切片 + 数据流 + 依赖图分析（确定性）→ LLM 因果解释（可选）
- **refactor**: AST 机械变换（确定性）→ LLM 策略选择（可选）

## P0 — 已实现 ✅

- [x] CLI 框架（`review` / `list-rules`）
- [x] tree-sitter 多语言集成（Rust / Python / Go / Dart / TypeScript）
- [x] 检测器：过长函数、unsafe 块、过长参数列表、未使用变量、缺失测试
- [x] 输出格式：终端 / JSON / STATUS.md
- [x] 配置系统：`.quanttide/code/contract.yaml` + `--rules`
- [x] 自举验证 + 77 测试 + 95% 覆盖率
- [x] 发布流水线：`cli/v*` tag → crates.io

## P1 — 架构升级

### review 增强
- [ ] 文件级忽略（行注释 / 配置 exclude）
- [ ] `--reflect` 标志（机械侦探：切片 + 数据流 + 依赖图）
- [ ] `--llm` 标志（LLM 二次审查 + 因果解释）
- [ ] 增量扫描（git diff 范围）

### contract 命令
- [ ] `contract init` — 交互式创建配置
- [ ] `contract list` — 替换 list-rules（JSON 输出）
- [ ] `contract validate` — 校验配置 vs 已知规则

## P2 — LLM 集成

- [ ] LLM 二次审查：排序、去重、语义规则
- [ ] LLM 因果解释：在证据链上做根因分析
- [ ] 纯 LLM 规则：安全漏洞、并发 bug、逻辑错误
- [ ] `--mode lint / llm / deep` 三种模式

## P3 — refactor

- [ ] 机械变换引擎：函数提取、重命名、内联、死代码删除
- [ ] dry-run / apply / 自动验证 / 回滚
- [ ] LLM 策略选择：复杂场景的代码生成

## 非目标

- 不做纯 LLM 的代码审查（规则引擎是安全网，必须优先运行）
- 不做自动修复（--apply 需要人类确认，默认 dry-run）
- 不承诺可复现性在 LLM 介入后完全不变（证据链部分确定，解释部分不确定）
