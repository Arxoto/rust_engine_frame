# few-shot 边界语义与 PausePrefab 干预测试

Type: task
Status: resolved

钉住 `few_shot_times.rs` 的边界行为:第 limit+1 次调用时内层触发器先成功触发、外层返回 `false` 并静默丢弃该次触发(`few_shot_times.rs:66-85`);并测试 `PausePrefab` 对 `tick` 的干预(冻结期间不推进,`pause_prefab.rs:89-162`)。当前均无测试。spec:`interface-demonstration-tests/spec.md`。

## 验收

- few-shot:limit 次内返回 `true`,第 limit+1 次返回 `false`,且行为被测试文档化(区分"未到时间"与"额度耗尽")。
- pause:暂停期间 `tick` 不推进进度,恢复后继续。
- 测试通过公开 trait 接口驱动。

## Answer

已实现（commit 45d3402）：
- `few_shot_times.rs` 2 个测试：limit 次内返回 `true`、第 limit+1 次静默丢弃（`few_shot_limited_trigger_drops_overflow`）；区分"未到时间"与"额度耗尽"（`few_shot_distinguishes_not_time_from_exhausted`）。
- `pause_prefab.rs` 2 个测试：暂停冻结期间 `tick` 不推进、恢复后继续（`pause_prefab_freezes_tick_while_paused`）；View/Control/Progress 委托（`pause_prefab_view_control_and_progress_delegate`）。
- 全部经公开 trait 接口（`Union<&PausePrefab, &Timer>` 组合视图）驱动。
