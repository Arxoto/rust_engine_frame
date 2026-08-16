# Spec: combat_units_system(战斗单位系统)——ECS 语义返工

Status: resolved

## Problem Statement

初版 `combat_units_system`(commit 067e24b)实现了「todo 可立即实现」五项需求,但存在**架构级问题**:

1. **命令式而非 ECS**:`load_shield` 内部直接调用 `Prop::refresh_bounds` / `Prop::apply_eff`,同步修改护盾。这不符合 ECS 的"组件由系统每帧统一处理"语义,也与伤害系统(`DamageEffectBuffer` → `merge_damages` → `apply_damages` 的缓冲+消费模式)相违背。
2. **`gen_shield_defence` 返回裸 `f64`**:丢失了效果的来源/id 信息,调用方还要自己组装 `PropBoundsEffect`,生成函数的职责不完整。
3. **`init_three_bars` 写入而非生成**:以 `&mut` 写入方式初始化,不返回所有权,不符合"组件生成"语义。
4. **即时变更的类型缺失**:`cost_magicka`/`try_cost_magicka`/`cut_stamina` 接收裸 `f64`,无法表达"按当前值百分比/按最大值百分比"的修改方式;`DamageCalc` 与新的通用计算类型是同一概念,存在重复。

## Solution

架构返工,将战斗单位系统对齐 ECS 语义:

### base_lib 新增(引擎无关,通用能力)

**`src/base_lib/eff_attr_prop/prop_alter_eff.rs`**:
- `PropAlterEffectType { Val, CurPer, MaxPer }`——通用的绝对/当前百分比/最大百分比计算方式(百分比参照物=**被改的 Prop 自身**)。
- `PropAlterEffect<S> { eff_type: PropAlterEffectType, eff: Effect<S> }`——对任意 Prop 的修改描述。
- `apply_prop_alter_eff(prop: &mut Prop, eff: PropAlterEffect<S>) -> PropAlterResult`——折算百分比后 `Prop::apply_eff`(直接应用,不入 buffer)。
- `PropAlterEffectBuffer<S>`——(**废弃,不实现**)经盘问决策:不新建 buffer,装载型复用 `DamageEffectBuffer`。

**`src/base_lib/eff_attr_prop/prop_systems.rs`**:
- `try_refresh_dirty_prop_bounds<S, Timer>(prop: &mut Prop, effs: &mut UpsertContainer<PropBoundsEffect<S, Timer>>)`——依 `effs.is_changed()` 调 `refresh_bounds` + `apply_bounds`,与 `attr_systems::try_refresh_dirty_attr` 对称。ECS 每帧由该 system 刷新护盾上限,不在装载函数内做。

### common_impl 修改

**`damages.rs`**(伤害管线扩展为通用修改管线):
- `DamageCalc` **删除**,`DamageEffect<S>` 的计算类型字段改为 `PropAlterEffectType`,字段名统一为 `eff_type`(原 `dmg_calc`),加注释说明曾为 `DamageCalc`、现为通用类型。
- `DamageType` 增加三个**装载/仅护盾**类型:
  - `OnlyShieldDefence` / `OnlyShieldSubstitute` / `OnlyShieldArcane`
  - 命名不含装载语义(仅表"只作用于该护盾"),与 `BrokeShieldX` 破盾语义区分。
- `merge_damages`:合并数组扩到 9(`Only*` 追加末尾);增加接收 `&ShieldSubstitute`(`OnlyShieldSubstitute` 的百分比参照物);`Only*` 百分比参照对应护盾。
- `apply_damages`:`Only*` 分支 `target_props` 指向单个对应护盾,复用 `apply_eff` **累加**正值(不设值;机制与伤害扣减对称)。
- `calc_damage_scale`:**对 `KarmaTruth` + 三个 `Only*` 跳过 `base_scale` 能量缩放**(直接返回 `damage_scale`);`Only*` 基础缩放为 1.0。
- `is_hurt_heal()`:`Only*` 返回 `false`(不记录死因)。
- 更新 `docs/adr/0001`(合并数组契约追加装载型位置)。

**`combat_units_system.rs`**:
- `init_three_bars` → **`gen_three_bars`**:返回 `ThreeBars { health, stamina, magicka }` 所有权(不含容器),入参不变(`&Strength, &Belief, &ThreeBarsConfig`)。
- `gen_shield_defence<S>(armor_hard: &ArmorHard, from_name: S, effect_name: S) -> PropBoundsEffect<S, StaticTimer>`:内部固化 `calc_defence_shield` 公式,组装 `UpperAdd` + `StaticTimer::inf()`,返回**上限效果**(不再返回裸 f64)。
- `load_shield<S>(shield_effs: &mut UpsertContainer<PropBoundsEffect<S, StaticTimer>>, damage_buffer: &mut DamageEffectBuffer<S>, bounds_eff: PropBoundsEffect<S, StaticTimer>, value_eff: DamageEffect<S>)`:纯编排——upsert `bounds_eff` 进 `shield_effs`,push `value_eff` 进 `damage_buffer`;**不调 `refresh_bounds`/`apply_eff`**。
- `cost_magicka` / `try_cost_magicka` / `cut_stamina`:改收 `PropAlterEffect<S>`,内部 `apply_prop_alter_eff` 直接应用;`try_cost_magicka` 保留"不足返回 None 且值不变"语义。

