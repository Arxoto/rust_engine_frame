# props / prop_bounds_eff / effects 单元测试

Type: task
Status: ready-for-agent

补齐 `base_lib/eff_attr_prop/` 下 `props.rs`、`prop_bounds_eff.rs`、`effects.rs` 的单元测试(当前 0 测试):`Prop::apply_eff`/`apply_eff_checked`/`refresh_bounds`/`current_is_zero`、`PropBoundsEffect` 的上下限映射(`prop_bounds_eff.rs:39-54`)、`EffectMean::which_nature` 的基线判断。spec:`interface-demonstration-tests/spec.md`。

## 验收

- 资源池瞬时变更、上下限钳制、下限为 0 时的归零判断各有测试。
- `PropBoundsEffectType`(UpperAdd/UpperPer/LowerAdd)映射到 `AttrEffectType` 的行为被钉住。
- `EffectMean` 的基线(加法 0、乘法 1)约定被测试固化。
