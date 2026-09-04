//! ## 衡量伤害公式的平衡性
//!
//! - 随着角色成长，【伤害成长】应该与【受伤上限】大致成正比
//! - 伤害公式中各个属性的根源属性应该合理分配，避免某一属性影响力过大
//!
//! ## 受伤上限
//!
//! 【受伤上限】 本质即 【生命值和护盾值】 的组合
//!
//! - 生命值 [`super::damages::Health`]
//!   - 直接正相关 [`Strength`] see [`calc_health_max`]
//! - 替身护盾 [`super::damages::ShieldSubstitute`]
//!   - 直接正相关 [`Belief`] todo 信念超过阈值才能激发替身护盾
//! - 防护护盾 [`super::damages::ShieldDefence`]
//!   - 直接正相关 [`ArmorHard`] see [`calc_defence_shield`]
//!   - 间接正相关 [`Strength`] todo 数值上盔甲坚韧与质量呈正相关，气力决定可穿戴质量，因此可以近似取代
//! - 奥术护盾 [`super::damages::ShieldArcane`]
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

use strum::EnumCount;

use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{
            attr_layers::AttrLayerEffTarget,
            compound_attr_systems::{self, CompoundAttr},
            effects::{EffId, Effect},
            modifier_collections::ModifiableAttr,
        },
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        damages::{
            SurvivalAttrEff, SurvivalAttrRef, SurvivalBoundRef, SurvivalEffBuffer,
            SurvivalEffTargets, SurvivalNormalizedAttrEff,
        },
        energies::Magicka,
        energy_systems,
    },
};

/// 提前计算伤害效果，可以实现“替某人承受伤害”的效果
///
/// 返回伤害类型，因为要根据伤害类型选择生效的目标属性
pub fn normalize_damage_eff<S: FixedName>(
    dmg_scale_attrs: DamageScaleAttrs,
    mut target_svv_attrs: SurvivalAttrRef,
    target_svv_bounds: SurvivalBoundRef,
    eff: SurvivalAttrEff<S>,
) -> SurvivalNormalizedAttrEff<S> {
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

    SurvivalNormalizedAttrEff {
        svv_targets: target_type,
        eff: Effect::new(from_name, effect_name, abs_eff_val * damage_scale),
    }
}

/// 推入伤害 buffer
pub fn push_damages<S: FixedName>(
    svv_eff_buffer: &mut SurvivalEffBuffer<S>,
    eff: SurvivalNormalizedAttrEff<S>,
) {
    svv_eff_buffer.push(eff);
}

/// 每帧计算伤害前都先进行同类合并，以减少计算次数
///
/// 同时合并可以规范伤害计算，具体原因如下
/// - 若先【物理伤害】，后【破盾伤害】，那么当两者加起来能够破盾时，实际伤害与顺序有关
/// - 【物理伤害】在前会导致后面的【破盾伤害】无效化
///
/// 详细探讨见 [`crate::base_lib::eff_attr::attr_systems`]
pub fn merge_damages<S: FixedName>(
    svv_eff_buffer: &mut SurvivalEffBuffer<S>,
) -> MergedSurvivalEffs<S> {
    let mut merged_svv_effs = MergedSurvivalEffs::<S>::default();

    for SurvivalNormalizedAttrEff { svv_targets, eff } in svv_eff_buffer.take_effs() {
        // 根据伤害类型找到聚合对象
        let merged_dmg = merged_svv_effs.get_mut(svv_targets);

        if let Some(merged_dmg) = merged_dmg {
            merged_dmg.merge_eff(eff);
        } else {
            *merged_dmg = Some(MergedSvvEffRecord::new(eff));
        }
    }

    merged_svv_effs
}

/// 对合并后的伤害效果计算伤害
pub fn apply_damages<S: FixedName>(
    mut merged_svv_effs: MergedSurvivalEffs<S>,
    mut attrs: SurvivalAttrRef,
    attr_bounds: SurvivalBoundRef,
) -> DamageInfo<S> {
    let mut dmg_info: DamageInfo<S> = DamageInfo::default();

    for svv_targets in SurvivalEffTargets::SORTED_ARRAY {
        let dmg_eff = merged_svv_effs.take(svv_targets);
        if let Some(dmg_eff) = dmg_eff {
            let MergedSvvEffRecord {
                danger_eff,
                merged_val,
            } = dmg_eff;

            let origin_heal = attrs.health.0.get_pending_value();

            compound_attr_systems::apply_alter(&mut attrs, &attr_bounds, svv_targets, merged_val);

            let current_heal = attrs.health.0.get_pending_value();
            let diff_heal = current_heal - origin_heal;
            if diff_heal < 0.0 {
                // 必须实际伤害到生命值才参与结算
                if let Some(hurt_eff) = dmg_info.max_hurt_heal_eff.as_mut() {
                    refresh_danger_eff(hurt_eff, danger_eff);
                } else {
                    dmg_info.max_hurt_heal_eff = Some(danger_eff);
                }
            }
        }
    }

    dmg_info
}

