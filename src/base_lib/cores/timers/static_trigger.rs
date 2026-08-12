//! 参考静态计时器实现的触发器

use crate::base_lib::cores::{
    design_patterns::Union,
    timers::{
        few_shot_times::FewShotTimes,
        static_timer::StaticTimeline,
        tiny_timer::{CyclicalTrigger, TimerControl, TimerProgress, TimerView},
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

    pub fn of_timer<'a>(
        &'a self,
        timeline: &'a StaticTimeline,
    ) -> Union<&'a Self, &'a StaticTimeline> {
        Union(self, timeline)
    }

    pub fn of_timer_mut<'a>(
        &'a mut self,
        timeline: &'a StaticTimeline,
    ) -> Union<&'a mut Self, &'a StaticTimeline> {
        Union(self, timeline)
    }
}

impl TimerProgress for Union<&InfiniteStaticTrigger, &StaticTimeline> {
    fn elapsed(&self) -> f64 {
        // 不考虑经过时间超过周期的情况，应该每帧先尝试触发消费掉余量，而后显示进度
        self.duration() - self.remaining()
    }

    fn remaining(&self) -> f64 {
        self.0.end_at - self.1.current_time()
    }

    fn duration(&self) -> f64 {
        self.0.cycle
    }

    fn progress(&self) -> f64 {
        1.0 - self.remaining() / self.duration()
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
