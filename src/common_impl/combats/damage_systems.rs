//! ## 衡量伤害公式的平衡性
//!
//! - 随着角色成长，【伤害成长】应该与【受伤上限】大致成正比
//! - 伤害公式中各个属性的根源属性应该合理分配，避免某一属性影响力过大
//!
//! ## 受伤上限
//!
//! 【受伤上限】 本质即 【生命值和护盾值】 的组合
//!
//! - 生命值 [`Health`]
//!   - 直接正相关 [`Strength`] see [`calc_health_max`]
//! - 替身护盾 [`ShieldSubstitute`]
//!   - 直接正相关 [`Belief`] todo 信念超过阈值才能激发替身护盾
//! - 防护护盾 [`ShieldDefence`]
//!   - 直接正相关 [`ArmorHard`] see [`calc_defence_shield`]
//!   - 间接正相关 [`Strength`] todo 数值上盔甲坚韧与质量呈正相关，气力决定可穿戴质量，因此可以近似取代
//! - 奥术护盾 [`ShieldArcane`]
//!   - 直接正相关 [`Belief`] todo
//!
//! 不同 【伤害类型】 [`SurvivalEffTargets`] 对应的 【生命值和护盾值】 see [`SurvivalEffTargets`]
//!
//! ## 伤害成长
//!
//! 不同 【伤害类型】 [`SurvivalEffTargets`] 对应的 【伤害缩放】 see [`calc_damage_scale`]
//!
//! ## 平衡性分析
//!
//! 从“玩家受击角度”进行数值平衡分析
//! （根据伤害类型找到对应的 【生命值和护盾值】 、再找到相关的成长属性，对比伤害成长来源，二者是否能相互抵消）
//!
//! - 真实伤害 [`SurvivalEffTargets::OnlyHealth`]
//!   - 受伤上限 正相关 [`Strength`]
//!   - 伤害成长 正相关 [`Strength`] or [`Belief`] （招式固有属性，不缩放，与角色收获相关，使用内禀属性代替）
//!   - 对于 [`Strength`] 成长是平衡的
//!   - 对于 [`Belief`] 成长【受击者不利】，算作差异性，不在此系统弥补
//!   - 由于其不平衡性，应注意避免数值膨胀，并在其他机制弥补，如：替死法术、冲击韧性机制、远程拉扯等
//! - 物理冲击 [`SurvivalEffTargets::PhysicsImpact`]
//!   - 受伤上限 正相关 [`Strength`] + [`Belief`]
//!   - 伤害成长 正相关 [`Strength`] （ [`WeaponMass`] 和 [`ArmorSoft`] 均为武器盔甲固有属性，设计边际递减）
//!   - 对于 [`Strength`] 成长是平衡的
//!   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
//! - 物理剪切 [`SurvivalEffTargets::PhysicsShears`]
//!   - 受伤上限 正相关 [`Strength`] * 2 + [`Belief`]
//!   - 伤害成长 正相关 [`Strength`] （ [`WeaponSharp`] 为武器固有属性，设计边际递减）
//!   - 对于 [`Strength`] 成长是平衡的
//!   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
//! - 魔法奥术 [`SurvivalEffTargets::MagickaArcane`]
//!   - 受伤上限 正相关 [`Belief`] * 2 + [`Strength`]
//!   - 伤害成长 正相关 [`Belief`]
//!   - 对于 [`Strength`] 成长【攻击者不利】，可令武器附带该类伤害
//!   - 对于 [`Belief`] 成长是平衡的

use crate::{
    base_lib::{
        cores::unify_types::{FLOAT_DEAD_ZONE, FixedName},
        eff_attr::{
            attr_layers::{AttrLayerEffTarget, AttrLayerEffTargetIter},
            bound_attrs::BoundRange,
            effects::Effect,
            modifier_collections::ModifiableAttr,
        },
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        damages::{
            DamageInfo, Health, HealthLower, HealthUpper, ShieldArcane, ShieldArcaneUpper,
            ShieldDefence, ShieldDefenceUpper, ShieldSubstitute, ShieldSubstituteUpper,
            SurvivalAttrEff, SurvivalAttrLayer, SurvivalEffBufferOld, SurvivalEffTargets,
        },
        energies::Magicka,
        energy_systems,
    },
};

use super::*;

const BOUNDED_ATTR_LOWER: f64 = 0.0;

#[derive(Debug)]
pub struct MergedSurvivalEffs<S: FixedName> {
    only_heal: Option<Effect<S>>,
    only_sub: Option<Effect<S>>,
    only_def: Option<Effect<S>>,
    only_arc: Option<Effect<S>>,
    phy_imp: Option<Effect<S>>,
    phy_she: Option<Effect<S>>,
    mgk_arc: Option<Effect<S>>,
}

