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
    ///
    /// 根据业务类型判断先处理逻辑还是先累加计时（也可以统一，区别不大）
    ///
    /// - 计时器是帮助：如容错时间等，范围应该尽量大，即计时滞后，因此应该先处理逻辑后累加计时
    /// - 计时器是限制：如冷却计算等，范围应该尽量小，即计时提前，因此应该先累加计数后处理逻辑
    ///
    /// 业务逻辑一般都是放在 _physics_process / FixedUpdate 里的
    ///
    /// - Godot 推荐，保证逻辑与物理引擎同步，且适配物理插值
    /// - Godot 中，先 _physics_process 而后【物理模拟】，其次 _process 最后【渲染】
    /// - _physics_process 中根据发生事件和业务逻辑生成物理效果，【物理模拟】时使效果生效
    fn tick(&mut self, delta: f64);
}

/// 可结束计时器
pub trait FlowingTimerReadonly {
    /// 计时结束
    fn is_finished(&self) -> bool;
}

/// 可结束计时器
pub trait FlowingTimer: FlowingTimerReadonly {
    /// 重置时间
    fn restart(&mut self);

    /// 提前结束
    fn finish(&mut self);
}

/// 可被冻结的计时器
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
    /// 尝试触发一次
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

    // 是否暂停透传 tag
    impl<T: TickTimer> FreezableTimerReadonly for ContextWrapper<&FreezableTickTag, &T> {
        fn is_frozen(&self) -> bool {
            self.inner.is_frozen()
        }
    }

    // 是否暂停透传 tag
    impl<T: TickTimer> FreezableTimerReadonly for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn is_frozen(&self) -> bool {
            self.inner.is_frozen()
        }
    }

    // 暂停功能透传 tag
    impl<T: TickTimer> FreezableTimer for ContextWrapper<&mut FreezableTickTag, &mut T> {
        fn freeze(&mut self) {
            self.inner.freeze();
        }

        fn resume(&mut self) {
            self.inner.resume();
        }
    }

    // 时间进度透传 timer
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

    // 时间进度透传 timer
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

    // 时间步进被 tag 代理决定是否调用 timer
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
            Self { value: 0, limit }
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

    // 是否结束透传 tag
    impl<T: FlowingTimer + CyclicalTimer> FlowingTimerReadonly
        for ContextWrapper<&FewShotCycleTag, &T>
    {
        fn is_finished(&self) -> bool {
            // 只关注有限循环本身，真实的计时器可能是无限循环的，对应判断可能会存在异常
            self.inner.is_finished()
        }
    }

    // 是否结束透传 tag
    impl<T: FlowingTimer + CyclicalTimer> FlowingTimerReadonly
        for ContextWrapper<&mut FewShotCycleTag, &mut T>
    {
        fn is_finished(&self) -> bool {
            // 只关注有限循环本身，真实的计时器可能是无限循环的，对应判断可能会存在异常
            self.inner.is_finished()
        }
    }

    // 开始结束同时修改 tag 和 timer
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

    // 循环触发同时检查 tag 和 timer
    impl<T: FlowingTimer + CyclicalTimer> CyclicalTimer
        for ContextWrapper<&mut FewShotCycleTag, &mut T>
    {
        fn try_trigger_once(&mut self) -> bool {
            // 因为 FewShotCycleTag 允许次数只会减小不会增长
            // 所以当他失败时代表之后的触发也必定失败，因此无需回退前面的 timer
            // 若 FewShotCycleTag 支持临时增加次数，会导致之后的触发存在一周期的误差
            // 而重启会同时重启两者的状态，因此没这个问题，因此设计为只支持重启不支持增加

            // 先尝试触发计数器，成功后尝试触发有限循环，两者都成功才算成功触发
            self.ctx.try_trigger_once() && self.inner.try_trigger_once()
        }
    }
}
