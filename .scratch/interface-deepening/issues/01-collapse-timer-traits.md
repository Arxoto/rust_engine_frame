# 收拢计时器 trait 面

Type: task
Status: ready-for-agent
Blocked by: interface-demonstration-tests/01, interface-demonstration-tests/02, interface-demonstration-tests/03

`base_lib/cores/timers/tiny_timer.rs` 当前 8 个 trait 支撑 30–80 行的具体计时器(接口 ≈ 实现):暂停 View/Control 围绕单布尔过度拆分(`TimerPauseView` 第 74 行 + `TimerPauseControl` 第 78 行,仅两个实现、一条调用链);`TimerProgress`(4 方法)+ `TimerView`(1 方法)让每个具体类型都要实现两遍只读;`CyclicalTrigger`(第 87 行)是单方法 trait 只被触发器实现。合并过度拆分,并论证原借用冲突理由(`tiny_timer.rs:16-17`)在新形态下是否仍成立。spec:`interface-deepening/spec.md`。

## 验收

- 阅读一个具体计时器所需的概念数明显下降。
- 删除测试通过:删掉的 trait 其行为测试迁移到新接口,复杂度集中在计时器模块内部。
- 等级 B 的计时器演示测试(回归基线)全部继续通过。
- 保留 `DependCtx` 关联类型方案(方案二)。
