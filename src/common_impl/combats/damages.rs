use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        attr_layers::{AttrLayerEffType, AttrLayerType},
        bound_attr_modifiers::BoundAttrModifier,
        bound_attrs::{BoundAttr, BoundRange},
        bounded_attr_effs::{AttrAlterEff, AttrAlterEffType},
        bounded_attrs::BoundedAttr,
        compound_attr_systems::{CompoundAttr, CompoundAttrBound},
        effects::Effect,
        modifier_collections::ModifierCollection,
    },
};

// region: 属性定义

/// 血量 被伤害系统控制； 基础值被【气力】的基础值影响
pub struct Health(pub BoundedAttr);

/// 替身护盾 被伤害系统控制；
pub struct ShieldSubstitute(pub BoundedAttr);

/// 防护护盾 被伤害系统控制；
pub struct ShieldDefence(pub BoundedAttr);

/// 奥术护盾 被伤害系统控制；
pub struct ShieldArcane(pub BoundedAttr);

pub const SHIELD_LOWER: f64 = 0.0;

pub struct HealthLower(pub BoundAttr);
pub struct HealthUpper(pub BoundAttr);
pub struct ShieldSubstituteUpper(pub BoundAttr);
pub struct ShieldDefenceUpper(pub BoundAttr);
pub struct ShieldArcaneUpper(pub BoundAttr);

pub struct HealthLowerEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct HealthUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct ShieldSubstituteUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct ShieldDefenceUpperEffs(pub ModifierCollection<BoundAttrModifier>);
pub struct ShieldArcaneUpperEffs(pub ModifierCollection<BoundAttrModifier>);

// endregion

// region: 属性层级

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

// endregion

// region: 效果定义

/// 生存效果生效目标
///
/// 百分比伤害计算时选取 `stop_at` 为基础进行计算
///
/// - 真实伤害 [`SurvivalEffType::OnlyHealth`]
///   - 伤害 [`Health`]
/// - 物理冲击 [`SurvivalEffType::PhysicsImpact`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`]
/// - 物理剪切 [`SurvivalEffType::PhysicsShears`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`] & [`ShieldDefence`]
/// - 魔法奥术 [`SurvivalEffType::MagickaArcane`]
///   - 伤害 [`Health`] & [`ShieldSubstitute`] & [`ShieldArcane`]
/// - 破盾专精伤害对应护盾
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCountMacro, EnumIter)]
pub enum SurvivalEffType {
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

impl SurvivalEffType {
    /// 按照生效顺序排序，通过单元测试保证排序准确性
    pub const SORTED_ARRAY: [Self; Self::COUNT] = [
        Self::OnlyShieldDefence,
        Self::OnlyShieldArcane,
        Self::OnlyShieldSubstitute,
        Self::PhysicsShears,
        Self::MagickaArcane,
        Self::PhysicsImpact,
        Self::OnlyHealth,
    ];
}

impl AttrLayerEffType for SurvivalEffType {
    type LayerType = SurvivalAttrLayer;

    fn start_at(&self) -> Self::LayerType {
        match self {
            SurvivalEffType::OnlyHealth => SurvivalAttrLayer::Health,
            SurvivalEffType::OnlyShieldSubstitute => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffType::OnlyShieldDefence => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffType::OnlyShieldArcane => SurvivalAttrLayer::ShieldArcane,
            SurvivalEffType::PhysicsImpact => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffType::PhysicsShears => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffType::MagickaArcane => SurvivalAttrLayer::ShieldArcane,
        }
    }

    fn stop_at(&self) -> Self::LayerType {
        match self {
            SurvivalEffType::OnlyHealth => SurvivalAttrLayer::Health,
            SurvivalEffType::OnlyShieldSubstitute => SurvivalAttrLayer::ShieldSubstitute,
            SurvivalEffType::OnlyShieldDefence => SurvivalAttrLayer::ShieldDefence,
            SurvivalEffType::OnlyShieldArcane => SurvivalAttrLayer::ShieldArcane,
            SurvivalEffType::PhysicsImpact => SurvivalAttrLayer::Health,
            SurvivalEffType::PhysicsShears => SurvivalAttrLayer::Health,
            SurvivalEffType::MagickaArcane => SurvivalAttrLayer::Health,
        }
    }
}

/// 生存类效果（伤害、治疗、护盾）
#[derive(Debug, Clone)]
pub struct SurvivalAttrEff<S: FixedName> {
    /// 伤害类型，伤害针对的哪层属性
    pub(super) svv_eff_type: SurvivalEffType,
    pub(super) alter_eff: AttrAlterEff<S>,
}

impl<S: FixedName> SurvivalAttrEff<S> {
    /// 构造单次伤害效果
    ///
    /// 推入 [`SurvivalEffBuffer`] 后由伤害系统消费。
    pub fn new(
        svv_eff_type: SurvivalEffType,
        alter_type: AttrAlterEffType,
        eff: Effect<S>,
    ) -> Self {
        Self {
            svv_eff_type,
            alter_eff: AttrAlterEff::new(alter_type, eff),
        }
    }

