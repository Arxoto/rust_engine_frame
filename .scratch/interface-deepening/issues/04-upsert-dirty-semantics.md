# UpsertContainer 统一 dirty 语义并收拢状态机

Type: task
Status: ready-for-agent
Blocked by: interface-demonstration-tests/06

`base_lib/eff_attr_prop/upsert_container.rs` 两条修改路径 dirty 语义矛盾:`iter_mut`(第 62-65 行)不置脏、`select_mut_ele`(第 114-125 行)无条件置脏,导致经 `iter_mut` ticking 会静默绕过 `try_refresh_dirty_attr`。清洞调度散在 4 处(阈值 `upsert_container.rs:141-155` + `UpsertContainerCleaner` 第 173-192 行 + 常量 `time_type::DEFAULT_REFRESH_PERIOD` + 胶水 `attr_systems::try_clean_hole`)。合并策略靠调用方 `update_logic` 闭包。统一语义(倾向"任何修改都置脏",或提供显式批量 API),将调度收进容器,提供默认合并策略方法。spec:`interface-deepening/spec.md`。

## 验收

- `iter_mut` 与 `select_mut_ele` 的脏语义一致(或批量 API 显式纳入脏跟踪),契约被测试钉住。
- 清洞调度与周期常量收进容器/容器模块,调用方不再跨 4 个文件拼装。
- 提供默认合并策略方法(如 `upsert_replace`),`equips.rs:138-140` 改用它。
- 现有 upsert 测试(含压力测试)全部继续通过,测试数只增不减。
