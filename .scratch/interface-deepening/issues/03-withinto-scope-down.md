# WithInto 降为私有或删除

Type: task
Status: resolved

`design_patterns.rs` 的 `WithInto<Ctx>`(第 49-53 行)是方案一的残留,公开导出但零生产调用,只活在 `design_patterns.rs` 自己的测试里(第 68-74、112-131、155-169 行)。将其降为私有(`pub(crate)` 或模块内)或删除;若 `motions/actions.rs` 仍用 `Union::new`,保留 `Union` 本身。spec:`interface-deepening/spec.md`。

## 验收

- `WithInto` 不再作为公开 trait 出现在 crate 接口中(或已删除)。
- `Union` 保留且构造方式统一(消除 `pub(super)` 元组构造与 `Union::new` 双写法,`design_patterns.rs:39`、`few_shot_times.rs:21`、`actions.rs:193-209`)。
- 相关测试迁移或删除,`cargo test --lib base_lib::cores::design_patterns` 通过。
