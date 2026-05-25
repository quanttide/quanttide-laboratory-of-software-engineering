# Skill 架构（设计记录）

## 区别：Skill 不是系统提示词

| | 系统提示词 | Skill |
|--|-----------|-------|
| 加载时机 | 会话启动时 | 调用时 |
| 上下文窗口 | 始终占用 | 不占用，直到被调用 |
| 见性 | LLM 始终可见 | 通过 catalog 可见，但 inactive |
| 触发方式 | 自动 | 按需（LLM 或用户） |
| 内容 | 稳定的元规则（角色、行为准则、如何使用 skill） | 领域指令 + 工具调用 + 动态结果 |

Skill 是**按需加载到用户提示层的工具结果**，不是预加载到系统提示层的指令。

## 定义

Skill = prompt + 工具调用指令 + 元数据，注册为 `.agents/skills/<name>/SKILL.md`。

每个已验证的工具组合模式对应一个 Skill。

## 目录结构

```
.agents/skills/<skill-name>/SKILL.md
  ├── frontmatter: name / description / disable-model-invocation
  └── body: 工具调用指令 + prompt
```

全局级：`~/.agents/skills/`
项目级：`.quanttide/agents/skills/`

## 生命周期

```
加载（项目打开时）
  → 解析 frontmatter（name + description）
  → 注册到 skill 列表
  → 选入 catalog（摘要信息给 LLM）
  → 不加载 body，不占上下文

调用（LLM 请求时）
  → LLM 从 catalog 中选中 skill
  → skill_tool { name: "analyse-security" }
  → 按 name 查找
  → 读取 body
  → 运行工具（backward_slice + dataflow）
  → 注入工具结果到 body
  → 返回给 LLM（作为工具调用结果）
  → 进入用户提示层
```

## 候选 Skill

| Skill | 工具组合 | 输入 | 输出 |
|-------|----------|------|------|
| `analyse-security` | backward_slice + dataflow | 文件 + 行号 | 安全检查报告 |
| `analyse-duplicate` | flatten_stmts | 函数节点 | 重复模式报告 |
| `analyse-consistency` | dataflow × N | 多变量路径 | 一致性检查报告 |
| `analyse-boundary` | flatten_stmts | 过长函数 | 职责拆分建议 |
| `analyse-impact` | forward_slice + call_graph | 定义行 | 变更影响范围 |

## 参考实现

Zed Agent Skills：

```
加载:  load_skills_from_directory()
注册:  解析 frontmatter → 存入 skill 列表
调用:  skill_tool { name: "analyse-security" }
       → 按 name 查找
       → 读取 body
       → render_skill_envelope() → XML 包络体
       → 授权确认 → 返回给 LLM
```
