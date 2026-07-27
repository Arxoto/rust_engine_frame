/// 效果描述 不实现具体效果
///
/// 可与计时器组合实现复杂效果：
/// - 持续效果、持续触发效果
/// - 延迟生效效果，计时结束后自动添加另一个效果
///
/// 效果可以堆叠，但对于如何叠加效，应该根据上层业务需求自己判断，不同堆叠优先级可以诞生有趣的策略，如：
///
/// - 不同效果的延迟、频率、层数上限、层数、强度不同：初始效果决定频率和层数上限、中间快速堆叠层数、最后施加高强度、选择快速冷却的效果延续时长
/// - 某效果根据延迟生效的时长增加伤害，施加重置延迟效果，在最后造成大量伤害，这种机制也可替换成堆叠效果组合（同时施加重置延迟和堆叠层数两种效果）
///
/// 注意：若允许不同来源的效果可叠加，那么必然会导致伤害结算存在误差：叠加产生的额外收益算谁的，这划分给谁都不合适，也许可以算成团队收益
#[derive(Clone, Debug)]
pub struct Effect<S> {
    /// 效果描述
    effect_desc: EffectDescriptor<S>,
    /// 效果值，部分效果的生效不取决于该值，但仍可根据正负判断是否增益
    effect_value: f64,
    /// 堆叠层数，这里不持有上限信息
    stack_value: i32,
}

/// 效果描述，包含【效果名称】和【来源】
#[derive(Clone, Debug)]
pub struct EffectDescriptor<S> {
    /// 效果来源，一般用于结算记录
    from_name: S,
    /// 效果名称，用作事件触发时主要根据名称生效作用
    effect_name: S,
}

impl<S> Effect<S> {
    pub fn new<T: Into<S>>(
        from_name: T,
        effect_name: T,
        effect_value: f64,
        stack_value: i32,
    ) -> Self {
        let from_name: S = from_name.into();
        let effect_name: S = effect_name.into();
        Self {
            effect_desc: EffectDescriptor {
                from_name,
                effect_name,
            },
            effect_value,
            stack_value,
        }
    }

    // region: getter

    pub fn get_desc(&self) -> &EffectDescriptor<S> {
        &self.effect_desc
    }

    pub fn get_from_name(&self) -> &S {
        &self.effect_desc.from_name
    }

    pub fn get_effect_name(&self) -> &S {
        &self.effect_desc.effect_name
    }

    pub fn get_effect_value(&self) -> f64 {
        self.effect_value
    }

    pub fn get_stack_value(&self) -> i32 {
        self.stack_value
    }

    // endregion

    /// 设置来源名称，有时候可将效果来源从个人更新为团队
    ///
    /// 注意，若 [`Effect::effect_name`] 和 [`Effect::from_name`] 一起用作索引哈希，那么就不应该修改
    pub fn set_from_name(&mut self, from_name: S) {
        self.effect_desc.from_name = from_name
    }

    /// 设置强度值
    pub fn set_eff_val(&mut self, effect_value: f64) {
        self.effect_value = effect_value
    }

    /// 设置强度值
    pub fn set_eff_val_by(&mut self, other: &Self) {
        self.effect_value = other.effect_value
    }

    /// 累加堆叠层数
    pub fn add_eff_stack_by(&mut self, other: &Self, stack_limit: i32) {
        self.stack_value = stack_limit.max(self.stack_value + other.stack_value)
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
    /// 增益效果
    Buff,
    /// 减益效果
    Debuff,
    /// 中性效果
    Neutral,
}

impl EffectMean {
    pub fn which_nature(value: f64, base_line: f64) -> Self {
        if value > base_line {
            Self::Buff
        } else if value < base_line {
            Self::Debuff
        } else {
            Self::Neutral
        }
    }

    /// 增益效果
    pub fn is_buff(&self) -> bool {
        matches!(self, Self::Buff)
    }

    /// 减益效果
    pub fn is_debuff(&self) -> bool {
        matches!(self, Self::Debuff)
    }

    /// 中性效果
    pub fn is_neutral(&self) -> bool {
        matches!(self, Self::Neutral)
    }
}
