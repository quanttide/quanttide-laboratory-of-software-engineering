# 代码重构智能体认知架构

## 核心问题

如何让一个 AI 智能体"懂"重构，像资深工程师一样思考？

答案不在算法里，在**知识结构**里。当前模块定义的是智能体的**认知框架**——它如何感知代码、如何分类问题、如何决策行动。

---

## 智能体的六元认知模型

```python
# 这是智能体的"认知骨架"，不是数据存储
RefactorMethod    # 知道"怎么做"——可执行的操作库
CodeSmell         # 知道"看什么"——对代码的异常感知
Correspondence    # 知道"什么连到什么"——问题与方案之间的映射
RefactorGoal      # 知道"为什么"——质量价值的判断标准
RefactorProcess   # 知道"先什么后什么"——行动的时间逻辑
SafetyNet         # 知道"什么不能做"——安全的边界约束
```

一个人类重构专家头脑中同时运作这 6 套知识系统，智能体也需要同样结构的认知能力。

---

## 认知模型详解（结合实例）

### 1. RefactorMethod — 操作知识

智能体知道"有哪些变换操作可用"，每个操作包含完整的执行知识：

```python
RefactorMethod(
    id="extract-function",        # 操作的唯一标识
    name="extract-function",
    label="提炼函数",              # 自然语言名称
    motivation="函数过长或包含需要注释才能理解的代码段",  # 何时触发
    steps=[                       # 执行步骤——智能体将 steps 展开为子任务序列
        "创建新函数",
        "复制代码",
        "处理变量作用域",
        "替换原处为调用",
    ],
    condition="提取后的函数名能清晰表达代码意图",  # 前提条件——智能体执行前必须验证
)
```

当前知识库包含 5 种操作：`extract-function`、`rename-variable`、`extract-class`、`move-function`、`replace-conditional-with-polymorphism`。

### 2. CodeSmell — 感知知识

智能体知道"代码中哪些模式需要警惕"，每种坏味道包含感知特征和关联方案：

```python
CodeSmell(
    id="long-function",
    name="long-function",
    label="过长函数",
    symptom="函数过长，难以理解、测试和维护",                  # 主观感受
    characteristic="函数超过一屏、包含多个层级的缩进、需要滚动才能看完",  # 客观特征——智能体的检测依据
    counter_methods=["extract-function", "replace-temp-with-query"],  # 候选方案——激活联想
)
```

当前知识库包含 5 种坏味道：`long-function`、`duplicate-code`、`large-class`、`long-parameter-list`、`shotgun-surgery`。

### 3. Correspondence — 映射知识

这是最关键的认知升级。智能体将"问题→方案"的映射关系从 CodeSmell 中分离为独立的认知单元，使映射本身成为可推理、可扩展的一等公民：

```python
Correspondence(
    id="long-function-to-extract-function",
    name="long-function-to-extract-function",
    label="从过长函数到提炼函数",
    source="long-function",       # 源：问题
    target="extract-function",    # 目标：方案
)
```

| 源（问题） | 目标（方案） |
|-----------|-------------|
| `long-function` | `extract-function` |
| `duplicate-code` | `extract-function` |
| `large-class` | `extract-class` |
| `long-parameter-list` | `preserve-whole-object` |
| `shotgun-surgery` | `rename-variable` |

将映射独立为 Correspondence 而非嵌入 CodeSmell.counter_methods，意味着智能体可以：
- **多对多映射** — 一个手法解决多种坏味道，一种坏味道有多种手法
- **映射可演化** — 新增映射无需修改原有模型
- **映射本身可推理** — 智能体可以问"这个映射合理吗？""有没有更好的映射？"

### 4. RefactorGoal — 价值知识

智能体知道"为什么而重构"，用可衡量的标准判断行动价值：

```python
RefactorGoal(
    id="readability",
    name="readability",
    label="可读性",
    criterion="新成员阅读代码后能在30分钟内理解其逻辑",  # 衡量标准——智能体评估效果的依据
    counter_methods=["rename-variable", "extract-function", "introduce-explaining-variable", "decompose-conditional"],
)
```

价值知识让智能体在多个候选方案中做权衡：是要可读性还是可维护性？

### 5. RefactorProcess — 时序知识

智能体知道"事情该按什么顺序做"，每个流程定义输入→步骤→输出的完整闭环：

```python
RefactorProcess(
    id="identify-smell",
    name="identify-smell",
    label="识别坏味道",
    steps=["阅读代码", "标记可疑结构", "确认坏味道类型"],
    inputs="源代码",
    outputs="坏味道清单（每个条目含位置、类型、严重程度）",
)
```

三个流程串联为完整的认知闭环：`identify-smell → choose-technique → execute-and-verify`。

### 6. SafetyNet — 边界知识

智能体知道"什么绝对不能做"，在行动边界内保障安全：

```python
SafetyNet(
    id="unit-test-coverage",
    name="unit-test-coverage",
    label="单元测试覆盖",
    requirement="重构涉及的代码路径≥80%覆盖率",
    tools=["pytest", "JUnit", "vitest"],
)
```

---

## 智能体的推理链

### 从感知到行动的完整路径

```
感知        看到一段 120 行的函数
              → 匹配 CodeSmell("long-function").characteristic
              → "函数超过一屏 → 这是过长函数"

联想        long-function 激活 Correspondence
              → "long-function → extract-function"
              → 也激活了 CodeSmell.counter_methods
              → "还可尝试 replace-temp-with-query"

评估        检查 RefactorMethod("extract-function").condition
              → "提取后的函数名能清晰表达代码意图"
              → 这段代码可以命名为 calculate_total() → 条件满足

规划        将 extract-function 展开为 RefactorMethod.steps
              → "创建新函数 → 复制代码 → 处理变量作用域 → 替换原处为调用"
              → 按 RefactorProcess("execute-and-verify") 组织执行

价值判断    用 RefactorGoal("readability") 衡量
              → "提炼后代码更短、意图更清晰 → 可读性提升"

安全约束    用 SafetyNet 检查前提
              → "测试覆盖率 ≥80%? → 通过"
              → "有 CI 流水线 → 可通过"
```

