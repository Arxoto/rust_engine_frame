# attr_systems 真实数据组合测试

Type: task
Status: ready-for-agent

`base_lib/eff_attr_prop/attr_systems.rs` 当前的唯一组合测试 `example_process_tick` 跑的是**空切片**(`attr_systems.rs:113`),`clean_expired_element → try_refresh_dirty_attr → try_clean_hole → try_reset_timeline` 的组合从未用真实数据演示,`try_reset_timeline` 无任何测试。用真实(非空)实体数据扩展或新增组合测试。spec:`interface-demonstration-tests/spec.md`。

## 验收

- 组合测试使用带效果的容器 + 时间线 + 过期效果,断言刷新、清理、时间线重置全链路。
- `try_reset_timeline` 有覆盖(含 `fix_timeline_diff` 对依赖计时器的影响)。
- 测试即"每实体 tick"的样板,可被等级 C 与 combats 沿用。
