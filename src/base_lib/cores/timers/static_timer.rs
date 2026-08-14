//! 基于绝对时间戳实现的计时器
//! - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
//! - 【注意】不应该对静态时钟和时间线包装实现暂停功能，应该在最外层单独操作时间线
//! - 【缺点】长时间运行可能导致精度不佳，需要定时重置时间线，并一同处理所有关联的静态时钟
//! - 适用于服务端验证、长期计时，静态时间戳可能需要更换非浮点类型防止误差累积，如 [`std::time::Duration`]
//! - 数量规模庞大的场景，将大量的循环触发的 TickTimer 转换为 StaticTimer 表示状态和少量触发式 TickTimer 用于结算

use crate::base_lib::cores::{
    design_patterns::DependCtx,
    timers::{
        tick_timer::TickTimer,
        tiny_timer::{TimerControl, TimerProgress, TimerView},
    },
    unify_types::time_type,
};

/// 静态计时器的参考时间线，暂停等功能在时间线上实现
#[derive(Clone, Debug)]
pub struct StaticTimeline(pub TickTimer);

impl Default for StaticTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticTimeline {
    /// 创建一个永不停止的计时器 用作静态计时器的基准
    pub fn new() -> Self {
        Self(TickTimer::new(time_type::MAX))
    }

    pub fn current_time(&self) -> time_type::T {
        self.0.elapsed(())
    }

    /// 确认没有依赖本时间线的计时器后，【重启时间线】，防止无限累加引起精度丢失
    pub fn reset_timeline(&mut self) {
        self.0.reset(());
    }
}

#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 计时器时长
    duration: time_type::T,
    /// 计时结束时间
    end_at: time_type::T,
}

impl StaticTimer {
    pub fn new(timeline: &StaticTimeline, duration: time_type::T) -> Self {
        Self {
            duration,
            end_at: timeline.current_time() + duration,
        }
    }
}

impl DependCtx for StaticTimer {
    type Ctx<'a> = &'a StaticTimeline;
}

impl TimerProgress for StaticTimer {
    fn elapsed(&self, ctx: &StaticTimeline) -> time_type::T {
        self.duration(ctx) - self.remaining(ctx)
    }

    fn remaining(&self, ctx: &StaticTimeline) -> time_type::T {
        (self.end_at - ctx.current_time()).min(time_type::ZERO)
    }

    fn duration(&self, _ctx: &StaticTimeline) -> time_type::T {
        self.duration
    }

    fn progress(&self, ctx: &StaticTimeline) -> f64 {
        1.0 - time_type::to_f64(self.remaining(ctx)) / time_type::to_f64(self.duration(ctx))
    }
}

impl TimerView for StaticTimer {
    fn is_completed(&self, ctx: &StaticTimeline) -> bool {
        ctx.current_time() >= self.end_at
    }
}

impl TimerControl for StaticTimer {
    fn reset(&mut self, ctx: &StaticTimeline) {
        self.end_at = ctx.current_time() + self.duration;
    }

    fn complete(&mut self, ctx: &StaticTimeline) {
        self.end_at = ctx.current_time();
    }
}
