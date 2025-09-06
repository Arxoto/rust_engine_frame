/// 效果描述 不实现具体效果
#[derive(Default, Clone)]
pub struct Effect<S> {
    /// 效果名称
    effect_name: S,
    /// 效果来源
    from_name: S,
    /// 效果值 部分效果的生效不取决于该值 但仍可根据正负判断是否增益
    value: f64,
}

impl<S> Effect<S> {
    pub fn new<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self {
            effect_name: effect_name.into(),
            from_name: from_name.into(),
            value,
        }
    }
}

// =================================================================================

/// 代理类型效果描述
pub trait ProxyEffect<S> {
    fn as_effect(&self) -> &Effect<S>;
    fn as_mut_effect(&mut self) -> &mut Effect<S>;

    fn get_effect_name(&self) -> &S {
        &self.as_effect().effect_name
    }

    fn set_effect_name(&mut self, v: S) {
        self.as_mut_effect().effect_name = v
    }

    fn get_from_name(&self) -> &S {
        &self.as_effect().from_name
    }

    fn set_from_name(&mut self, v: S) {
        self.as_mut_effect().from_name = v
    }

    fn get_value(&self) -> f64 {
        self.as_effect().value
    }

    fn set_value(&mut self, v: f64) {
        self.as_mut_effect().value = v
    }
}

impl<S> ProxyEffect<S> for Effect<S> {
    fn as_effect(&self) -> &Effect<S> {
        self
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        self
    }
}