impl<S: FixedName> Default for MergedSurvivalEffs<S> {
    fn default() -> Self {
        Self {
            only_heal: None,
            only_sub: None,
            only_def: None,
            only_arc: None,
            phy_imp: None,
            phy_she: None,
            mgk_arc: None,
        }
    }
}

type MergedSurvivalEffArray<S> = [(SurvivalEffTargets, Option<Effect<S>>); 7];

impl<S: FixedName> MergedSurvivalEffs<S> {
    /// 顺序与 [`crate::base_lib::eff_attr::attr_layers::attr_layer_system::rank_attr_layer_eff`] 一样
    ///
    /// check see `tests::check_survival_eff_slice`
    pub fn into_slice(self) -> MergedSurvivalEffArray<S> {
        [
            (SurvivalEffTargets::OnlyShieldDefence, self.only_def),
            (SurvivalEffTargets::OnlyShieldArcane, self.only_arc),
            (SurvivalEffTargets::OnlyShieldSubstitute, self.only_sub),
            (SurvivalEffTargets::PhysicsShears, self.phy_she),
            (SurvivalEffTargets::MagickaArcane, self.mgk_arc),
            (SurvivalEffTargets::PhysicsImpact, self.phy_imp),
            (SurvivalEffTargets::OnlyHealth, self.only_heal),
        ]
    }
}

pub struct DamageTargetAttrs<'a> {
    pub target_heal: &'a Health,
    pub target_shield_sub: &'a ShieldSubstitute,
    pub target_shield_def: &'a ShieldDefence,
    pub target_shield_arc: &'a ShieldArcane,
    pub target_heal_upper: &'a HealthUpper,
    pub target_shield_sub_upper: &'a ShieldSubstituteUpper,
    pub target_shield_def_upper: &'a ShieldDefenceUpper,
    pub target_shield_arc_upper: &'a ShieldArcaneUpper,
}

/// 每帧计算伤害前都先进行同类合并
///
/// 合并方便伤害计算，具体原因如下
/// - 若先【物理伤害】，后【破盾伤害】，那么当两者加起来能够破盾时，实际伤害与顺序有关
/// - 【物理伤害】在前会导致后面的【破盾伤害】无效化
///
/// 详细探讨见 [`crate::base_lib::eff_attr::attr_systems`]
pub fn merge_damages<S: FixedName>(
    survival_eff_buffer: &mut SurvivalEffBufferOld<S>,
    damage_target_attrs: DamageTargetAttrs,
) -> MergedSurvivalEffs<S> {
    let DamageTargetAttrs {
        target_heal,
        target_shield_sub,
        target_shield_def,
        target_shield_arc,
        target_heal_upper,
        target_shield_sub_upper,
        target_shield_def_upper,
        target_shield_arc_upper,
    } = damage_target_attrs;

    let mut merged_svv_effs = MergedSurvivalEffs::<S>::default();

    // get the ownership
    for dmg_eff in survival_eff_buffer.0.drain(..) {
        let SurvivalAttrEff {
            target_type,
            alter_eff,
        } = dmg_eff;

        // 根据伤害类型找到百分比参照物
        let (base_bounded, base_bound) = match target_type.stop_at() {
            SurvivalAttrLayer::Health => (&target_heal.0, &target_heal_upper.0),
            SurvivalAttrLayer::ShieldSubstitute => {
                (&target_shield_sub.0, &target_shield_sub_upper.0)
            }
            SurvivalAttrLayer::ShieldDefence => (&target_shield_def.0, &target_shield_def_upper.0),
            SurvivalAttrLayer::ShieldArcane => (&target_shield_arc.0, &target_shield_arc_upper.0),
        };

        // 根据伤害算法计算伤害绝对值
        let abs_eff_val = alter_eff.calc_alter_val(base_bounded, base_bound);

        // 根据伤害类型找到聚合对象
        let merged_dmg = match target_type {
            SurvivalEffTargets::OnlyHealth => &mut merged_svv_effs.only_heal,
            SurvivalEffTargets::OnlyShieldSubstitute => &mut merged_svv_effs.only_sub,
            SurvivalEffTargets::OnlyShieldDefence => &mut merged_svv_effs.only_def,
            SurvivalEffTargets::OnlyShieldArcane => &mut merged_svv_effs.only_arc,
            SurvivalEffTargets::PhysicsImpact => &mut merged_svv_effs.phy_imp,
            SurvivalEffTargets::PhysicsShears => &mut merged_svv_effs.phy_she,
            SurvivalEffTargets::MagickaArcane => &mut merged_svv_effs.mgk_arc,
        };

        // 累加绝对值
        if let Some(merged_dmg) = merged_dmg {
            merged_dmg.set_effect_value(merged_dmg.get_effect_value() + abs_eff_val);
        } else {
            let mut eff = alter_eff.take_eff();
            eff.set_effect_value(abs_eff_val);
            *merged_dmg = Some(eff);
        }
    }

    merged_svv_effs
}

