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
    fn elapsed(&self) -> f64 {
        self.elapsed
    }

    fn remaining(&self) -> f64 {
        self.duration - self.elapsed
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn progress(&self) -> f64 {
        self.elapsed / self.duration
    }
}

impl TimerView for TickTimer {
    fn is_completed(&self) -> bool {
        self.elapsed >= self.duration
    }
}

impl TimerControl for TickTimer {
    fn reset(&mut self) {
        self.elapsed = 0.0
    }

    fn complete(&mut self) {
        self.elapsed = self.duration
    }
}
