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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::unify_types::time_type;

    /// 初始状态：elapsed 为 0、remaining 为一个周期、进度 0、无法结束
    #[test]
    fn infinite_trigger_initial_state() {
        let t = InfiniteTickTrigger::new(time_type::unit::<3>());
        assert_eq!(t.elapsed(()), time_type::ZERO);
        assert_eq!(t.remaining(()), time_type::unit::<3>());
        assert_eq!(t.duration(()), time_type::unit::<3>());
        assert_eq!(t.progress(()), 0.0 / 3.0);
        assert!(!t.is_completed(()));
    }

    /// 未到周期时触发失败且不消耗；越过周期后成功触发并消费一个周期
    #[test]
    fn infinite_trigger_try_trigger_consumes_cycle() {
        let mut t = InfiniteTickTrigger::new(time_type::unit::<3>());
        t.tick(time_type::unit::<2>());
        assert!(!t.try_trigger_once(())); // 未到时间
        assert_eq!(t.elapsed(()), time_type::unit::<2>()); // 失败不消耗余量

        t.tick(time_type::unit::<2>());
        assert_eq!(t.elapsed(()), time_type::unit::<4>());
        assert!(t.try_trigger_once(())); // 越过周期，成功并消费
        assert_eq!(t.elapsed(()), time_type::unit::<1>());
    }

    /// elapsed 不钳制、可越过周期累积；越过后的进度 >1，需先触发消费
    #[test]
    fn infinite_trigger_ticks_without_clamp() {
        let mut t = InfiniteTickTrigger::new(time_type::unit::<3>());
        t.tick(time_type::unit::<2>());
        t.tick(time_type::unit::<2>());
        assert_eq!(t.elapsed(()), time_type::unit::<4>());
        assert_eq!(t.progress(()), 4.0 / 3.0);
    }

    /// reset 归零；complete 是空操作（无限触发无法结束）
    #[test]
    fn infinite_trigger_reset_and_complete_noop() {
        let mut t = InfiniteTickTrigger::new(time_type::unit::<3>());
        t.tick(time_type::unit::<2>());
        t.reset(());
        assert_eq!(t.elapsed(()), time_type::ZERO);

        t.tick(time_type::unit::<2>());
        t.complete(());
        assert_eq!(t.elapsed(()), time_type::unit::<2>());
        assert!(!t.is_completed(()));
    }
}
