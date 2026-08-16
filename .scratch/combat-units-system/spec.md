# Spec: combat_units_system(战斗单位系统)

Status: resolved

## Problem Statement

`src/common_impl/combats/combat_units.rs` 的模块文档列出的「todo 可立即实现」五项需求目前全部没有代码实现,战斗单位的生命周期能力(三维初始化、护盾生成/装载、能量花费、削韧)缺失,上层无法驱动战斗单位进入可战斗状态:

1. 根据内禀属性初始化三维(`Health`/`Stamina`/`Magicka`)。
2. 根据外赋属性生成防护护盾(影响最大值和当前值)。
3. 装载奥术护盾和替身护盾。
4. 花费能量与尝试花费(`try_cost_magicka`)。
5. 削韧(`cut_stamina`,冲击-平衡)。

同时存在一个模块组织约束需要满足:**`combats.rs` 内各模块按 `pub mod` 声明顺序单向依赖——上面的模块不能 `use` 下面的模块,下面的可以 `use` 上面的模块(模块间不得存在循环依赖)。** 因此新代码的落点必须在该约束下确定。

## Solution

新增系统模块 `src/common_impl/combats/combat_units_system.rs`,模块声明加在 `combats.rs` 的 `pub mod damages;` **之后**(成为 `combats` 中最下面的模块,可单向 `use` 其上所有模块,无环)。沿用 `equips.rs` 的 `equip_system`、`damages.rs` 的 `damage_system` 惯例——系统逻辑独立成子模块,不与数据声明混排。

模块提供六个公开函数,对应五个 todo:

| 函数 | 对应 todo | 职责 |
|---|---|---|
| `init_three_bars` | ① 按内禀初始化三维 | 由 `Strength`/`Belief` + 配置参数算出三维并写入 `Health`/`Stamina`/`Magicka` |
| `gen_shield_defence` | ② 按外赋生成防护护盾 | 固化公式 `calc_defence_shield(&ArmorHard)`;**只生成护盾值,不装载**,由调用方手动装载 |
| `load_shield` | ②③ 装载护盾(防护/奥术/替身通用) | 传入完整 `Effect<S>`,同时改护盾最大值与当前值 |
| `cost_magicka` | ④ 花费能量 | 硬扣,返回实际生效值 |
| `try_cost_magicka` | ④ 尝试花费 | 软扣,能量不足返回 `None` |
| `cut_stamina` | ⑤ 削韧 | 负数入参削减平衡,返回实际生效值 |

## User Stories

1. As 一个游戏上层调用方, I want 角色创建时调用一个函数即可按内禀属性初始化血量/平衡/能量三维, so that 角色从出生起就是可战斗状态。
2. As 一个游戏上层调用方, I want 根据外赋属性(装备坚韧)生成防护护盾值, so that 我可以把它作为护盾装载的来源。
3. As 一个游戏上层调用方, I want 用一个统一的装载函数把任意护盾值装入防护/奥术/替身护盾(同时改最大值和当前值), so that 玩家施放护盾法术时护盾立即就位。
4. As 一个游戏上层调用方, I want `cost_magicka`/`try_cost_magicka` 支持硬扣与软扣两种能量花费, so that 系统级强制扣除与施法前置检查都能表达。
5. As 一个游戏上层调用方, I want `cut_stamina` 削减平衡, so that 冲击伤害可以破坏敌方架势、营造输出窗口。
6. As 一个框架维护者, I want 新系统模块放在 `damages` 之后、单向 use 上方模块, so that 模块间无环依赖、符合 `combats.rs` 的 `pub mod` 顺序约束。
7. As 一个框架维护者, I want 系统函数不含数值公式与魔法数字, so that 平衡数值由上层配置表驱动、可独立调整。

## Implementation Decisions

- **放置**:新建 `combat_units_system.rs`,在 `combats.rs` 的 `pub mod damages;` 之后声明。该模块 `use` `combat_inherents`/`combat_additions`/`combat_units`/`damages` 及 `base_lib`,全部在其上方,单向无环。不把逻辑塞进 `combat_units.rs`(那里若 `use damages` 将构成"上 use 下"的环依赖违规)。
- **`init_three_bars`**:单函数、整包初始化(不拆三个)。配置参数(血量/能量的 base+scale、`stamina_max`、`magicka_energy_level`)聚合进 `ThreeBarsConfig` 结构体作为单一入参(避免 clippy `too_many_arguments`,配置由上层传入——**不含魔法数字**)。内部:
  - 血量 `current = max`(经 `damage_system::calc_health_max`)。
  - 平衡 `current = max = stamina_max`(与任何内禀无关)。
  - 能量 `current = 0`、`max = calc_magicka_max`。
  - 直接构造 `Prop` 写入三个 newtype,不经过容器。
