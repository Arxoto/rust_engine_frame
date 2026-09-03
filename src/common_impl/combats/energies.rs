use std::marker::PhantomData;

use strum_macros::EnumIter;

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        attr_layers::{AttrLayerEffTarget, AttrLayerType},
        bound_attr_modifiers::BoundAttrModifier,
        bound_attrs::{BoundAttr, BoundRange},
        bounded_attrs::BoundedAttr,
        compound_attr_systems::{CompoundAttr, CompoundAttrBound},
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
pub struct ExternalEnergyUpper(pub BoundAttr);
pub struct ExternalEnergyUpperEffs(pub ModifierCollection<BoundAttrModifier>);

pub const MAGICKA_ENERGY_LOWER: f64 = 0.0;

// endregion

// region: 属性层级

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

// endregion

// region: 效果定义

/// 能量消耗统一路径
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum EnergyEffTargets {
    #[default]
    Standard,
}

impl AttrLayerEffTarget for EnergyEffTargets {
    type Layer = EnergyAttrLayer;

    fn start_at(&self) -> Self::Layer {
        match self {
            EnergyEffTargets::Standard => EnergyAttrLayer::ExternalEnergy,
        }
    }

    fn stop_at(&self) -> Self::Layer {
        match self {
            EnergyEffTargets::Standard => EnergyAttrLayer::Magicka,
        }
    }
}

// endregion

// region: 能量消耗效果 Buffer

/// 由于能量消耗逻辑比较简单，因此未了节省内存空间，不用列表存储实际效果（为了兼容，保留泛型）
#[derive(Debug)]
pub struct EnergyEffBuffer<S: FixedName>(f64, PhantomData<S>);

impl<S: FixedName> Default for EnergyEffBuffer<S> {
    fn default() -> Self {
        Self(0.0, PhantomData)
    }
}

impl<S: FixedName> EnergyEffBuffer<S> {
    pub fn push_delta_val(&mut self, v: f64) {
        self.0 += v;
    }

    pub fn take_delta_val(&mut self) -> f64 {
        let delta_val = self.0;
        self.0 = 0.0;
        delta_val
    }
}

// endregion

// region: 实现复合属性

pub struct EnergyAttrRef<'a> {
    pub magicka: &'a mut Magicka,
    pub ex_energy: &'a mut ExternalEnergy,
}

impl CompoundAttr<EnergyEffTargets> for EnergyAttrRef<'_> {
    fn get_attr_mut(
        &mut self,
        target_layer: <EnergyEffTargets as AttrLayerEffTarget>::Layer,
    ) -> &mut BoundedAttr {
        match target_layer {
            EnergyAttrLayer::Magicka => &mut self.magicka.0,
            EnergyAttrLayer::ExternalEnergy => &mut self.ex_energy.0,
        }
    }
}

pub struct EnergyBoundRef<'a> {
    pub magicka_upper: &'a MagickaUpper,
    pub ex_energy_upper: &'a ExternalEnergyUpper,
}

impl CompoundAttrBound<EnergyEffTargets> for EnergyBoundRef<'_> {
    fn gen_bound_range(
        &self,
        target_layer: <EnergyEffTargets as AttrLayerEffTarget>::Layer,
    ) -> BoundRange {
        match target_layer {
            EnergyAttrLayer::Magicka => {
                BoundRange::new(MAGICKA_ENERGY_LOWER, &self.magicka_upper.0)
            }
            EnergyAttrLayer::ExternalEnergy => {
                BoundRange::new(MAGICKA_ENERGY_LOWER, &self.ex_energy_upper.0)
            }
        }
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

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::base_lib::eff_attr::attr_layers::attr_layer_system;

    use super::*;

    #[test]
    fn check_attr_layer() {
        for ele in EnergyAttrLayer::iter() {
            attr_layer_system::check_attr_layer(ele);
        }
    }

    #[test]
    fn check_attr_eff() {
        for ele in EnergyEffTargets::iter() {
            attr_layer_system::check_attr_layer_eff_target(ele);
        }
    }

    // 因为目前能量消耗只有一种路径，因此没有预置顺序，无需检查效果排序
    #[test]
    fn check_attr_eff_sort() {}
}
