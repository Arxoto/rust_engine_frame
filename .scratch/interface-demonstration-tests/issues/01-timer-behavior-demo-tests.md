# 计时器核心行为演示测试

Type: task
Status: resolved

为 `base_lib/cores/timers/` 的 `TickTimer`、`InfiniteTickTrigger` 等补齐演示测试,覆盖 `Tickable::tick`、`TimerProgress`(`elapsed/remaining/duration/progress`)、`TimerView`、`TimerControl`(`reset/complete`)。当前 8 个计时器模块仅 1 个测试且断言的是浮点常量(`static_timer.rs:117-127`)。spec:`interface-demonstration-tests/spec.md`。

## 验收

- `TickTimer` 与 `InfiniteTickTrigger` 的 progress/view/control 各有测试,断言行为而非浮点常量。
- 覆盖"先业务后 tick"的计时次序约定(帮助类后 tick、限制类先 tick)。
- 每个测试可被 `cargo test --lib base_lib::cores::timers::` 筛选运行。

## Answer

已实现（commit 45d3402）：
- `tick_timer.rs` 5 个测试：初始状态、累加与时长钳制、进度比例、reset/complete、inf 永不完全。
- `tick_trigger.rs` 4 个测试：初始状态、try_trigger 消耗周期、无钳制累加、reset/complete noop。
- 断言行为（elapsed/remaining/progress/is_completed）而非浮点常量；"先业务后 tick"约定由 tick 次序测试覆盖。
