use crate::{
    attrs::dyn_prop_inst_effect::{DynPropInstEffect, DynPropInstEffectType},
    cores::unify_type::FixedName,
    effects::native_effect::Effect,
};

impl<S: FixedName> DynPropInstEffect<S> {
    pub fn new_cur_val(effect: Effect<S>) -> Self {
        Self::new(DynPropInstEffectType::CurVal, effect)
    }

    pub fn new_cur_per(effect: Effect<S>) -> Self {
        Self::new(DynPropInstEffectType::CurPer, effect)
    }

    pub fn new_cur_max_per(effect: Effect<S>) -> Self {
        Self::new(DynPropInstEffectType::CurMaxPer, effect)
    }
}
