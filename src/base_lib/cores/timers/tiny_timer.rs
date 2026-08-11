use crate::base_lib::cores::design_patterns::Union;

/// tick 驱动
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

/// 计时器进度只读视图
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

/// 计时器状态只读视图
pub trait TimerView {
    /// 计时结束
    fn is_completed(&self) -> bool;
}

/// 计时器状态变更控制
pub trait TimerControl {
    /// 重置计时
    fn reset(&mut self);

    /// 结束计时
    fn complete(&mut self);
}

/// 计时器暂停状态
pub trait TimerPauseView {
    /// 是否暂停
    fn is_paused(&self) -> bool;
}

/// 计时器暂停与恢复控制
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

// 默认实现

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
