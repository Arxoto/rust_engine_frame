# StaticTimer / StaticTimeline 生命周期测试

Type: task
Status: resolved

为 `base_lib/cores/timers/static_timer.rs` 的 `StaticTimer`/`StaticTimeline` 补齐测试:绝对时间戳比较、`reset_timeline_and_get_diff` 与 `fix_timeline_diff` 的漂移修正生命周期(参考 `attr_systems::try_reset_timeline`)。当前唯一测试只断言 `f64::MAX + 0.1`。spec:`interface-demonstration-tests/spec.md`。

## 验收

- `StaticTimer` 的 `elapsed/remaining/progress/is_completed/reset/complete`(经 `&StaticTimeline` 上下文)各有测试。
- 时间线重置 → 全部依赖计时器 `fix_timeline_diff` 后,相对时间读数保持不变。
- 测试覆盖"先业务后 tick"的约定。

## Answer

已实现（commit 45d3402，另含 e678c91 的 StaticTimer remaining bug 修复）：
- `static_timer.rs` 5 个测试：初始状态、随时间线推进（elapsed/remaining/progress）、reset/complete、`reset_timeline_and_get_diff` → 全部 `fix_timeline_diff` 后相对读数不变（`static_timer_fix_timeline_diff_preserves_relative_readings`）、inf 永不完全。
- 生命周期两层覆盖：`static_timer.rs` 内联的任意 diff 相对读数不变 + `attr_systems.rs` 端到端 `try_reset_timeline`（大 tick 越过一年门槛）。
