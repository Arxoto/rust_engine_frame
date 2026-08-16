//! 定义计时器的核心功能，并提供基础的逻辑复用
//!
//! 组合预制体的逻辑复用有多种方案
//! - 均使用泛型结构体
//!   - 结构体内部字段控制是否暂停或有限触发，同时持有纯计时器的所有权
//!   - 会导致 tag 与特定 timer 所有权绑定
//!     - 【可暂停】需要干预 tick 行为
//!     - 【有限触发】需要干预 重置、强制结束、尝试触发 行为
//!   - 考虑到 tick 行为 和 重置、强制结束功能 一般被同一个结构体所实现，因此泛型结构的代码需要重复 2*2-1 = 3 遍（扩展行为的排列组合）
//!   - 考虑到代码段复制（泛型结构体）比透传调用（组合方式）更不可维护，因此不考虑该方案
//! - 宏实现代码复用
//!   - 考虑到调试难度和可读性，暂不考虑
//! - Blanket impl 自动实现
//!   - 自动为“持有预制体”的结构体实现对应特征
//!   - 需要配合 private::Sealed 私有封装防止下游重复实现，否则可能导致同一特征冲突实现
//!   - 无限循环和有限循环的逻辑，由于实现同一特征，因此判定存在冲突，无法优雅解决
//! - 组合间接实现代码复用，使用时临时生成代理
//!   - 部分特征函数需要持有可变引用，此时会导致不同特征使用时抢占同一份数据，需要将只读和可变特征函数分离定义
//!   - 要满足可变权限，那么组合生成的代理必须持有可变引用，此时只读函数中无法创建代理实例，权限截断，同样需要只读可变分离
//!   - 但是只读和可变特征分离会导致代码量增加
//!
//! 目前选择“组合”方案，并对【有限触发】功能做集成实现（他用到的地方比【可暂停】功能少得多）
//!
//! trait 面按能力边界拆分，不进行合并。 8 个 trait 对应不同能力(进度模型/完成状态/手动控制/循环触发/暂停读/暂停写/tick/HasTimer)。
//! - `TimerProgress` (进度) 与 `TimerView` (完成状态) 是两种能力， `TimerControl` 与 `CyclicalTrigger` 同理， `TickTimer`/`StaticTimer` 无循环触发能力
//! - 合并会强迫仅实现单能力的类型实现无语义方法，并且组合方案生成的临时引用代理要求只读和可变能力需要进行区分，故刻意保持拆分(决议见 `.scratch/interface-deepening/issues/01`)。

use crate::base_lib::cores::{design_patterns::DependCtx, unify_types::time_type};

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
    ///
    /// 调用次序是【上层】的约定，本 trait 只累加时间；同一 1s 计时器，两种次序在边界帧语义不同：
    ///
    /// ```
    /// # use rust_engine_frame::base_lib::cores::timers::tick_timer::TickTimer;
    /// # use rust_engine_frame::base_lib::cores::timers::tiny_timer::{Tickable, TimerView};
    /// # use rust_engine_frame::base_lib::cores::unify_types::time_type;
    /// // 限制类（冷却）：先 tick 后业务 —— 同一帧到达即可用
    /// let mut cooldown = TickTimer::new(time_type::unit::<1>());
    /// cooldown.tick(time_type::unit::<1>()); // 先累加
    /// assert!(cooldown.is_completed(()));    // 后判断，帧 N 即可用
    ///
    /// // 帮助类（容错窗口）：先业务后 tick —— 窗口宽一帧
    /// let mut grace = TickTimer::new(time_type::unit::<1>());
    /// assert!(!grace.is_completed(()));      // 先判断，帧 N 仍可用
    /// grace.tick(time_type::unit::<1>());    // 后累加
    /// assert!(grace.is_completed(()));       // 帧 N+1 才不可用
    /// ```
    fn tick(&mut self, delta: time_type::T);
}

/// 计时器【进度】只读视图
pub trait TimerProgress: DependCtx {
    /// 经过多长时间
    fn elapsed(&self, ctx: Self::Ctx<'_>) -> time_type::T;

    /// 剩余时长
    fn remaining(&self, ctx: Self::Ctx<'_>) -> time_type::T;

    /// 总持续时长
    fn duration(&self, ctx: Self::Ctx<'_>) -> time_type::T;

    /// 进度比例
    fn progress(&self, ctx: Self::Ctx<'_>) -> f64;
}

/// 计时器【状态】只读视图
pub trait TimerView: DependCtx {
    /// 计时结束
    fn is_completed(&self, ctx: Self::Ctx<'_>) -> bool;
}

/// 计时器【状态】变更控制
pub trait TimerControl: DependCtx {
    /// 重置计时
    fn reset(&mut self, ctx: Self::Ctx<'_>);

    /// 结束计时
    fn complete(&mut self, ctx: Self::Ctx<'_>);
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
pub trait CyclicalTrigger: DependCtx {
    /// 尝试触发一次
    fn try_trigger_once(&mut self, ctx: Self::Ctx<'_>) -> bool;
}

/// 拥有计时器，一个类型只能实现一次该特征
pub trait HasTimer {
    type Timer;

    fn get_timer(&self) -> &Self::Timer;

    fn get_timer_mut(&mut self) -> &mut Self::Timer;
}
