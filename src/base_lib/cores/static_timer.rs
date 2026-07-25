//! 基于绝对时间戳实现的计时器
//! - 【缺点】不支持循环触发，代码实现和使用复杂
//! - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
//! - 适用于服务端验证、长期计时，注意需要修改时间类型为高精度类型，如 [`std::time::Duration`]
//! - 数量规模庞大的场景，将大量的循环触发的 TickTimer 转换为 StaticTimer 表示状态和少量触发式 TickTimer 用于结算

use crate::base_lib::cores::{design_patterns::ContextWrapper, tiny_timer::{FlowingTimer, FlowingTimerReadonly, TinyTimer}};

#[derive(Clone, Copy, Debug)]
pub struct StaticTimeline(pub f64);

impl StaticTimeline {
    pub fn tick(&mut self, delta: f64) {
        self.0 += delta;
    }

    // todo 暂停功能放这里
}

#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 计时器时长
    duration: f64,
    /// 计时结束时间
    end_at: f64,
}

impl StaticTimer {
    pub fn new(timeline: StaticTimeline, duration: f64) -> Self {
        Self {
            duration,
            end_at: timeline.0 + duration,
        }
    }
}

impl TinyTimer for ContextWrapper<&StaticTimer, &StaticTimeline> {
    fn get_time(&self) -> f64 {
        self.get_time_limit() - self.get_time_left()
    }

    fn get_time_left(&self) -> f64 {
        (self.inner.end_at - self.ctx.0).min(0.0)
    }

    fn get_time_limit(&self) -> f64 {
        self.inner.duration
    }

    fn get_time_ratio(&self) -> f64 {
        1.0 - self.get_time_left() / self.get_time_limit()
    }
}

// 如有必要可实现 TickTimer 但是里面的 tick 应该是空逻辑

impl FlowingTimerReadonly for ContextWrapper<&StaticTimer, &StaticTimeline> {
    fn is_finished(&self) -> bool {
        self.ctx.0 >= self.inner.end_at
    }
}

impl FlowingTimerReadonly for ContextWrapper<&mut StaticTimer, &mut StaticTimeline> {
    fn is_finished(&self) -> bool {
        self.ctx.0 >= self.inner.end_at
    }
}

impl FlowingTimer for ContextWrapper<&mut StaticTimer, &mut StaticTimeline> {
    fn restart(&mut self) {
        self.inner.end_at = self.ctx.0 + self.inner.duration;
    }

    fn finish(&mut self) {
        self.inner.end_at = self.ctx.0;
    }
}