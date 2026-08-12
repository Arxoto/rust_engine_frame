//! 基于绝对时间戳实现的计时器
//! - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
//! - 【注意】不应该对静态时钟和时间线包装实现暂停功能，应该在最外层单独操作时间线
//! - 【缺点】长时间运行可能导致精度不佳，需要定时重置时间线，并一同处理所有关联的静态时钟
//! - 适用于服务端验证、长期计时，静态时间戳可能需要更换非浮点类型防止误差累积，如 [`std::time::Duration`]
//! - 数量规模庞大的场景，将大量的循环触发的 TickTimer 转换为 StaticTimer 表示状态和少量触发式 TickTimer 用于结算

use crate::base_lib::cores::{
    design_patterns::{Union, UnitedInto},
    timers::{
        tick_timer::TickTimer,
        tiny_timer::{TimerControl, TimerProgress, TimerView},
    },
};

/// 静态计时器的参考时间线，暂停等功能在时间线上实现
#[derive(Clone, Debug)]
pub struct StaticTimeline(pub TickTimer);

impl StaticTimeline {
    /// 创建一个永不停止的计时器 用作静态计时器的基准
    pub fn new() -> Self {
        Self(TickTimer::new(f64::INFINITY))
    }

    pub fn current_time(&self) -> f64 {
        self.0.elapsed()
    }

    /// 确认没有依赖本时间线的计时器后，【重启时间线】，防止无限累加引起精度丢失
    pub fn reset_timeline(&mut self) {
        self.0.reset();
    }
}

#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 计时器时长
    duration: f64,
    /// 计时结束时间
    end_at: f64,
}

impl StaticTimer {
    pub fn new(timeline: &StaticTimeline, duration: f64) -> Self {
        Self {
            duration,
            end_at: timeline.current_time() + duration,
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

impl TimerProgress for Union<&StaticTimer, &StaticTimeline> {
    fn elapsed(&self) -> f64 {
        self.duration() - self.remaining()
    }

    fn remaining(&self) -> f64 {
        (self.0.end_at - self.1.current_time()).min(0.0)
    }

    fn duration(&self) -> f64 {
        self.0.duration
    }

    fn progress(&self) -> f64 {
        1.0 - self.remaining() / self.duration()
    }
}

impl TimerView for Union<&StaticTimer, &StaticTimeline> {
    fn is_completed(&self) -> bool {
        self.1.current_time() >= self.0.end_at
    }
}

impl TimerControl for Union<&mut StaticTimer, &StaticTimeline> {
    fn reset(&mut self) {
        self.0.end_at = self.1.current_time() + self.0.duration;
    }

    fn complete(&mut self) {
        self.0.end_at = self.1.current_time();
    }
}

type Stl = StaticTimeline;

impl<'a, 'b> UnitedInto<&'b Stl, Union<&'a StaticTimer, &'b Stl>> for &'a StaticTimer {
    fn unite_into(self, w: &'b Stl) -> Union<&'a StaticTimer, &'b Stl> {
        Union(self, w)
    }
}

impl<'a, 'b> UnitedInto<&'b Stl, Union<&'a mut StaticTimer, &'b Stl>> for &'a mut StaticTimer {
    fn unite_into(self, w: &'b Stl) -> Union<&'a mut StaticTimer, &'b Stl> {
        Union(self, w)
    }
}
