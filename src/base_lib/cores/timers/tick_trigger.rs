//! 参考累加计时器实现的触发器

use crate::base_lib::cores::{
    design_patterns::{DependCtx, Union},
    timers::{
        few_shot_times::FewShotTimes,
        tiny_timer::{CyclicalTrigger, Tickable, TimerControl, TimerProgress, TimerView},
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

impl DependCtx for InfiniteTickTrigger {
    type Ctx<'a> = ();
}

impl TimerProgress for InfiniteTickTrigger {
    fn elapsed(&self, _: ()) -> f64 {
        // 不考虑经过时间超过周期的情况，应该每帧先尝试触发消费掉余量，而后显示进度
        self.elapsed
    }

    fn remaining(&self, _: ()) -> f64 {
        self.cycle - self.elapsed(())
    }

    fn duration(&self, _: ()) -> f64 {
        self.cycle
    }

    fn progress(&self, _: ()) -> f64 {
        self.elapsed(()) / self.duration(())
    }
}

impl TimerView for InfiniteTickTrigger {
    fn is_completed(&self, _: ()) -> bool {
        // 无法结束
        false
    }
}

impl TimerControl for InfiniteTickTrigger {
    fn reset(&mut self, _: ()) {
        self.elapsed = 0.0
    }

    fn complete(&mut self, _: ()) {
        // do nothing
    }
}

impl CyclicalTrigger for InfiniteTickTrigger {
    fn try_trigger_once(&mut self, _: ()) -> bool {
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
    inf_trigger: InfiniteTickTrigger,
}

impl FewShotTickTrigger {
    pub fn new(cycle: f64, limit_time: u32) -> Self {
        Self {
            few_shot: FewShotTimes::new(limit_time),
            inf_trigger: InfiniteTickTrigger::new(cycle),
        }
    }
}

impl Tickable for FewShotTickTrigger {
    fn tick(&mut self, delta: f64) {
        self.inf_trigger.tick(delta);
    }
}

impl DependCtx for FewShotTickTrigger {
    type Ctx<'a> = ();
}

impl TimerProgress for FewShotTickTrigger {
    fn elapsed(&self, _: ()) -> f64 {
        self.inf_trigger.elapsed(())
    }

    fn remaining(&self, _: ()) -> f64 {
        self.inf_trigger.remaining(())
    }

    fn duration(&self, _: ()) -> f64 {
        self.inf_trigger.duration(())
    }

    fn progress(&self, _: ()) -> f64 {
        self.inf_trigger.progress(())
    }
}

impl TimerView for FewShotTickTrigger {
    fn is_completed(&self, _: ()) -> bool {
        Union(&self.few_shot, &self.inf_trigger).is_completed(())
    }
}

impl TimerControl for FewShotTickTrigger {
    fn reset(&mut self, _: ()) {
        Union(&mut self.few_shot, &mut self.inf_trigger).reset(())
    }

    fn complete(&mut self, _: ()) {
        Union(&mut self.few_shot, &mut self.inf_trigger).complete(())
    }
}

impl CyclicalTrigger for FewShotTickTrigger {
    fn try_trigger_once(&mut self, _: ()) -> bool {
        Union(&mut self.few_shot, &mut self.inf_trigger).try_trigger_once(())
    }
}
