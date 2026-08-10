use crate::base_lib::cores::{
    tick_timer::{TinyTickTimer, InfTickTrigger},
    tiny_timer::{few_show_cycle::FewShotCycleTag, freezable_tick::FreezableTickTag},
};

// 面向对象时可直接使用， ECS 时可考虑拆分

/// 可暂停的计时器
#[derive(Clone, Debug)]
pub struct FreezeTickTimer {
    pub freezable_tick_tag: FreezableTickTag,
    pub tick_timer: TinyTickTimer,
}

impl FreezeTickTimer {
    pub fn new(limit: f64) -> Self {
        Self {
            freezable_tick_tag: FreezableTickTag::default(),
            tick_timer: TinyTickTimer::new(limit),
        }
    }
}

/// 可暂停的无限触发器
#[derive(Clone, Debug)]
pub struct FreezeInfTickTrigger {
    pub freezable_tick_tag: FreezableTickTag,
    pub tick_timer: InfTickTrigger,
}

impl FreezeInfTickTrigger {
    pub fn new(limit: f64) -> Self {
        Self {
            freezable_tick_tag: FreezableTickTag::default(),
            tick_timer: InfTickTrigger::new(limit),
        }
    }
}

/// 可暂停的有限触发器
#[derive(Clone, Debug)]
pub struct FreezeFewShotTickTrigger {
    pub freezable_tick_tag: FreezableTickTag,
    pub few_shot_cycle_tag: FewShotCycleTag,
    pub tick_timer: InfTickTrigger,
}

impl FreezeFewShotTickTrigger {
    pub fn new(time_limit: f64, few_shot: u32) -> Self {
        Self {
            freezable_tick_tag: FreezableTickTag::default(),
            few_shot_cycle_tag: FewShotCycleTag::new(few_shot),
            tick_timer: InfTickTrigger::new(time_limit),
        }
    }
}

pub mod freeze_tick_timer_impl {
    use crate::base_lib::cores::{
        design_patterns::WithContext,
        tick_timer_builders::FreezeTickTimer,
        tiny_timer::{
            FlowingTimer, FlowingTimerReadonly, FreezableTimer, FreezableTimerReadonly, TickTimer,
            TinyTimer,
        },
    };

    impl TinyTimer for FreezeTickTimer {
        fn get_time(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time()
        }

        fn get_time_left(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_left()
        }

        fn get_time_limit(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_limit()
        }

        fn get_time_ratio(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_ratio()
        }
    }

    impl TickTimer for FreezeTickTimer {
        fn tick(&mut self, delta: f64) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .tick(delta);
        }
    }

    impl FlowingTimerReadonly for FreezeTickTimer {
        fn is_finished(&self) -> bool {
            self.tick_timer.is_finished()
        }
    }

    impl FlowingTimer for FreezeTickTimer {
        fn restart(&mut self) {
            self.tick_timer.restart();
        }

        fn finish(&mut self) {
            self.tick_timer.finish();
        }
    }

    impl FreezableTimerReadonly for FreezeTickTimer {
        fn is_frozen(&self) -> bool {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .is_frozen()
        }
    }

    impl FreezableTimer for FreezeTickTimer {
        fn freeze(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .freeze();
        }

        fn resume(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .resume();
        }
    }
}

pub mod freeze_cycle_tick_timer_impl {
    use crate::base_lib::cores::{
        design_patterns::WithContext,
        tick_timer_builders::FreezeInfTickTrigger,
        tiny_timer::{
            CyclicalTimer, FlowingTimer, FlowingTimerReadonly, FreezableTimer,
            FreezableTimerReadonly, TickTimer, TinyTimer,
        },
    };

    impl TinyTimer for FreezeInfTickTrigger {
        fn get_time(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time()
        }

        fn get_time_left(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_left()
        }

        fn get_time_limit(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_limit()
        }

        fn get_time_ratio(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_ratio()
        }
    }

    impl TickTimer for FreezeInfTickTrigger {
        fn tick(&mut self, delta: f64) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .tick(delta);
        }
    }

    impl FlowingTimerReadonly for FreezeInfTickTrigger {
        fn is_finished(&self) -> bool {
            self.tick_timer.is_finished()
        }
    }

    impl FlowingTimer for FreezeInfTickTrigger {
        fn restart(&mut self) {
            self.tick_timer.restart();
        }

        fn finish(&mut self) {
            self.tick_timer.finish();
        }
    }

    impl FreezableTimerReadonly for FreezeInfTickTrigger {
        fn is_frozen(&self) -> bool {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .is_frozen()
        }
    }

    impl FreezableTimer for FreezeInfTickTrigger {
        fn freeze(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .freeze();
        }

        fn resume(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .resume();
        }
    }

    impl CyclicalTimer for FreezeInfTickTrigger {
        fn try_trigger_once(&mut self) -> bool {
            self.tick_timer.try_trigger_once()
        }
    }
}

pub mod freeze_few_shot_tick_timer_impl {
    use crate::base_lib::cores::{
        design_patterns::WithContext,
        tick_timer_builders::FreezeFewShotTickTrigger,
        tiny_timer::{
            CyclicalTimer, FlowingTimer, FlowingTimerReadonly, FreezableTimer,
            FreezableTimerReadonly, TickTimer, TinyTimer,
        },
    };

    impl TinyTimer for FreezeFewShotTickTrigger {
        fn get_time(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time()
        }

        fn get_time_left(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_left()
        }

        fn get_time_limit(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_limit()
        }

        fn get_time_ratio(&self) -> f64 {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .get_time_ratio()
        }
    }

    impl TickTimer for FreezeFewShotTickTrigger {
        fn tick(&mut self, delta: f64) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .tick(delta);
        }
    }

    impl FlowingTimerReadonly for FreezeFewShotTickTrigger {
        fn is_finished(&self) -> bool {
            self.few_shot_cycle_tag
                .with_ctx(&self.tick_timer)
                .is_finished()
        }
    }

    impl FlowingTimer for FreezeFewShotTickTrigger {
        fn restart(&mut self) {
            self.few_shot_cycle_tag
                .with_ctx_mut(&mut self.tick_timer)
                .restart();
        }

        fn finish(&mut self) {
            self.few_shot_cycle_tag
                .with_ctx_mut(&mut self.tick_timer)
                .finish();
        }
    }

    impl FreezableTimerReadonly for FreezeFewShotTickTrigger {
        fn is_frozen(&self) -> bool {
            self.freezable_tick_tag
                .with_ctx(&self.tick_timer)
                .is_frozen()
        }
    }

    impl FreezableTimer for FreezeFewShotTickTrigger {
        fn freeze(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .freeze();
        }

        fn resume(&mut self) {
            self.freezable_tick_tag
                .with_ctx_mut(&mut self.tick_timer)
                .resume();
        }
    }

    impl CyclicalTimer for FreezeFewShotTickTrigger {
        fn try_trigger_once(&mut self) -> bool {
            self.few_shot_cycle_tag
                .with_ctx_mut(&mut self.tick_timer)
                .try_trigger_once()
        }
    }
}

// todo test