    pub fn new_from_alter_eff(svv_eff_type: SurvivalEffType, alter_eff: AttrAlterEff<S>) -> Self {
        Self {
            svv_eff_type,
            alter_eff,
        }
    }
}

// endregion

// region: 效果 buffer

/// 归一化后的效果
///
/// 使用 [`super::damage_systems::normalize_damage_eff`] 进行归一化
#[derive(Debug)]
pub struct SurvivalNormalizedAttrEff<S: FixedName> {
    pub(super) svv_eff_type: SurvivalEffType,
    pub(super) eff: Effect<S>,
}

#[derive(Debug)]
pub struct SurvivalEffBuffer<S: FixedName>(Vec<SurvivalNormalizedAttrEff<S>>);

impl<S: FixedName> Default for SurvivalEffBuffer<S> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<S: FixedName> SurvivalEffBuffer<S> {
    pub fn push(&mut self, eff: SurvivalNormalizedAttrEff<S>) {
        self.0.push(eff);
    }

    pub fn take_effs(&mut self) -> impl Iterator<Item = SurvivalNormalizedAttrEff<S>> {
        self.0.drain(..)
    }
}

// endregion

// region: 实现复合属性

pub struct SurvivalAttrRef<'a> {
    pub health: &'a mut Health,
    pub shield_substitute: &'a mut ShieldSubstitute,
    pub shield_defence: &'a mut ShieldDefence,
    pub shield_arcane: &'a mut ShieldArcane,
}

impl CompoundAttr<SurvivalEffType> for SurvivalAttrRef<'_> {
    fn get_attr_mut(
        &mut self,
        layer_type: <SurvivalEffType as AttrLayerEffType>::LayerType,
    ) -> &mut BoundedAttr {
        match layer_type {
            SurvivalAttrLayer::Health => &mut self.health.0,
            SurvivalAttrLayer::ShieldSubstitute => &mut self.shield_substitute.0,
            SurvivalAttrLayer::ShieldDefence => &mut self.shield_defence.0,
            SurvivalAttrLayer::ShieldArcane => &mut self.shield_arcane.0,
        }
    }
}

pub struct SurvivalBoundRef<'a> {
    pub health_lower: &'a HealthLower,
    pub health_upper: &'a HealthUpper,
    pub shield_substitute_upper: &'a ShieldSubstituteUpper,
    pub shield_defence_upper: &'a ShieldDefenceUpper,
    pub shield_arcane_upper: &'a ShieldArcaneUpper,
}

impl SurvivalBoundRef<'_> {
    #[inline]
    pub fn get_upper(
        &self,
        layer_type: <SurvivalEffType as AttrLayerEffType>::LayerType,
    ) -> &BoundAttr {
        match layer_type {
            SurvivalAttrLayer::Health => &self.health_upper.0,
            SurvivalAttrLayer::ShieldSubstitute => &self.shield_substitute_upper.0,
            SurvivalAttrLayer::ShieldDefence => &self.shield_defence_upper.0,
            SurvivalAttrLayer::ShieldArcane => &self.shield_arcane_upper.0,
        }
    }
}

impl CompoundAttrBound<SurvivalEffType> for SurvivalBoundRef<'_> {
    fn gen_bound_range(
        &self,
        layer_type: <SurvivalEffType as AttrLayerEffType>::LayerType,
    ) -> BoundRange {
        match layer_type {
            SurvivalAttrLayer::Health => {
                BoundRange::new(&self.health_lower.0, self.get_upper(layer_type))
            }
            SurvivalAttrLayer::ShieldSubstitute => {
                BoundRange::new(SHIELD_LOWER, self.get_upper(layer_type))
            }
            SurvivalAttrLayer::ShieldDefence => {
                BoundRange::new(SHIELD_LOWER, self.get_upper(layer_type))
            }
            SurvivalAttrLayer::ShieldArcane => {
                BoundRange::new(SHIELD_LOWER, self.get_upper(layer_type))
            }
        }
    }
}

// endregion

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::base_lib::eff_attr::attr_layers::attr_layer_system;

    use super::*;

    // 在单元测试中检查复合属性的层级规划，以避免运行时开销

    #[test]
    fn check_attr_layer() {
        for ele in SurvivalAttrLayer::iter() {
            attr_layer_system::check_attr_layer(ele);
        }
    }

    #[test]
    fn check_attr_eff() {
        for ele in SurvivalEffType::iter() {
            attr_layer_system::check_attr_layer_eff_type(ele);
        }
    }

    /// 检查预设的效果生效顺序
    #[test]
    fn check_attr_eff_sort() {
        let builtin_array = SurvivalEffType::SORTED_ARRAY.to_vec();

        let mut checked_array: Vec<SurvivalEffType> = SurvivalEffType::iter().collect();
        checked_array.sort_by(attr_layer_system::rank_attr_layer_eff);

        assert_eq!(builtin_array, checked_array);
    }
}
