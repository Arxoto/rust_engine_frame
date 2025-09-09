use crate::{
    attrs::dyn_prop_period_effect::{DynPropPeriodEffect, DynPropPeriodEffectType},
    cores::unify_type::FixedName,
    effects::{native_duration::Duration, native_effect::Effect},
};

impl<S: FixedName> DynPropPeriodEffect<S> {
    pub fn new_val(effect: (Effect<S>, Duration), period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::Val, effect, period_time)
    }

    pub fn new_cur_per(effect: (Effect<S>, Duration), period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::CurPer, effect, period_time)
    }

    pub fn new_max_per(effect: (Effect<S>, Duration), period_time: f64) -> Self {
        Self::new(DynPropPeriodEffectType::MaxPer, effect, period_time)
    }

    pub fn new_cur_val_to_val(
        effect: (Effect<S>, Duration),
        to_val: f64,
        period_time: f64,
    ) -> Self {
        Self::new(
            DynPropPeriodEffectType::CurValToVal(to_val),
            effect,
            period_time,
        )
    }
}
