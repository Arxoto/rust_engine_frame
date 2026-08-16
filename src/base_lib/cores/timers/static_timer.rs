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
        Self(TickTimer::new(time_type::INFINITY))
    }

    pub fn current_time(&self) -> time_type::T {
        self.0.elapsed(())
    }

    /// 确认没有依赖本时间线的计时器后，【重启时间线】，防止无限累加引起精度丢失
    pub fn reset_timeline_and_get_diff(&mut self) -> time_type::T {
        let diff = self.current_time();
        self.0.reset(());
        diff
    }
}

/// 绝对时间戳计时器，经 [`StaticTimeline`] 上下文读取
///
/// 时间线在帧末推进（先业务后 tick），帧内业务读取的是推进前的时间，
/// 因此容错窗口会覆盖「到达时长的那一帧」：
///
/// ```
/// # use rust_engine_frame::base_lib::cores::timers::static_timer::{StaticTimer, StaticTimeline};
/// # use rust_engine_frame::base_lib::cores::timers::tiny_timer::{Tickable, TimerView};
/// # use rust_engine_frame::base_lib::cores::unify_types::time_type;
/// let mut timeline = StaticTimeline::new();
/// let t = StaticTimer::new(&timeline, time_type::unit::<1>());
///
/// // 帧 N：业务先读，未过期
/// assert!(!t.is_completed(&timeline));
/// timeline.0.tick(time_type::unit::<1>()); // 帧末推进时间线
/// // 帧 N+1：业务再读，已过期
/// assert!(t.is_completed(&timeline));
/// ```
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

    pub fn inf() -> Self {
        Self {
            duration: time_type::INFINITY,
            end_at: time_type::INFINITY,
        }
    }

    #[cfg(feature = "time_type_f64")]
    pub fn fix_timeline_diff(&mut self, diff: time_type::T) {
        self.end_at -= diff;
    }

    #[cfg(feature = "time_type_duration")]
    pub fn fix_timeline_diff(&mut self, diff: time_type::T) {
        self.end_at = self.end_at.saturating_sub(diff)
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
        (self.end_at - ctx.current_time()).max(time_type::ZERO)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::{timers::tiny_timer::Tickable, unify_types::time_type};

    /// 初始状态：elapsed 为 0、remaining 为全长、进度 0、未完成
    #[test]
    fn static_timer_initial_state() {
        let timeline = StaticTimeline::new();
        let t = StaticTimer::new(&timeline, time_type::unit::<5>());
        assert_eq!(t.elapsed(&timeline), time_type::ZERO);
        assert_eq!(t.remaining(&timeline), time_type::unit::<5>());
        assert_eq!(t.duration(&timeline), time_type::unit::<5>());
        assert_eq!(t.progress(&timeline), 0.0 / 5.0);
        assert!(!t.is_completed(&timeline));
    }

    /// 时间线推进后 remaining 减少、elapsed 增加；到达时长后完成
    #[test]
    fn static_timer_tracks_timeline_progress() {
        let mut timeline = StaticTimeline::new();
        let t = StaticTimer::new(&timeline, time_type::unit::<5>());

        timeline.0.tick(time_type::unit::<2>());
        assert_eq!(t.elapsed(&timeline), time_type::unit::<2>());
        assert_eq!(t.remaining(&timeline), time_type::unit::<3>());
        assert!(!t.is_completed(&timeline));

        timeline.0.tick(time_type::unit::<3>());
        assert!(t.is_completed(&timeline));
        assert_eq!(t.remaining(&timeline), time_type::ZERO);
        assert_eq!(t.progress(&timeline), 5.0 / 5.0);
    }

    /// reset 从当前时间重新起算；complete 立即结束
    #[test]
    fn static_timer_reset_and_complete() {
        let mut timeline = StaticTimeline::new();
        let mut t = StaticTimer::new(&timeline, time_type::unit::<5>());
        timeline.0.tick(time_type::unit::<2>());

        t.reset(&timeline);
        assert_eq!(t.remaining(&timeline), time_type::unit::<5>());
        assert!(!t.is_completed(&timeline));

        t.complete(&timeline);
        assert!(t.is_completed(&timeline));
        assert_eq!(t.remaining(&timeline), time_type::ZERO);
        assert_eq!(t.elapsed(&timeline), time_type::unit::<5>());
    }

    /// 时间线重置（漂移修正）：对中飞行计时器 fix_timeline_diff 后，相对时间读数保持不变
    #[test]
    fn static_timer_fix_timeline_diff_preserves_relative_readings() {
        let mut timeline = StaticTimeline::new();
        timeline.0.tick(time_type::unit::<100>()); // 时间线先走一段

        // 中飞行计时器：创建于 t=100，时长 200，结束于 t=300
        let mut t = StaticTimer::new(&timeline, time_type::unit::<200>());
        timeline.0.tick(time_type::unit::<50>()); // 推进到 t=150

        // 重置前的相对读数
        let elapsed_before = t.elapsed(&timeline);
        let remaining_before = t.remaining(&timeline);
        assert_eq!(elapsed_before, time_type::unit::<50>());
        assert_eq!(remaining_before, time_type::unit::<150>());

        // 时间线重置，返回漂移差
        let diff = timeline.reset_timeline_and_get_diff();
        assert_eq!(diff, time_type::unit::<150>());
        assert_eq!(timeline.current_time(), time_type::ZERO);

        // 依赖计时器修正 diff 后，相对读数不变
        t.fix_timeline_diff(diff);
        assert_eq!(t.elapsed(&timeline), elapsed_before);
        assert_eq!(t.remaining(&timeline), remaining_before);
        assert_eq!(t.progress(&timeline), 50.0 / 200.0);
    }

    /// 无限时长计时器：永不完成
    #[test]
    fn static_timer_infinite_never_completes() {
        let mut timeline = StaticTimeline::new();
        let t = StaticTimer::inf();
        timeline.0.tick(time_type::unit::<3>());
        assert!(!t.is_completed(&timeline));
    }
}
