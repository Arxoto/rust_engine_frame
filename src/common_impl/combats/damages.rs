use strum_macros::EnumIter;

use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr_prop::{
            effects::Effect,
            multi_prop::MultiPropEffTargets,
            prop_alter_eff::{PropAlterEffect, PropAlterEffectType},
        },
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        combat_units::{
            Health, Magicka, ShieldArcane, ShieldDefence, ShieldSubstitute, SurvivalPropLayer,
        },
    },
};

/// 存放伤害或治疗效果的 buffer
#[derive(Debug)]
pub struct SurvivalEffBuffer<S: FixedName>(Vec<SurvivalPropEffect<S>>);

/// 伤害信息，表示每次伤害造成的影响
#[derive(Debug)]
pub struct DamageInfo<S: FixedName> {
    /// 首次造成伤害的来源和效果名称，用于统计死因
    pub first_hurt_heal_from_eff: Option<(S, S)>,
}

/// 生存类效果（伤害、治疗、护盾）
#[derive(Debug, Clone)]
pub struct SurvivalPropEffect<S: FixedName> {
    /// 伤害类型，伤害针对的哪些目标
    target_type: SurvivalEffTargets,
    /// 伤害生效方式（绝对值或是百分比）
    ///
    /// 与 [`crate::base_lib::eff_attr_prop::prop_alter_eff::PropAlterEffect`] 共享同一计算语义
    alter_type: PropAlterEffectType,
    eff: Effect<S>,
}

impl<S: FixedName> SurvivalPropEffect<S> {
    /// 构造单次伤害效果
    ///
    /// 推入 [`SurvivalEffBuffer`] 后由伤害系统消费。
    #[must_use]
    pub fn new(
        target_type: SurvivalEffTargets,
        alter_type: PropAlterEffectType,
        eff: Effect<S>,
    ) -> Self {
        Self {
            target_type,
            alter_type,
            eff,
        }
    }

    pub fn new_from_alter_eff(target_type: SurvivalEffTargets, eff: PropAlterEffect<S>) -> Self {
        Self {
            target_type,
            alter_type: eff.get_type(),
            eff: eff.own_eff(),
        }
    }
}

impl<S: FixedName> Default for SurvivalEffBuffer<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: FixedName> SurvivalEffBuffer<S> {
    /// 构造空的伤害缓冲
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// 推入一次伤害效果
    pub fn push(&mut self, dmg_eff: SurvivalPropEffect<S>) {
        self.0.push(dmg_eff);
    }

    /// 缓冲内伤害效果数量
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 缓冲是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// 生存效果生效目标
///
/// 百分比伤害计算时选取 `stop_at` 为基础进行计算
///
/// - 真实伤害 [`SurvivalEffTargets::OnlyHealth`]
///   - 伤害 [`Health`]
/// - 物理冲击 [`SurvivalEffTargets::PhysicsImpact`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`]
/// - 物理剪切 [`SurvivalEffTargets::PhysicsShears`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`] & [`ShieldDefence`]
/// - 魔法奥术 [`SurvivalEffTargets::MagickaArcane`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`] & [`ShieldArcane`]
/// - 破盾专精伤害对应护盾
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum SurvivalEffTargets {
    /// 仅作用于生命值，可用作真实伤害或治疗
    OnlyHealth,
    /// 仅作用于替身护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldSubstitute,
    /// 仅作用于防护护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldDefence,
    /// 仅作用于奥术护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldArcane,

    /// 物理冲击 沉重
    PhysicsImpact,
    /// 物理剪切 尖锐
    PhysicsShears,
    /// 魔法奥术
    MagickaArcane,
}

impl MultiPropEffTargets for SurvivalEffTargets {
    type Layer = SurvivalPropLayer;

    fn start_at(&self) -> Self::Layer {
        match self {
            SurvivalEffTargets::OnlyHealth => SurvivalPropLayer::Health,
            SurvivalEffTargets::OnlyShieldSubstitute => SurvivalPropLayer::ShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence => SurvivalPropLayer::ShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane => SurvivalPropLayer::ShieldArcane,
            SurvivalEffTargets::PhysicsImpact => SurvivalPropLayer::ShieldSubstitute,
            SurvivalEffTargets::PhysicsShears => SurvivalPropLayer::ShieldDefence,
            SurvivalEffTargets::MagickaArcane => SurvivalPropLayer::ShieldArcane,
        }
    }

