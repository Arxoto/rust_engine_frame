use crate::effects::native_duration::{Duration, ProxyDuration};
use crate::effects::native_effect::{Effect, ProxyEffect};

pub struct EffectBuilder;

impl EffectBuilder {
    /// 瞬时效果
    pub fn new_instant<S, T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Effect<S> {
        Effect::new(from_name, effect_name, value)
    }

    /// 持久效果（无限存在）
    pub fn new_infinite<S, T: Into<S>>(
        from_name: T,
        effect_name: T,
        value: f64,
    ) -> (Effect<S>, Duration) {
        (
            Effect::new(from_name, effect_name, value),
            Duration::new_infinite(),
        )
    }

    /// 持久效果（持续时间）
    pub fn new_duration<S, T: Into<S>>(
        from_name: T,
        effect_name: T,
        value: f64,
        duration_time: f64,
    ) -> (Effect<S>, Duration) {
        (
            Effect::new(from_name, effect_name, value),
            Duration::new_duration(duration_time),
        )
    }
}

// =================================================================================

/// 代理类型 持续型效果
pub trait ProxyDurationEffect<S: Clone>: ProxyEffect<S> + ProxyDuration {
    /// 刷新
    fn refresh_with_name<T: ProxyEffect<S>>(&mut self, eff: &T) {
        self.set_from_name(eff.get_from_name().clone());
        self.restart_life();
    }

    /// 刷新并赋值
    fn refresh_with_name_value<T: ProxyEffect<S>>(&mut self, eff: &T) {
        self.set_value(eff.get_value());
        self.refresh_with_name(eff);
    }

    /// 刷新并赋值堆叠 返回叠加后的层数
    fn refresh_with_name_value_stack<T: ProxyDurationEffect<S>>(&mut self, eff: &T) -> i64 {
        // 设计 延迟和周期等不重置 以此顺序可最大收益 低延迟高频次（初始确定上限）-高层数（快速叠加层数）-高加成（最终伤害）-快速冷却（持续刷新时间过渡）
        self.refresh_with_name_value(eff);
        self.try_add_stack(eff.get_stack())
    }
}

// =================================================================================

type DurationEffect<S> = (Effect<S>, Duration);

impl<S> ProxyEffect<S> for DurationEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        &self.0
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        &mut self.0
    }
}

impl<S> ProxyDuration for DurationEffect<S> {
    fn as_duration(&self) -> &Duration {
        &self.1
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        &mut self.1
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DurationEffect<S> {}
