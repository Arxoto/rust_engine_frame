/// 效果描述 不实现具体效果
#[derive(Default, Clone, Debug)]
pub struct Effect<S> {
    /// 效果名称
    pub(crate) effect_name: S,
    /// 效果来源
    pub(crate) from_name: S,
    /// 效果值 部分效果的生效不取决于该值 但仍可根据正负判断是否增益
    pub(crate) value: f64,
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

#[derive(Clone, Copy)]
pub enum EffectNature {
    /// 增益效果
    Buff,
    /// 减益效果
    Debuff,
    /// 中性效果
    Neutral,
}

impl EffectNature {
    /// 根据基线判断效果值为增益减益
    pub fn which_nature(value: f64, base_line: f64) -> EffectNature {
        if value > base_line {
            EffectNature::Buff
        } else if value < base_line {
            EffectNature::Debuff
        } else {
            EffectNature::Neutral
        }
    }
}

// =================================================================================

/// 代理类型 效果描述
pub trait ProxyEffect<S> {
    fn as_effect(&self) -> &Effect<S>;
    fn as_mut_effect(&mut self) -> &mut Effect<S>;

    #[inline]
    fn get_effect_name(&self) -> &S {
        &self.as_effect().effect_name
    }

    #[inline]
    fn set_effect_name(&mut self, v: S) {
        self.as_mut_effect().effect_name = v
    }

    #[inline]
    fn get_from_name(&self) -> &S {
        &self.as_effect().from_name
    }

    #[inline]
    fn set_from_name(&mut self, v: S) {
        self.as_mut_effect().from_name = v
    }

    #[inline]
    fn get_value(&self) -> f64 {
        self.as_effect().value
    }

    #[inline]
    fn set_value(&mut self, v: f64) {
        self.as_mut_effect().value = v
    }

    // ===========================
    // 业务逻辑
    // ===========================

    /// 判断效果为增益减益
    ///
    /// 默认通过值进行判断 参照物默认为 0.0 根据业务逻辑自行覆盖
    #[inline]
    fn which_nature(&self) -> EffectNature {
        EffectNature::which_nature(self.get_value(), 0.0)
    }

    #[inline]
    fn nature_is_buff(&self) -> bool {
        matches!(self.which_nature(), EffectNature::Buff)
    }

    #[inline]
    fn nature_is_debuff(&self) -> bool {
        matches!(self.which_nature(), EffectNature::Debuff)
    }

    #[inline]
    fn nature_is_neutral(&self) -> bool {
        matches!(self.which_nature(), EffectNature::Neutral)
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
