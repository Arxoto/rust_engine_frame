use crate::{attrs::dyn_prop_inst_effect::{DynPropInstEffect, DynPropInstEffectType}, cores::unify_type::FixedName, effects::native_effect::Effect};

impl<S> DynPropInstEffect<S>
where
    S: FixedName,
{
    
    // =================================================================================

    pub fn new_instant_cur_val(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_instant(DynPropInstEffectType::CurVal, from_name, effect_name, value)
    }

    pub fn new_instant_cur_per(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_instant(DynPropInstEffectType::CurPer, from_name, effect_name, value)
    }

    pub fn new_instant_cur_max_per(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_instant(DynPropInstEffectType::CurMaxPer, from_name, effect_name, value)
    }
    
    // =================================================================================

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