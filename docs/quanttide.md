# quanttide + quanttide-agent 库用法教程

## 安装

```bash
pip install quanttide quanttide-agent
```

`quanttide-agent` 依赖 `httpx`、`pydantic>=2.0`。
`quanttide` 依赖 `platformdirs`。

---

## quanttide — 基础设施

### LocalStorage — 跨平台目录管理

自动遵循 XDG（Linux）、macOS、Windows 标准目录。

```python
from quanttide import LocalStorage

store = LocalStorage("my-app")

store.config_dir  # ~/.config/my-app/
store.data_dir    # ~/.local/share/my-app/
store.state_dir   # ~/.local/state/my-app/
store.cache_dir   # ~/.cache/my-app/
store.log_dir     # ~/.cache/my-app/log/
store.runtime_dir # /run/user/1000/my-app/ (fallback: cache_dir/run)
```

环境变量覆写：`{APP}_{CATEGORY}_HOME`（全大写），例如：

```bash
export MY_APP_DATA_HOME=/tmp
export MY_APP_CONFIG_HOME=./config
```

### Pydantic Field 类型

配合 Pydantic v2 使用的字段类型注解：

```python
from pydantic import BaseModel
from quanttide import IdField, NameField, TitleField, DescriptionField, LabelField, OrderField
from quanttide import CreatedAtField, UpdatedAtField

class Project(BaseModel):
    id: IdField                    # UUID
    name: NameField                # ≤100, slug 风格
    title: TitleField              # ≤255
    desc: DescriptionField         # 长文本
    label: LabelField              # ≤50
    order: OrderField              # int ≥1
    created_at: CreatedAtField     # datetime
    updated_at: UpdatedAtField     # datetime
```

---

## quanttide-agent — LLM 集成

### 配置

支持环境变量/`.env` 文件加载。字段见 `Settings`：

```bash
# .env 或环境变量
LLM_MODEL="deepseek-v4-flash"
LLM_BASE_URL="https://api.deepseek.com"
LLM_API_KEY="sk-..."
```

也可在代码中直接传入：

```python
from quanttide_agent import LLM

llm = LLM(model="deepseek-v4-flash", base_url="https://api.deepseek.com", api_key="sk-...")
```

优先级：构造函数参数 > 环境变量 > 默认值。

### 基础：LLM.complete

```python
from quanttide_agent import LLM, Message

llm = LLM()

# 方式 1：纯字符串
resp = llm.complete("你好")
print(resp.content)       # str
print(resp.model)         # 实际使用的模型
print(resp.finish_reason) # "stop"
print(resp.usage)         # Usage(input_tokens=..., output_tokens=...)

# 方式 2：Message 列表
messages = [
    Message(role="system", content="你是助手"),
    Message(role="user", content="你好"),
]
resp = llm.complete(messages)

# 方式 3：dict 列表
resp = llm.complete([
    {"role": "system", "content": "你是助手"},
    {"role": "user", "content": "你好"},
])
```

#### 参数控制

```python
resp = llm.complete(
    "解释 Python 装饰器",
    temperature=0.3,
    max_tokens=2000,
    top_p=0.9,
    stop=["\n\n"],
    frequency_penalty=0.5,
    presence_penalty=0.5,
    thinking=True,                 # 启用深度思考
    reasoning_effort="max",        # "low" | "medium" | "high" | "max"
)
```

### Message

```python
from quanttide_agent import Message

msg = Message(role="user", content="hi")
msg.to_dict()  # {"role": "user", "content": "hi"}

# 支持 role: "system" | "user" | "assistant" | "tool"
# tool_call_id 可选，用于 tool response
```

### Tool / ToolSchema — 工具定义

