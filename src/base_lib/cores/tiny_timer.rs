//! 定义计时器的核心功能，并提供基础的逻辑复用
//! 
//! 组合预制体的逻辑复用其实有两种方案
//! - Blanket impl 自动实现
//!   - 需要配合 private::Sealed 私有封装防止下游重复实现，否则可能导致同一特征冲突实现
//!   - 无限循环和有限循环的逻辑，由于实现同一特征，因此判定存在冲突，无法优雅解决
//! - 组合间接实现代码复用，使用时临时生成代理
//!   - 部分特征函数需要持有可变引用，此时会导致不同特征使用时抢占同一份数据，需要将只读和可变特征函数分离定义
//!   - 要满足可变权限，那么组合生成的代理必须持有可变引用，此时只读函数中无法创建代理实例，权限截断，同样需要只读可变分离
//!   - 但是只读和可变特征分离会导致代码量增加

use crate::base_lib::cores::design_patterns::ContextWrapper;

// region: 抽象定义

/// Progress 进度计时器
pub trait TinyTimer {
    /// 已经经过了多少时间
    fn get_time(&self) -> f64;

    /// 剩余时长
    fn get_time_left(&self) -> f64;

    /// 计时上限
    fn get_time_limit(&self) -> f64;

    /// 进度比例
    fn get_time_ratio(&self) -> f64;
}

/// 基于增量累加时间实现的计时器
pub trait TickTimer {
    /// 时间流逝
    fn tick(&mut self, delta: f64);
}

pub trait FlowingTimerReadonly {
    /// 计时结束
    fn is_finished(&self) -> bool;
}

/// 有状态计时器
pub trait FlowingTimer: FlowingTimerReadonly {
    /// 重置时间
    fn restart(&mut self);

    /// 提前结束
    fn finish(&mut self);
}

pub trait FreezableTimerReadonly {
    /// 是否被冻结
    fn is_frozen(&self) -> bool;
}

/// 可被冻结的计时器
pub trait FreezableTimer: FreezableTimerReadonly {
    /// 冻结时间
    fn freeze(&mut self);

    /// 恢复计时
    fn resume(&mut self);
}

/// 可循环触发的计时器
pub trait CyclicalTimer {
    /// 触发次数
    fn try_trigger_once(&mut self) -> bool;
}

// endregion

pub mod freezable_tick {
    use super::*;

    /// 冻结预制体，实现 [`FreezableTimer`] [`TickTimer`]
    #[derive(Clone, Debug)]
    pub struct FreezableTickTag(bool);

    impl Default for FreezableTickTag {
        fn default() -> Self {
            // 默认不冻结
            Self(false)
        }
    }

    impl FreezableTimerReadonly for FreezableTickTag {
        fn is_frozen(&self) -> bool {
            self.0
        }
    }

    impl FreezableTimer for FreezableTickTag {
        fn freeze(&mut self) {
            self.0 = true
        }

        fn resume(&mut self) {
            self.0 = false
        }
    }

    impl<T: TickTimer> FreezableTimerReadonly for ContextWrapper<&FreezableTickTag, &T> {
        fn is_frozen(&self) -> bool {
            self.inner.is_frozen()
        }
    }

    impl<T: TickTimer> FreezableTimerReadonly for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn is_frozen(&self) -> bool {
            self.inner.is_frozen()
        }
    }

    impl<T: TickTimer> FreezableTimer for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn freeze(&mut self) {
            self.inner.freeze();
        }

        fn resume(&mut self) {
            self.inner.resume();
        }
    }

    impl<T: TinyTimer> TinyTimer for ContextWrapper<&FreezableTickTag, &T> {
        fn get_time(&self) -> f64 {
            self.ctx.get_time()
        }

        fn get_time_left(&self) -> f64 {
            self.ctx.get_time_left()
        }

        fn get_time_limit(&self) -> f64 {
            self.ctx.get_time_limit()
        }

        fn get_time_ratio(&self) -> f64 {
            self.ctx.get_time_ratio()
        }
    }

    impl<T: TinyTimer> TinyTimer for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn get_time(&self) -> f64 {
            self.ctx.get_time()
        }

        fn get_time_left(&self) -> f64 {
            self.ctx.get_time_left()
        }

        fn get_time_limit(&self) -> f64 {
            self.ctx.get_time_limit()
        }

        fn get_time_ratio(&self) -> f64 {
            self.ctx.get_time_ratio()
        }
    }

    impl<T: TickTimer> TickTimer for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn tick(&mut self, delta: f64) {
            // 时间冻结时不步进
            if !self.is_frozen() {
                self.ctx.tick(delta);
            }
        }
    }
}

pub mod few_show_cycle {
    use super::*;

    /// 有限循环预制体，实现 [`FlowingTimer`] [`CyclicalTimer`]
    #[derive(Clone, Debug)]
    pub struct FewShotCycleTag {
        value: u32,
        limit: u32,
    }

    impl FewShotCycleTag {
        pub fn new(limit: u32) -> FewShotCycleTag {
            Self {
                value: 0,
                limit,
            }
        }
    }

    impl FlowingTimerReadonly for FewShotCycleTag {
        fn is_finished(&self) -> bool {
            self.value >= self.limit
        }
    }

    impl FlowingTimer for FewShotCycleTag {
        fn restart(&mut self) {
            self.value = 0;
        }

        fn finish(&mut self) {
            self.value = self.limit;
        }
    }

    impl CyclicalTimer for FewShotCycleTag {
        fn try_trigger_once(&mut self) -> bool {
            if self.is_finished() {
                false
            } else {
                self.value += 1;
                true
            }
        }
    }

    impl<T: FlowingTimer + CyclicalTimer> FlowingTimerReadonly
        for ContextWrapper<&FewShotCycleTag, &T>
    {
        fn is_finished(&self) -> bool {
            // 只关注有限循环本身，真实的计时器可能是无限循环的，对应判断可能会存在异常
            self.inner.is_finished()
        }
    }

    impl<T: FlowingTimer + CyclicalTimer> FlowingTimerReadonly
        for ContextWrapper<&mut FewShotCycleTag, &mut T>
    {
        fn is_finished(&self) -> bool {
            // 只关注有限循环本身，真实的计时器可能是无限循环的，对应判断可能会存在异常
            self.inner.is_finished()
        }
    }

    impl<T: FlowingTimer + CyclicalTimer> FlowingTimer
        for ContextWrapper<&mut FewShotCycleTag, &mut T>
    {
        fn restart(&mut self) {
            self.ctx.restart();
            self.inner.restart();
        }

        fn finish(&mut self) {
            self.ctx.finish();
            self.inner.finish();
        }
    }

    impl<T: FlowingTimer + CyclicalTimer> CyclicalTimer
        for ContextWrapper<&mut FewShotCycleTag, &mut T>
    {
        fn try_trigger_once(&mut self) -> bool {
            // 先尝试触发计数器，成功后尝试触发有限循环，两者都成功才算成功触发
            self.ctx.try_trigger_once() && self.inner.try_trigger_once()
        }
    }
}
