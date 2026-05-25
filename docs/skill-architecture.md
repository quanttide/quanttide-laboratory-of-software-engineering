# Skill 架构（设计记录）

## 定义

Skill = prompt + 元数据 + 工具调用指令，注册为 `.agents/skills/<name>/SKILL.md`。

每个已验证的工具组合模式对应一个 Skill。

## 目录结构

```
.agents/skills/<skill-name>/SKILL.md
  ├── frontmatter: name / description / disable-model-invocation
  └── body: prompt + 工具调用指令
```

全局级：`~/.agents/skills/`
项目级：`.quanttide/agents/skills/`

## 生命周期

```
加载（项目打开时）
  → 解析 frontmatter（name + description）
  → 存入 skill 列表
  → 选入 catalog（受上下文窗口限制）

调用（LLM 请求时）
  → 按 name 查找 skill
  → 读取 body
  → 运行工具 → 注入结果
  → 返回给 LLM
```

body 延迟读取，不占系统 prompt。每次调用实时解析。

## 候选 Skill

| Skill | 工具组合 | 输入 | 输出 |
|-------|----------|------|------|
| `analyse-security` | backward_slice + dataflow | 文件 + 行号 | 安全检查报告 |
| `analyse-duplicate` | flatten_stmts | 函数节点 | 重复模式报告 |
| `analyse-consistency` | dataflow × N | 多变量路径 | 一致性检查报告 |
| `analyse-boundary` | flatten_stmts | 过长函数 | 职责拆分建议 |
| `analyse-impact` | forward_slice + call_graph | 定义行 | 变更影响范围 |

## 参考实现

Zed Agent Skills 的接入方式：

```
加载:  load_skills_from_directory()
注册:  解析 frontmatter → 存入 skill 列表
调用:  skill_tool { name: "analyse-security" }
       → 按 name 查找
       → 读取 body
       → render_skill_envelope() → XML 包络体
       → 授权确认 → 返回给 LLM
```
