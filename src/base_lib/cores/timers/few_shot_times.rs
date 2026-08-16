use crate::base_lib::cores::{
    design_patterns::{DependCtx, Union},
    timers::tiny_timer::{CyclicalTrigger, TimerControl, TimerView},
};

/// 有限循环预制体，干预 [`TimerView`] [`TimerControl`] [`CyclicalTrigger`]
#[derive(Clone, Debug)]
pub(super) struct FewShotTimes {
    current: u32,
    limit: u32,
}

impl FewShotTimes {
    pub fn new(limit: u32) -> Self {
        Self { current: 0, limit }
    }

    #[rustfmt::skip]
    #[inline]
    pub fn of_timer_view<'a, T: DependCtx>(&self, t: &T) -> impl TimerView<Ctx<'a> = T::Ctx<'a>> {
        Union::new(self, t)
    }

    #[rustfmt::skip]
    #[inline]
    pub fn of_timer_control<'a, T: TimerControl>(&mut self, t: &mut T) -> impl TimerControl<Ctx<'a> = T::Ctx<'a>> {
        Union::new(self, t)
    }

    #[rustfmt::skip]
    #[inline]
    pub fn of_cyclical_trigger<'a, T: CyclicalTrigger>(&mut self, t: &mut T) -> impl CyclicalTrigger<Ctx<'a> = T::Ctx<'a>> {
        Union::new(self, t)
    }
}

impl<T: DependCtx> DependCtx for Union<&FewShotTimes, &T> {
    type Ctx<'a> = T::Ctx<'a>;
}

impl<T: DependCtx> DependCtx for Union<&mut FewShotTimes, &mut T> {
    type Ctx<'a> = T::Ctx<'a>;
}

impl<T: DependCtx> TimerView for Union<&FewShotTimes, &T> {
    fn is_completed(&self, _: T::Ctx<'_>) -> bool {
        // 只关注有限循环本身，真实的触发器可能是无限循环的，对应判断可能会存在异常
        self.0.current >= self.0.limit
    }
}

// 开始结束同时修改 prefab 和 timer
impl<T: TimerControl> TimerControl for Union<&mut FewShotTimes, &mut T> {
    fn reset(&mut self, ctx: Self::Ctx<'_>) {
        self.0.current = 0;
        self.1.reset(ctx);
    }

    fn complete(&mut self, ctx: Self::Ctx<'_>) {
        self.0.current = self.0.limit;
        self.1.complete(ctx);
    }
}

// 循环触发同时检查 prefab 和 timer
impl<T: CyclicalTrigger> CyclicalTrigger for Union<&mut FewShotTimes, &mut T> {
    fn try_trigger_once(&mut self, ctx: Self::Ctx<'_>) -> bool {
        // 因为 FewShotTimes 允许次数只会减小不会增长
        // 所以当他失败时代表之后的触发也必定失败，因此无需回退前面的 timer
        // 若 FewShotTimes 支持临时增加次数，会导致之后的触发存在一周期的误差
        // 反观重启会同时重启两者的状态，因此没这个问题，所以设计为只支持重启不支持增加

        // 先判断计时器能否触发，成功后尝试触发有限循环，两者都成功才算成功触发
        if !self.1.try_trigger_once(ctx) {
            return false;
        }

        let u = Union::new(&*self.0, &*self.1);
        if u.is_completed(ctx) {
            false
        } else {
            self.0.current += 1;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::{
        timers::{
            tick_trigger::InfiniteTickTrigger,
            tiny_timer::{CyclicalTrigger, Tickable, TimerProgress, TimerView},
        },
        unify_types::time_type,
    };

    /// limit 次内触发成功;第 limit+1 次返回 false,且内层触发器已消耗一次(静默丢弃)
    #[test]
    fn few_shot_limited_trigger_drops_overflow() {
        let mut few_shot = FewShotTimes::new(2);
        let mut trigger = InfiniteTickTrigger::new(time_type::unit::<3>());

        trigger.tick(time_type::unit::<3>()); // 到达一个周期
        assert!(
            few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        ); // 第 1 次
        trigger.tick(time_type::unit::<3>());
        assert!(
            few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        ); // 第 2 次
        assert!(few_shot.of_timer_view(&trigger).is_completed(())); // 额度耗尽

        // 第 3 次:内层先成功触发(消耗周期),外层返回 false 并静默丢弃该次触发
        trigger.tick(time_type::unit::<3>());
        assert!(
            !few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        );
        assert_eq!(trigger.elapsed(()), time_type::ZERO); // 内层确实消费了一个周期
    }

    /// 区分「未到时间」与「额度耗尽」:未到时间内层未触发(elapsed 不变),额度耗尽内层消耗但被丢弃
    #[test]
    fn few_shot_distinguishes_not_time_from_exhausted() {
        let mut few_shot = FewShotTimes::new(1);
        let mut trigger = InfiniteTickTrigger::new(time_type::unit::<3>());

        // 未到时间:触发失败且 elapsed 不变
        trigger.tick(time_type::unit::<2>());
        assert!(
            !few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        );
        assert_eq!(trigger.elapsed(()), time_type::unit::<2>()); // 内层未消耗

        // 额度耗尽:内层已触发(消耗周期)但被外层丢弃
        trigger.tick(time_type::unit::<1>()); // 到达周期
        assert!(
            few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        ); // 第 1 次(也是最后一次)
        assert!(few_shot.of_timer_view(&trigger).is_completed(()));
        trigger.tick(time_type::unit::<3>()); // 再到一个周期
        assert!(
            !few_shot
                .of_cyclical_trigger(&mut trigger)
                .try_trigger_once(())
        );
        assert_eq!(trigger.elapsed(()), time_type::ZERO); // 内层消费了一个周期,触发被丢弃
    }
}
