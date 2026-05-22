# AI 集成指南

CodeAgent 专做**机械性重构**（Review→Reflect→Refactor 循环），AI 编程助手（如 OpenCode）擅长**语义理解**。两者组合可以形成"生成/修改 → 自动清洗 → AI 复核"的闭环，比各自单独用更可靠。

## 为什么需要 AI

CodeAgent 的 Review 只检查语法（`py_compile`）和静态坏味道，不保证重构后的代码逻辑正确。例如提取函数时，如果新函数引用了外层作用域的变量，工具不会报错——编译仍然通过，但语义可能变了。AI 可以理解代码意图，发现这类问题。

---

<!-- ==================== 面向人类用户 ==================== -->

## 配合工作流

### 场景 1：生成代码后自动清洗

1. 让 AI 生成一段 Python 代码（如"写一个订单处理函数"）
2. AI 自动调用工具扫描生成的代码
3. 发现坏味道 → 工具执行修复 → AI 将修复后的代码展示给你
4. 你只需 review 最终结果

### 场景 2：交互式重构复核

1. 你对 AI 说："用重构工具扫描并修复这个文件"
2. AI 执行工具，得到修复结果
3. AI 读取改动前后的差异，检查语义是否被破坏（变量作用域、闭包引用等）
4. 对可疑变化，AI 提醒你确认，或自行进一步调整

### 场景 3：大范围重构的安全网

1. AI 做大规模代码迁移（升级框架、替换库）
2. 每次修改后自动运行工具检测
3. 如果检测到新增坏味道，AI 决定是回滚修改，还是接受并让工具修复
4. 形成"AI 修改 → 工具检查 → 工具修复 → 再次检查"的自动纠偏环

---

<!-- ==================== 面向 AI Agent 开发者 ==================== -->

## 集成方式

### 方式一：OpenCode Slash Command（推荐）

在项目根目录创建 `.opencode/commands/smell-fix.md`：

```markdown
# code-cleanup

运行 CodeAgent 扫描当前文件/目录的 Python 坏味道并自动修复。

## 用法

`/code-cleanup [文件或目录路径]`

## 执行流程

1. 确定工具路径：根据实际安装位置调整。如果是 `examples/default` 结构，执行 `python src/main.py <路径>`；否则替换为实际的可执行路径
2. CodeAgent 自动执行 Review→Reflect→Refactor 循环
3. 解析 Agent 输出，提取以下信息：
   - 发现了几个坏味道，分别是什么类型、在什么位置
   - 哪些修复成功，哪些失败
   - 是否有增量坏味道（重构引入的新问题）
4. 对每个成功的修复，读取修改前后的差异（`git diff`）
5. 检查语义正确性：
   - 提取函数是否引用了外层变量
   - 重命名是否遗漏了引用
6. 向用户展示：
   ```
   ## 扫描结果：发现 2 个坏味道
   - long-function @ my_code.py:5-38 → extract-function ✅ 已修复
   - long-parameter-list @ my_code.py:40 → 跳过（无可用修复手法）
   
   ## 需注意
   提取的函数 `extracted_func` 访问了外部变量 `counter`，请确认逻辑正确。
   
   重构后引入了 1 个新坏味道：[long-function] L42-55，建议检视。
   ```
7. 如果用户不满意某个修复，提供选项让用户选择是否还原

## 验证

- 运行 `python -m py_compile <目标文件>` 确认语法合法
- 有条件时运行 `pytest` 确认测试通过
```

### 方式二：Pre-commit Hook

将 CodeAgent 注册为 git pre-commit hook，在提交代码前自动触发：

```yaml
# .pre-commit-config.yaml
-   repo: local
    hooks:
    -   id: code-cleanup
        name: CodeAgent cleanup
        entry: python src/main.py
        language: system
        files: \.py$
```

> **风险警告**：CodeAgent 没有 `--dry-run` 模式，注册为 pre-commit hook 后每次提交都会**直接修改文件**。建议先在单个文件上测试确认行为符合预期，再启用 hook。Agent 的无限循环保护会避免同一坏味道被反复修复，但不会阻止 hook 修改文件。

## 注意事项

- CodeAgent 的编译检查通过 ≠ 重构正确。请始终让 AI（或你自己）复核语义。
- CodeAgent 只改同一文件内的代码，跨文件重构需要 AI 额外处理。
- CodeAgent 有无限循环保护（成功和失败的修改都会被标记已尝试），但 AI 复核时仍需注意是否有反复修改同一段代码的风险。
- 未配置 LLM API Key 时 CodeAgent 正常运作（L1 规则模式），但 AI 复核需要依赖自身的语义理解能力。