pub struct DamageCalcAttrs<'a> {
    pub source_strength: &'a Strength,
    pub source_belief: &'a Belief,
    pub source_magicka: &'a Magicka,
    pub source_weapon_sharp: &'a WeaponSharp,
    pub source_weapon_mass: &'a WeaponMass,
    pub target_armor_soft: &'a ArmorSoft,
}

pub struct DamageTargetMutAttrs<'a> {
    pub target_heal: &'a mut Health,
    pub target_shield_sub: &'a mut ShieldSubstitute,
    pub target_shield_def: &'a mut ShieldDefence,
    pub target_shield_arc: &'a mut ShieldArcane,
    pub target_heal_upper: &'a HealthUpper,
    pub target_heal_lower: &'a HealthLower,
    pub target_shield_sub_upper: &'a ShieldSubstituteUpper,
    pub target_shield_def_upper: &'a ShieldDefenceUpper,
    pub target_shield_arc_upper: &'a ShieldArcaneUpper,
}

/// 对合并后的伤害效果计算伤害
pub fn apply_damages<S: FixedName>(
    merged_svv_effs: MergedSurvivalEffs<S>,
    damage_calc_attrs: DamageCalcAttrs,
    damage_target_attrs: DamageTargetMutAttrs,
) -> DamageInfo<S> {
    let DamageTargetMutAttrs {
        target_heal,
        target_shield_sub,
        target_shield_def,
        target_shield_arc,
        target_heal_upper,
        target_heal_lower,
        target_shield_sub_upper,
        target_shield_def_upper,
        target_shield_arc_upper,
    } = damage_target_attrs;

    let mut dmg_info: DamageInfo<S> = DamageInfo::default();
    let svv_effs = merged_svv_effs.into_slice();
    for (svv_eff_target, dmg_eff) in svv_effs {
        if let Some(mut dmg_eff) = dmg_eff {
            // 根据伤害类型计算缩放比例
            // todo 这一步移到伤害合并处
            let dmg_scale = damage_systems::calc_damage_scale(svv_eff_target, &damage_calc_attrs);

            let mut real_dmg = dmg_scale * dmg_eff.get_effect_value();
            dmg_eff.set_effect_value(real_dmg); // 更新为实际伤害
            let mut is_hurt_heal = false;
            let targets_iter = AttrLayerEffTargetIter::from(svv_eff_target);
            for svv_layer in targets_iter {
                let (attr, upper, lower) = match svv_layer {
                    SurvivalAttrLayer::Health => (
                        &mut target_heal.0,
                        target_heal_upper.0.get_current(),
                        target_heal_lower.0.get_current(),
                    ),
                    SurvivalAttrLayer::ShieldSubstitute => (
                        &mut target_shield_sub.0,
                        target_shield_sub_upper.0.get_current(),
                        BOUNDED_ATTR_LOWER,
                    ),
                    SurvivalAttrLayer::ShieldDefence => (
                        &mut target_shield_def.0,
                        target_shield_def_upper.0.get_current(),
                        BOUNDED_ATTR_LOWER,
                    ),
                    SurvivalAttrLayer::ShieldArcane => (
                        &mut target_shield_arc.0,
                        target_shield_arc_upper.0.get_current(),
                        BOUNDED_ATTR_LOWER,
                    ),
                };
                let old_val = attr.get_pending_value();
                attr.apply_alter(real_dmg);
                attr.clamp_by(BoundRange::new(lower, upper));
                let diff_val = attr.get_pending_value() - old_val;
                real_dmg -= diff_val;

                // todo 优化成匹配最后一个
                // 实际伤害到了生命值
                if matches!(svv_layer, SurvivalAttrLayer::Health) && diff_val < -FLOAT_DEAD_ZONE {
                    is_hurt_heal = true;
                }
            }

            if is_hurt_heal {
                // todo 由于这里是合并后的伤害，因此这里计算得到的不准
                if let Some(hurt_by) = dmg_info.max_hurt_heal_eff.as_mut() {
                    // 伤害是负数 取最小值
                    if dmg_eff.get_effect_value() < hurt_by.get_effect_value() {
                        *hurt_by = dmg_eff;
                    }
                } else {
                    dmg_info.max_hurt_heal_eff = Some(dmg_eff);
                }
            }
        }
    }

    dmg_info
}

