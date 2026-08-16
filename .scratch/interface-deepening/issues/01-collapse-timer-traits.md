# 收拢计时器 trait 面

Type: task
Status: resolved
Blocked by: interface-demonstration-tests/01, interface-demonstration-tests/02, interface-demonstration-tests/03

`base_lib/cores/timers/tiny_timer.rs` 当前 8 个 trait 支撑 30–80 行的具体计时器(接口 ≈ 实现):暂停 View/Control 围绕单布尔过度拆分(`TimerPauseView` 第 74 行 + `TimerPauseControl` 第 78 行,仅两个实现、一条调用链);`TimerProgress`(4 方法)+ `TimerView`(1 方法)让每个具体类型都要实现两遍只读;`CyclicalTrigger`(第 87 行)是单方法 trait 只被触发器实现。合并过度拆分,并论证原借用冲突理由(`tiny_timer.rs:16-17`)在新形态下是否仍成立。spec:`interface-deepening/spec.md`。

## 验收

- 阅读一个具体计时器所需的概念数明显下降。
- 删除测试通过:删掉的 trait 其行为测试迁移到新接口,复杂度集中在计时器模块内部。
- 等级 B 的计时器演示测试(回归基线)全部继续通过。
- 保留 `DependCtx` 关联类型方案(方案二)。

## Answer

**决议:trait 面收拢不实施(决策不实施),trait 拆分保持现状。** 经逐 trait 核对能力边界,「合并过度拆分」的判断不成立,合并反而会耦合不同能力:

- `TimerProgress`(进度模型:elapsed/remaining/duration/progress)vs `TimerView`(完成状态:is_completed)是**两种能力**。若存在「仅状态、无进度」的合法类型。合并会强迫它实现 4 个无语义方法。
- `TimerControl` vs `CyclicalTrigger` 同理:`TickTimer`/`StaticTimer` 没有循环触发能力;用默认值 `false` 合并,只是把耦合藏进默认方法。
- `TimerPauseView`/`TimerPauseControl` 的拆分与模块文档「只读与可变必须分离」原则一致(组合代理只读/可变权限截断),不合并。
- 「阅读一个具体计时器需理解约 11 个概念」的根源不是 trait 数量,而是 `DependCtx` GAT + `Union` 组合的机制性负担 —— 它们是承重结构(spec 明确不推翻),靠合并 trait 名字消不掉。合并省下的 prefab impl 块(每 prefab 1–2 个)不值能力耦合的代价。

**实际交付**:
- 补全 `FewShotStaticTrigger` 的 `TimerProgress`:透传内层 `InfiniteStaticTrigger`(反映当前周期内的相对时间);`is_completed` 仍由 few-shot 额度决定,不来自内层(内层永不完成)。见 `static_trigger.rs`。
- 新增演示测试 `few_shot_static_trigger_progress_delegates_and_completion_from_quota`,计时器模块测试 19 个全绿。

**备注(未纳入本决议)**:暂停 Union 死代理(`PausePrefab::of_timer_pause_view`/`of_timer_pause_control` 及其 Union impl,纯转发 prefab、只在测试中使用)是可选的清理项,如需可另开 issue。
