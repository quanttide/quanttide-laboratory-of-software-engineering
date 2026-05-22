# 代码重构分析工具

## 这是什么

一个可编程的代码重构分析工具。输入源码，输出结构化的坏味道清单和对应的重构方案。

## 用法

### 扫描代码坏味道

```bash
python src/main.py
```

对 `sample.py` 运行检测 + 规划 + 执行流水线：

```
发现 3 个坏味道
  [long-function] sample.py:43-77  severity=0.42
  [long-parameter-list] sample.py:38-40  severity=0.40
  [large-class] sample.py:3-35  severity=0.65

规划了 2 步重构
  1. extract-class → large-class
  2. extract-function → long-function
```

### 运行测试

```bash
python -m pytest tests/ -v        # 17 项单元测试
python -m pytest integrated_tests/ -v  # 4 项集成测试
```

集成测试在 2157 行的真实 TypeScript 代码上验证流水线，检出 24 个超长函数。

## 架构

```
examples/
  code_refactor.py          知识定义（类型 + 实例数据）

src/                        核心逻辑
  detectors.py              坏味道检测器
  planner.py                重构规划器
  transformers.py           代码变换器
  session.py                流程编排

tests/                      单元测试
integrated_tests/           集成测试
docs/                       文档
```

## 局限性

- 检测器仅支持 Python AST，TS 检测是正则近似
- 仅实现了重命名变量和提炼函数两种变换
- 条件检查（`_check_condition`）是乐观放行，未真实评估
- 不是 AI，没有推理能力，所有规则都是硬编码
