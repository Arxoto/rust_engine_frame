# DamageEffect / DamageEffectBuffer 公开构造路径

Type: task
Status: resolved

为 `common_impl/combats/damages.rs` 的 `DamageEffect<S>`(第 23-28 行,字段私有)与 `DamageEffectBuffer<S>(Vec<...>)`(第 14 行,元组字段私有)提供公开构造路径,使 `merge_damages`/`apply_damages` 能被上层喂入输入。spec:`constructible-public-inputs/spec.md`。

## 验收

- 上层可构造 `DamageEffect` 与 `DamageEffectBuffer`,无需访问私有字段。
- `merge_damages` / `apply_damages` 可被进程内测试以黑盒方式调用并断言结果。
- 至少一个 doc test 或单元测试演示完整"构造 → 调用 → 断言"链路。
- 不改伤害公式与合并次序的实现行为。

## Answer

已实现（commit 见 git log）：
- `DamageEffect::new(dmg_type, dmg_calc, eff)` 公开构造路径。
- `DamageEffectBuffer::new()` / `push()` / `len()` / `is_empty()` + `Default`。
- 两个黑盒演示测试 `test_damage_pipeline_constructible_merge` / `_apply`（仅用公开 API，覆盖 merge 合并与 apply 全链路、死因记录）。