- **`gen_shield_defence`**:固化 `damage_system::calc_defence_shield(&ArmorHard)`,返回 `f64` 护盾值,**不装载**。调用方取得该值后自行构造 `Effect<S>` 并手动调 `load_shield`。
- **`load_shield`**:单个**泛型**函数,三种护盾通用,签名直接操作 `&mut Prop` + `&mut UpsertContainer<PropBoundsEffect<S, StaticTimer>>` + `Effect<S>`(调用方传 `&mut shield.0`、`&mut shield_effs.0`,与 `pub` 字段风格一致)。内部按「效果驱动」机制:
  1. 用传入 `Effect<S>` 构造 `PropBoundsEffect(UpperAdd)` 并 upsert 进容器(id 由 `Effect` 的 from/eff 名决定,重复装载同 id 幂等覆盖)。
  2. `refresh_bounds` 重算上限。
  3. `apply_eff(V)` 提升当前值至装载值(**同时改 max 和 current**,满足 todo 原文)。
- **奥术/替身护盾只做装载,不做生成**:其数值公式是给数值策划的要求(见 `damages.rs` 文档注释的「直接正相关 Belief」及阈值激发设定),不是代码要求;调用方自行算好值传入 `load_shield`。
- **`cost_magicka` / `try_cost_magicka`**:分别委托 `Prop::apply_eff` / `Prop::apply_eff_checked`。`cost_magicka` 返回 `PropAlterResult`(实际生效值);`try_cost_magicka` 返回 `Option<PropAlterResult>`,能量不足返回 `None` 且不改值。
- **`cut_stamina`**:入参为**负数**(减益用负值,符合 `EffectMean::Bad` 语义),委托 `Prop::apply_eff`,返回 `PropAlterResult`。
- **全项目约定(本 spec 确立,需同步进 CONTEXT.md)**:
  1. **负数入参**:凡减益/伤害类效果入参,调用方传负值(与 `EffectMeaning` 的 `Bad` 语义一致)。
  2. **完整 `Effect<S>`**:凡涉及效果的**持久**上层封装,入参默认传完整 `Effect<S>`(含 from/eff 名),除非特殊要求;不传裸 `f64` 丢失来源记录。**即时(非持久)资源变更例外**(如扣能量/削韧),无 id 参与 upsert,收裸数值。

## Testing Decisions

- **好测试的标准**:通过公开接口「构造公开输入 → 调用 `pub fn` → 断言可观察行为」演示用法并钉住行为(即「接口即测试面」接缝,等级 A/B 已有先例)。不访问私有字段。
- **被测模块**:`common_impl/combats/combat_units_system.rs`。
- **现有先例**:`damages.rs` 的 `damage_system` 测试(`apply_one` 构造链)、`equips.rs` 的 `equip_weapon_chain_reflects_to_attr`(equip → refresh → 断言)、`props.rs` 的 `apply_eff`/`apply_eff_checked` 测试。
- **硬性要求**:六个 `pub fn` 每个至少一个演示测试;可被 `cargo test --lib common_impl::combats::` 筛选运行;浮点断言按项目约定选可精确表示的数、无容差。
- **建议测试**:
  - `init_three_bars`:给定 `Strength`/`Belief` 与配置参数,断言三维的 max/current(血量满、平衡满、能量 0)。
  - `gen_shield_defence`:给定 `ArmorHard`,断言返回 `calc_defence_shield` 同值;且调用后护盾未被装载(隔离测试)。
  - `load_shield`:分别对防护/奥术/替身装载,断言 max 与 current 同时变为装载值;同 id 重复装载幂等覆盖(用不同值验证)。
  - `cost_magicka` / `try_cost_magicka`:硬扣超限被钳制并返回实际值;软扣能量不足返回 `None` 且值不变、充足时返回 `Some`。
  - `cut_stamina`:负数入参削减、正数入参(若调用)会抬升——按 `EffectMeaning` 语义断言。

## Out of Scope

- 不实现 `combat_units.rs`「todo 后续系统性实现」的内容(角色周期性效果、血量百分比恢复、平衡固定值恢复延迟、能量固定值削减延迟)。
- 不实现奥术/替身护盾的数值公式(留给数值策划)。
- 不修改 `damages.rs` 现有公式与 `calc_*` 函数行为(`gen_shield_defence` 只复用 `calc_defence_shield`)。
- 不引入引擎绑定或新增第三方依赖。
- 不为平衡性调整引入配置资源表;配置参数均为函数入参。

## Further Notes

- 本 spec 是 `combat_units.rs` roadmap 的第一块:**先让三维/护盾/能量/平衡可被驱动,后续系统性恢复效果再在容器与计时器上叠加**。
- 两条全项目约定(负数入参、完整 `Effect<S>`)记录于此,按 `/grill-with-docs` 的纸面记录流程应在实现阶段同步进 `CONTEXT.md`(通过 `/domain-modeling`)。
- 实现后更新 `combat_units.rs` 模块文档:将「todo 可立即实现」五项标记为已实现,指向 `combat_units_system` 的对应函数。
