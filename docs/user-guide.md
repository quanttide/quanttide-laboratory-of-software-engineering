# 用户文档规划

## 定位

面向普通用户的代码重构工具说明书，回答"这工具能做什么"而非"这代码怎么工作的"。

## 文档结构

```
1. 身份卡 — 一行说明工具是什么

2. 三步走 — 三种功能按使用流程排列
   2.1 发现坏味道
       - 过长函数（超过 30 行）
       - 过长参数列表（超过 5 个参数）
       - 过大类（超过 10 个方法或超过 10 个字段）
       - 来源：test_detect_smells_on_fixture
   2.2 规划修复方案
       - 检测后自动匹配重构手法，按严重程度排序
       - 来源：test_plan_from_detection
   2.3 自动修复代码
       - 重命名变量、提取函数
       - 修复后编译检查 + 语义验证
       - 来源：test_full_pipeline
       - 来源：test_detect_no_smells_on_clean（零误报）
       - 来源：test_full_pipeline ast.parse（不破坏语法）

3. 信任底线 — 什么情况下工具不误报

4. 能力边界 — 不做的事
   - 不改 TypeScript（只读）
   - 不修改跨文件代码
   - 不接 LLM 做决策（可空运行）
```

## 事实源

所有内容直接来自集成测试断言，不引用源码实现。

| 文档章节 | 对应集成测试 | 断言依据 |
|---------|-------------|---------|
| 2.1 发现坏味道 | `test_detect_smells_on_fixture` | `"long-function" in smell_ids` 等 |
| 2.1 阈值精确值 | fixtures/sample.py 的构造方式 | `len(smells) >= 3` + smell_id 断言 |
| 2.2 规划修复方案 | `test_plan_from_detection` | `step.method_id in known_methods` |
| 2.3 自动修复代码 | `test_full_pipeline` | `result.status in ("success", "failed")` |
| 3 信任底线 | `test_detect_no_smells_on_clean` | `len(smells) == 0` |
| 3 修复不破坏语法 | `test_full_pipeline` | `ast.parse(work_file.read_text())` |

## 写作约束

| 维度 | 要求 |
|------|------|
| 视角 | 产品经理向普通用户介绍能力 |
| 篇幅 | 不超过 30 行正文 |
| 语言 | 中文，无代码块，无架构图，无实现细节 |
| 生成方式 | 脚本从集成测试 docstring 自动生成 |
