use crate::base_lib::cores::{
    design_patterns::Union,
    timers::{
        few_shot_times::FewShotTimes,
        tiny_timer::{CyclicalTrigger, Tickable, TimerControl, TimerView},
    },
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
pub struct FewShotTickTrigger {
    few_shot: FewShotTimes,
    inner: InfiniteTickTrigger,
}

impl FewShotTickTrigger {
    pub fn new(cycle: f64, limit_time: u32) -> Self {
        Self {
            few_shot: FewShotTimes::new(limit_time),
            inner: InfiniteTickTrigger::new(cycle),
        }
    }
}

impl Tickable for FewShotTickTrigger {
    fn tick(&mut self, delta: f64) {
        self.inner.tick(delta);
    }
}

impl TimerView for FewShotTickTrigger {
    fn is_completed(&self) -> bool {
        Union(&self.few_shot, &self.inner).is_completed()
    }
}

impl TimerControl for FewShotTickTrigger {
    fn reset(&mut self) {
        Union(&mut self.few_shot, &mut self.inner).reset()
    }

    fn complete(&mut self) {
        Union(&mut self.few_shot, &mut self.inner).complete()
    }
}

impl CyclicalTrigger for FewShotTickTrigger {
    fn try_trigger_once(&mut self) -> bool {
        Union(&mut self.few_shot, &mut self.inner).try_trigger_once()
    }
}
