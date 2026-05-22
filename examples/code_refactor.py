"""代码重构知识库 — 将 knowledge 翻译为可编程的类型和实例。

用法：
    from docs.gallery.wisdom.code_refactor import (
        RefactorMethod, CodeSmell, RefactorGoal,
        refactoring_techniques, code_smells,
    )
"""

from pydantic import BaseModel, Field


class RefactorMethod(BaseModel):
    """重构手法 — 具体的代码变换操作，每种手法有明确的动机、步骤、效果和适用条件。"""
    id: str
    name: str
    label: str
    motivation: str = Field(description="为什么需要这个手法")
    steps: list[str] = Field(description="执行步骤序列")
    condition: str = Field(description="适用条件")


class CodeSmell(BaseModel):
    """代码坏味道 — 代码中表明存在更深层问题的表面症状。"""
    id: str
    name: str
    label: str
    symptom: str = Field(description="症状表现")
    characteristic: str = Field(description="识别特征")
    counter_methods: list[str] = Field(description="对应的重构手法")


class RefactorGoal(BaseModel):
    """重构目标 — 重构希望达成的质量维度。"""
    id: str
    name: str
    label: str
    criterion: str = Field(description="衡量标准")
    counter_methods: list[str] = Field(description="对应手法")


class RefactorProcess(BaseModel):
    """重构流程 — 重构的操作步骤序列。"""
    id: str
    name: str
    label: str
    steps: list[str] = Field(description="流程步骤序列")
    inputs: str = Field(description="输入")
    outputs: str = Field(description="输出")


class SafetyNet(BaseModel):
    """重构安全网 — 确保重构不改变外部行为的保障机制。"""
    id: str
    name: str
    label: str
    requirement: str = Field(description="要求")
    tools: list[str] = Field(description="工具/技术")


refactoring_techniques = [
    RefactorMethod(
        id="extract-function",
        name="extract-function",
        label="提炼函数",
        motivation="函数过长或包含需要注释才能理解的代码段",
        steps=["创建新函数", "复制代码", "处理变量作用域", "替换原处为调用"],
        condition="提取后的函数名能清晰表达代码意图",
    ),
    RefactorMethod(
        id="rename-variable",
        name="rename-variable",
        label="重命名变量",
        motivation="变量名模糊、缩写或误导",
        steps=["搜索所有引用", "逐个替换", "运行测试"],
        condition="名称反映用途而非类型",
    ),
    RefactorMethod(
        id="extract-class",
        name="extract-class",
        label="提炼类",
        motivation="类承担了多个不相关的职责（单一职责原则违反）",
        steps=["创建新类", "在新类中声明字段和方法", "建立原类到新类的引用", "编译测试"],
        condition="拆分后的每个类都能用一句话描述其职责",
    ),
    RefactorMethod(
        id="move-function",
        name="move-function",
        label="搬移函数",
        motivation="函数使用了另一个类/模块的比自身更多的上下文",
        steps=["检查源和目标上下文", "复制函数", "建立引用", "删除原函数"],
        condition="移动后函数的内聚性提高",
    ),
    RefactorMethod(
        id="replace-conditional-with-polymorphism",
        name="replace-conditional-with-polymorphism",
        label="以多态取代条件表达式",
        motivation="多个条件分支根据对象类型执行不同行为",
        steps=["为每种类型创建子类", "将条件分支逻辑移到子类方法", "删除原条件表达式"],
        condition="类型变化频率高于行为变化频率",
    ),
]