/// 伤害缩放
///
/// ## 设定
///
/// - 真实伤害 [`SurvivalEffTargets::OnlyHealth`] 或护盾专精
///   - 不缩放
/// - 物理冲击 [`SurvivalEffTargets::PhysicsImpact`]
///   - 直接正相关 ([`Strength`] + [`WeaponMass`]) / [`ArmorSoft`]
///   - 同时正相关 [`Magicka`]
/// - 物理剪切 [`SurvivalEffTargets::PhysicsShears`]
///   - 直接正相关 [`Strength`] * [`WeaponSharp`]
///   - 同时正相关 [`Magicka`]
/// - 魔法奥术 [`SurvivalEffTargets::MagickaArcane`]
///   - 直接正相关 [`Belief`]
///   - 同时正相关 [`Magicka`]
pub(super) fn calc_damage_scale(
    svv_eff_target: SurvivalEffTargets,
    damage_calc_attrs: &DamageCalcAttrs,
) -> f64 {
    let DamageCalcAttrs {
        source_strength,
        source_belief,
        source_magicka,
        source_weapon_sharp,
        source_weapon_mass,
        target_armor_soft,
    } = damage_calc_attrs;

    // 真实伤害与护盾专精不受能量加成
    let damage_scale = match svv_eff_target {
        SurvivalEffTargets::OnlyHealth
        | SurvivalEffTargets::OnlyShieldSubstitute
        | SurvivalEffTargets::OnlyShieldDefence
        | SurvivalEffTargets::OnlyShieldArcane => {
            return 1.0;
        }
        SurvivalEffTargets::PhysicsImpact => {
            (source_strength.0.get_current() + source_weapon_mass.0.get_current())
                / target_armor_soft.0.get_current()
        }
        SurvivalEffTargets::PhysicsShears => {
            source_strength.0.get_current() * source_weapon_sharp.0.get_current()
        }
        SurvivalEffTargets::MagickaArcane => source_belief.0.get_current(),
    };

    // 尽量保证增幅
    // 能量越高伤害越高 不使用双方能量差是为了防止在高能量状态下，小怪低能量形成的碾压，导致堆怪没威胁
    let scale_by_magicka = 0.0_f64.max(1.0 + energy_systems::calc_magicka_scale(source_magicka));

    damage_scale * scale_by_magicka
}

/// [`Strength`] 影响 [`Health`]
#[inline]
pub fn calc_health_max(health_base: f64, health_scale: f64, strength: &Strength) -> f64 {
    health_base + health_scale * strength.0.get_origin()
}

/// [`ArmorHard`] 影响 [`ShieldDefence`]
#[inline]
pub fn calc_defence_shield(armor_hard: &ArmorHard) -> f64 {
    armor_hard.0.get_current()
}

#[cfg(test)]
mod tests {
    use crate::base_lib::eff_attr::{
        bound_attrs::BoundAttr, bounded_attr_effs::AttrAlterEffType, bounded_attrs::BoundedAttr,
        effects::EffId, stat_attrs::StatAttr,
    };

    use super::*;

    // region: 测试脚手架

    /// 目标四资源
    struct Targets {
        heal: Health,
        sub: ShieldSubstitute,
        def: ShieldDefence,
        arc: ShieldArcane,
        heal_upper: HealthUpper,
        heal_lower: HealthLower,
        sub_upper: ShieldSubstituteUpper,
        def_upper: ShieldDefenceUpper,
        arc_upper: ShieldArcaneUpper,
    }

    impl Targets {
        /// 四资源均满值 100/100
        fn full() -> Self {
            Self {
                heal: Health(BoundedAttr::new(100.0)),
                sub: ShieldSubstitute(BoundedAttr::new(100.0)),
                def: ShieldDefence(BoundedAttr::new(100.0)),
                arc: ShieldArcane(BoundedAttr::new(100.0)),
                heal_upper: HealthUpper(BoundAttr::new(100.0)),
                heal_lower: HealthLower(BoundAttr::new(0.0)),
                sub_upper: ShieldSubstituteUpper(BoundAttr::new(100.0)),
                def_upper: ShieldDefenceUpper(BoundAttr::new(100.0)),
                arc_upper: ShieldArcaneUpper(BoundAttr::new(100.0)),
            }
        }

        fn as_dmg_target_attrs(&self) -> DamageTargetAttrs<'_> {
            DamageTargetAttrs {
                target_heal: &self.heal,
                target_shield_sub: &self.sub,
                target_shield_def: &self.def,
                target_shield_arc: &self.arc,
                target_heal_upper: &self.heal_upper,
                target_shield_sub_upper: &self.sub_upper,
                target_shield_def_upper: &self.def_upper,
                target_shield_arc_upper: &self.arc_upper,
            }
        }

