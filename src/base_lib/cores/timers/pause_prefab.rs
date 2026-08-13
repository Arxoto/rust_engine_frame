use crate::base_lib::cores::{
    design_patterns::Union,
    timers::tiny_timer::{
        CyclicalTrigger, Tickable, TimerControl, TimerPauseControl, TimerPauseView, TimerProgress,
        TimerView,
    },
};

/// 冻结预制体，能对所有计时器类型进行代理，干预 [`Tickable::tick`]
#[derive(Clone, Debug)]
pub struct PausePrefab(bool);

impl Default for PausePrefab {
    fn default() -> Self {
        // 默认不冻结
        Self(false)
    }
}

impl TimerPauseView for PausePrefab {
    fn is_paused(&self) -> bool {
        self.0
    }
}

impl TimerPauseControl for PausePrefab {
    fn pause(&mut self) {
        self.0 = true
    }

    fn resume(&mut self) {
        self.0 = false
    }
}

// region: fn for union

impl PausePrefab {
    #[inline]
    pub fn of_tickable<T: Tickable>(&self, t: &mut T) -> impl Tickable {
        Union(self, t)
    }

    #[inline]
    pub fn of_timer_pause_view<T>(&self, t: &T) -> impl TimerPauseView {
        Union(self, t)
    }

    #[inline]
    pub fn of_timer_pause_control<T>(&mut self, t: &T) -> impl TimerPauseControl {
        Union(self, t)
    }

    #[inline]
    pub fn of_timer_progress<T: TimerProgress>(&self, t: &T) -> impl TimerProgress {
        Union(self, t)
    }

    #[inline]
    pub fn of_timer_view<T: TimerView>(&self, t: &T) -> impl TimerView {
        Union(self, t)
    }

    #[inline]
    pub fn of_timer_control<T: TimerControl>(&self, t: &mut T) -> impl TimerControl {
        Union(self, t)
    }

    #[inline]
    pub fn of_cyclical_trigger<T: CyclicalTrigger>(&self, t: &mut T) -> impl CyclicalTrigger {
        Union(self, t)
    }
}

// endregion

// region: impl for union

// 根据 prefab 决定是否调用 timer tick
impl<T: Tickable> Tickable for Union<&PausePrefab, &mut T> {
    fn tick(&mut self, delta: f64) {
        if !self.0.is_paused() {
            self.1.tick(delta);
        }
    }
}

// 是否暂停透传 prefab
impl<T> TimerPauseView for Union<&PausePrefab, &T> {
    fn is_paused(&self) -> bool {
        self.0.is_paused()
    }
}

// 暂停恢复功能透传 prefab
impl<T> TimerPauseControl for Union<&mut PausePrefab, &T> {
    fn pause(&mut self) {
        self.0.pause();
    }

    fn resume(&mut self) {
        self.0.resume();
    }
}

// 时间进度透传 timer
impl<T: TimerProgress> TimerProgress for Union<&PausePrefab, &T> {
    type Ctx<'a> = T::Ctx<'a>;

    fn elapsed<'a>(&self, ctx: Self::Ctx<'a>) -> f64 {
        self.1.elapsed(ctx)
    }

    fn remaining<'a>(&self, ctx: Self::Ctx<'a>) -> f64 {
        self.1.remaining(ctx)
    }

    fn duration<'a>(&self, ctx: Self::Ctx<'a>) -> f64 {
        self.1.duration(ctx)
    }

    fn progress<'a>(&self, ctx: Self::Ctx<'a>) -> f64 {
        self.1.progress(ctx)
    }
}

impl<T: TimerView> TimerView for Union<&PausePrefab, &T> {
    type Ctx<'a> = T::Ctx<'a>;

    fn is_completed<'a>(&self, ctx: Self::Ctx<'a>) -> bool {
        self.1.is_completed(ctx)
    }
}

impl<T: TimerControl> TimerControl for Union<&PausePrefab, &mut T> {
    type Ctx<'a> = T::Ctx<'a>;

    fn reset<'a>(&mut self, ctx: Self::Ctx<'a>) {
        self.1.reset(ctx);
    }

    fn complete<'a>(&mut self, ctx: Self::Ctx<'a>) {
        self.1.complete(ctx);
    }
}

impl<T: CyclicalTrigger> CyclicalTrigger for Union<&PausePrefab, &mut T> {
    type Ctx<'a> = T::Ctx<'a>;

    fn try_trigger_once<'a>(&mut self, ctx: Self::Ctx<'a>) -> bool {
        self.1.try_trigger_once(ctx)
    }
}

// endregion
