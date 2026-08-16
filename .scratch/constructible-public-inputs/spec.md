# Spec: 可构造的公开输入(等级 A)

Status: ready-for-agent

## Problem Statement

本项目定位为"无游戏引擎依赖"的游戏通用底层开发框架,游戏业务逻辑由上层调用 `pub fn` 来驱动(见 README「项目介绍」)。但当前部分 `pub fn` 的入参类型**无法构造**——字段私有、无公开构造器——导致上层根本无法调用它们,接口成为虚假承诺。

这直接违反两条新确立的规范:

- README「项目介绍」:"游戏业务逻辑通过上层调用 `pub fn` 来驱动"。
- CLAUDE.md「Testing」第一条约定:"An unconstructible public input type counts as a violation: a `pub fn` whose parameters cannot be built is an interface an upper layer cannot call — public types feeding `pub fn`s need a public construction path."

具体违规点:

| 公开接口 | 问题 |
|---|---|
| `damages::merge_damages` / `apply_damages` | 入参 `DamageEffect`、`DamageEffectBuffer` 字段私有、无构造器,伤害管线无法被上层喂入输入 |
| `actions::ActionSwitcher::register_action` | 入参 `ActionData` 字段私有、无构造器,`register_action` 是死 API |
| `player_controller::PlayerCharacterController` | 无构造器、无只读访问,连"演示如何使用"都做不到 |
| `props::Prop` 文档承诺的工作流 | 文档指导上层执行 `refresh_bounds` → `apply_bounds`,但 `apply_bounds` 是私有的,承诺的公开序列不可达 |

## Solution

为所有公开 `pub fn` 的入参提供**公开构造路径**,使上层能:构造输入 → 调用 `pub fn` → 观察结果。并用 doc test 或单元测试演示这条路径(即"接口即测试面"接缝)。

补构造路径**不改动实现行为**(不碰伤害公式、动作切换逻辑),只打通入口。

## User Stories

1. As 一个游戏上层调用方, I want `DamageEffect` / `DamageEffectBuffer` 有公开构造路径, so that 我可以调用 `merge_damages` / `apply_damages` 处理每帧伤害。
2. As 一个游戏上层调用方, I want `ActionData` 有公开构造路径(`new` 或 builder,覆盖 id/priority/order/进入条件/状态标签), so that 我可以调用 `register_action` 注册动作。
3. As 一个游戏上层调用方, I want `PlayerCharacterController` 可构造(基于 `PlayerInput`), so that 我可以把它作为游戏内角色的控制样例直接使用。
4. As 一个游戏上层调用方, I want 文档承诺的 `Prop` 工作流(`refresh_bounds` → `apply_bounds`)可执行, so that 我可以按文档重算并钳制资源上下限。
5. As 一个游戏上层调用方, I want 每个补了构造路径的 `pub fn` 配有 doc test 或单元测试演示用法, so that 我可以照测试示例调用。
6. As 一个维护者, I want 不可构造的公开类型被规范与测试拦截, so that 未来不再产生死接口。

## Implementation Decisions

- **构造路径以公开 API 形式提供,不引入新抽象**:为 `DamageEffect`/`DamageEffectBuffer` 增加公开构造函数或 builder;字段公开或私有按需而定。
- **`ActionData`** 增加公开构造函数(或 builder),包含 `id`、`priority`、进入条件 `TinyTag`、`state_tags`;`order`(注册顺序)由 `register_action` 注册时自动赋值,无需作为入参。
- **`PlayerCharacterController`** 增加公开构造器(入参为 `PlayerInput`)与必要的只读访问;若保持黑盒,需提供可演示的公开方法。
- **`Prop`** 将 `apply_bounds` 提升为公开方法(或提供等价的公开 re-clamp 方法),并修正文档,使承诺的序列真正可执行。
- **不改实现行为**:伤害公式、动作切换逻辑、属性计算保持不变,仅打通入口。
- **共享测试接缝**:crate 公开接口 + 进程内 doc/单元测试,不依赖任何引擎。

## Testing Decisions

- **好测试的标准**:只测外部可观察行为 —— 通过公开构造路径构造输入,调用 `pub fn`,断言返回值/状态变化;不访问私有字段,不触及内部实现。
- **被测模块**:`common_impl/combats/damages.rs`、`base_lib/motions/actions.rs`、`base_lib/motions/player_controller.rs`、`base_lib/eff_attr_prop/props.rs`。
- **现有先例**:`eff_attr_prop` 内大量内联 `#[cfg(test)]` 测试(attr_eff 15 个、upsert_container 14 个)就是"构造 → 调用 → 断言"模式的先例;doc test 可参考 `base_lib/cores/unify_types.rs`。
- **硬性要求**:每个被补构造路径的 `pub fn` 至少配一个演示测试,且可被 `cargo test --lib <模块>::` 独立筛选运行。

## Out of Scope

- 不实现 `combat_units.rs` 的 roadmap(三维初始化、护盾生成、`try_cost_magicka`、削韧 `cut_stamina`)。
- 不实现 `motions` 的 action→behaviour 聚合。
- 不改动计时器 trait 体系(属等级 C)。
- 不引入引擎绑定或新增第三方依赖。
- 不为"可测试性"修改内部实现结构。

## Further Notes

- 本 spec 是等级 B 的门票:先让接口可构造,等级 B 才能写演示测试。
- 补构造路径过程中若发现文档与实现偏差(如 `apply_bounds`),顺手修正文档,并在 spec 的 Comments 中记录。
- 与等级 B、C 共享同一测试接缝:"crate 公开接口 + 进程内测试"。