### 约束延续
- 模块放置不变:`combat_units_system` 声明于 `damages` 之后,单向 use 上方,无环依赖。
- 负数入参、完整 `Effect<S>`(即时变更除外)两条 CONTEXT 约定继续生效。
- 注释/提交信息中文;浮点断言无容差。

## User Stories

1. As 一个游戏上层调用方, I want 角色创建时 `gen_three_bars` 返回三维所有权, so that 我可以直接将其装配到实体。
2. As 一个游戏上层调用方, I want `gen_shield_defence` 返回就绪的 `PropBoundsEffect`, so that 我无需自行组装护盾上限效果。
3. As 一个游戏上层调用方, I want `load_shield` 只把上限效果放入容器、把当前值效果放入伤害缓冲, so that 护盾上限由每帧 system 依脏标签刷新,当前值由伤害管线统一消费,符合 ECS。
4. As 一个游戏上层调用方, I want `cost_magicka`/`cut_stamina` 接受 `PropAlterEffect`, so that 我可以表达"扣当前值百分比/扣最大值百分比"等修改方式。
5. As 一个框架维护者, I want 护盾装载复用 `DamageEffectBuffer` + `Only*` 类型, so that 不新增平行 buffer,伤害管线成为通用的按类型路由修改管线。
6. As 一个框架维护者, I want 上限刷新在 `prop_systems` 独立成 system, so that 与 `attr_systems` 对称、职责清晰。

## Implementation Decisions

- **`PropAlterEffect`/`PropAlterEffectType`/`apply_prop_alter_eff` 放 `prop_alter_eff.rs`**:base_lib 通用能力,不特定于 combat。
- **`PropAlterEffectBuffer` 不实现**(盘问决策):装载型复用 `DamageEffectBuffer`。
- **上限刷新放 `prop_systems.rs`**:新建文件,与 `attr_systems` 按类型分文件。
- **`DamageCalc` 删除、替换为 `PropAlterEffectType`**:两个类型是同一概念(绝对/当前百分比/最大百分比),消除重复。
- **`DamageEffect.eff_type` 字段**:统一命名,加注释说明来源。
- **`Only*` 类型**:装载型在 `merge_damages` 数组末尾;`calc_damage_scale` 跳过能量缩放;`apply_damages` 单目标累加;`is_hurt_heal=false`。
- **`gen_three_bars`**:返回结构体所有权,不入参 `&mut`。
- **`gen_shield_defence`**:内部固化公式,返回完整 `PropBoundsEffect`。
- **`load_shield`**:纯编排(upsert + push),不含刷新/应用逻辑。
- **cost/cut**:`PropAlterEffect` 直接应用,`try_cost_magicka` 保留不足返回 None。
- ADR-0001 同步合并数组契约。

## Testing Decisions

- **好测试的标准**:公开接口「构造公开输入 → 调用 `pub fn` → 断言可观察行为」,不碰私有字段。
- **被测模块**:`base_lib/eff_attr_prop/prop_alter_eff.rs`、`base_lib/eff_attr_prop/prop_systems.rs`、`common_impl/combats/damages.rs`、`common_impl/combats/combat_units_system.rs`。
- **现有先例**:`damages.rs` 的 `apply_one` 构造链、`attr_systems.rs` 的 `real_data_per_entity_tick`、`props.rs` 的 `apply_eff_checked` 测试。
- **硬性要求**:每个新增/改签名的 `pub fn` 至少一个演示测试;`cargo test --lib` 全绿;`cargo clippy --all-targets` 新文件无警告;`cargo fmt --check` 通过。
- **建议测试**:
  - `prop_alter_eff`:Val/CurPer/MaxPer 三种折算(参照目标 Prop 当前/最大值);直接应用返回实际生效值。
  - `prop_systems`:`try_refresh_dirty_prop_bounds` 依脏标签刷新上限并钳制当前值;无脏不动作。
  - `damages`:`Only*` 装载单目标累加、`is_hurt_heal=false`、`calc_damage_scale` 跳过能量缩放、合并数组扩到 9。
  - `combat_units_system`:`gen_three_bars` 返回结构体;`gen_shield_defence` 返回就绪 `PropBoundsEffect`;`load_shield` 只 upsert+push(不调 refresh/apply);cost/cut 用 `PropAlterEffect` 表达百分比。

## Out of Scope

- 不实现 `combat_units.rs`「todo 后续系统性实现」(周期性效果、恢复机制)。
- 不实现奥术/替身护盾的数值公式(留给数值策划)。
- 不改变伤害公式数值(`calc_health_max`/`calc_magicka_max`/`calc_defence_shield` 行为)。
- 不引入引擎绑定或新增第三方依赖。
- 不为 `Prop` 添加"设当前值"方法(装载用累加语义)。

## Further Notes

- 本次为对 commit 067e24b 的架构返工,git 历史保留初版。
- 返工后 `combat_units.rs` 文档不再罗列已实现功能(用户原则:靠读代码结构)。
- 两条 CONTEXT 约定(负数入参、完整 Effect)继续生效,不因返工改动。
