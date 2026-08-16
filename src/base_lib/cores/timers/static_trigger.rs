//! 参考静态计时器实现的触发器

use crate::base_lib::cores::{
    design_patterns::DependCtx,
    timers::{
        few_shot_times::FewShotTimes,
        static_timer::StaticTimeline,
        tiny_timer::{CyclicalTrigger, TimerControl, TimerProgress, TimerView},
    },
    unify_types::time_type,
};

#[derive(Clone, Debug)]
pub struct InfiniteStaticTrigger {
    /// 周期
    cycle: time_type::T,
    /// 下个触发时刻
    end_at: time_type::T,
}

impl InfiniteStaticTrigger {
    pub fn new(timeline: &StaticTimeline, cycle: time_type::T) -> Self {
        Self {
            cycle,
            end_at: timeline.current_time() + cycle,
        }
    }
}

impl DependCtx for InfiniteStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;
}

impl TimerProgress for InfiniteStaticTrigger {
    fn elapsed(&self, ctx: &StaticTimeline) -> time_type::T {
        // 不考虑经过时间超过周期的情况，应该每帧先尝试触发消费掉余量，而后显示进度
        self.duration(ctx) - self.remaining(ctx)
    }

    fn remaining(&self, ctx: &StaticTimeline) -> time_type::T {
        self.end_at - ctx.current_time()
    }

    fn duration(&self, _ctx: &StaticTimeline) -> time_type::T {
        self.cycle
    }

    fn progress(&self, ctx: &StaticTimeline) -> f64 {
        1.0 - time_type::to_f64(self.remaining(ctx)) / time_type::to_f64(self.duration(ctx))
    }
}

impl TimerView for InfiniteStaticTrigger {
    fn is_completed(&self, _ctx: &StaticTimeline) -> bool {
        // 无法结束
        false
    }
}

impl TimerControl for InfiniteStaticTrigger {
    fn reset(&mut self, ctx: &StaticTimeline) {
        self.end_at = ctx.current_time() + self.cycle;
    }

    fn complete(&mut self, _ctx: &StaticTimeline) {
        // do nothing
    }
}

impl CyclicalTrigger for InfiniteStaticTrigger {
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
    pub fn new(timeline: &StaticTimeline, cycle: time_type::T, limit_time: u32) -> Self {
        Self {
            few_shot: FewShotTimes::new(limit_time),
            inf_tg: InfiniteStaticTrigger::new(timeline, cycle),
        }
    }
}

impl DependCtx for FewShotStaticTrigger {
    type Ctx<'a> = &'a StaticTimeline;
}

impl TimerProgress for FewShotStaticTrigger {
    #[rustfmt::skip]
    fn elapsed(&self, ctx: &StaticTimeline) -> time_type::T {
        self.inf_tg.elapsed(ctx)
    }

    #[rustfmt::skip]
    fn remaining(&self, ctx: &StaticTimeline) -> time_type::T {
        self.inf_tg.remaining(ctx)
    }

    #[rustfmt::skip]
    fn duration(&self, ctx: &StaticTimeline) -> time_type::T {
        self.inf_tg.duration(ctx)
    }

    #[rustfmt::skip]
    fn progress(&self, ctx: &StaticTimeline) -> f64 {
        self.inf_tg.progress(ctx)
    }
}

impl TimerView for FewShotStaticTrigger {
    #[rustfmt::skip]
    fn is_completed(&self, ctx: &StaticTimeline) -> bool {
        self.few_shot.of_timer_view(&self.inf_tg).is_completed(ctx)
    }
}

impl TimerControl for FewShotStaticTrigger {
    #[rustfmt::skip]
    fn reset(&mut self, ctx: &StaticTimeline) {
        self.few_shot.of_timer_control(&mut self.inf_tg).reset(ctx)
    }

    #[rustfmt::skip]
    fn complete(&mut self, ctx: &StaticTimeline) {
        self.few_shot.of_timer_control(&mut self.inf_tg).complete(ctx)
    }
}

impl CyclicalTrigger for FewShotStaticTrigger {
    #[rustfmt::skip]
    fn try_trigger_once(&mut self, ctx: &StaticTimeline) -> bool {
        self.few_shot.of_cyclical_trigger(&mut self.inf_tg).try_trigger_once(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::timers::tiny_timer::Tickable;

    /// FewShotStaticTrigger 的进度透传内层 InfiniteStaticTrigger;完成状态由 few-shot 额度决定
    #[test]
    fn few_shot_static_trigger_progress_delegates_and_completion_from_quota() {
        let mut timeline = StaticTimeline::new();
        let mut t = FewShotStaticTrigger::new(&timeline, time_type::unit::<3>(), 2);

        // 初始:elapsed 0、remaining 一个周期、进度 0、未完成
        assert_eq!(t.elapsed(&timeline), time_type::ZERO);
        assert_eq!(t.remaining(&timeline), time_type::unit::<3>());
        assert_eq!(t.duration(&timeline), time_type::unit::<3>());
        assert!(!t.is_completed(&timeline));

        // 推进 2s:进度跟随内层触发器
        timeline.0.tick(time_type::unit::<2>());
        assert_eq!(t.elapsed(&timeline), time_type::unit::<2>());
        assert_eq!(t.remaining(&timeline), time_type::unit::<1>());
        assert_eq!(t.progress(&timeline), 1.0 - 1.0 / 3.0); // 同构表达式
        assert!(!t.is_completed(&timeline));

        // 到达周期触发第 1 次:内层周期被消费,elapsed 归零
        timeline.0.tick(time_type::unit::<1>());
        assert!(t.try_trigger_once(&timeline));
        assert_eq!(t.elapsed(&timeline), time_type::ZERO);
        assert!(!t.is_completed(&timeline));

        // 第 2 次触发后额度耗尽:is_completed 来自 few-shot 额度,而非内层(内层永不完成)
        timeline.0.tick(time_type::unit::<3>());
        assert!(t.try_trigger_once(&timeline));
        assert!(t.is_completed(&timeline));
        assert!(!t.inf_tg.is_completed(&timeline));
    }
}
