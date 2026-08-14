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

    // todo 参考 PausePrefab 实现 Union 包装函数
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

        let u = Union(&*self.0, &*self.1);
        if u.is_completed(ctx) {
            false
        } else {
            self.0.current += 1;
            true
        }
    }
}
