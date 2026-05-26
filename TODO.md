# TODO

## 证据链实验

- [ ] 模式 D：正向证据链（raw → trimmed → parts → price → total）vs 反向追溯链
- [ ] 模式 E：不同粒度追溯链（2 层 vs 全路径）
- [ ] 模式 F：多变量并列呈现（price 和 qty 的 dataflow 并排放）
- [ ] 反面案例：什么情况下完整代码反而让 LLM 忽略关键问题？
- [ ] 证据不足时 LLM 编造分析的频率统计

## 假阳性过滤

- [ ] 在真实项目上跑 review，统计 LLM dismiss 率
- [ ] 人工验证 dismiss 是否合理
- [ ] 通用 prompt vs 专用 prompt 对比

## 工具

- [ ] `reflect` 命令原型
- [ ] 置信度变量名表扩展（当前只覆盖了 process_order 的变量）
