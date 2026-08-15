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
