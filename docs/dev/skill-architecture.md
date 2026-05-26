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

Skill 只预设**工具调用**（如何收集证据），不预设**分析维度**（看什么），让 LLM 自主发现。

| Skill | 工具集 | 输入 | 说明 |
|-------|--------|------|------|
| `analyse-line` | backward_slice + dataflow + forward_slice | 文件 + 行号 | 给证据，让 LLM 自己发现是安全问题还是重复还是别的 |
| `analyse-function` | flatten_stmts + call_graph + type_info | 函数名 | 同上 |

单一 skill，不按分析维度拆分。`parts[2]` 的越界 bug 证明了 LLM 在充足的证据下能自己发现问题，不需要预设「你要检查安全性」。

## 质量问题

reflect 的 D 级输出和 A 级输出在语气上几乎不可区分。LLM 说"职责不够单一"和"缺少长度检查会导致 panic"听起来一样自信，但前者空洞、后者精确。

防御机制：
1. **验证闭环** — reflect 的输出必须能转化为具体的 refactor 动作，否则应标记为低置信度
2. **证据锚定** — reflect 输出中引用行号/变量名的比例 > 50% 才算合格
3. **人类审核** — 允许用户 dismiss 空洞的 reflect 输出

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