    fn stop_at(&self) -> Self::Layer {
        match self {
            SurvivalEffTargets::OnlyHealth => SurvivalPropLayer::Health,
            SurvivalEffTargets::OnlyShieldSubstitute => SurvivalPropLayer::ShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence => SurvivalPropLayer::ShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane => SurvivalPropLayer::ShieldArcane,
            SurvivalEffTargets::PhysicsImpact => SurvivalPropLayer::Health,
            SurvivalEffTargets::PhysicsShears => SurvivalPropLayer::Health,
            SurvivalEffTargets::MagickaArcane => SurvivalPropLayer::Health,
        }
    }
}

impl SurvivalEffTargets {
    /// 能否对血量造成伤害
    pub fn is_hurt_heal(&self) -> bool {
        self.stop_at() == SurvivalPropLayer::Health
    }
}

impl<S: FixedName> Default for DamageInfo<S> {
    fn default() -> Self {
        Self {
            first_hurt_heal_from_eff: None,
        }
    }
}

pub struct MagickaEnergyLevel(f64, f64, f64);

impl MagickaEnergyLevel {
    pub const fn new(l0: f64, l1: f64, l2: f64) -> MagickaEnergyLevel {
        MagickaEnergyLevel(l0, l1, l2)
    }

    pub fn max_energy(&self, v: f64) -> f64 {
        if v <= self.0 {
            self.0
        } else if v <= self.1 {
            self.1
        } else {
            self.2
        }
    }
}

/// ## 如何衡量伤害公式是否平衡
///
/// - 随着角色成长，【伤害成长】应该与【受伤上限】大致成正比
/// - 伤害公式中各个属性的根源属性应该合理分配，避免某一属性影响力过大
///
/// ## 受伤上限
///
/// 【受伤上限】 本质即 【生命值和护盾值】 的组合
///
/// - 生命值 [`Health`]
///   - 直接正相关 [`Strength`] see [`damage_system::calc_health_max`]
/// - 替身护盾 [`ShieldSubstitute`]
///   - 直接正相关 [`Belief`] todo 信念超过阈值才能激发替身护盾
/// - 防护护盾 [`ShieldDefence`]
///   - 直接正相关 [`ArmorHard`] see [`damage_system::calc_defence_shield`]
///   - 间接正相关 [`Strength`] todo 数值上盔甲坚韧与质量呈正相关，气力决定可穿戴质量，因此可以近似取代
/// - 奥术护盾 [`ShieldArcane`]
///   - 直接正相关 [`Belief`] todo
///
/// 不同 【伤害类型】 [`SurvivalEffTargets`] 对应的 【生命值和护盾值】 see [`SurvivalEffTargets`]
///
/// ## 伤害成长
///
/// 不同 【伤害类型】 [`SurvivalEffTargets`] 对应的 【伤害缩放】 see [`damage_system::calc_damage_scale`]
///
/// ## 平衡性分析
///
/// 从“玩家受击角度”进行数值平衡分析
/// （根据伤害类型找到对应的 【生命值和护盾值】 、再找到相关的成长属性，对比伤害成长来源，二者是否能相互抵消）
///
/// - 真实伤害 [`SurvivalEffTargets::OnlyHealth`]
///   - 受伤上限 正相关 [`Strength`]
///   - 伤害成长 正相关 [`Strength`] or [`Belief`] （招式固有属性，不缩放，与角色收获相关，使用内禀属性代替）
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【受击者不利】，算作差异性，不在此系统弥补
///   - 由于其不平衡性，应注意避免数值膨胀，并在其他机制弥补，如：替死法术、冲击韧性机制、远程拉扯等
/// - 物理冲击 [`SurvivalEffTargets::PhysicsImpact`]
///   - 受伤上限 正相关 [`Strength`] + [`Belief`]
///   - 伤害成长 正相关 [`Strength`] （ [`WeaponMass`] 和 [`ArmorSoft`] 均为武器盔甲固有属性，设计边际递减）
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
/// - 物理剪切 [`SurvivalEffTargets::PhysicsShears`]
///   - 受伤上限 正相关 [`Strength`] * 2 + [`Belief`]
///   - 伤害成长 正相关 [`Strength`] （ [`WeaponSharp`] 为武器固有属性，设计边际递减）
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
/// - 魔法奥术 [`SurvivalEffTargets::MagickaArcane`]
///   - 受伤上限 正相关 [`Belief`] * 2 + [`Strength`]
///   - 伤害成长 正相关 [`Belief`]
///   - 对于 [`Strength`] 成长【攻击者不利】，可令武器附带该类伤害
///   - 对于 [`Belief`] 成长是平衡的
pub mod damage_system {
    use crate::{
        base_lib::eff_attr_prop::multi_prop::MultiPropEffTargetIter,
        common_impl::combats::combat_units::SurvivalPropLayer,
    };

    use super::*;

    const MAGICKA_BASELINE: f64 = 100.0;

    #[derive(Debug)]
    pub struct MergedSurvivalEffs<S: FixedName> {
        dmg_only_heal: Option<Effect<S>>,
        dmg_only_sub: Option<Effect<S>>,
        dmg_only_def: Option<Effect<S>>,
        dmg_only_arc: Option<Effect<S>>,
        dmg_phy_imp: Option<Effect<S>>,
        dmg_phy_she: Option<Effect<S>>,
        dmg_mgk_arc: Option<Effect<S>>,
    }

