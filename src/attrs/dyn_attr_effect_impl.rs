use crate::{
    attrs::dyn_attr_effect::{DynAttrEffect, DynAttrEffectType},
    cores::unify_type::FixedName,
    effects::{native_duration::Duration, native_effect::Effect},
};

impl<S: FixedName> DynAttrEffect<S> {
    pub fn new_basic_add(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynAttrEffectType::BasicAdd, effect)
    }

    pub fn new_final_multi(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynAttrEffectType::FinalMulti, effect)
    }

    pub fn new_basic_percent(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynAttrEffectType::BasicPercent, effect)
    }

    pub fn new_final_percent(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynAttrEffectType::FinalPercent, effect)
    }
}
