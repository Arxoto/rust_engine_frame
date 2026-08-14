//! prop 属性的上下限效果，表现为血量上下限

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        attr_eff::{AttrEffId, AttrEffect, AttrEffectType},
        effects::{Effect, EffectMeaning},
        upsert_container::Upsert,
    },
};

/// Prop 属性边界效果的类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropBoundsEffectType {
    UpperAdd,
    UpperPer,
    LowerAdd,
}

/// Prop 属性边界效果的修改对象
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropBoundsEffectTarget {
    Upper,
    Lower,
}

/// Prop 属性边界效果，本质为 [`AttrEffect`] ，但只支持的维度有限制
///
/// 若想在修改上限的同时修改实际值，那么需要同时生成【修改上限】的效果和【修改实际值】的效果
///
/// 为了保证两者修改效果一致，限制修改维度只能基于基础值修改（不会被放大缩小产生偏差）
#[derive(Clone, Debug)]
pub struct PropBoundsEffect<S: FixedName, Timer> {
    target: PropBoundsEffectTarget,
    eff: AttrEffect<S, Timer>,
}

impl<S: FixedName, Timer> PropBoundsEffect<S, Timer> {
    pub fn new(eff_type: PropBoundsEffectType, effect: Effect<S>, duration: Timer) -> Self {
        match eff_type {
            PropBoundsEffectType::UpperAdd => Self {
                target: PropBoundsEffectTarget::Upper,
                eff: AttrEffect::new(AttrEffectType::BasicAdd, effect, duration),
            },
            PropBoundsEffectType::UpperPer => Self {
                target: PropBoundsEffectTarget::Upper,
                eff: AttrEffect::new(AttrEffectType::BasicPer, effect, duration),
            },
            PropBoundsEffectType::LowerAdd => Self {
                target: PropBoundsEffectTarget::Lower,
                eff: AttrEffect::new(AttrEffectType::BasicAdd, effect, duration),
            },
        }
    }

    pub fn get_target(&self) -> PropBoundsEffectTarget {
        self.target
    }

    pub fn get_eff(&self) -> &AttrEffect<S, Timer> {
        &self.eff
    }
}

impl<S: FixedName, Timer> Upsert for PropBoundsEffect<S, Timer> {
    type Id = AttrEffId<S>;

    fn gen_id(&self) -> Self::Id {
        self.eff.gen_id()
    }

    fn matched_id(&self, id: &Self::Id) -> bool {
        self.eff.matched_id(id)
    }

    fn has_same_id(&self, other: &Self) -> bool {
        self.eff.has_same_id(&other.eff)
    }
}

impl<S: FixedName, Timer> EffectMeaning for PropBoundsEffect<S, Timer> {
    fn which_nature(&self) -> super::effects::EffectMean {
        self.eff.which_nature()
    }
}