code_smells = [
    CodeSmell(
        id="long-function",
        name="long-function",
        label="过长函数",
        symptom="函数过长，难以理解、测试和维护",
        characteristic="函数超过一屏、包含多个层级的缩进、需要滚动才能看完",
        counter_methods=["extract-function", "replace-temp-with-query"],
    ),
    CodeSmell(
        id="duplicate-code",
        name="duplicate-code",
        label="重复代码",
        symptom="相同的代码结构在多个位置出现",
        characteristic="复制粘贴的代码块、仅在细节上不同的相似结构",
        counter_methods=["extract-function", "move-statements", "extract-superclass"],
    ),
    CodeSmell(
        id="large-class",
        name="large-class",
        label="过大类",
        symptom="类承担了过多职责，字段和方法数量过大",
        characteristic="类字段超过10个、方法超过20个、难以用一句话描述类的职责",
        counter_methods=["extract-class", "extract-subclass", "replace-with-observer"],
    ),
    CodeSmell(
        id="long-parameter-list",
        name="long-parameter-list",
        label="过长参数列表",
        symptom="函数的参数过多（超过3-4个），调用和维护困难",
        characteristic="参数超过5个、存在相邻参数经常同时变化",
        counter_methods=["preserve-whole-object", "replace-parameter-with-query", "introduce-parameter-object"],
    ),
    CodeSmell(
        id="shotgun-surgery",
        name="shotgun-surgery",
        label="霰弹式修改",
        symptom="一个改动导致多个类都需要修改",
        characteristic="修改一个功能需要在3个以上的文件中做改动、忘记修改某个文件会导致bug",
        counter_methods=["move-function", "inline-class", "inline-function"],
    ),
]

refactor_goals = [
    RefactorGoal(
        id="readability", name="readability", label="可读性",
        criterion="新成员阅读代码后能在30分钟内理解其逻辑",
        counter_methods=["rename-variable", "extract-function", "introduce-explaining-variable", "decompose-conditional"],
    ),
    RefactorGoal(
        id="maintainability", name="maintainability", label="可维护性",
        criterion="功能变更涉及的文件数≤3",
        counter_methods=["extract-class", "move-function", "modularize"],
    ),
    RefactorGoal(
        id="testability", name="testability", label="可测试性",
        criterion="函数无副作用或有明确的副作用边界、依赖可注入",
        counter_methods=["dependency-injection", "extract-interface", "functional-refactoring"],
    ),
]

refactor_processes = [
    RefactorProcess(
        id="identify-smell", name="identify-smell", label="识别坏味道",
        steps=["阅读代码", "标记可疑结构", "确认坏味道类型"],
        inputs="源代码",
        outputs="坏味道清单（每个条目含位置、类型、严重程度）",
    ),
    RefactorProcess(
        id="choose-technique", name="choose-technique", label="选择重构手法",
        steps=["根据坏味道类型查找对应手法", "评估手法适用性（目标、风险、副作用）", "确定执行顺序"],
        inputs="坏味道清单",
        outputs="重构方案（手法序列+预期效果）",
    ),
    RefactorProcess(
        id="execute-and-verify", name="execute-and-verify", label="执行并验证",
        steps=["应用手法（每次只做一步）", "运行测试", "提交"],
        inputs="重构方案",
        outputs="重构后的代码+绿色流水线",
    ),
]

safety_nets = [
    SafetyNet(
        id="unit-test-coverage", name="unit-test-coverage", label="单元测试覆盖",
        requirement="重构涉及的代码路径≥80%覆盖率",
        tools=["pytest", "JUnit", "vitest"],
    ),
    SafetyNet(
        id="static-type-checking", name="static-type-checking", label="静态类型检查",
        requirement="编译器或类型检查工具在重构时及早发现接口不匹配",
        tools=["mypy", "pyright", "TypeScript compiler", "rustc"],
    ),
    SafetyNet(
        id="ci-pipeline", name="ci-pipeline", label="持续集成流水线",
        requirement="所有测试通过→自动构建→部署到staging",
        tools=["GitHub Actions", "GitLab CI", "Jenkins"],
    ),
]


if __name__ == "__main__":
    print(f"重构手法: {len(refactoring_techniques)} 项")
    print(f"代码坏味道: {len(code_smells)} 项")
    print(f"重构目标: {len(refactor_goals)} 项")
    print(f"重构流程: {len(refactor_processes)} 项")
    print(f"重构安全网: {len(safety_nets)} 项")