    impl<S: FixedName> Default for MergedSurvivalEffs<S> {
        fn default() -> Self {
            Self {
                dmg_only_heal: None,
                dmg_only_sub: None,
                dmg_only_def: None,
                dmg_only_arc: None,
                dmg_phy_imp: None,
                dmg_phy_she: None,
                dmg_mgk_arc: None,
            }
        }
    }

    type MergedSurvivalEffArray<S> = [(SurvivalEffTargets, Option<Effect<S>>); 7];

    impl<S: FixedName> MergedSurvivalEffs<S> {
        /// 顺序与 [`crate::base_lib::eff_attr_prop::multi_prop::multi_prop_system::rank_multi_prop_eff`] 一样
        ///
        /// check see `tests::check_survival_eff_slice`
        pub fn into_slice(self) -> MergedSurvivalEffArray<S> {
            [
                (SurvivalEffTargets::OnlyShieldDefence, self.dmg_only_def),
                (SurvivalEffTargets::OnlyShieldArcane, self.dmg_only_arc),
                (SurvivalEffTargets::OnlyShieldSubstitute, self.dmg_only_sub),
                (SurvivalEffTargets::PhysicsShears, self.dmg_phy_she),
                (SurvivalEffTargets::MagickaArcane, self.dmg_mgk_arc),
                (SurvivalEffTargets::PhysicsImpact, self.dmg_phy_imp),
                (SurvivalEffTargets::OnlyHealth, self.dmg_only_heal),
            ]
        }
    }

    /// 每帧计算伤害前都先进行同类合并
    ///
    /// 合并方便伤害计算，具体原因如下
    /// - 若先【物理伤害】，后【破盾伤害】，那么当两者加起来能够破盾时，实际伤害与顺序有关
    /// - 【物理伤害】在前会导致后面的【破盾伤害】无效化
    ///
    /// 详细探讨见 [`crate::base_lib::eff_attr_prop::multi_prop`]
    pub fn merge_damages<S: FixedName>(
        survival_eff_buffer: &mut SurvivalEffBuffer<S>,
        target_health: &Health,
        target_shield_substitute: &ShieldSubstitute,
        target_shield_defence: &ShieldDefence,
        target_shield_arcane: &ShieldArcane,
    ) -> MergedSurvivalEffs<S> {
        let mut merged_survival_effs = MergedSurvivalEffs::<S>::default();

        // get the ownership
        for dmg_eff in survival_eff_buffer.0.drain(0..) {
            let SurvivalPropEffect {
                target_type,
                alter_type,
                mut eff,
            } = dmg_eff;

            // 根据伤害类型找到聚合对象
            let merged_dmg = match target_type {
                SurvivalEffTargets::OnlyHealth => &mut merged_survival_effs.dmg_only_heal,
                SurvivalEffTargets::OnlyShieldSubstitute => &mut merged_survival_effs.dmg_only_sub,
                SurvivalEffTargets::OnlyShieldDefence => &mut merged_survival_effs.dmg_only_def,
                SurvivalEffTargets::OnlyShieldArcane => &mut merged_survival_effs.dmg_only_arc,
                SurvivalEffTargets::PhysicsImpact => &mut merged_survival_effs.dmg_phy_imp,
                SurvivalEffTargets::PhysicsShears => &mut merged_survival_effs.dmg_phy_she,
                SurvivalEffTargets::MagickaArcane => &mut merged_survival_effs.dmg_mgk_arc,
            };

            // 提前获取原始效果值
            let origin_eff_val = eff.get_effect_value();
            // 预处理聚合对象，移走所有权
            if merged_dmg.is_none() {
                eff.set_effect_value(0.0);
                *merged_dmg = Some(eff);
            }

            // 根据伤害类型找到百分比参照物
            let base_prop = match target_type.stop_at() {
                SurvivalPropLayer::Health => &target_health.0,
                SurvivalPropLayer::ShieldSubstitute => &target_shield_substitute.0,
                SurvivalPropLayer::ShieldDefence => &target_shield_defence.0,
                SurvivalPropLayer::ShieldArcane => &target_shield_arcane.0,
            };

            // 根据伤害算法计算伤害绝对值
            let abs_eff_val = alter_type.calc_alter_val(origin_eff_val, base_prop);

            // 累加绝对值
            if let Some(merged_dmg) = merged_dmg {
                merged_dmg.set_effect_value(merged_dmg.get_effect_value() + abs_eff_val);
            }
        }

        merged_survival_effs
    }

    pub struct DamageAppliedAttrProps<'a> {
        pub source_strength: &'a Strength,
        pub source_belief: &'a Belief,
        pub source_magicka: &'a Magicka,
        pub source_weapon_sharp: &'a WeaponSharp,
        pub source_weapon_mass: &'a WeaponMass,
        pub target_armor_soft: &'a ArmorSoft,
    }

