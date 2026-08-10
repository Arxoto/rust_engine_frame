//! 基于绝对时间戳实现的计时器
//! - 【缺点】不支持循环触发，需配合统一时间线一起使用
//! - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
//! - 适用于服务端验证、长期计时，静态时间戳可能需要更换非浮点类型防止误差累积，如 [`std::time::Duration`]
//! - 数量规模庞大的场景，将大量的循环触发的 TickTimer 转换为 StaticTimer 表示状态和少量触发式 TickTimer 用于结算

use crate::base_lib::cores::{
    design_patterns::ContextWrapper,
    tick_timer_builders::FreezeTickTimer,
    tiny_timer::{FlowingTimer, FlowingTimerReadonly, TinyTimer},
};

/// 静态计时器的参考时间线，暂停等功能在时间线上实现
#[derive(Clone, Debug)]
pub struct StaticTimeline(pub FreezeTickTimer);

impl StaticTimeline {
    /// 创建一个永不停止的计时器 用作静态计时器的基准
    pub fn new() -> Self {
        Self(FreezeTickTimer::new(f64::INFINITY))
    }

    pub fn current_time(&self) -> f64 {
        self.0.get_time()
    }

    /// 确认没有依赖本时间线的计时器后，【重启时间线】，防止无限累加引起精度丢失
    pub fn restart_timeline(&mut self) {
        self.0.tick_timer.restart();
    }
}

#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 计时器时长
    duration: f64,
    /// 计时结束时间
    end_at: f64,
}

impl StaticTimer {
    pub fn new(timeline: &StaticTimeline, duration: f64) -> Self {
        Self {
            duration,
            end_at: timeline.current_time() + duration,
        }
    }
}

pub trait HasStaticTimer {
    fn get_static_timer(&self) -> &StaticTimer;
    fn get_static_timer_mut(&mut self) -> &mut StaticTimer;
}

impl TinyTimer for ContextWrapper<&StaticTimer, &StaticTimeline> {
    fn get_time(&self) -> f64 {
        self.get_time_limit() - self.get_time_left()
    }

    fn get_time_left(&self) -> f64 {
        (self.inner.end_at - self.ctx.current_time()).min(0.0)
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
        self.ctx.current_time() >= self.inner.end_at
    }
}

impl FlowingTimerReadonly for ContextWrapper<&mut StaticTimer, &mut StaticTimeline> {
    fn is_finished(&self) -> bool {
        self.ctx.current_time() >= self.inner.end_at
    }
}

impl FlowingTimer for ContextWrapper<&mut StaticTimer, &mut StaticTimeline> {
    fn restart(&mut self) {
        self.inner.end_at = self.ctx.current_time() + self.inner.duration;
    }

    fn finish(&mut self) {
        self.inner.end_at = self.ctx.current_time();
    }
}
