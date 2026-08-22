/// 效果描述 不实现具体效果
///
/// 可与计时器组合实现复杂效果：
/// - 持续效果、持续触发效果
/// - 延迟生效效果，计时结束后自动添加另一个效果
///
/// （堆叠逻辑交由上层实现，这里注释暂且保留，待上层实现后转移） todo
///
/// 效果可以堆叠，但对于如何叠加效，应该根据上层业务需求自己判断，不同堆叠优先级可以诞生有趣的策略，如：
///
/// - 不同效果的延迟、频率、层数上限、层数、强度不同：初始效果决定频率和层数上限、中间快速堆叠层数、最后施加高强度、选择快速冷却的效果延续时长
/// - 某效果根据延迟生效的时长增加伤害，施加重置延迟效果，在最后造成大量伤害，这种机制也可替换成堆叠效果组合（同时施加重置延迟和堆叠层数两种效果）
///
/// 注意：若允许不同来源的效果可叠加，那么必然会导致伤害结算存在误差：叠加产生的额外收益算谁的，这划分给谁都不合适，也许可以算成团队收益
#[derive(Clone, Debug)]
pub struct Effect<S> {
    /// 效果来源，始终是角色名称，一般用于结算记录
    from_name: S,
    /// 效果名称，一般与来源效果名称相关，用作事件触发时主要根据名称生效作用
    effect_name: S,
    /// 效果值，部分效果的生效不取决于该值，但仍可根据正负判断是否增益
    effect_value: f64,
}

impl<S> Effect<S> {
    pub fn new(from_name: S, effect_name: S, effect_value: f64) -> Self {
        Self {
            from_name,
            effect_name,
            effect_value,
        }
    }

    pub fn new_form<T: Into<S>>(from_name: T, effect_name: T, effect_value: f64) -> Self {
        let from_name: S = from_name.into();
        let effect_name: S = effect_name.into();
        Self {
            from_name,
            effect_name,
            effect_value,
        }
    }

    // region: getter setter

    pub fn take_from_eff_name(self) -> (S, S) {
        (self.from_name, self.effect_name)
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
