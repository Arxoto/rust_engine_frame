# few-shot 边界语义与 PausePrefab 干预测试

Type: task
Status: ready-for-agent

钉住 `few_shot_times.rs` 的边界行为:第 limit+1 次调用时内层触发器先成功触发、外层返回 `false` 并静默丢弃该次触发(`few_shot_times.rs:66-85`);并测试 `PausePrefab` 对 `tick` 的干预(冻结期间不推进,`pause_prefab.rs:89-162`)。当前均无测试。spec:`interface-demonstration-tests/spec.md`。

## 验收

- few-shot:limit 次内返回 `true`,第 limit+1 次返回 `false`,且行为被测试文档化(区分"未到时间"与"额度耗尽")。
- pause:暂停期间 `tick` 不推进进度,恢复后继续。
- 测试通过公开 trait 接口驱动。