### 链式推理的具体实例

```python
# 智能体内部的认知活动，不是代码执行，而是概念激活的路径
perception:  CodeSmell("long-function")
               ↓
association: Correspondence("long-function-to-extract-function")
               ↓
             CodeSmell("long-function").counter_methods
               → ["extract-function", "replace-temp-with-query"]
               ↓
evaluation:  RefactorMethod("extract-function").condition
               → "提取后的函数名能清晰表达代码意图" → True
               ↓
             RefactorMethod("extract-function").motivation
               → "函数过长或包含需要注释才能理解的代码段" → 匹配
               ↓
execution:   RefactorMethod("extract-function").steps
               → ["创建新函数", "复制代码", "处理变量作用域", "替换原处为调用"]
               ↓
verification: RefactorGoal("readability").criterion
               → "新成员阅读代码后能在30分钟内理解其逻辑"
```

这不是流水线，而是智能体**瞬时的认知活动**——多种知识同时激活、相互验证、形成决策。

---

## 知识即推理

传统工具把知识当"配置"，智能体把知识当"推理路径"：

```
Correspondence.source
    ↓
智能体看到问题，自动映射到解决方案
    ↓
这不是查表，是"联想"——一个概念激活相关联的概念
    ↓
RefactorMethod.condition
    ↓
智能体评估候选项的适用性——"条件满足吗？"
    ↓
RefactorMethod.steps
    ↓
智能体将操作展开为时间序列——"先做什么，再做什么"
    ↓
RefactorGoal.criterion
    ↓
智能体判断是否值得做——"达到目标了吗？"
```

每个字段都是智能体推理链上的一个节点。

---

## 认知能力进化路径

### 当前（知识结构化）

智能体拥有专家的知识结构（6 类模型 + 具体实例），但知识是静态的。智能体需要人工读取这些数据并据此推理。

### 下一阶段（感知自动化）

让智能体能在真实代码中**主动识别**坏味道：

```python
class AgentPerception:
    """
    接收原始代码，输出激活的 CodeSmell 实例。
    感知依据是 CodeSmell.characteristic——智能体用这个字段训练或配置其检测能力。
    """
    def scan(self, code: str) -> list[ActivatedSmell]:
        # 对每个 CodeSmell，检查代码是否匹配其 characteristic
        # e.g. long-function: 函数行数 > 屏幕高度
        # e.g. long-parameter-list: 参数个数 > 5
        for smell in code_smells:
            if matches(code, smell.characteristic):
                activated.append(ActivatedSmell(smell_id=smell.id, location=...))
```

### 再下一阶段（判断自动化）

让智能体能自主**评估和决策**：

```python
class AgentJudgment:
    """
    根据激活的坏味道，通过 Correspondence 映射到手法，再按 RefactorGoal 排序。
    """
    def decide(self, smells: list[ActivatedSmell], goals: list[str]) -> Plan:
        # 1. 通过 Correspondence 将 smells 映射到 methods
        # 2. 检查每个 method 的 condition 是否满足
        # 3. 用 RefactorGoal 对方案排序（优先级）
        # 4. 按 RefactorProcess 编排执行顺序
        for smell in smells:
            for corr in correspondences:
                if corr.source == smell.smell_id:
                    method = find_method(corr.target)
                    if evaluate(method.condition):
                        plan.add_step(method)
```

### 最终阶段（反思与学习）

智能体从执行结果中更新自己的知识——不仅是更新实例数据，甚至能发现新的坏味道模式、建立新的 Correspondence：

```python
class AgentReflection:
    """
    执行后评估效果，更新内部知识。
    """
    def reflect(self, plan: Plan, result: ExecutionResult):
        # 这个手法在这个场景下效果好吗？
        # 是否应该更新 Correspondence（新的问题→方案映射）？
        # 是否发现了新的 CodeSmell pattern？
        # RefactorMethod.condition 是否过于宽松/严格？
        ...
```

---

## 关键设计原则

1. **认知先于功能** — 先定义智能体"知道什么"，再定义它"能做什么"。功能是认知的外化。

2. **知识即推理媒介** — 每个字段不仅是存储，更是推理链条上的节点。设计字段时思考"智能体会怎样用它推理"。例如：
   - `CodeSmell.characteristic` = 智能体的检测模式
   - `Correspondence.source/target` = 智能体的联想路径
   - `RefactorMethod.condition` = 智能体的前置验证
   - `RefactorGoal.criterion` = 智能体的效果评估

3. **映射独立化** — Correspondence 将"问题→方案"提升为一等公民，使映射本身可被智能体推理和演化，而非嵌入 CodeSmell 内部的固定列表。

4. **自省能力** — 智能体能"讲述"自己的推理过程（因为推理路径就是知识链接路径）：
   - "我在第42行检测到了过长函数（CodeSmell）"
   - "通过 Correspondence 映射到 extract-function"
   - "我选择提炼函数，因为该段代码可以被命名为 calculate_discount（condition 满足）"
   - "我选择先做提取再做重命名，因为重命名依赖于提取后的代码（RefactorProcess）"

5. **渐进复杂化** — 从静态知识 → 感知 → 判断 → 反思，逐步赋予智能体更完整的认知闭环。