    /// 对合并后的伤害效果计算伤害
    pub fn apply_damages<S: FixedName>(
        merged_survival_effs: MergedSurvivalEffs<S>,
        damage_applied_attr_props: DamageAppliedAttrProps,
        target_health: &mut Health,
        target_shield_substitute: &mut ShieldSubstitute,
        target_shield_defence: &mut ShieldDefence,
        target_shield_arcane: &mut ShieldArcane,
    ) -> DamageInfo<S> {
        let DamageAppliedAttrProps {
            source_strength,
            source_belief,
            source_magicka,
            source_weapon_sharp,
            source_weapon_mass,
            target_armor_soft,
        } = damage_applied_attr_props;

        let mut dmg_info: DamageInfo<S> = DamageInfo::default();
        let svv_effs = merged_survival_effs.into_slice();
        for (svv_eff_target, dmg_eff) in svv_effs {
            if let Some(dmg_eff) = dmg_eff {
                // 根据伤害类型计算缩放比例
                let dmg_scale = damage_system::calc_damage_scale(
                    svv_eff_target,
                    source_strength,
                    source_belief,
                    source_magicka,
                    source_weapon_sharp,
                    source_weapon_mass,
                    target_armor_soft,
                );

                let mut real_dmg = dmg_scale * dmg_eff.get_effect_value();
                let targets_iter = MultiPropEffTargetIter::from(svv_eff_target);
                for svv_layer in targets_iter {
                    let prop = match svv_layer {
                        SurvivalPropLayer::Health => &mut target_health.0,
                        SurvivalPropLayer::ShieldSubstitute => &mut target_shield_substitute.0,
                        SurvivalPropLayer::ShieldDefence => &mut target_shield_defence.0,
                        SurvivalPropLayer::ShieldArcane => &mut target_shield_arcane.0,
                    };
                    let res = prop.apply_eff(real_dmg);
                    real_dmg -= res.real_eff_val;
                }

                if dmg_info.first_hurt_heal_from_eff.is_none() && svv_eff_target.is_hurt_heal() {
                    dmg_info.first_hurt_heal_from_eff = Some(dmg_eff.own_from_eff_name());
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
    pub fn calc_damage_scale(
        svv_eff_target: SurvivalEffTargets,
        source_strength: &Strength,
        source_belief: &Belief,
        source_magicka: &Magicka,
        source_weapon_sharp: &WeaponSharp,
        source_weapon_mass: &WeaponMass,
        target_armor_soft: &ArmorSoft,
    ) -> f64 {
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

        // 能量越高伤害越高 不使用双方能量差是为了防止在高能量状态下，小怪低能量形成的碾压，导致堆怪没威胁
        let base_scale = 0.0_f64.max(1.0 + source_magicka.0.get_current() / MAGICKA_BASELINE);

        damage_scale * base_scale
    }

    /// [`Strength`] 影响 [`Health`]
    pub fn calc_health_max(health_base: f64, health_scale: f64, strength: &Strength) -> f64 {
        health_base + health_scale * strength.0.get_origin()
    }

    /// [`Belief`] 影响【原始能量】
    pub fn calc_magicka_value(magicka_base: f64, magicka_scale: f64, belief: &Belief) -> f64 {
        magicka_base + magicka_scale * belief.0.get_origin()
    }

    /// 【原始能量】影响 [`Magicka`]
    pub fn calc_magicka_max(
        magicka_base: f64,
        magicka_scale: f64,
        belief: &Belief,
        magicka_energy_level: &MagickaEnergyLevel,
    ) -> f64 {
        let magicka_value = calc_magicka_value(magicka_base, magicka_scale, belief);
        magicka_energy_level.max_energy(magicka_value)
    }

    /// [`ArmorHard`] 影响 [`ShieldDefence`]
    pub fn calc_defence_shield(armor_hard: &ArmorHard) -> f64 {
        armor_hard.0.get_current()
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::damage_system::{
        DamageAppliedAttrProps, MergedSurvivalEffs, apply_damages, calc_damage_scale,
        calc_defence_shield, calc_health_max, calc_magicka_max, calc_magicka_value, merge_damages,
    };
    use super::{
        DamageInfo, MagickaEnergyLevel, SurvivalEffBuffer, SurvivalEffTargets, SurvivalPropEffect,
    };
    use crate::base_lib::eff_attr_prop::multi_prop::{MultiPropEffTargetIter, multi_prop_system};
    use crate::base_lib::eff_attr_prop::{
        attrs::Attr, effects::Effect, prop_alter_eff::PropAlterEffectType, props::Prop,
    };
    use crate::common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        combat_units::{
            Health, Magicka, ShieldArcane, ShieldDefence, ShieldSubstitute, SurvivalPropLayer,
        },
    };

    #[test]
    fn check_survival_eff() {
        for targets in SurvivalEffTargets::iter() {
            multi_prop_system::check_multi_prop_eff_targets(targets);
        }
    }

    #[test]
    fn check_survival_eff_slice() {
        let effs: MergedSurvivalEffs<String> = MergedSurvivalEffs::default();
        let svv_effs = effs.into_slice();
        assert_eq!(svv_effs.len(), SurvivalEffTargets::iter().len());

        let svv_targets: Vec<_> = svv_effs.into_iter().map(|e| e.0).collect();

        let mut svv_targets_cloned: Vec<SurvivalEffTargets> = svv_targets.clone();
        svv_targets_cloned.sort_by(multi_prop_system::rank_multi_prop_eff);
        assert_eq!(svv_targets, svv_targets_cloned);
    }

    // region: 测试脚手架

    /// 目标四资源
    struct Targets {
        health: Health,
        sub: ShieldSubstitute,
        defence: ShieldDefence,
        arc: ShieldArcane,
    }

    impl Targets {
        /// 四资源均满值 100/100
        fn full() -> Self {
            Self {
                health: Health(Prop::new(100.0, 100.0, 0.0)),
                sub: ShieldSubstitute(Prop::new(100.0, 100.0, 0.0)),
                defence: ShieldDefence(Prop::new(100.0, 100.0, 0.0)),
                arc: ShieldArcane(Prop::new(100.0, 100.0, 0.0)),
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
                strength: Strength(Attr::new(1.0)),
                belief: Belief(Attr::new(1.0)),
                magicka: Magicka(Prop::new(0.0, 100.0, 0.0)),
                weapon_sharp: WeaponSharp(Attr::new(1.0)),
                weapon_mass: WeaponMass(Attr::new(1.0)),
                armor_soft: ArmorSoft(Attr::new(2.0)),
            }
        }

        fn as_props(&self) -> DamageAppliedAttrProps<'_> {
            DamageAppliedAttrProps {
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
        buffer: &mut SurvivalEffBuffer<&'static str>,
        targets: &mut Targets,
        attrs: &TestAttrs,
    ) -> DamageInfo<&'static str> {
        let merged = merge_damages(
            buffer,
            &targets.health,
            &targets.sub,
            &targets.defence,
            &targets.arc,
        );
        apply_damages(
            merged,
            attrs.as_props(),
            &mut targets.health,
            &mut targets.sub,
            &mut targets.defence,
            &mut targets.arc,
        )
    }

    /// PropAboutSurvivalEffTarget 未实现 PartialEq，用 matches! 判断同一资源
    fn same_prop(a: SurvivalPropLayer, b: SurvivalPropLayer) -> bool {
        matches!(
            (a, b),
            (SurvivalPropLayer::Health, SurvivalPropLayer::Health)
                | (
                    SurvivalPropLayer::ShieldSubstitute,
                    SurvivalPropLayer::ShieldSubstitute
                )
                | (
                    SurvivalPropLayer::ShieldDefence,
                    SurvivalPropLayer::ShieldDefence
                )
                | (
                    SurvivalPropLayer::ShieldArcane,
                    SurvivalPropLayer::ShieldArcane
                )
        )
    }

    // endregion

    // region: SurvivalEffTargets 方法（基于文档注释）

    /// target_types：各伤害类型的目标资源与文档一致
    #[test]
    fn target_types_match_documented_targets() {
        // 单资源类型：文档「仅作用于」对应单一资源
        assert_target_types(SurvivalEffTargets::OnlyHealth, &[SurvivalPropLayer::Health]);
        assert_target_types(
            SurvivalEffTargets::OnlyShieldSubstitute,
            &[SurvivalPropLayer::ShieldSubstitute],
        );
        assert_target_types(
            SurvivalEffTargets::OnlyShieldDefence,
            &[SurvivalPropLayer::ShieldDefence],
        );
        assert_target_types(
            SurvivalEffTargets::OnlyShieldArcane,
            &[SurvivalPropLayer::ShieldArcane],
        );

        // 复合类型：文档列出的受伤上限 —— 剪切伤 Def/Sub/Health、冲击伤 Sub/Health、奥术伤 Arc/Sub/Health
        assert_target_types(
            SurvivalEffTargets::PhysicsShears,
            &[
                SurvivalPropLayer::ShieldDefence,
                SurvivalPropLayer::ShieldSubstitute,
                SurvivalPropLayer::Health,
            ],
        );
        assert_target_types(
            SurvivalEffTargets::PhysicsImpact,
            &[
                SurvivalPropLayer::ShieldSubstitute,
                SurvivalPropLayer::Health,
            ],
        );
        assert_target_types(
            SurvivalEffTargets::MagickaArcane,
            &[
                SurvivalPropLayer::ShieldArcane,
                SurvivalPropLayer::ShieldSubstitute,
                SurvivalPropLayer::Health,
            ],
        );
    }

    fn assert_target_types(dmg_type: SurvivalEffTargets, expected: &[SurvivalPropLayer]) {
        let targets: Vec<_> = MultiPropEffTargetIter::from(dmg_type).collect();
        assert_eq!(targets.len(), expected.len(), "{dmg_type:?} 目标个数不符");
        for (got, want) in targets.iter().zip(expected) {
            assert!(
                same_prop(*got, *want),
                "{dmg_type:?} 目标 {got:?} 应为 {want:?}"
            );
        }
    }

    // endregion

    // region: 伤害缩放与数值公式

    /// calc_damage_scale：真实伤害与护盾专精恒为 1.0，不受属性/能量加成
    #[test]
    fn calc_damage_scale_is_constant_one_for_only_types() {
        let attrs = TestAttrs {
            strength: Strength(Attr::new(5.0)),
            belief: Belief(Attr::new(7.0)),
            magicka: Magicka(Prop::new(200.0, 200.0, 0.0)),
            weapon_sharp: WeaponSharp(Attr::new(3.0)),
            weapon_mass: WeaponMass(Attr::new(4.0)),
            armor_soft: ArmorSoft(Attr::new(1.0)),
        };
        for dmg_type in [
            SurvivalEffTargets::OnlyHealth,
            SurvivalEffTargets::OnlyShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane,
        ] {
            assert_eq!(
                calc_damage_scale(
                    dmg_type,
                    &attrs.strength,
                    &attrs.belief,
                    &attrs.magicka,
                    &attrs.weapon_sharp,
                    &attrs.weapon_mass,
                    &attrs.armor_soft,
                ),
                1.0,
                "{dmg_type:?} 缩放应恒为 1.0"
            );
        }
    }

    /// calc_damage_scale：复合伤害公式（能量系数 = 1 + 能量/100，能量基准 100）
    #[test]
    fn calc_damage_scale_composite_formulas() {
        let attrs = TestAttrs {
            strength: Strength(Attr::new(2.0)),
            belief: Belief(Attr::new(3.0)),
            magicka: Magicka(Prop::new(100.0, 200.0, 0.0)),
            weapon_sharp: WeaponSharp(Attr::new(4.0)),
            weapon_mass: WeaponMass(Attr::new(5.0)),
            armor_soft: ArmorSoft(Attr::new(2.0)),
        };
        let (s, b, m, w_s, w_m, a_s) = (
            &attrs.strength,
            &attrs.belief,
            &attrs.magicka,
            &attrs.weapon_sharp,
            &attrs.weapon_mass,
            &attrs.armor_soft,
        );
        let energy_scale = 1.0 + 100.0 / 100.0;
        // 物理剪切 = 气力 * 锋利 * 能量系数
        assert_eq!(
            calc_damage_scale(SurvivalEffTargets::PhysicsShears, s, b, m, w_s, w_m, a_s),
            2.0 * 4.0 * energy_scale
        );
        // 物理冲击 = (气力 + 质量) / 柔韧 * 能量系数
        assert_eq!(
            calc_damage_scale(SurvivalEffTargets::PhysicsImpact, s, b, m, w_s, w_m, a_s),
            (2.0 + 5.0) / 2.0 * energy_scale
        );
        // 魔法奥术 = 信念 * 能量系数
        assert_eq!(
            calc_damage_scale(SurvivalEffTargets::MagickaArcane, s, b, m, w_s, w_m, a_s),
            3.0 * energy_scale
        );
    }

    /// calc_health_max：Strength 影响 Health，health_base + health_scale * strength.origin
    #[test]
    fn calc_health_max_scales_with_strength_origin() {
        let strength = Strength(Attr::new(10.0));
        assert_eq!(calc_health_max(100.0, 5.0, &strength), 150.0);
    }

    /// calc_magicka_value：Belief 影响原始能量，magicka_base + magicka_scale * belief.origin
    #[test]
    fn calc_magicka_value_scales_with_belief_origin() {
        let belief = Belief(Attr::new(10.0));
        assert_eq!(calc_magicka_value(50.0, 3.0, &belief), 80.0);
    }

    /// calc_magicka_max：先算原始能量，再按能级取对应层级上限
    #[test]
    fn calc_magicka_max_takes_energy_level() {
        let levels = MagickaEnergyLevel::new(100.0, 200.0, 300.0);
        // 原始能量 50 + 3*10 = 80 → 第一能级上限 100
        let belief = Belief(Attr::new(10.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 100.0);
        // 原始能量 50 + 3*50 = 200 → 第二能级上限 200
        let belief = Belief(Attr::new(50.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 200.0);
        // 原始能量 50 + 3*80 = 290 → 第三能级上限 300
        let belief = Belief(Attr::new(80.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 300.0);
    }

    /// calc_defence_shield：ArmorHard 影响 ShieldDefence，返回 armor_hard 当前值
    #[test]
    fn calc_defence_shield_uses_armor_hard_current() {
        let armor_hard = ArmorHard(Attr::new(25.0));
        assert_eq!(calc_defence_shield(&armor_hard), 25.0);
    }

    // endregion

    // region: 伤害计算（各类型在对应 prop 生效 + 破盾传递）

    /// OnlyHealth：绝对值伤害直接作用于血量
    #[test]
    fn only_health_hits_health_directly() {
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("attacker", "real_dmg", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert!(buffer.is_empty());
        assert_eq!(targets.health.0.get_current(), 60.0);
        assert_eq!(targets.sub.0.get_current(), 100.0);
        assert_eq!(targets.defence.0.get_current(), 100.0);
        assert_eq!(targets.arc.0.get_current(), 100.0);
    }

    /// OnlyHealth：治疗正效果值加回血量，超上限被钳制
    #[test]
    fn only_health_heals_up_to_cap() {
        let mut targets = Targets::full();
        targets.health.0.apply_eff(-30.0); // 先扣到 70
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("healer", "heal", 40.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.health.0.get_current(), 100.0); // 70 + 40 = 110 → 钳制到 100
    }

    /// OnlyShield*：只作用于对应护盾，其余资源不变
    #[test]
    fn only_shield_types_hit_only_their_shield() {
        // 替身护盾
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            PropAlterEffectType::Val,
            Effect::new("a", "break_sub", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.sub.0.get_current(), 60.0);
        assert_eq!(targets.health.0.get_current(), 100.0);
        assert_eq!(targets.defence.0.get_current(), 100.0);
        assert_eq!(targets.arc.0.get_current(), 100.0);

        // 防护护盾
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldDefence,
            PropAlterEffectType::Val,
            Effect::new("a", "break_def", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.defence.0.get_current(), 60.0);
        assert_eq!(targets.health.0.get_current(), 100.0);
        assert_eq!(targets.sub.0.get_current(), 100.0);

        // 奥术护盾
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldArcane,
            PropAlterEffectType::Val,
            Effect::new("a", "break_arc", -40.0),
        ));
        let mut targets = Targets::full();
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.arc.0.get_current(), 60.0);
        assert_eq!(targets.health.0.get_current(), 100.0);
    }

    /// PhysicsShears：防护护盾 → 替身护盾 → 血量，依次穿透
    #[test]
    fn physics_shears_breaks_defence_then_substitute_then_health() {
        let mut targets = Targets {
            health: Health(Prop::new(100.0, 100.0, 0.0)),
            sub: ShieldSubstitute(Prop::new(20.0, 100.0, 0.0)),
            defence: ShieldDefence(Prop::new(30.0, 100.0, 0.0)),
            arc: ShieldArcane(Prop::new(100.0, 100.0, 0.0)),
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("a", "shear", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.defence.0.get_current(), 0.0); // 吸收 30
        assert_eq!(targets.sub.0.get_current(), 0.0); // 吸收 20
        assert_eq!(targets.health.0.get_current(), 50.0); // 剩余 50 落血
        assert_eq!(targets.arc.0.get_current(), 100.0); // 奥术护盾不受物理剪切影响
    }

    /// PhysicsImpact：替身护盾 → 血量，依次穿透
    #[test]
    fn physics_impact_breaks_substitute_then_health() {
        let mut targets = Targets {
            health: Health(Prop::new(100.0, 100.0, 0.0)),
            sub: ShieldSubstitute(Prop::new(30.0, 100.0, 0.0)),
            defence: ShieldDefence(Prop::new(100.0, 100.0, 0.0)),
            arc: ShieldArcane(Prop::new(100.0, 100.0, 0.0)),
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::PhysicsImpact,
            PropAlterEffectType::Val,
            Effect::new("a", "impact", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.sub.0.get_current(), 0.0); // 吸收 30
        assert_eq!(targets.health.0.get_current(), 30.0); // 剩余 70 落血
        assert_eq!(targets.defence.0.get_current(), 100.0); // 防护护盾不受物理冲击影响
    }

    /// MagickaArcane：奥术护盾 → 替身护盾 → 血量，依次穿透
    #[test]
    fn magicka_arcane_breaks_arcane_then_substitute_then_health() {
        let mut targets = Targets {
            health: Health(Prop::new(100.0, 100.0, 0.0)),
            sub: ShieldSubstitute(Prop::new(20.0, 100.0, 0.0)),
            defence: ShieldDefence(Prop::new(100.0, 100.0, 0.0)),
            arc: ShieldArcane(Prop::new(30.0, 100.0, 0.0)),
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::MagickaArcane,
            PropAlterEffectType::Val,
            Effect::new("a", "arcane", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(targets.arc.0.get_current(), 0.0); // 吸收 30
        assert_eq!(targets.sub.0.get_current(), 0.0); // 吸收 20
        assert_eq!(targets.health.0.get_current(), 50.0); // 剩余 50 落血
        assert_eq!(targets.defence.0.get_current(), 100.0); // 防护护盾不受魔法奥术影响
    }

    // endregion

    // region: 同类合并

    /// merge_damages：同类伤害合并后一次结算，缓冲被清空
    #[test]
    fn merge_damages_sums_same_type_and_drains_buffer() {
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("a", "hit1", -30.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("a", "hit2", -20.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert!(buffer.is_empty());
        assert_eq!(targets.health.0.get_current(), 50.0); // -30 与 -20 合并为 -50
    }

    /// merge_damages：百分比按目标自身折算后与绝对值合并
    #[test]
    fn merge_damages_mixes_absolute_and_percent() {
        let mut targets = Targets::full(); // 血量当前值 100
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("a", "flat", -30.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::CurPer,
            Effect::new("a", "cut", -0.2),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::MaxPer,
            Effect::new("a", "slash", -0.5),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        // -30 + (-0.2*100) + (-0.5*100) = -100
        assert_eq!(targets.health.0.get_current(), 0.0);
    }

    // endregion

    // region: OnlyShield* 与复合伤害同帧的顺序（避免破盾伤害浪费）

    /// OnlyShieldDefence 与 PhysicsShears 同帧：破盾伤害必须先于复合伤害结算，
    /// 否则 PhysicsShears 先破防护盾，OnlyShieldDefence 会对空盾浪费伤害
    #[test]
    fn combined_only_shield_defence_applies_before_physics_shears() {
        let mut targets = Targets {
            defence: ShieldDefence(Prop::new(50.0, 100.0, 0.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldDefence,
            PropAlterEffectType::Val,
            Effect::new("a", "break_def", -100.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("a", "shear", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 若顺序颠倒，PhysicsShears 先打掉 50 防护，OnlyShieldDefence 的 -100 全部浪费，
        // 替身只会剩 50；正确顺序下替身被 PhysicsShears 打空、血量无伤
        assert_eq!(targets.defence.0.get_current(), 0.0);
        assert_eq!(targets.sub.0.get_current(), 0.0);
        assert_eq!(targets.health.0.get_current(), 100.0);
    }

    /// OnlyShieldSubstitute 与 PhysicsImpact 同帧：破盾伤害先结算
    #[test]
    fn combined_only_shield_substitute_applies_before_physics_impact() {
        let mut targets = Targets {
            sub: ShieldSubstitute(Prop::new(50.0, 100.0, 0.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            PropAlterEffectType::Val,
            Effect::new("a", "break_sub", -100.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::PhysicsImpact,
            PropAlterEffectType::Val,
            Effect::new("a", "impact", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 错误顺序下 PhysicsImpact 先波及替身与血量（替身 50、血量 50），OnlyShieldSubstitute 浪费；
        // 正确顺序下替身被破盾伤害清空，冲击伤害全部落血
        assert_eq!(targets.sub.0.get_current(), 0.0);
        assert_eq!(targets.health.0.get_current(), 0.0);
    }

    /// OnlyShieldArcane 与 MagickaArcane 同帧：破盾伤害先结算
    #[test]
    fn combined_only_shield_arcane_applies_before_magicka_arcane() {
        let mut targets = Targets {
            arc: ShieldArcane(Prop::new(50.0, 100.0, 0.0)),
            ..Targets::full()
        };
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldArcane,
            PropAlterEffectType::Val,
            Effect::new("a", "break_arc", -100.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::MagickaArcane,
            PropAlterEffectType::Val,
            Effect::new("a", "arcane", -100.0),
        ));
        run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());

        // 错误顺序下 MagickaArcane 先波及奥术 50、替身 50、血量 50，OnlyShieldArcane 浪费；
        // 正确顺序下奥术被破盾伤害清空，魔法伤害全部落在替身、血量无伤
        assert_eq!(targets.arc.0.get_current(), 0.0);
        assert_eq!(targets.sub.0.get_current(), 0.0);
        assert_eq!(targets.health.0.get_current(), 100.0);
    }

    // endregion

    // region: HealthInfo 死因来源

    /// HealthInfo：记录第一个「伤血/治疗」效果作为死因来源；纯护盾伤害不记录
    #[test]
    fn apply_damages_records_first_hurt_heal_source() {
        // 纯护盾伤害 → 无死因来源
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            PropAlterEffectType::Val,
            Effect::new("a", "break_sub", -30.0),
        ));
        let info = run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(info.first_hurt_heal_from_eff, None);

        // 混入真实伤害 → 记录真实伤害的 (来源, 效果名)
        let mut targets = Targets::full();
        let mut buffer = SurvivalEffBuffer::new();
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyShieldSubstitute,
            PropAlterEffectType::Val,
            Effect::new("a", "break_sub", -30.0),
        ));
        buffer.push(SurvivalPropEffect::new(
            SurvivalEffTargets::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("b", "real_dmg", -30.0),
        ));
        let info = run_damage(&mut buffer, &mut targets, &TestAttrs::scale_one());
        assert_eq!(info.first_hurt_heal_from_eff, Some(("b", "real_dmg")));
    }

    // endregion
}
