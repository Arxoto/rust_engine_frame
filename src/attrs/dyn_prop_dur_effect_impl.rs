use crate::{
    attrs::dyn_prop_dur_effect::{DynPropDurEffect, DynPropDurEffectType},
    cores::unify_type::FixedName,
    effects::{native_duration::Duration, native_effect::Effect},
};

impl<S: FixedName> DynPropDurEffect<S> {
    pub fn new_max_val(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynPropDurEffectType::MaxVal, effect)
    }

    pub fn new_max_per(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynPropDurEffectType::MaxPer, effect)
    }

    pub fn new_min_val(effect: (Effect<S>, Duration)) -> Self {
        Self::new(DynPropDurEffectType::MinVal, effect)
    }
}
