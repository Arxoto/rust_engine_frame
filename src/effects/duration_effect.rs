use crate::effects::native_duration::{Duration, ProxyDuration};
use crate::effects::native_effect::{Effect, ProxyEffect};

/// 持续型效果
#[derive(Default, Clone)]
pub struct DurationEffect<S> {
    effect: Effect<S>,
    duration: Duration,
}

impl<S> ProxyEffect<S> for DurationEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        &self.effect
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        &mut self.effect
    }
}

impl<S> ProxyDuration for DurationEffect<S> {
    fn as_duration(&self) -> &Duration {
        &self.duration
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        &mut self.duration
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

impl<S: Clone> ProxyDurationEffect<S> for DurationEffect<S> {}

impl<S> DurationEffect<S> {
    /// 无限存在的效果
    pub fn new_infinite<T: Into<S>>(from_name: T, effect_name: T, value: f64) -> Self {
        Self {
            effect: Effect::new(effect_name, from_name, value),
            duration: Duration::new_infinite(),
        }
    }

    /// 持续一段时间的效果
    pub fn new_duration<T: Into<S>>(from_name: T, effect_name: T, value: f64, duration_time: f64) -> Self {
        Self {
            effect: Effect::new(effect_name, from_name, value),
            duration: Duration::new_duration(duration_time),
        }
    }

    pub fn new(effect: Effect<S>, duration: Duration) -> Self {
        Self { effect, duration }
    }
}