```python
from quanttide_agent import Tool, ToolSchema

# 仅 schema（用于 LLM function calling）
schema = ToolSchema(
    name="get_weather",
    description="获取天气",
    parameters={
        "type": "object",
        "properties": {
            "location": {"type": "string"},
        },
        "required": ["location"],
    },
)

# Tool = ToolSchema + executor（用于 ReActAgent）
def get_weather(args: dict) -> str:
    location = args.get("location", "")
    return f"{location} 的天气是 25°C"

tool = Tool(
    name="get_weather",
    description="获取天气",
    parameters={
        "type": "object",
        "properties": {
            "location": {"type": "string"},
        },
        "required": ["location"],
    },
    executor=get_weather,
)

tool.execute({"location": "北京"})  # 调用 executor
```

### LLM Function Calling（原生 API）

```python
tools = [
    ToolSchema(
        name="calculator",
        description="四则运算",
        parameters={
            "type": "object",
            "properties": {
                "expr": {"type": "string", "description": "表达式"},
            },
            "required": ["expr"],
        },
    ),
]

resp = llm.complete("计算 123*456", tools=tools, tool_choice="auto")
print(resp.content)       # 可能为空（模型决定调用工具）
print(resp.tool_calls)    # [ToolCall(id="...", name="calculator", arguments='{"expr":"123*456"}')]
```

### ReActAgent — 思考-行动-观察循环

适用于需要多步推理+工具调用的场景。使用 ReAct 协议格式。

```python
from quanttide_agent import ReActAgent, LLM, Tool, Message, ActionParser

llm = LLM()
tools = [
    Tool(name="search", description="搜索", executor=lambda args: f"结果: {args}"),
    Tool(name="calculate", description="计算", executor=lambda args: str(eval(args.get("expr", "0")))),
]

agent = ReActAgent(llm, tools)
# 可选自定义 ActionParser：
# agent = ReActAgent(llm, tools, parser=ActionParser(
#     key_action_name="Action", key_action_args="Input",
# ))

result = agent.run([
    Message(role="system", content=ReActAgent.system_prompt("工具列表...")),
    Message(role="user", content="计算 2+3 并搜索 Python"),
])
print(result)
```

ReAct 协议格式（LLM 输出）：

```
Thought: 我需要计算 2+3
Action name: calculate
Action args: {"expr": "2+3"}
```

获得最终结果时：

```
Thought: 我得到答案了
Final Answer: 5
```

### Action / ActionParser — 动作解析

```python
from quanttide_agent import Action, ActionParser

parser = ActionParser()
# 默认解析格式：
#   Action name: xxx
#   Action args: {...}

action = parser.parse("""Action name: validate
Action args: {"domain": "test"}""")
# Action(name="validate", args={"domain": "test"})

# 自定义格式
parser2 = ActionParser(key_action_name="Tool", key_action_args="Input")
```

### 完整示例：LLM 条件检查

```python
from quanttide_agent import LLM, Message

llm = LLM()
code = "def foo(a, b, c, d, e, f, g): pass"

resp = llm.complete([
    Message(role="system", content=(
        "你是一个代码重构专家。判断是否可以对给定代码应用指定的重构方法。"
        "只回复 yes 或 no。"
    )),
    Message(role="user", content=(
        f"代码：\n```python\n{code}\n```\n"
        "重构方法：extract-function（将一段代码提取为独立函数）\n"
        "前提条件：代码块包含至少 2 个语句，不是单行 def。\n"
        "是否满足条件？"
    )),
], temperature=0.1)

print(resp.content)  # yes / no
```

---

## 最佳实践

1. **LLM 实例复用**：全局只创建一个 `LLM` 实例，不要每次调用都新建。
2. **System Prompt**：放在第一个 `Message(role="system")`，明确给出角色、规则、输出格式。
3. **Temperature**：条件判断用低 temperature（0.0-0.2），创意类用高（0.7-0.9）。
4. **环境变量**：API Key 通过 `.env` 文件或环境变量注入，不硬编码。
5. **错误处理**：`LLM.complete` 失败时抛 `LLMError`，retry 参数可设自动重试次数。
6. **Token 追踪**：`resp.usage` 包含 `input_tokens`/`output_tokens`/`reasoning_tokens`。
7. **ReActAgent**：适合需要多步工具调用的场景；单次判断用 `LLM.complete` 更简单。
