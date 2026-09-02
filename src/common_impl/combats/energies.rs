use std::ops::{Deref, DerefMut};

use strum_macros::EnumIter;

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        attr_layers::AttrLayerType, bound_attr_modifiers::BoundAttrModifier,
        bound_attrs::BoundAttr, bounded_attr_effs::AttrAlterEff, bounded_attrs::BoundedAttr,
        modifier_collections::ModifierCollection,
    },
};

// region: 属性定义

/// 魔能（气势） 被战时评价系统控制； 基础值被【信念】的基础值和能级系统影响
pub struct Magicka(pub BoundedAttr);

/// 外部能源 环境逸散的自由态能量
pub struct ExternalEnergy(pub BoundedAttr);

pub struct MagickaUpper(pub BoundAttr);
pub struct MagickaUpperEffs(pub ModifierCollection<BoundAttrModifier>);

// endregion

// region: 能量消耗效果 Buffer

/// 能量消耗统一路径，因此无需自定义效果类型
#[derive(Debug)]
pub struct EnergyEffBuffer<S: FixedName>(Vec<AttrAlterEff<S>>);

impl<S: FixedName> Default for EnergyEffBuffer<S> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<S: FixedName> Deref for EnergyEffBuffer<S> {
    type Target = Vec<AttrAlterEff<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: FixedName> DerefMut for EnergyEffBuffer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// endregion

// region: 魔法能级

/// 魔法能级划分
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

// endregion

// region: 能量消耗逻辑

/// 能量消耗：目前仅一种消耗逻辑，只有一种路径
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum EnergyAttrLayer {
    Magicka,
    ExternalEnergy,
}

impl AttrLayerType for EnergyAttrLayer {
    fn get_next(&self) -> Self {
        match self {
            EnergyAttrLayer::Magicka => Self::Magicka,
            EnergyAttrLayer::ExternalEnergy => Self::Magicka,
        }
    }

    fn get_layer(&self) -> u8 {
        match self {
            EnergyAttrLayer::Magicka => 0,
            EnergyAttrLayer::ExternalEnergy => 1,
        }
    }
}

impl EnergyAttrLayer {
    /// 能量消耗统一路径
    #[inline]
    pub fn start_at() -> Self {
        Self::ExternalEnergy
    }
}

// endregion

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        base_lib::eff_attr::{attr_layers::attr_layer_system, stat_attrs::StatAttr},
        common_impl::combats::{
            combat_inherents::Belief,
            energy_systems::{calc_magicka_max, calc_magicka_value},
        },
    };

    use super::*;

    /// 检查能量属性层级，因为能耗目前只有一种类型，因此不对效果做检查
    #[test]
    fn check_energy_layer() {
        for ele in EnergyAttrLayer::iter() {
            attr_layer_system::check_attr_layer(ele);
        }
    }

    /// calc_magicka_value：Belief 影响原始能量，magicka_base + magicka_scale * belief.origin
    #[test]
    fn calc_magicka_value_scales_with_belief_origin() {
        let belief = Belief(StatAttr::new(10.0));
        assert_eq!(calc_magicka_value(50.0, 3.0, &belief), 80.0);
    }

    /// calc_magicka_max：先算原始能量，再按能级取对应层级上限
    #[test]
    fn calc_magicka_max_takes_energy_level() {
        let levels = MagickaEnergyLevel::new(100.0, 200.0, 300.0);
        // 原始能量 50 + 3*10 = 80 → 第一能级上限 100
        let belief = Belief(StatAttr::new(10.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 100.0);
        // 原始能量 50 + 3*50 = 200 → 第二能级上限 200
        let belief = Belief(StatAttr::new(50.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 200.0);
        // 原始能量 50 + 3*80 = 290 → 第三能级上限 300
        let belief = Belief(StatAttr::new(80.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 300.0);
    }
}
