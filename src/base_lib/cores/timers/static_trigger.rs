use crate::base_lib::cores::{
    design_patterns::Union,
    timers::{
        few_shot_times::FewShotTimes,
        static_timer::StaticTimeline,
        tiny_timer::{CyclicalTrigger, TimerControl, TimerView},
    },
};

#[derive(Clone, Debug)]
pub struct InfiniteStaticTrigger {
    /// 周期
    cycle: f64,
    /// 下个触发时刻
    end_at: f64,
}

impl InfiniteStaticTrigger {
    pub fn new(timeline: &StaticTimeline, cycle: f64) -> Self {
        Self {
            cycle,
            end_at: timeline.current_time() + cycle,
        }
    }
}

impl TimerView for Union<&InfiniteStaticTrigger, &StaticTimeline> {
    fn is_completed(&self) -> bool {
        // 无法结束
        false
    }
}

impl TimerControl for Union<&mut InfiniteStaticTrigger, &StaticTimeline> {
    fn reset(&mut self) {
        self.0.end_at = self.1.current_time() + self.0.cycle;
    }

    fn complete(&mut self) {
        // do nothing
    }
}

impl CyclicalTrigger for Union<&mut InfiniteStaticTrigger, &StaticTimeline> {
    fn try_trigger_once(&mut self) -> bool {
        if self.0.end_at <= self.1.current_time() {
            self.0.end_at += self.0.cycle;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct FewShotStaticTrigger {
    few_shot: FewShotTimes,
    inf_tg: InfiniteStaticTrigger,
}

impl FewShotStaticTrigger {
    pub fn new(timeline: &StaticTimeline, cycle: f64, limit_time: u32) -> Self {
        Self {
            few_shot: FewShotTimes::new(limit_time),
            inf_tg: InfiniteStaticTrigger::new(timeline, cycle),
        }
    }
}

impl TimerView for Union<&FewShotStaticTrigger, &StaticTimeline> {
    fn is_completed(&self) -> bool {
        Union(&self.0.few_shot, &Union(&self.0.inf_tg, self.1)).is_completed()
    }
}

impl TimerControl for Union<&mut FewShotStaticTrigger, &StaticTimeline> {
    fn reset(&mut self) {
        Union(&mut self.0.few_shot, &mut Union(&mut self.0.inf_tg, self.1)).reset()
    }

    fn complete(&mut self) {
        Union(&mut self.0.few_shot, &mut Union(&mut self.0.inf_tg, self.1)).complete()
    }
}

impl CyclicalTrigger for Union<&mut FewShotStaticTrigger, &StaticTimeline> {
    fn try_trigger_once(&mut self) -> bool {
        Union(&mut self.0.few_shot, &mut Union(&mut self.0.inf_tg, self.1)).try_trigger_once()
    }
}
