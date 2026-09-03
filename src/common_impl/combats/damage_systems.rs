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
            compound_attr_systems::CompoundAttr,
            effects::{EffId, Effect},
            modifier_collections::ModifiableAttr,
        },
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        damages::{
            DamageInfo, Health, HealthLower, HealthUpper, ShieldArcane, ShieldArcaneUpper,
            ShieldDefence, ShieldDefenceUpper, ShieldSubstitute, ShieldSubstituteUpper,
            SurvivalAttrEff, SurvivalAttrLayer, SurvivalAttrRef, SurvivalBoundRef,
            SurvivalEffBuffer, SurvivalEffTargets,
        },
        energies::Magicka,
        energy_systems,
    },
};

const BOUNDED_ATTR_LOWER: f64 = 0.0;

/// 提前计算伤害效果，可以实现“替某人承受伤害”的效果
///
/// 返回伤害类型，因为要根据伤害类型选择生效的目标属性
pub fn normalize_damage_eff<S: FixedName>(
    dmg_scale_attrs: DamageScaleAttrs,
    mut target_svv_attrs: SurvivalAttrRef,
    target_svv_bounds: SurvivalBoundRef,
    eff: SurvivalAttrEff<S>,
) -> (SurvivalEffTargets, Effect<S>) {
    let SurvivalAttrEff {
        target_type,
        alter_eff,
    } = eff;

    // 根据伤害类型找到百分比参照物
    let bottom_layer = target_type.stop_at();
    let base_attr = target_svv_attrs.get_attr_mut(bottom_layer);
    let base_upper = target_svv_bounds.get_upper(bottom_layer);

    // 根据伤害算法计算伤害绝对值
    let abs_eff_val = alter_eff.calc_alter_val(base_attr, base_upper);

    // 根据伤害类型计算缩放比例
    let damage_scale = calc_damage_scale(target_type, dmg_scale_attrs);

    let EffId {
        from_name,
        effect_name,
    } = alter_eff.take_eff().take_from_eff_name();

    (
        target_type,
        Effect::new(from_name, effect_name, abs_eff_val * damage_scale),
    )
}

pub fn push_damages<S: FixedName>(
    svv_eff_buffer: &mut SurvivalEffBuffer<S>,
    eff: (SurvivalEffTargets, Effect<S>),
) {
    svv_eff_buffer.push(eff);
}

// region: 合并效果

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

/// 每帧计算伤害前都先进行同类合并
///
/// 合并方便伤害计算，具体原因如下
/// - 若先【物理伤害】，后【破盾伤害】，那么当两者加起来能够破盾时，实际伤害与顺序有关
/// - 【物理伤害】在前会导致后面的【破盾伤害】无效化
///
/// 详细探讨见 [`crate::base_lib::eff_attr::attr_systems`]
pub fn merge_damages<S: FixedName>(
    svv_eff_buffer: &mut SurvivalEffBuffer<S>,
) -> MergedSurvivalEffs<S> {
    let mut merged_svv_effs = MergedSurvivalEffs::<S>::default();

    for (target_type, eff) in svv_eff_buffer.take_effs() {
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

        // todo 在这里找到每个类型的伤害最大效果，后续根据合并后的伤害是否真正伤害到血量，再找到伤害最大值
        // 累加绝对值
        if let Some(merged_dmg) = merged_dmg {
            let delta_val = eff.get_effect_value();
            let origin_val = merged_dmg.get_effect_value();
            merged_dmg.set_effect_value(origin_val + delta_val);
        } else {
            *merged_dmg = Some(eff);
        }
    }

    merged_svv_effs
}

// endregion

pub struct DamageScaleAttrs<'a> {
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
    _damage_calc_attrs: DamageScaleAttrs,
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
            // let dmg_scale = damage_systems::calc_damage_scale(svv_eff_target, &damage_calc_attrs);
            let dmg_scale = 1.0;

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

                // todo 放在循环外，必须实际伤害到了生命值（击破护盾）才参与结算
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
fn calc_damage_scale(
    svv_eff_target: SurvivalEffTargets,
    damage_scale_attrs: DamageScaleAttrs,
) -> f64 {
    let DamageScaleAttrs {
        source_strength,
        source_belief,
        source_magicka,
        source_weapon_sharp,
        source_weapon_mass,
        target_armor_soft,
    } = damage_scale_attrs;

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
