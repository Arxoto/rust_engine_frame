# 计时器核心行为演示测试

Type: task
Status: ready-for-agent

为 `base_lib/cores/timers/` 的 `TickTimer`、`InfiniteTickTrigger` 等补齐演示测试,覆盖 `Tickable::tick`、`TimerProgress`(`elapsed/remaining/duration/progress`)、`TimerView`、`TimerControl`(`reset/complete`)。当前 8 个计时器模块仅 1 个测试且断言的是浮点常量(`static_timer.rs:117-127`)。spec:`interface-demonstration-tests/spec.md`。

## 验收

- `TickTimer` 与 `InfiniteTickTrigger` 的 progress/view/control 各有测试,断言行为而非浮点常量。
- 覆盖"先业务后 tick"的计时次序约定(帮助类后 tick、限制类先 tick)。
- 每个测试可被 `cargo test --lib base_lib::cores::timers::` 筛选运行。
