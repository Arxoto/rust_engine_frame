use crate::effects::native_effect::{Effect, ProxyEffect};

/// 瞬时效果
#[derive(Default, Clone)]
pub struct InstantEffect<S>(Effect<S>);

impl<S> ProxyEffect<S> for InstantEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        &self.0
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        &mut self.0
    }
}

impl<S> InstantEffect<S> {
    /// 瞬时效果
    pub fn new_instant<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self(Effect::new(from_name, effect_name, value))
    }

    pub fn new(effect: Effect<S>) -> Self {
        Self(effect)
    }
}
