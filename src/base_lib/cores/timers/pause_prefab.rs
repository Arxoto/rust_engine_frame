use crate::base_lib::cores::{
    design_patterns::Union,
    timers::tiny_timer::{
        CyclicalTrigger, Tickable, TimerControl, TimerPauseControl, TimerPauseView, TimerProgress,
        TimerView,
    },
};

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

// region: impl for union

impl<T: Tickable> Tickable for Union<&PausePrefab, &mut T> {
    fn tick(&mut self, delta: f64) {
        if !self.0.is_paused() {
            self.1.tick(delta);
        }
    }
}

impl<T> TimerPauseView for Union<&PausePrefab, &T> {
    fn is_paused(&self) -> bool {
        self.0.is_paused()
    }
}

impl<T> TimerPauseControl for Union<&mut PausePrefab, &T> {
    fn pause(&mut self) {
        self.0.pause();
    }

    fn resume(&mut self) {
        self.0.resume();
    }
}

impl<T: TimerProgress> TimerProgress for Union<&PausePrefab, &T> {
    fn elapsed(&self) -> f64 {
        self.1.elapsed()
    }

    fn remaining(&self) -> f64 {
        self.1.remaining()
    }

    fn duration(&self) -> f64 {
        self.1.duration()
    }

    fn progress(&self) -> f64 {
        self.1.progress()
    }
}

impl<T: TimerView> TimerView for Union<&PausePrefab, &T> {
    fn is_completed(&self) -> bool {
        self.1.is_completed()
    }
}

impl<T: TimerControl> TimerControl for Union<&PausePrefab, &mut T> {
    fn reset(&mut self) {
        self.1.reset();
    }

    fn complete(&mut self) {
        self.1.complete();
    }
}

impl<T: CyclicalTrigger> CyclicalTrigger for Union<&PausePrefab, &mut T> {
    fn try_trigger_once(&mut self) -> bool {
        self.1.try_trigger_once()
    }
}

// endregion