        fn as_dmg_target_mut_attrs(&mut self) -> DamageTargetMutAttrs<'_> {
            DamageTargetMutAttrs {
                target_heal: &mut self.heal,
                target_shield_sub: &mut self.sub,
                target_shield_def: &mut self.def,
                target_shield_arc: &mut self.arc,
                target_heal_upper: &self.heal_upper,
                target_heal_lower: &self.heal_lower,
                target_shield_sub_upper: &self.sub_upper,
                target_shield_def_upper: &self.def_upper,
                target_shield_arc_upper: &self.arc_upper,
            }
        }
    }

    /// 攻击方/目标属性组合
    struct TestAttrs {
        strength: Strength,
        belief: Belief,
        magicka: Magicka,
        weapon_sharp: WeaponSharp,
        weapon_mass: WeaponMass,
        armor_soft: ArmorSoft,
    }

    impl TestAttrs {
        /// 气力/信念/锋利/质量 = 1，柔韧 = 2，能量 = 0
        ///
        /// → 物理剪切 1*1、物理冲击 (1+1)/2、魔法奥术 1，能量系数 = 1，三种复合伤害缩放均为 1
        fn scale_one() -> Self {
            Self {
                strength: Strength(StatAttr::new(1.0)),
                belief: Belief(StatAttr::new(1.0)),
                magicka: Magicka(BoundedAttr::new(0.0)),
                weapon_sharp: WeaponSharp(StatAttr::new(1.0)),
                weapon_mass: WeaponMass(StatAttr::new(1.0)),
                armor_soft: ArmorSoft(StatAttr::new(2.0)),
            }
        }

        fn as_dmg_calc_attrs(&self) -> DamageCalcAttrs<'_> {
            DamageCalcAttrs {
                source_strength: &self.strength,
                source_belief: &self.belief,
                source_magicka: &self.magicka,
                source_weapon_sharp: &self.weapon_sharp,
                source_weapon_mass: &self.weapon_mass,
                target_armor_soft: &self.armor_soft,
            }
        }
    }

    /// 一次「push → merge → apply」完整走查，返回 HealthInfo
    fn run_damage(
        buffer: &mut SurvivalEffBufferOld<&'static str>,
        targets: &mut Targets,
        attrs: &TestAttrs,
    ) -> DamageInfo<&'static str> {
        let merged = merge_damages(buffer, targets.as_dmg_target_attrs());
        apply_damages(
            merged,
            attrs.as_dmg_calc_attrs(),
            targets.as_dmg_target_mut_attrs(),
        )
    }

    // endregion

    // region: SurvivalEffTargets 方法（基于文档注释）

    /// target_types：各伤害类型的目标资源与文档一致
    #[test]
    fn target_types_match_documented_targets() {
        // 单资源类型：文档「仅作用于」对应单一资源
        assert_target_types(SurvivalEffTargets::OnlyHealth, &[SurvivalAttrLayer::Health]);
        assert_target_types(
            SurvivalEffTargets::OnlyShieldSubstitute,
            &[SurvivalAttrLayer::ShieldSubstitute],
        );
        assert_target_types(
            SurvivalEffTargets::OnlyShieldDefence,
            &[SurvivalAttrLayer::ShieldDefence],
        );
        assert_target_types(
            SurvivalEffTargets::OnlyShieldArcane,
            &[SurvivalAttrLayer::ShieldArcane],
        );

        // 复合类型：文档列出的受伤上限 —— 剪切伤 Def/Sub/Health、冲击伤 Sub/Health、奥术伤 Arc/Sub/Health
        assert_target_types(
            SurvivalEffTargets::PhysicsShears,
            &[
                SurvivalAttrLayer::ShieldDefence,
                SurvivalAttrLayer::ShieldSubstitute,
                SurvivalAttrLayer::Health,
            ],
        );
        assert_target_types(
            SurvivalEffTargets::PhysicsImpact,
            &[
                SurvivalAttrLayer::ShieldSubstitute,
                SurvivalAttrLayer::Health,
            ],
        );
        assert_target_types(
            SurvivalEffTargets::MagickaArcane,
            &[
                SurvivalAttrLayer::ShieldArcane,
                SurvivalAttrLayer::ShieldSubstitute,
                SurvivalAttrLayer::Health,
            ],
        );
    }

    fn assert_target_types(dmg_type: SurvivalEffTargets, expected: &[SurvivalAttrLayer]) {
        let targets: Vec<_> = AttrLayerEffTargetIter::from(dmg_type).collect();
        assert_eq!(targets.len(), expected.len(), "{dmg_type:?} 目标个数不符");
        for (got, want) in targets.iter().zip(expected) {
            assert_eq!(got, want, "{dmg_type:?} 目标 {got:?} 应为 {want:?}");
        }
    }

    // endregion

    // region: 伤害缩放与数值公式

    /// calc_damage_scale：真实伤害与护盾专精恒为 1.0，不受属性/能量加成
    #[test]
    fn calc_damage_scale_is_constant_one_for_only_types() {
        let attrs = TestAttrs {
            strength: Strength(StatAttr::new(5.0)),
            belief: Belief(StatAttr::new(7.0)),
            magicka: Magicka(BoundedAttr::new(200.0)),
            weapon_sharp: WeaponSharp(StatAttr::new(3.0)),
            weapon_mass: WeaponMass(StatAttr::new(4.0)),
            armor_soft: ArmorSoft(StatAttr::new(1.0)),
        };
        for dmg_type in [
            SurvivalEffTargets::OnlyHealth,
            SurvivalEffTargets::OnlyShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane,
        ] {
            assert_eq!(
                calc_damage_scale(dmg_type, &attrs.as_dmg_calc_attrs()),
                1.0,
                "{dmg_type:?} 缩放应恒为 1.0"
            );
        }
    }

    /// calc_damage_scale：复合伤害公式（能量系数 = 1 + 能量/100，能量基准 100）
    #[test]
    fn calc_damage_scale_composite_formulas() {
        let attrs = TestAttrs {
            strength: Strength(StatAttr::new(2.0)),
            belief: Belief(StatAttr::new(3.0)),
            magicka: Magicka(BoundedAttr::new(100.0)),
            weapon_sharp: WeaponSharp(StatAttr::new(4.0)),
            weapon_mass: WeaponMass(StatAttr::new(5.0)),
            armor_soft: ArmorSoft(StatAttr::new(2.0)),
        };

        let energy_scale = 1.0 + 100.0 / 100.0;
        // 物理剪切 = 气力 * 锋利 * 能量系数
        assert_eq!(
            calc_damage_scale(
                SurvivalEffTargets::PhysicsShears,
                &attrs.as_dmg_calc_attrs()
            ),
            2.0 * 4.0 * energy_scale
        );
        // 物理冲击 = (气力 + 质量) / 柔韧 * 能量系数
        assert_eq!(
            calc_damage_scale(
                SurvivalEffTargets::PhysicsImpact,
                &attrs.as_dmg_calc_attrs()
            ),
            (2.0 + 5.0) / 2.0 * energy_scale
        );
        // 魔法奥术 = 信念 * 能量系数
        assert_eq!(
            calc_damage_scale(
                SurvivalEffTargets::MagickaArcane,
                &attrs.as_dmg_calc_attrs()
            ),
            3.0 * energy_scale
        );
    }

    /// calc_health_max：Strength 影响 Health，health_base + health_scale * strength.origin
    #[test]
    fn calc_health_max_scales_with_strength_origin() {
        let strength = Strength(StatAttr::new(10.0));
        assert_eq!(calc_health_max(100.0, 5.0, &strength), 150.0);
    }

    /// calc_defence_shield：ArmorHard 影响 ShieldDefence，返回 armor_hard 当前值
    #[test]
    fn calc_defence_shield_uses_armor_hard_current() {
        let armor_hard = ArmorHard(StatAttr::new(25.0));
        assert_eq!(calc_defence_shield(&armor_hard), 25.0);
    }

    // endregion

    // region: 伤害计算（各类型在对应属性生效 + 破盾传递）

    /// OnlyHealth：绝对值伤害直接作用于血量
    #[test]
    fn only_health_hits_health_directly() {
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("attacker", "real_dmg", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert!(buffer.is_empty());
        assert_eq!(targets.heal.0.get_pending_value(), 60.0);
        assert_eq!(targets.sub.0.get_pending_value(), 100.0);
        assert_eq!(targets.def.0.get_pending_value(), 100.0);
        assert_eq!(targets.arc.0.get_pending_value(), 100.0);
    }

    /// OnlyHealth：治疗正效果值加回血量，超上限被钳制
    #[test]
    fn only_health_heals_up_to_cap() {
        let mut targets = Targets::full();
        targets.heal.0.apply_alter(-30.0); // 先扣到 70
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("healer", "heal", 40.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.heal.0.get_pending_value(), 100.0); // 70 + 40 = 110 → 钳制到 100
    }

    /// OnlyShield*：只作用于对应护盾，其余资源不变
    #[test]
    fn only_shield_types_hit_only_their_shield() {
        // 替身护盾
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            AttrAlterEffType::Val,
            Effect::new("a", "break_sub", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.sub.0.get_pending_value(), 60.0);
        assert_eq!(targets.heal.0.get_pending_value(), 100.0);
        assert_eq!(targets.def.0.get_pending_value(), 100.0);
        assert_eq!(targets.arc.0.get_pending_value(), 100.0);

        // 防护护盾
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldDefence,
            AttrAlterEffType::Val,
            Effect::new("a", "break_def", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.def.0.get_pending_value(), 60.0);
        assert_eq!(targets.heal.0.get_pending_value(), 100.0);
        assert_eq!(targets.sub.0.get_pending_value(), 100.0);

        // 奥术护盾
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldArcane,
            AttrAlterEffType::Val,
            Effect::new("a", "break_arc", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.arc.0.get_pending_value(), 60.0);
        assert_eq!(targets.heal.0.get_pending_value(), 100.0);
    }

    /// PhysicsShears：防护护盾 → 替身护盾 → 血量，依次穿透
    #[test]
    fn physics_shears_breaks_defence_then_substitute_then_health() {
        let mut targets = Targets {
            heal: Health(BoundedAttr::new(100.0)),
            sub: ShieldSubstitute(BoundedAttr::new(20.0)),
            def: ShieldDefence(BoundedAttr::new(30.0)),
            arc: ShieldArcane(BoundedAttr::new(100.0)),
            heal_upper: HealthUpper(BoundAttr::new(100.0)),
            heal_lower: HealthLower(BoundAttr::new(0.0)),
            sub_upper: ShieldSubstituteUpper(BoundAttr::new(100.0)),
            def_upper: ShieldDefenceUpper(BoundAttr::new(100.0)),
            arc_upper: ShieldArcaneUpper(BoundAttr::new(100.0)),
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsShears,
            AttrAlterEffType::Val,
            Effect::new("a", "shear", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.def.0.get_pending_value(), 0.0); // 吸收 30
        assert_eq!(targets.sub.0.get_pending_value(), 0.0); // 吸收 20
        assert_eq!(targets.heal.0.get_pending_value(), 50.0); // 剩余 50 落血
        assert_eq!(targets.arc.0.get_pending_value(), 100.0); // 奥术护盾不受物理剪切影响
    }

    /// PhysicsImpact：替身护盾 → 血量，依次穿透
    #[test]
    fn physics_impact_breaks_substitute_then_health() {
        let mut targets = Targets {
            heal: Health(BoundedAttr::new(100.0)),
            sub: ShieldSubstitute(BoundedAttr::new(30.0)),
            def: ShieldDefence(BoundedAttr::new(100.0)),
            arc: ShieldArcane(BoundedAttr::new(100.0)),
            heal_upper: HealthUpper(BoundAttr::new(100.0)),
            heal_lower: HealthLower(BoundAttr::new(0.0)),
            sub_upper: ShieldSubstituteUpper(BoundAttr::new(100.0)),
            def_upper: ShieldDefenceUpper(BoundAttr::new(100.0)),
            arc_upper: ShieldArcaneUpper(BoundAttr::new(100.0)),
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsImpact,
            AttrAlterEffType::Val,
            Effect::new("a", "impact", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.sub.0.get_pending_value(), 0.0); // 吸收 30
        assert_eq!(targets.heal.0.get_pending_value(), 30.0); // 剩余 70 落血
        assert_eq!(targets.def.0.get_pending_value(), 100.0); // 防护护盾不受物理冲击影响
    }

    /// MagickaArcane：奥术护盾 → 替身护盾 → 血量，依次穿透
    #[test]
    fn magicka_arcane_breaks_arcane_then_substitute_then_health() {
        let mut targets = Targets {
            heal: Health(BoundedAttr::new(100.0)),
            sub: ShieldSubstitute(BoundedAttr::new(20.0)),
            def: ShieldDefence(BoundedAttr::new(100.0)),
            arc: ShieldArcane(BoundedAttr::new(30.0)),
            heal_upper: HealthUpper(BoundAttr::new(100.0)),
            heal_lower: HealthLower(BoundAttr::new(0.0)),
            sub_upper: ShieldSubstituteUpper(BoundAttr::new(100.0)),
            def_upper: ShieldDefenceUpper(BoundAttr::new(100.0)),
            arc_upper: ShieldArcaneUpper(BoundAttr::new(100.0)),
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::MagickaArcane,
            AttrAlterEffType::Val,
            Effect::new("a", "arcane", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.arc.0.get_pending_value(), 0.0); // 吸收 30
        assert_eq!(targets.sub.0.get_pending_value(), 0.0); // 吸收 20
        assert_eq!(targets.heal.0.get_pending_value(), 50.0); // 剩余 50 落血
        assert_eq!(targets.def.0.get_pending_value(), 100.0); // 防护护盾不受魔法奥术影响
    }

    // endregion

    // region: 同类合并

    /// merge_damages：同类伤害合并后一次结算，缓冲被清空
    #[test]
    fn merge_damages_sums_same_type_and_drains_buffer() {
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("a", "hit1", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("a", "hit2", -20.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert!(buffer.is_empty());
        assert_eq!(targets.heal.0.get_pending_value(), 50.0); // -30 与 -20 合并为 -50
    }

    /// merge_damages：百分比按目标自身折算后与绝对值合并
    #[test]
    fn merge_damages_mixes_absolute_and_percent() {
        let mut targets = Targets::full(); // 血量当前值 100
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("a", "flat", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::CurPer,
            Effect::new("a", "cut", -0.2),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::MaxPer,
            Effect::new("a", "slash", -0.5),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        // -30 + (-0.2*100) + (-0.5*100) = -100
        assert_eq!(targets.heal.0.get_pending_value(), 0.0);
    }

    // endregion

    // region: OnlyShield* 与复合伤害同帧的顺序（避免破盾伤害浪费）

    /// OnlyShieldDefence 与 PhysicsShears 同帧：破盾伤害必须先于复合伤害结算，
    /// 否则 PhysicsShears 先破防护盾，OnlyShieldDefence 会对空盾浪费伤害
    #[test]
    fn combined_only_shield_defence_applies_before_physics_shears() {
        let mut targets = Targets {
            def: ShieldDefence(BoundedAttr::new(50.0)),
            def_upper: ShieldDefenceUpper(BoundAttr::new(100.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldDefence,
            AttrAlterEffType::Val,
            Effect::new("a", "break_def", -100.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsShears,
            AttrAlterEffType::Val,
            Effect::new("a", "shear", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 若顺序颠倒，PhysicsShears 先打掉 50 防护，OnlyShieldDefence 的 -100 全部浪费，
        // 替身只会剩 50；正确顺序下替身被 PhysicsShears 打空、血量无伤
        assert_eq!(targets.def.0.get_pending_value(), 0.0);
        assert_eq!(targets.sub.0.get_pending_value(), 0.0);
        assert_eq!(targets.heal.0.get_pending_value(), 100.0);
    }

    /// OnlyShieldSubstitute 与 PhysicsImpact 同帧：破盾伤害先结算
    #[test]
    fn combined_only_shield_substitute_applies_before_physics_impact() {
        let mut targets = Targets {
            sub: ShieldSubstitute(BoundedAttr::new(50.0)),
            sub_upper: ShieldSubstituteUpper(BoundAttr::new(100.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            AttrAlterEffType::Val,
            Effect::new("a", "break_sub", -100.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsImpact,
            AttrAlterEffType::Val,
            Effect::new("a", "impact", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 错误顺序下 PhysicsImpact 先波及替身与血量（替身 50、血量 50），OnlyShieldSubstitute 浪费；
        // 正确顺序下替身被破盾伤害清空，冲击伤害全部落血
        assert_eq!(targets.sub.0.get_pending_value(), 0.0);
        assert_eq!(targets.heal.0.get_pending_value(), 0.0);
    }

    /// OnlyShieldArcane 与 MagickaArcane 同帧：破盾伤害先结算
    #[test]
    fn combined_only_shield_arcane_applies_before_magicka_arcane() {
        let mut targets = Targets {
            arc: ShieldArcane(BoundedAttr::new(50.0)),
            arc_upper: ShieldArcaneUpper(BoundAttr::new(100.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldArcane,
            AttrAlterEffType::Val,
            Effect::new("a", "break_arc", -100.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::MagickaArcane,
            AttrAlterEffType::Val,
            Effect::new("a", "arcane", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 错误顺序下 MagickaArcane 先波及奥术 50、替身 50、血量 50，OnlyShieldArcane 浪费；
        // 正确顺序下奥术被破盾伤害清空，魔法伤害全部落在替身、血量无伤
        assert_eq!(targets.arc.0.get_pending_value(), 0.0);
        assert_eq!(targets.sub.0.get_pending_value(), 0.0);
        assert_eq!(targets.heal.0.get_pending_value(), 100.0);
    }

    // endregion

    // region: HealthInfo 死因来源

    /// HealthInfo：记录第一个「伤血/治疗」效果作为死因来源；纯护盾伤害不记录
    #[test]
    fn apply_damages_records_first_hurt_heal_source() {
        // 纯护盾伤害 → 无死因来源
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            AttrAlterEffType::Val,
            Effect::new("a", "break_sub", -30.0),
        ));
        let info = run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert!(info.max_hurt_heal_eff.is_none());

        // 混入真实伤害和切割伤害 → 记录真实伤害，切割伤害没有击穿护盾
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            AttrAlterEffType::Val,
            Effect::new("a", "break_sub", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("b", "real_dmg", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsShears,
            AttrAlterEffType::Val,
            Effect::new("b", "shear", -60.0),
        ));
        let info = run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(
            info.max_hurt_heal_eff.unwrap().take_from_eff_name(),
            EffId {
                from_name: "b",
                effect_name: "real_dmg"
            }
        );

        // 混入真实伤害和冲击伤害 → 记录冲击伤害，冲击伤害击穿护盾并且伤害值大于真实伤害
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBufferOld::new();
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            AttrAlterEffType::Val,
            Effect::new("a", "break_sub", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::OnlyHealth,
            AttrAlterEffType::Val,
            Effect::new("b", "real_dmg", -30.0),
        ));
        buffer.push(SurvivalAttrEff::new(
            SurvivalEffTargets::PhysicsImpact,
            AttrAlterEffType::Val,
            Effect::new("b", "impact", -110.0),
        ));
        let info = run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(
            info.max_hurt_heal_eff.unwrap().take_from_eff_name(),
            EffId {
                from_name: "b",
                effect_name: "impact"
            }
        );
    }

    // endregion
}
