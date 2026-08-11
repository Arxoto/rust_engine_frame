use crate::base_lib::cores::{
    design_patterns::{Union, UnitedInto}, timers::{
        tick_timer::TickTimer, tiny_timer::{TimerControl, TimerProgress, TimerView},
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
