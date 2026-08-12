//! 定义计时器的核心功能，并提供基础的逻辑复用
//!
//! 组合预制体的逻辑复用有多种方案
//! - 均使用泛型结构体
//!   - 会导致 tag 与特定 timer 所有权绑定
//!     - 【可暂停】需要干预 tick 行为
//!     - 【有限触发】需要干预 重置、强制结束、尝试触发 行为
//!   - 考虑到 tick 行为 和 重置、强制结束功能 一般被同一个结构体所实现，因此泛型结构的代码需要重复 2*2-1 = 3 遍（扩展行为的排列组合）
//!   - 考虑到代码段复制（泛型结构体）比透传调用（组合方式）更不可维护，因此不考虑该方案
//! - 宏实现代码复用
//!   - 考虑到调试难度和可读性，暂不考虑
//! - Blanket impl 自动实现
//!   - 需要配合 private::Sealed 私有封装防止下游重复实现，否则可能导致同一特征冲突实现
//!   - 无限循环和有限循环的逻辑，由于实现同一特征，因此判定存在冲突，无法优雅解决
//! - 组合间接实现代码复用，使用时临时生成代理
//!   - 部分特征函数需要持有可变引用，此时会导致不同特征使用时抢占同一份数据，需要将只读和可变特征函数分离定义
//!   - 要满足可变权限，那么组合生成的代理必须持有可变引用，此时只读函数中无法创建代理实例，权限截断，同样需要只读可变分离
//!   - 但是只读和可变特征分离会导致代码量增加
//!
//! 目前选择组合方案，并对【有限触发】功能做集成实现（他用到的地方比【可暂停】功能少得多）

use crate::base_lib::cores::design_patterns::Union;

/// tick 每帧驱动
pub trait Tickable {
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

/// 计时器【进度】只读视图
pub trait TimerProgress {
    /// 经过多长时间
    fn elapsed(&self) -> f64;

    /// 剩余时长
    fn remaining(&self) -> f64;

    /// 总持续时长
    fn duration(&self) -> f64;

    /// 进度比例
    fn progress(&self) -> f64;
}

/// 计时器【状态】只读视图
pub trait TimerView {
    /// 计时结束
    fn is_completed(&self) -> bool;
}

/// 计时器【状态】变更控制
pub trait TimerControl {
    /// 重置计时
    fn reset(&mut self);

    /// 结束计时
    fn complete(&mut self);
}

/// 计时器【暂停状态】
pub trait TimerPauseView {
    /// 是否暂停
    fn is_paused(&self) -> bool;
}

/// 计时器【暂停与恢复】控制
pub trait TimerPauseControl {
    /// 暂停计时
    fn pause(&mut self);

    /// 恢复计时
    fn resume(&mut self);
}

/// 循环触发器
pub trait CyclicalTrigger {
    /// 尝试触发一次
    fn try_trigger_once(&mut self) -> bool;
}

/// 拥有计时器，一个类型只能实现一次该特征
pub trait HasTimer {
    type Timer;

    fn get_timer(&self) -> &Self::Timer;

    fn get_timer_mut(&mut self) -> &mut Self::Timer;
}

// region: impl for Union<T, ()>

impl<T: TimerProgress> TimerProgress for Union<&T, ()> {
    fn elapsed(&self) -> f64 {
        self.0.elapsed()
    }

    fn remaining(&self) -> f64 {
        self.0.remaining()
    }

    fn duration(&self) -> f64 {
        self.0.duration()
    }

    fn progress(&self) -> f64 {
        self.0.progress()
    }
}

impl<T: TimerView> TimerView for Union<&T, ()> {
    fn is_completed(&self) -> bool {
        self.0.is_completed()
    }
}

impl<T: TimerControl> TimerControl for Union<&mut T, ()> {
    fn reset(&mut self) {
        self.0.reset()
    }

    fn complete(&mut self) {
        self.0.complete()
    }
}

impl<T: TimerPauseView> TimerPauseView for Union<&T, ()> {
    fn is_paused(&self) -> bool {
        self.0.is_paused()
    }
}

impl<T: TimerPauseControl> TimerPauseControl for Union<&mut T, ()> {
    fn pause(&mut self) {
        self.0.pause()
    }

    fn resume(&mut self) {
        self.0.resume()
    }
}

// endregion
