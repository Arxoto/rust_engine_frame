use crate::{attrs::dyn_prop_period_effect::{DynPropPeriodEffect, DynPropPeriodEffectType}, cores::unify_type::FixedName, effects::duration_effect::DurationEffect};

impl<S> DynPropPeriodEffect<S>
where
    S: FixedName,
{

    // =================================================================================

    pub fn new_infinite_cur_val(from_name: S, effect_name: S, value: f64, period_time: f64) -> Self {
        Self::new_infinite(DynPropPeriodEffectType::CurVal, from_name, effect_name, value, period_time)
    }

    pub fn new_infinite_cur_per(from_name: S, effect_name: S, value: f64, period_time: f64) -> Self {
        Self::new_infinite(DynPropPeriodEffectType::CurPer, from_name, effect_name, value, period_time)
    }

    pub fn new_infinite_cur_max_per(from_name: S, effect_name: S, value: f64, period_time: f64) -> Self {
        Self::new_infinite(DynPropPeriodEffectType::CurMaxPer, from_name, effect_name, value, period_time)
    }

    pub fn new_infinite_cur_val_to_val(from_name: S, effect_name: S, value: f64, to_val: f64, period_time: f64) -> Self {
        Self::new_infinite(DynPropPeriodEffectType::CurValToVal(to_val), from_name, effect_name, value, period_time)
    }

    // =================================================================================

    pub fn new_duration_cur_val(from_name: S, effect_name: S, value: f64, duration_time: f64, period_time: f64) -> Self {
        Self::new_duration(DynPropPeriodEffectType::CurVal, from_name, effect_name, value, duration_time, period_time)
    }

    pub fn new_duration_cur_per(from_name: S, effect_name: S, value: f64, duration_time: f64, period_time: f64) -> Self {
        Self::new_duration(DynPropPeriodEffectType::CurPer, from_name, effect_name, value, duration_time, period_time)
    }

    pub fn new_duration_cur_max_per(from_name: S, effect_name: S, value: f64, duration_time: f64, period_time: f64) -> Self {
        Self::new_duration(DynPropPeriodEffectType::CurMaxPer, from_name, effect_name, value, duration_time, period_time)
    }

    pub fn new_duration_cur_val_to_val(from_name: S, effect_name: S, value: f64, to_val: f64, duration_time: f64, period_time: f64) -> Self {
        Self::new_duration(DynPropPeriodEffectType::CurValToVal(to_val), from_name, effect_name, value, duration_time, period_time)
    }

    // =================================================================================

    pub fn new_cur_val(effect: DurationEffect<S>, period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::CurVal, effect, period_time)
    }

    pub fn new_cur_per(effect: DurationEffect<S>, period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::CurPer, effect, period_time)
    }

    pub fn new_cur_max_per(effect: DurationEffect<S>, period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::CurMaxPer, effect, period_time)
    }

    pub fn new_cur_val_to_val(effect: DurationEffect<S>, to_val: f64, period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::CurValToVal(to_val), effect, period_time)
    }
}