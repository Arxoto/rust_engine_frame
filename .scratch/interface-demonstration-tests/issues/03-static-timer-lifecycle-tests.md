# StaticTimer / StaticTimeline 生命周期测试

Type: task
Status: ready-for-agent

为 `base_lib/cores/timers/static_timer.rs` 的 `StaticTimer`/`StaticTimeline` 补齐测试:绝对时间戳比较、`reset_timeline_and_get_diff` 与 `fix_timeline_diff` 的漂移修正生命周期(参考 `attr_systems::try_reset_timeline`)。当前唯一测试只断言 `f64::MAX + 0.1`。spec:`interface-demonstration-tests/spec.md`。

## 验收

- `StaticTimer` 的 `elapsed/remaining/progress/is_completed/reset/complete`(经 `&StaticTimeline` 上下文)各有测试。
- 时间线重置 → 全部依赖计时器 `fix_timeline_diff` 后,相对时间读数保持不变。
- 测试覆盖"先业务后 tick"的约定。
