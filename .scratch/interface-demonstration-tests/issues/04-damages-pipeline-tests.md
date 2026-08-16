# 伤害管线演示测试

Type: task
Status: ready-for-agent
Blocked by: constructible-public-inputs/01

为 `common_impl/combats/damages.rs` 的 `merge_damages`/`apply_damages`/`calc_damage_scale`/`calc_health_max`/`calc_magicka_max`/`calc_defence_shield` 写黑盒演示测试,覆盖合并次序、护盾命中次序、致死判定(`DamageInfo`)与平衡公式。当前 combats 整棵树 0 测试。依赖等级 A 的 `DamageEffect` 构造路径。spec:`interface-demonstration-tests/spec.md`。

## 验收

- 通过公开构造路径喂入伤害输入,断言 `Prop` 变化与 `DamageInfo`。
- 覆盖 6 种 `DamageType` 与致死/护盾分支。
- 公式常量(`MAGICKA_BASELINE` 等)与 `combats.rs` 平衡文档保持一致,测试即文档。