pub fn try_damage<S: FixedName>(
    mut attrs: SurvivalAttrRef,
    attr_bounds: SurvivalBoundRef,
    must_ge: f64,
    eff: SurvivalNormalizedAttrEff<S>,
) -> bool {
    let delta_val = eff.eff.get_effect_value();

    compound_attr_systems::apply_alter_safety(
        &mut attrs,
        &attr_bounds,
        eff.svv_targets,
        delta_val,
        must_ge,
    )
}

/// [`Strength`] 影响 [`super::damages::Health`]
#[inline]
pub fn calc_health_max(health_base: f64, health_scale: f64, strength: &Strength) -> f64 {
    health_base + health_scale * strength.0.get_origin()
}

/// [`ArmorHard`] 影响 [`super::damages::ShieldDefence`]
#[inline]
pub fn calc_defence_shield(armor_hard: &ArmorHard) -> f64 {
    armor_hard.0.get_current()
}

/// 伤害缩放（根据伤害类型计算缩放比例）
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

/// 自动识别刷新伤害值最大的效果
fn refresh_danger_eff<S: FixedName>(current_eff: &mut Effect<S>, new_eff: Effect<S>) {
    // 伤害值最大，记录效果值最小的
    if new_eff.get_effect_value() < current_eff.get_effect_value() {
        *current_eff = new_eff;
    }
}

/// 根据伤害类型计算缩放比例时，需要用到的属性
pub struct DamageScaleAttrs<'a> {
    pub source_strength: &'a Strength,
    pub source_belief: &'a Belief,
    pub source_magicka: &'a Magicka,
    pub source_weapon_sharp: &'a WeaponSharp,
    pub source_weapon_mass: &'a WeaponMass,
    pub target_armor_soft: &'a ArmorSoft,
}

/// 某一类伤害的合并记录
#[derive(Debug)]
struct MergedSvvEffRecord<S: FixedName> {
    /// 伤害值最大的效果，用于记录致命伤
    danger_eff: Effect<S>,
    /// 合并伤害
    merged_val: f64,
}

impl<S: FixedName> MergedSvvEffRecord<S> {
    fn new(eff: Effect<S>) -> Self {
        let merged_val = eff.get_effect_value();
        Self {
            danger_eff: eff,
            merged_val,
        }
    }

    fn merge_eff(&mut self, eff: Effect<S>) {
        self.merged_val += eff.get_effect_value();
        refresh_danger_eff(&mut self.danger_eff, eff);
    }
}

/// 所有伤害类型的合并记录
#[derive(Debug)]
pub struct MergedSurvivalEffs<S: FixedName>(
    [Option<MergedSvvEffRecord<S>>; SurvivalEffTargets::COUNT],
);

impl<S: FixedName> Default for MergedSurvivalEffs<S> {
    fn default() -> Self {
        Self([const { None }; SurvivalEffTargets::COUNT])
    }
}

impl<S: FixedName> MergedSurvivalEffs<S> {
    /// 不允许重复，并且固定在 0 - N 之间
    #[inline]
    const fn get_index(svv_targets: SurvivalEffTargets) -> usize {
        match svv_targets {
            SurvivalEffTargets::OnlyHealth => 0,
            SurvivalEffTargets::OnlyShieldSubstitute => 1,
            SurvivalEffTargets::OnlyShieldDefence => 2,
            SurvivalEffTargets::OnlyShieldArcane => 3,
            SurvivalEffTargets::PhysicsImpact => 4,
            SurvivalEffTargets::PhysicsShears => 5,
            SurvivalEffTargets::MagickaArcane => 6,
        }
    }

    fn get_mut(&mut self, svv_targets: SurvivalEffTargets) -> &mut Option<MergedSvvEffRecord<S>> {
        let index = Self::get_index(svv_targets);
        &mut self.0[index]
    }

    fn take(&mut self, svv_targets: SurvivalEffTargets) -> Option<MergedSvvEffRecord<S>> {
        let index = Self::get_index(svv_targets);
        self.0[index].take()
    }
}

/// 伤害信息，表示每次伤害造成的影响
///
/// 这里不自动判断血量是否为零，因为还在 pending 阶段，管线后续可能还会修改
#[derive(Debug)]
pub struct DamageInfo<S: FixedName> {
    /// 对生命造成的最大伤害的效果，用于统计死因
    pub max_hurt_heal_eff: Option<Effect<S>>,
}

impl<S: FixedName> Default for DamageInfo<S> {
    fn default() -> Self {
        Self {
            max_hurt_heal_eff: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn merged_svv_effs_index() {
        let mut merged_survival_effs = MergedSurvivalEffs::default();

        // 对所有类型遍历并获取，不报错说明索引都在 0 - N 之间
        for ele in SurvivalEffTargets::iter() {
            let merged_svv_eff_record = merged_survival_effs.get_mut(ele);
            *merged_svv_eff_record = Some(MergedSvvEffRecord::new(Effect::new(
                "from_name",
                "effect_name",
                0.0,
            )));
        }

        // 全部都是 some 说明索引没有重复
        for ele in merged_survival_effs.0 {
            assert!(ele.is_some())
        }
    }
}
