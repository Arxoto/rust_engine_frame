use strum_macros::EnumIter;

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        attr_layers::{AttrLayerEffTarget, AttrLayerType},
        bound_attr_modifiers::BoundAttrModifier,
        bound_attrs::BoundAttr,
        bounded_attr_effs::{AttrAlterEff, AttrAlterEffType},
        bounded_attrs::BoundedAttr,
        effects::Effect,
        modifier_collections::ModifierCollection,
    },
};

// region: 属性

/// 替身护盾 被伤害系统控制；
pub struct ShieldSubstitute(pub BoundedAttr);

/// 防护护盾 被伤害系统控制；
pub struct ShieldDefence(pub BoundedAttr);

/// 奥术护盾 被伤害系统控制；
pub struct ShieldArcane(pub BoundedAttr);

/// 血量 被伤害系统控制； 基础值被【气力】的基础值影响
pub struct Health(pub BoundedAttr);

pub struct ShieldSubstituteUpper(pub BoundAttr);
pub struct ShieldDefenceUpper(pub BoundAttr);
pub struct ShieldArcaneUpper(pub BoundAttr);
pub struct HealthUpper(pub BoundAttr);
pub struct HealthLower(pub BoundAttr);

pub struct ShieldSubstituteUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct ShieldDefenceUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct ShieldArcaneUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct HealthUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct HealthLowerEffs(pub ModifierCollection<BoundAttrModifier>);

// endregion

/// 生存属性类型（生命值、护盾）
///
/// 生命值护盾的层级关系的值是业务约定
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum SurvivalAttrLayer {
    Health,
    ShieldSubstitute,
    ShieldDefence,
    ShieldArcane,
}

impl AttrLayerType for SurvivalAttrLayer {
    fn get_next(&self) -> Self {
        match self {
            SurvivalAttrLayer::Health => Self::Health,
            SurvivalAttrLayer::ShieldSubstitute => Self::Health,
            SurvivalAttrLayer::ShieldDefence => Self::ShieldSubstitute,
            SurvivalAttrLayer::ShieldArcane => Self::ShieldSubstitute,
        }
    }

    fn get_layer(&self) -> u8 {
        match self {
            SurvivalAttrLayer::Health => 0,
            SurvivalAttrLayer::ShieldSubstitute => 1,
            SurvivalAttrLayer::ShieldDefence => 2,
            SurvivalAttrLayer::ShieldArcane => 2,
        }
    }
}

/// 存放伤害或治疗效果的 buffer
#[derive(Debug)]
pub struct SurvivalEffBuffer<S: FixedName>(pub Vec<SurvivalAttrEff<S>>);

/// 伤害信息，表示每次伤害造成的影响
///
/// 这里不自动判断血量是否为零，因为还在 pending 阶段，管线后续可能还会修改
#[derive(Debug)]
pub struct DamageInfo<S: FixedName> {
    /// 对生命造成的最大伤害的效果，用于统计死因
    pub max_hurt_heal_eff: Option<Effect<S>>,
}

/// 生存类效果（伤害、治疗、护盾）
#[derive(Debug, Clone)]
pub struct SurvivalAttrEff<S: FixedName> {
    /// 伤害类型，伤害针对的哪层属性
    pub target_type: SurvivalEffTargets,
    /// 伤害生效方式（绝对值或是百分比）
    pub alter_type: AttrAlterEffType,
    pub eff: Effect<S>,
}

impl<S: FixedName> SurvivalAttrEff<S> {
    /// 构造单次伤害效果
    ///
    /// 推入 [`SurvivalEffBuffer`] 后由伤害系统消费。
    pub fn new(
        target_type: SurvivalEffTargets,
        alter_type: AttrAlterEffType,
        eff: Effect<S>,
    ) -> Self {
        Self {
            target_type,
            alter_type,
            eff,
        }
    }

    pub fn new_from_alter_eff(target_type: SurvivalEffTargets, eff: AttrAlterEff<S>) -> Self {
        Self {
            target_type,
            alter_type: eff.get_type(),
            eff: eff.take_eff(),
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
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// 推入一次伤害效果
    pub fn push(&mut self, dmg_eff: SurvivalAttrEff<S>) {
        self.0.push(dmg_eff);
    }

    /// 缓冲内伤害效果数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 缓冲是否为空
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

impl AttrLayerEffTarget for SurvivalEffTargets {
    type Layer = SurvivalAttrLayer;

    fn start_at(&self) -> Self::Layer {
        match self {
            SurvivalEffTargets::OnlyHealth => SurvivalAttrLayer::Health,
            SurvivalEffTargets::OnlyShieldSubstitute => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane => SurvivalAttrLayer::ShieldArcane,
            SurvivalEffTargets::PhysicsImpact => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffTargets::PhysicsShears => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffTargets::MagickaArcane => SurvivalAttrLayer::ShieldArcane,
        }
    }

    fn stop_at(&self) -> Self::Layer {
        match self {
            SurvivalEffTargets::OnlyHealth => SurvivalAttrLayer::Health,
            SurvivalEffTargets::OnlyShieldSubstitute => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffTargets::OnlyShieldDefence => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffTargets::OnlyShieldArcane => SurvivalAttrLayer::ShieldArcane,
            SurvivalEffTargets::PhysicsImpact => SurvivalAttrLayer::Health,
            SurvivalEffTargets::PhysicsShears => SurvivalAttrLayer::Health,
            SurvivalEffTargets::MagickaArcane => SurvivalAttrLayer::Health,
        }
    }
}

impl SurvivalEffTargets {
    /// 能否对血量造成伤害
    pub fn is_hurt_heal(&self) -> bool {
        self.stop_at() == SurvivalAttrLayer::Health
    }
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

    use crate::{
        base_lib::eff_attr::attr_layers::attr_layer_system,
        common_impl::combats::damage_systems::MergedSurvivalEffs,
    };

    use super::*;

    // 在单元测试中检查复合属性的层级规划，以避免运行时开销

    #[test]
    fn check_survival_layer() {
        for ele in SurvivalAttrLayer::iter() {
            attr_layer_system::check_attr_layer(ele);
        }
    }

    #[test]
    fn check_survival_eff() {
        for targets in SurvivalEffTargets::iter() {
            attr_layer_system::check_attr_layer_eff_target(targets);
        }
    }

    /// 检查预设的效果生效顺序
    #[test]
    fn check_survival_eff_slice() {
        let effs: MergedSurvivalEffs<String> = MergedSurvivalEffs::default();
        let svv_effs = effs.into_slice();
        assert_eq!(svv_effs.len(), SurvivalEffTargets::iter().len());

        let svv_targets: Vec<_> = svv_effs.into_iter().map(|e| e.0).collect();

        let mut svv_targets_cloned: Vec<SurvivalEffTargets> = svv_targets.clone();
        svv_targets_cloned.sort_by(attr_layer_system::rank_attr_layer_eff);
        assert_eq!(svv_targets, svv_targets_cloned);
    }
}
