use crate::base_lib::eff_attr_prop::effects::Effect;

pub trait EffectMergeLogic<S> {
    fn merge(&self, origin: &mut Effect<S>, other: &Effect<S>);
}

pub struct ResetValue;

impl<S> EffectMergeLogic<S> for ResetValue {
    fn merge(&self, origin: &mut Effect<S>, other: &Effect<S>) {
        origin.set_eff_val_by(other);
    }
}

pub struct ResetFromName;

impl<S: Clone> EffectMergeLogic<S> for ResetFromName {
    fn merge(&self, origin: &mut Effect<S>, other: &Effect<S>) {
        origin.set_from_name(other.get_from_name().clone());
    }
}

pub struct StackWithLimit(pub i32);

impl<S> EffectMergeLogic<S> for StackWithLimit {
    fn merge(&self, origin: &mut Effect<S>, other: &Effect<S>) {
        origin.add_eff_stack_by(other, self.0);
    }
}
