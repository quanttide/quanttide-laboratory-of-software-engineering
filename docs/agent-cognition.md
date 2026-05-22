# 代码重构智能体认知架构

## 核心问题

如何让一个 AI 智能体"懂"重构，像资深工程师一样思考？

答案不在算法里，在**知识结构**里。当前模块定义的是智能体的**认知框架**——它如何感知代码、如何分类问题、如何决策行动。

---

## 智能体的五元认知模型

```python
# 这是智能体的"认知骨架"，不是数据存储
RefactorMethod   # 知道"怎么做"——可执行的操作库
CodeSmell        # 知道"看什么"——对代码的异常感知
RefactorGoal     # 知道"为什么"——质量价值的判断标准
RefactorProcess  # 知道"先什么后什么"——行动的时间逻辑
SafetyNet        # 知道"什么不能做"——安全的边界约束
```

一个人类重构专家头脑中同时运作这 5 套知识系统，智能体也需要同样结构的认知能力。

---

## 智能体的推理链

```
感知层      看到一段代码 → 激活 CodeSmell 知识
              "这个函数 120 行 → long-function"

诊断层      激活的 CodeSmell 链接到 RefactorMethod
              "long-function → extract-function"
              "condition: 提取后的函数名能清晰表达意图"

规划层      将多个 RefactorMethod 组织为 RefactorProcess
              "先提取函数 → 再重命名变量 → 运行测试"

价值判断    用 RefactorGoal 衡量是否值得做
              "可读性提升 vs 引入风险"

安全制约    用 SafetyNet 约束行动边界
              "测试覆盖 ≥80% 才能执行"
```

这不是流水线，而是智能体**瞬时的认知活动**——感知、联想、判断、规划、约束同时发生。

---

## 知识即推理

传统工具把知识当"配置"，智能体把知识当"推理路径"：

```
CodeSmell.counter_methods
    ↓
智能体看见坏味道时，自动激活对应的手法候选项
    ↓
这不是查表，是"联想"——一个概念激活相关联的概念
    ↓
RefactorMethod.condition
    ↓
智能体评估候选项的适用性——"条件满足吗？"
    ↓
RefactorProcess.steps
    ↓
智能体将操作展开为时间序列——"先做什么，再做什么"
```

每个字段都是智能体推理链上的一个节点。

---

## 认知能力进化路径

### 当前（知识结构化）

智能体拥有专家的知识结构，但知识是静态的。

### 下一阶段（感知自动化）

让智能体能在真实代码中**主动识别**坏味道：

```python
# 智能体的"眼睛"
class AgentPerception:
    """
    接收原始代码，输出激活的 CodeSmell 实例。
    这不是简单的规则匹配，而是基于 pattern recognition 的感知。
    """
    def scan(self, code: str) -> list[ActivatedSmell]:
        # 激活哪些 CodeSmell？严重程度？位置在哪？
        ...
```

### 再下一阶段（判断自动化）

让智能体能自主**评估和决策**：

```python
class AgentJudgment:
    """
    根据激活的坏味道，选择手法，编排顺序。
    这不是 if-else，而是基于目标和约束的推理。
    """
    def decide(self, smells: list[ActivatedSmell], goals: list[str]) -> Plan:
        # trade-off: 多个坏味道先处理哪个？
        # conflict: 两个手法是否冲突？
        # risk: 这个变换安全吗？
        ...
```

### 最终阶段（反思与学习）

智能体从执行结果中更新自己的知识：

```python
class AgentReflection:
    """
    执行后评估效果，更新内部知识。
    """
    def reflect(self, plan: Plan, result: ExecutionResult):
        # 这个手法在这个场景下效果好吗？
        # 更新 condition？
        # 发现新的坏味道 pattern？
        ...
```

---

## 关键设计原则

1. **认知先于功能** — 先定义智能体"知道什么"，再定义它"能做什么"。功能是认知的外化。

2. **知识即推理媒介** — 每个字段不仅是存储，更是推理链条上的节点。设计字段时思考"智能体会怎样用它推理"。

3. **自省能力** — 智能体能"讲述"自己的推理过程（因为推理路径就是知识链接路径）：
   - "我在第42行检测到了过长函数"
   - "我选择提炼函数，因为该段代码可以被命名为 calculate_discount"
   - "我选择先做提取再做重命名，因为重命名依赖于提取后的代码"

4. **渐进复杂化** — 从静态知识 → 感知 → 判断 → 反思，逐步赋予智能体更完整的认知闭环。
