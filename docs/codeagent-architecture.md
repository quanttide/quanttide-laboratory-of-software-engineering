# CodeAgent：基于 Reflexion 的 3R 架构

## 核心循环

```
Review（检测）
  │
  ▼ finding
Reflect（理解）
  │
  ▼ 根因
Refactor（修复）
  │
  ▼ 修改后代码
Review（验证——重新检测）
```

对应 Reflexion (Shinn et al., 2023)：

| Reflexion | 3R CodeAgent | 工具 |
|-----------|-------------|------|
| **Act** | **Review** | 规则引擎（tree-sitter / cargo check）→ finding |
| **Observe** | finding | 行号、级别、规则 ID、代码片段 |
| **Reflect** | **Reflect** | backward_slice / dataflow / call_graph + LLM → 根因理解 |
| **Act with reflection** | **Refactor** | rename / 提取函数 / 安全机制 |
| （下一轮） | **Review（验证）** | 重新检测 → 确认修复有效或仍有问题 |

## 各阶段职责

### Review（检测）

**Act** — 规则引擎执行确定性扫描。

输入：源码
输出：`Vec<Finding>`（文件、行号、级别、规则ID、消息）
确定：✅ 可复现，无 LLM

### Reflect（理解）

**Observe + Reflect** — 机械工具提供结构化证据，LLM 做根因推理。

输入：finding + 代码
过程：
1. `backward_slice` 追溯变量定义链
2. `dataflow` 追踪值路径
3. `flatten_stmts` 展平函数体
4. `call_graph` 分析调用关系
5. LLM 结合证据做根因分析
输出：根因描述 + 修复建议
确定：⚠️ 机械部分可复现，LLM 部分不可复现

### Refactor（修复）

**Act with reflection** — 基于理解执行代码变换。

输入：根因 + 修复建议
过程：
1. LLM 生成 target code（新函数名、新逻辑）
2. 机械变换执行（rename / 提取函数 / 替换文本）
3. 安全机制：`--dry-run` → `--apply` → 自动验证
输出：patch
确定：⚠️ 机械变换可复现，代码生成不可复现

### Review（验证）

**下一轮 Act** — 重新运行检测确认修复。

输入：修改后的代码
输出：新的 finding 列表 → 与上一轮对比
确定：✅ 可复现

## 核心设计原则

1. **Review 是安全网** — 无论 LLM 是否参与，review 层必须能独立运行
2. **Reflect 必须绑定 Finding/Evidence** — 没有精确行号、变量名、数据流路径的 reflect 输出是无效的。跨文件/跨模块推理会退化为空洞的架构建议
3. **Reflect 工具是证据链** — backward_slice 等不依赖 LLM，但 LLM 消费它们的输出
4. **Refactor 需要人类审核** — `--dry-run` 默认，`--apply` 确认
5. **Review 闭环** — 任何修改后都要重新 review 验证

## 与 Skill 架构的关系

每个阶段可封装为 Skill：

```
.agents/skills/code-review/SKILL.md         → Review
.agents/skills/code-reflect/SKILL.md        → Reflect
.agents/skills/code-refactor/SKILL.md       → Refactor
.agents/skills/code-verify/SKILL.md         → Review（验证）
```

## 当前状态

| 阶段 | 成熟度 | 说明 |
|------|--------|------|
| Review | ✅ 已发布 | v0.2.0，89 测试 |
| Reflect 工具 | ✅ 实验验证 | backward_slice / dataflow / call_graph 等 |
| Reflect + LLM | ⚠️ 模式验证 | 3 种组合模式验证通过 |
| Refactor rename | ✅ 可用 | --dry-run / --apply |
| Refactor 提取函数 | ❌ 未实现 | 依赖 LLM 生成代码 |
| Review 验证闭环 | ❌ 未实现 | 需要自动对比前后 finding |
