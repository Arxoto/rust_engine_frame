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
}

impl TimerProgress for InfiniteStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn elapsed(&self, ctx: &StaticTimeline) -> f64 {
        // 不考虑经过时间超过周期的情况，应该每帧先尝试触发消费掉余量，而后显示进度
        self.duration(ctx) - self.remaining(ctx)
    }

    fn remaining(&self, ctx: &StaticTimeline) -> f64 {
        self.end_at - ctx.current_time()
    }

    fn duration(&self, _ctx: &StaticTimeline) -> f64 {
        self.cycle
    }

    fn progress(&self, ctx: &StaticTimeline) -> f64 {
        1.0 - self.remaining(ctx) / self.duration(ctx)
    }
}

impl TimerView for InfiniteStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn is_completed(&self, _ctx: &StaticTimeline) -> bool {
        // 无法结束
        false
    }
}

impl TimerControl for InfiniteStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn reset(&mut self, ctx: &StaticTimeline) {
        self.end_at = ctx.current_time() + self.cycle;
    }

    fn complete(&mut self, _ctx: &StaticTimeline) {
        // do nothing
    }
}

impl CyclicalTrigger for InfiniteStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn try_trigger_once(&mut self, ctx: &StaticTimeline) -> bool {
        if self.end_at <= ctx.current_time() {
            self.end_at += self.cycle;
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

impl TimerView for FewShotStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn is_completed(&self, _ctx: &StaticTimeline) -> bool {
        Union(&self.few_shot, &self.inf_tg).is_completed(())
    }
}

impl TimerControl for FewShotStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn reset(&mut self, ctx: &StaticTimeline) {
        Union(&mut self.few_shot, &mut self.inf_tg).reset(ctx)
    }

    fn complete(&mut self, ctx: &StaticTimeline) {
        Union(&mut self.few_shot, &mut self.inf_tg).complete(ctx)
    }
}

impl CyclicalTrigger for FewShotStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;

    fn try_trigger_once(&mut self, ctx: &StaticTimeline) -> bool {
        Union(&mut self.few_shot, &mut self.inf_tg).try_trigger_once(ctx)
    }
}
