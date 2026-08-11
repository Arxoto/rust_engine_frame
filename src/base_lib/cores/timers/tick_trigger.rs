use crate::base_lib::cores::timers::tiny_timer::{
    CyclicalTrigger, Tickable, TimerControl, TimerView,
};

/// 简单无限触发器
#[derive(Clone, Debug)]
pub struct InfiniteTickTrigger {
    elapsed: f64,
    cycle: f64,
}

impl InfiniteTickTrigger {
    pub fn new(cycle: f64) -> Self {
        Self {
            elapsed: 0.0,
            cycle,
        }
    }
}

impl Tickable for InfiniteTickTrigger {
    fn tick(&mut self, delta: f64) {
        // 不限制上限
        self.elapsed += delta
    }
}

impl TimerView for InfiniteTickTrigger {
    fn is_completed(&self) -> bool {
        // 无法结束
        false
    }
}

impl TimerControl for InfiniteTickTrigger {
    fn reset(&mut self) {
        self.elapsed = 0.0
    }

    fn complete(&mut self) {
        // do nothing
    }
}

impl CyclicalTrigger for InfiniteTickTrigger {
    fn try_trigger_once(&mut self) -> bool {
        if self.elapsed >= self.cycle {
            self.elapsed -= self.cycle;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct FewShotTrigger {
    current_time: u32,
    limit_time: u32,
    inner: InfiniteTickTrigger,
}

impl FewShotTrigger {
    pub fn new(cycle: f64, limit_time: u32) -> FewShotTrigger {
        Self {
            current_time: 0,
            limit_time,
            inner: InfiniteTickTrigger::new(cycle),
        }
    }
}

impl TimerView for FewShotTrigger {
    fn is_completed(&self) -> bool {
        self.current_time >= self.limit_time
    }
}

impl TimerControl for FewShotTrigger {
    fn reset(&mut self) {
        self.current_time = 0;
        self.inner.reset();
    }

    fn complete(&mut self) {
        self.current_time = self.limit_time;
        self.inner.complete();
    }
}

impl CyclicalTrigger for FewShotTrigger {
    fn try_trigger_once(&mut self) -> bool {
        if !self.inner.try_trigger_once() {
            return false;
        }

        if self.is_completed() {
            false
        } else {
            self.current_time += 1;
            true
        }
    }
}
