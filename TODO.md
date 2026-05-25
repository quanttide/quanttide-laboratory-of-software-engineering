# TODO — qtcloud-code-cli

## 高优

- [ ] tree-sitter 语言解析器集成（`tree-sitter-rust` 绑定）
- [ ] `qtcloud-code scan` 递归扫描目录下所有 `.rs` 文件
- [ ] Rust 检测器 1：未使用变量检测
- [ ] Rust 检测器 2：过于宽泛的 `unsafe` 块检测
- [ ] 输出格式：JSON（`--format json`）和终端文本
- [ ] 错误处理：非 UTF-8 文件、解析失败、权限拒绝
- [ ] 集成测试：扫描已知的坏代码样本

## 中优

- [ ] `LanguageParser` trait 提取，支持多语言
- [ ] tree-sitter-python 绑定
- [ ] tree-sitter-go 绑定
- [ ] tree-sitter-dart 绑定
- [ ] 通用检测器：过长函数、过长参数列表
- [ ] `--rules` 选择启用的检测器
- [ ] `.qtcloud-code.toml` 配置文件

## 低优

- [ ] CI 集成 GitHub Action
- [ ] pre-commit hook 支持
- [ ] 增量扫描（git diff 范围）
- [ ] 基线模式（`--baseline baseline.json`）
- [ ] 多线程并行扫描
