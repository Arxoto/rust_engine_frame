//! 基于增量累加时间实现的计时器
//! - 【缺点】长时间累加可能存在误差（每帧 delta 导致的误差累积）
//! - 每帧调用 `tick` 方法来更新计时器状态，逻辑简单清晰
//! - 适用于需要知道进度（动画特效）、短生命周期、局部时间调速等场景

use crate::base_lib::cores::{
    design_patterns::DependCtx,
    timers::tiny_timer::{Tickable, TimerControl, TimerProgress, TimerView},
    unify_types::time_type,
};

/// 简单计时器
#[derive(Clone, Debug)]
pub struct TickTimer {
    elapsed: time_type::T,
    duration: time_type::T,
}

impl TickTimer {
    pub fn new(duration: time_type::T) -> Self {
        Self {
            elapsed: time_type::ZERO,
            duration,
        }
    }

    pub fn inf() -> Self {
        Self {
            elapsed: time_type::ZERO,
            duration: time_type::INFINITY,
        }
    }
}

impl Tickable for TickTimer {
    fn tick(&mut self, delta: time_type::T) {
        // 限制最大值避免超限
        self.elapsed = self.duration.min(self.elapsed + delta)
    }
}

impl DependCtx for TickTimer {
    type Ctx<'a> = ();
}

impl TimerProgress for TickTimer {
    fn elapsed(&self, _: ()) -> time_type::T {
        self.elapsed
    }

    fn remaining(&self, _: ()) -> time_type::T {
        self.duration - self.elapsed
    }

    fn duration(&self, _: ()) -> time_type::T {
        self.duration
    }

    fn progress(&self, _: ()) -> f64 {
        time_type::to_f64(self.elapsed) / time_type::to_f64(self.duration)
    }
}

impl TimerView for TickTimer {
    fn is_completed(&self, _: ()) -> bool {
        self.elapsed >= self.duration
    }
}

impl TimerControl for TickTimer {
    fn reset(&mut self, _: ()) {
        self.elapsed = time_type::ZERO
    }

    fn complete(&mut self, _: ()) {
        self.elapsed = self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::unify_types::time_type;

    /// 初始状态：elapsed 为 0、remaining 为全长、进度 0、未完成
    #[test]
    fn tick_timer_initial_state() {
        let t = TickTimer::new(time_type::unit::<5>());
        assert_eq!(t.elapsed(()), time_type::ZERO);
        assert_eq!(t.remaining(()), time_type::unit::<5>());
        assert_eq!(t.duration(()), time_type::unit::<5>());
        assert_eq!(t.progress(()), 0.0 / 5.0);
        assert!(!t.is_completed(()));
    }

    /// 累加计时到全长后钳制、不再超限；进度上限 1
    #[test]
    fn tick_timer_accumulates_and_clamps_at_duration() {
        let mut t = TickTimer::new(time_type::unit::<5>());
        t.tick(time_type::unit::<2>());
        assert_eq!(t.elapsed(()), time_type::unit::<2>());
        assert_eq!(t.remaining(()), time_type::unit::<3>());
        assert!(!t.is_completed(()));

        t.tick(time_type::unit::<3>());
        assert_eq!(t.elapsed(()), time_type::unit::<5>());
        assert_eq!(t.remaining(()), time_type::ZERO);
        assert!(t.is_completed(()));

        // 超时后继续 tick，elapsed 不超上限
        t.tick(time_type::unit::<10>());
        assert_eq!(t.elapsed(()), time_type::unit::<5>());
        assert_eq!(t.progress(()), 5.0 / 5.0);
    }

    /// 进度比例随时间线性增长
    #[test]
    fn tick_timer_progress_ratio() {
        let mut t = TickTimer::new(time_type::unit::<5>());
        t.tick(time_type::unit::<2>());
        assert_eq!(t.progress(()), 2.0 / 5.0);
        t.tick(time_type::unit::<1>());
        assert_eq!(t.progress(()), 3.0 / 5.0);
    }

    /// reset 归零回到初始；complete 直接推进到结束
    #[test]
    fn tick_timer_reset_and_complete() {
        let mut t = TickTimer::new(time_type::unit::<5>());
        t.tick(time_type::unit::<2>());

        t.reset(());
        assert_eq!(t.elapsed(()), time_type::ZERO);
        assert!(!t.is_completed(()));

        t.complete(());
        assert!(t.is_completed(()));
        assert_eq!(t.elapsed(()), time_type::unit::<5>());
        assert_eq!(t.remaining(()), time_type::ZERO);
    }

    /// 无限时长计时器：elapsed 不被时长钳制、永不完成（用作时间线基准）
    #[test]
    fn tick_timer_infinite_never_completes() {
        let mut t = TickTimer::inf();
        t.tick(time_type::unit::<3>());
        assert_eq!(t.elapsed(()), time_type::unit::<3>());
        assert!(!t.is_completed(()));
    }
}
