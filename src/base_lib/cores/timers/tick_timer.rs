//! 基于增量累加时间实现的计时器
//! - 【缺点】长时间累加可能存在误差（每帧 delta 导致的误差累积）
//! - 每帧调用 `tick` 方法来更新计时器状态，逻辑简单清晰
//! - 适用于需要知道进度（动画特效）、短生命周期、局部时间调速等场景

use crate::base_lib::cores::timers::tiny_timer::{
    Tickable, TimerControl, TimerProgress, TimerView,
};

/// 简单计时器
#[derive(Clone, Debug)]
pub struct TickTimer {
    elapsed: f64,
    duration: f64,
}

impl TickTimer {
    pub fn new(duration: f64) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }
}

impl Tickable for TickTimer {
    fn tick(&mut self, delta: f64) {
        // 限制最大值避免超限
        self.elapsed = self.duration.min(self.elapsed + delta)
    }
}

impl TimerProgress for TickTimer {
    type Ctx<'a> = ();

    fn elapsed(&self, _: ()) -> f64 {
        self.elapsed
    }

    fn remaining(&self, _: ()) -> f64 {
        self.duration - self.elapsed
    }

    fn duration(&self, _: ()) -> f64 {
        self.duration
    }

    fn progress(&self, _: ()) -> f64 {
        self.elapsed / self.duration
    }
}

impl TimerView for TickTimer {
    type Ctx<'a> = ();

    fn is_completed(&self, _: ()) -> bool {
        self.elapsed >= self.duration
    }
}

impl TimerControl for TickTimer {
    type Ctx<'a> = ();

    fn reset(&mut self, _: ()) {
        self.elapsed = 0.0
    }

    fn complete(&mut self, _: ()) {
        self.elapsed = self.duration
    }
}
