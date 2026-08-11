use crate::base_lib::cores::{
    design_patterns::Union,
    timers::tiny_timer::{CyclicalTrigger, TimerControl, TimerView},
};

#[derive(Clone, Debug)]
pub(super) struct FewShotTimes {
    current: u32,
    limit: u32,
}

impl FewShotTimes {
    pub fn new(limit: u32) -> Self {
        Self { current: 0, limit }
    }
}

impl<T> TimerView for Union<&FewShotTimes, &T> {
    fn is_completed(&self) -> bool {
        self.0.current >= self.0.limit
    }
}

impl<T: TimerControl> TimerControl for Union<&mut FewShotTimes, &mut T> {
    fn reset(&mut self) {
        self.0.current = 0;
        self.1.reset();
    }

    fn complete(&mut self) {
        self.0.current = self.0.limit;
        self.1.complete();
    }
}

impl<T: CyclicalTrigger> CyclicalTrigger for Union<&mut FewShotTimes, &mut T> {
    fn try_trigger_once(&mut self) -> bool {
        if !self.1.try_trigger_once() {
            return false;
        }

        let u = Union(&*self.0, &*self.1);
        if u.is_completed() {
            false
        } else {
            self.0.current += 1;
            true
        }
    }
}
