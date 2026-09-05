use crate::base_lib::{cores::unify_types::FixedName, eff_attr::upserts::Upsert};

/// 效果描述 不实现具体效果
#[derive(Clone, Debug)]
pub struct Effect<S: FixedName> {
    /// 效果来源，始终是角色名称，一般用于结算记录
    from_name: S,
    /// 效果名称，一般与来源效果名称相关，用作事件触发时主要根据名称生效作用
    effect_name: S,
    /// 效果值，部分效果的生效不取决于该值，但仍可根据正负判断是否增益
    effect_value: f64,
}

impl<S: FixedName> Effect<S> {
    pub fn new(from_name: S, effect_name: S, effect_value: f64) -> Self {
        Self {
            from_name,
            effect_name,
            effect_value,
        }
    }

    pub fn new_from(from_name: impl Into<S>, effect_name: impl Into<S>, effect_value: f64) -> Self {
        let from_name: S = from_name.into();
        let effect_name: S = effect_name.into();
        Self {
            from_name,
            effect_name,
            effect_value,
        }
    }

    // region: getter setter

    pub fn take_from_eff_name(self) -> EffId<S> {
        EffId {
            from_name: self.from_name,
            effect_name: self.effect_name,
        }
    }

    pub fn get_from_name(&self) -> &S {
        &self.from_name
    }

    pub fn get_effect_name(&self) -> &S {
        &self.effect_name
    }

    pub fn get_effect_value(&self) -> f64 {
        self.effect_value
    }

    pub fn set_effect_value(&mut self, v: f64) {
        self.effect_value = v;
    }

    // endregion
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffId<S: FixedName> {
    pub from_name: S,
    pub effect_name: S,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffIdRef<'a, S: FixedName> {
    pub from_name: &'a S,
    pub effect_name: &'a S,
}

impl<'a, S: FixedName> Copy for EffIdRef<'a, S> {}

impl<S: FixedName> Upsert for Effect<S> {
    type Id = EffId<S>;
    type IdRef<'a>
        = EffIdRef<'a, S>
    where
        Self: 'a;

    fn gen_id(&self) -> Self::Id {
        EffId {
            from_name: self.from_name.clone(),
            effect_name: self.effect_name.clone(),
        }
    }

    fn id_ref<'a>(&'a self) -> Self::IdRef<'a> {
        EffIdRef {
            from_name: &self.from_name,
            effect_name: &self.effect_name,
        }
    }
}

/// 判断增益或减益效果
pub trait EffectMeaning {
    /// 判断增益或减益效果
    fn which_nature(&self) -> EffectMean;
}

/// 增益或减益效果标识
#[derive(Clone, Copy, Debug)]
pub enum EffectMean {
    /// 减益效果
    Bad,
    /// 增益效果
    Good,
    /// 中性效果
    Neutral,
}

impl EffectMean {
    pub fn which_nature(value: f64, base_line: f64) -> Self {
        if value > base_line {
            Self::Good
        } else if value < base_line {
            Self::Bad
        } else {
            Self::Neutral
        }
    }

    /// 增益效果
    pub fn is_good(&self) -> bool {
        matches!(self, Self::Good)
    }

    /// 减益效果
    pub fn is_bad(&self) -> bool {
        matches!(self, Self::Bad)
    }

    /// 中性效果
    pub fn is_neutral(&self) -> bool {
        matches!(self, Self::Neutral)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EffectMean 基线：value 高于基线为增益、低于为减益、等于为中性
    /// （加法类基线 0 、乘法类基线 1 ）
    #[test]
    fn effect_mean_which_nature_by_baseline() {
        // 加法运算 基线 1
        assert!(EffectMean::which_nature(5.0, 0.0).is_good());
        assert!(EffectMean::which_nature(-5.0, 0.0).is_bad());
        assert!(EffectMean::which_nature(0.0, 0.0).is_neutral());

        // 乘法运算 基线 1
        assert!(EffectMean::which_nature(1.5, 1.0).is_good());
        assert!(EffectMean::which_nature(0.5, 1.0).is_bad());
        assert!(EffectMean::which_nature(1.0, 1.0).is_neutral());
    }
}
