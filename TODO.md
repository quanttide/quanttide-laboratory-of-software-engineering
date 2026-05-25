# TODO — qtcloud-code-cli

## P0 已完成 ✅

- [x] CLI 命令框架（`scan` / `list-rules`）
- [x] tree-sitter-rust 集成，Rust 代码解析
- [x] 递归扫描目录下所有 `.rs` 文件
- [x] 检测器：过长函数（>30 行）
- [x] 检测器：宽泛 unsafe 块（>5 条语句）
- [x] 输出格式：JSON（`--format json`）和终端文本
- [x] 错误处理：解析失败、权限拒绝时跳过
- [x] 集成测试：使用 `examples/default` 作为 fixture

## 高优

- [ ] 检测器：未使用变量（需要语义分析 / 走 rustc 路径）
- [ ] file_path 输出改为绝对路径或相对 CWD
- [ ] 多语言：`LanguageParser` trait 提取后，加 Python 支持

## 中优

- [ ] tree-sitter-go 绑定
- [ ] tree-sitter-dart 绑定
- [ ] tree-sitter-typescript 绑定
- [ ] 通用检测器：过长参数列表
- [ ] `--rules` 选择启用的检测器
- [ ] `.qtcloud-code.toml` 配置文件

## 低优

- [ ] CI 集成 GitHub Action
- [ ] pre-commit hook 支持
- [ ] 增量扫描（git diff 范围）
- [ ] 基线模式（`--baseline baseline.json`）
- [ ] 多线程并行扫描
