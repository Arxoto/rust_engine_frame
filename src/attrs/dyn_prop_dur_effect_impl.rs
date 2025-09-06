use crate::{attrs::dyn_prop_dur_effect::{DynPropDurEffect, DynPropDurEffectType}, cores::unify_type::FixedName, effects::duration_effect::DurationEffect};

impl<S> DynPropDurEffect<S>
where
    S: FixedName,
{

    // =================================================================================

    pub fn new_infinite_max_val(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_infinite(DynPropDurEffectType::MaxVal, from_name, effect_name, value)
    }

    pub fn new_infinite_max_per(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_infinite(DynPropDurEffectType::MaxPer, from_name, effect_name, value)
    }

    pub fn new_infinite_min_val(from_name: S, effect_name: S, value: f64) -> Self {
        Self::new_infinite(DynPropDurEffectType::MinVal, from_name, effect_name, value)
    }

    // =================================================================================

    pub fn new_duration_max_val(from_name: S, effect_name: S, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynPropDurEffectType::MaxVal, from_name, effect_name, value, duration_time)
    }

    pub fn new_duration_max_per(from_name: S, effect_name: S, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynPropDurEffectType::MaxPer, from_name, effect_name, value, duration_time)
    }

    pub fn new_duration_min_val(from_name: S, effect_name: S, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynPropDurEffectType::MinVal, from_name, effect_name, value, duration_time)
    }

    // =================================================================================

    pub fn new_max_val(effect: DurationEffect<S>) -> Self {
        Self::new(DynPropDurEffectType::MaxVal, effect)
    }

    pub fn new_max_per(effect: DurationEffect<S>) -> Self {
        Self::new(DynPropDurEffectType::MaxPer, effect)
    }

    pub fn new_min_val(effect: DurationEffect<S>) -> Self {
        Self::new(DynPropDurEffectType::MinVal, effect)
    }
}