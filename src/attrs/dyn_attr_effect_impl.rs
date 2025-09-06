use crate::{attrs::dyn_attr_effect::{DynAttrEffect, DynAttrEffectType}, effects::duration_effect::DurationEffect};

impl<S: Clone> DynAttrEffect<S> {

    // =================================================================================

    pub fn new_infinite_basic_add<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self::new_infinite(DynAttrEffectType::BasicAdd, from_name, effect_name, value)
    }

    pub fn new_infinite_final_multi<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self::new_infinite(DynAttrEffectType::FinalMulti, from_name, effect_name, value)
    }

    pub fn new_infinite_basic_percent<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self::new_infinite(DynAttrEffectType::BasicPercent, from_name, effect_name, value)
    }

    pub fn new_infinite_final_percent<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self::new_infinite(DynAttrEffectType::FinalPercent, from_name, effect_name, value)
    }

    // =================================================================================

    pub fn new_duration_basic_add<T: Into<S>>(from_name: T, effect_name: T, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynAttrEffectType::BasicAdd, from_name, effect_name, value, duration_time)
    }

    pub fn new_duration_final_multi<T: Into<S>>(from_name: T, effect_name: T, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynAttrEffectType::FinalMulti, from_name, effect_name, value, duration_time)
    }

    pub fn new_duration_basic_percent<T: Into<S>>(from_name: T, effect_name: T, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynAttrEffectType::BasicPercent, from_name, effect_name, value, duration_time)
    }

    pub fn new_duration_final_percent<T: Into<S>>(from_name: T, effect_name: T, value: f64, duration_time: f64) -> Self {
        Self::new_duration(DynAttrEffectType::FinalPercent, from_name, effect_name, value, duration_time)
    }

    // =================================================================================

    pub fn new_basic_add(effect: DurationEffect<S>) -> Self {
        Self::new(DynAttrEffectType::BasicAdd, effect)
    }

    pub fn new_final_multi(effect: DurationEffect<S>) -> Self {
        Self::new(DynAttrEffectType::FinalMulti, effect)
    }

    pub fn new_basic_percent(effect: DurationEffect<S>) -> Self {
        Self::new(DynAttrEffectType::BasicPercent, effect)
    }

    pub fn new_final_percent(effect: DurationEffect<S>) -> Self {
        Self::new(DynAttrEffectType::FinalPercent, effect)
    }
}
