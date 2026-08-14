//! 参考累加计时器实现的触发器

use crate::base_lib::cores::{
    design_patterns::DependCtx,
    timers::{
        few_shot_times::FewShotTimes,
        tiny_timer::{CyclicalTrigger, Tickable, TimerControl, TimerProgress, TimerView},
    },
    unify_types::time_type,
};

/// 简单无限触发器
#[derive(Clone, Debug)]
pub struct InfiniteTickTrigger {
    elapsed: time_type::T,
    cycle: time_type::T,
}

impl InfiniteTickTrigger {
    pub fn new(cycle: time_type::T) -> Self {
        Self {
            elapsed: time_type::ZERO,
            cycle,
        }
    }
}

impl Tickable for InfiniteTickTrigger {
    fn tick(&mut self, delta: time_type::T) {
        // 不限制上限
        self.elapsed += delta
    }
}

impl DependCtx for InfiniteTickTrigger {
    type Ctx<'a> = ();
}

impl TimerProgress for InfiniteTickTrigger {
    fn elapsed(&self, _: ()) -> time_type::T {
        // 不考虑经过时间超过周期的情况，应该每帧先尝试触发消费掉余量，而后显示进度
        self.elapsed
    }

    fn remaining(&self, _: ()) -> time_type::T {
        self.cycle - self.elapsed(())
    }

    fn duration(&self, _: ()) -> time_type::T {
        self.cycle
    }

    fn progress(&self, _: ()) -> f64 {
        time_type::to_f64(self.elapsed(())) / time_type::to_f64(self.duration(()))
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
        self.elapsed = time_type::ZERO
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
    pub fn new(cycle: time_type::T, limit_time: u32) -> Self {
        Self {
            few_shot: FewShotTimes::new(limit_time),
            inf_trigger: InfiniteTickTrigger::new(cycle),
        }
    }
}

impl Tickable for FewShotTickTrigger {
    fn tick(&mut self, delta: time_type::T) {
        self.inf_trigger.tick(delta);
    }
}

impl DependCtx for FewShotTickTrigger {
    type Ctx<'a> = ();
}

impl TimerProgress for FewShotTickTrigger {
    fn elapsed(&self, _: ()) -> time_type::T {
        self.inf_trigger.elapsed(())
    }

    fn remaining(&self, _: ()) -> time_type::T {
        self.inf_trigger.remaining(())
    }

    fn duration(&self, _: ()) -> time_type::T {
        self.inf_trigger.duration(())
    }

    fn progress(&self, _: ()) -> f64 {
        self.inf_trigger.progress(())
    }
}

impl TimerView for FewShotTickTrigger {
    #[rustfmt::skip]
    fn is_completed(&self, _: ()) -> bool {
        self.few_shot.of_timer_view(&self.inf_trigger).is_completed(())
    }
}

impl TimerControl for FewShotTickTrigger {
    #[rustfmt::skip]
    fn reset(&mut self, _: ()) {
        self.few_shot.of_timer_control(&mut self.inf_trigger).reset(())
    }

    #[rustfmt::skip]
    fn complete(&mut self, _: ()) {
        self.few_shot.of_timer_control(&mut self.inf_trigger).complete(())
    }
}

impl CyclicalTrigger for FewShotTickTrigger {
    #[rustfmt::skip]
    fn try_trigger_once(&mut self, _: ()) -> bool {
        self.few_shot.of_cyclical_trigger(&mut self.inf_trigger).try_trigger_once(())
    }
}
