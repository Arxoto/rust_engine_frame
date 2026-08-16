# Spec: 接口演示测试(等级 B)

Status: ready-for-agent

## Problem Statement

README「项目介绍」声明:"部分方法可能没有在本项目内被直接调用,但是会有文档测试或单元测试演示如何使用。" CLAUDE.md「Testing」第一条约定要求**每个 `pub fn` 都由 doc test 或单元测试演示**,即使仓库内无调用方。

当前演示覆盖存在大面积空缺:

| 区域 | 现状 |
|---|---|
| `base_lib/cores/timers/` 8 个模块 | 仅 1 个测试,且断言的是浮点常量(`f64::MAX + 0.1`)而非计时器行为;`elapsed/remaining/progress/reset/complete`、`fix_timeline_diff`、few-shot 边界、暂停干预全部无演示 |
| `common_impl/combats/` 整棵树 | 0 测试;平衡公式(受力上限/伤害成长)只活在注释里 |
| `base_lib/eff_attr_prop/` | `props.rs`、`prop_bounds_eff.rs`、`effects.rs` 0 测试 |
| `base_lib/motions/` | `actions.rs`、`player_controller.rs` 0 测试 |
| `attr_systems.rs` 组合 | 唯一组合测试跑的是**空切片**,"每实体 tick"从未用真实数据演示 |
| Bevy 集成测试 | hello-world 样板,未驱动任何框架逻辑,"无引擎依赖"承诺未被验证 |

## Solution

按"构造输入 → 调用 `pub fn` → 断言可观察行为"补齐演示测试,覆盖所有公开接口;并在 Bevy 侧增加一份驱动真实框架逻辑的集成测试,验证"同一套 `base_lib` 逻辑不依赖引擎即可运行"。

测试即说明书:补测试的过程会暴露文档与实现的偏差(如 `props::apply_bounds` 的案例),此类偏差随测试一并修正。

## User Stories

1. As 一个游戏上层调用方, I want 每个计时器 trait(`elapsed/remaining/progress/reset/complete`、暂停/恢复)有测试演示, so that 我可以照示例使用计时器。
2. As 一个游戏上层调用方, I want few-shot 边界语义(第 limit+1 次触发被静默丢弃)有测试钉住, so that 我理解超发行为且不被"未到时间/额度耗尽"的歧义坑到。
3. As 一个游戏上层调用方, I want `PausePrefab` 对 `tick` 的干预有测试, so that 我了解冻结效果如何生效。
4. As 一个游戏上层调用方, I want `StaticTimer`/`StaticTimeline` 的生命周期(`reset_timeline_and_get_diff`/`fix_timeline_diff`)有测试, so that 我可以安全地做长期计时。
5. As 一个游戏上层调用方, I want 伤害平衡公式(`merge_damages`/`apply_damages`/`calc_damage_scale`/`calc_health_max` 等)有测试, so that 我可以信任伤害数值。
6. As 一个游戏上层调用方, I want 装备→属性效果链(equip 写入容器 → 刷新进 `Attr::current`)有测试, so that 穿上装备的效果可验证。
7. As 一个游戏上层调用方, I want `props`/`prop_bounds_eff`/`effects` 有测试, so that 资源池与上下限逻辑可验证。
8. As 一个游戏上层调用方, I want `attr_systems` 的组合(clean→refresh_dirty→clean_hole→reset_timeline)用真实实体数据演示, so that 我可以照着拼每实体 tick。
9. As 一个维护者, I want Bevy 侧有一份驱动真实框架逻辑的集成测试, so that "无引擎依赖"承诺被验证。
10. As 一个维护者, I want 新增测试遵循"先业务后 tick"的计时次序约定, so that 行为与项目约定一致。
11. As 一个维护者, I want 每个新增测试可被 `cargo test --lib <模块>::` 独立筛选, so that 失败定位准确。

## Implementation Decisions

- **测试通过公开接口驱动,不因可测性暴露内部字段。**
- **timers**:为 `TickTimer`/`StaticTimer`/`InfiniteTickTrigger`/`FewShotStaticTrigger` 等补齐演示测试;断言行为(进度、边界、暂停),而非浮点常量。
- **combats**:为 `damages` 管线(合并次序、护盾命中次序、致死判定)与装备→属性效果链写演示测试 —— 前提是等级 A 的公开构造路径就绪。
- **props / prop_bounds_eff / effects**:补齐单元测试,覆盖 `apply_eff`/`apply_eff_checked`/`refresh_bounds`/`current_is_zero` 与 `PropBoundsEffect` 的上下限映射。
- **attr_systems**:用真实(非空)实体数据扩展 `example_process_tick` 或新增组合测试,覆盖 `try_reset_timeline` 与时间线生命周期。
- **Bevy 集成测试**:新增一份把 `base_lib` 真实逻辑(如属性效果/计时器)挂到 Bevy System 上的测试,替代纯 hello-world 样板。
- 沿用现有 `#[cfg(test)]` 内联测试与 `tests/` 集成测试结构;计时次序遵循"先业务后 tick"(帮助类计时器后 tick、限制类先 tick)。

## Testing Decisions

- **好测试的标准**:演示用法 + 断言可观察行为。优先 doc test(作为文档的一部分),复杂流程用内联单元测试。测试只测外部行为,不测实现细节。
- **被测模块**:timers 全部、combats(damages/equips/combat_units)、props、prop_bounds_eff、effects、attr_systems、actions、player_controller。
- **现有先例**:`eff_attr_prop` 内联测试(attr_eff/upsert_container)、`motions/controllers.rs` 与 `behaviours.rs` 的"输入 → 输出"测试、`tests/` 集成测试结构。
- **硬性要求**:每个新增测试可独立通过筛选命令运行;加深后测试数量只增不减(等级 C 依赖本 spec 的测试作为回归基线)。

## Out of Scope

- 不引入 mock 框架或新测试依赖(除非确有必要的例外,需在 Comments 记录)。
- 不重写既有业务逻辑,只补演示与钉住行为。
- 等级 A 未完成的构造路径,其对应演示测试不在本 spec 强制要求(A 完成即自动触发)。
- 不设计新的测试抽象层。

## Further Notes

- 本 spec 依赖等级 A 就绪的公开构造路径,建议 A 完成后再大规模补测试。
- 补测试是找出"文档承诺与实现偏差"的最有效手段(已发现 `props::apply_bounds` 一例),发现即修正文档。
- 与等级 A、C 共享同一测试接缝:"crate 公开接口 + 进程内测试"。
