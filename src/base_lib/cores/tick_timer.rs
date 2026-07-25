//! 基于增量累加时间实现的计时器
//! - 【缺点】长时间累加可能存在误差
//! - 每帧调用 `tick` 方法来更新计时器状态，逻辑简单清晰
//! - 适用于需要知道进度（动画特效）、短生命周期、局部时间调速等场景

use crate::base_lib::cores::tiny_timer::{
    CyclicalTimer, FlowingTimer, FlowingTimerReadonly, TickTimer, TinyTimer,
};

/// 有限增长的 [`TickTimer`] 默认实现 [`FlowingTimer`]
#[derive(Clone, Debug)]
pub struct TickTimerFinite {
    time: f64,
    time_limit: f64,
}

impl TickTimerFinite {
    pub fn new(limit: f64) -> Self {
        Self {
            time: 0.0,
            time_limit: limit,
        }
    }
}

impl TinyTimer for TickTimerFinite {
    fn get_time(&self) -> f64 {
        self.time
    }

    fn get_time_left(&self) -> f64 {
        self.time_limit - self.time
    }

    fn get_time_limit(&self) -> f64 {
        self.time_limit
    }

    fn get_time_ratio(&self) -> f64 {
        self.time / self.time_limit
    }
}

impl TickTimer for TickTimerFinite {
    fn tick(&mut self, delta: f64) {
        // 有限累加
        self.time = self.time_limit.min(self.time + delta)
    }
}

impl FlowingTimerReadonly for TickTimerFinite {
    fn is_finished(&self) -> bool {
        self.time >= self.time_limit
    }
}

impl FlowingTimer for TickTimerFinite {
    fn restart(&mut self) {
        self.time = 0.0
    }

    fn finish(&mut self) {
        self.time = self.time_limit
    }
}

/// 无限增长的 [`TickTimer`] 默认实现 [`FlowingTimer`] 额外实现 [`CyclicalTimer`] 无限循环
#[derive(Clone, Debug)]
pub struct TickTimerInfinite {
    time: f64,
    time_limit: f64,
}

impl TickTimerInfinite {
    pub fn new(limit: f64) -> Self {
        Self {
            time: 0.0,
            time_limit: limit,
        }
    }
}

impl TinyTimer for TickTimerInfinite {
    fn get_time(&self) -> f64 {
        // when time is INF, return NAN
        // cause ratio to be NAN, left to be NAN
        self.time % self.time_limit
    }

    fn get_time_left(&self) -> f64 {
        self.time_limit - self.get_time()
    }

    fn get_time_limit(&self) -> f64 {
        self.time_limit
    }

    fn get_time_ratio(&self) -> f64 {
        self.get_time() / self.get_time_limit()
    }
}

impl TickTimer for TickTimerInfinite {
    fn tick(&mut self, delta: f64) {
        // 无限累加
        self.time += delta
    }
}

impl FlowingTimerReadonly for TickTimerInfinite {
    fn is_finished(&self) -> bool {
        // 无法结束
        false
    }
}

impl FlowingTimer for TickTimerInfinite {
    fn restart(&mut self) {
        self.time = 0.0
    }

    fn finish(&mut self) {
        // do nothing
    }
}

impl CyclicalTimer for TickTimerInfinite {
    fn try_trigger_once(&mut self) -> bool {
        if self.time >= self.time_limit {
            self.time -= self.time_limit;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    #[test]
    fn test_f64() {
        let number: f64 = 7.0;

        assert_eq!(number.min(f64::INFINITY), number);
        assert_eq!(number.min(f64::NAN), number);
        assert_eq!(number.max(f64::NAN), number);

        assert!(f64::INFINITY == f64::INFINITY);
        assert!(!(f64::INFINITY > f64::INFINITY));
        assert!(!(f64::INFINITY < f64::INFINITY));
        assert!(number != f64::NAN);
        assert!(!(number > f64::NAN));
        assert!(!(number < f64::NAN));

        assert_eq!(number + f64::INFINITY, f64::INFINITY);
        assert_eq!(number - f64::INFINITY, f64::NEG_INFINITY);
        assert_eq!(f64::INFINITY + f64::INFINITY, f64::INFINITY);
        assert!((f64::INFINITY - f64::INFINITY).is_nan());
        assert!((number + f64::NAN).is_nan());
        assert!((number - f64::NAN).is_nan());

        assert_eq!(number / f64::INFINITY, 0.0);
        assert_eq!(f64::INFINITY / number, f64::INFINITY);
        assert!((number / f64::NAN).is_nan());
        assert!((f64::NAN / number).is_nan());

        assert_eq!(number % f64::INFINITY, number);
        assert!((f64::INFINITY % number).is_nan());
        assert!((f64::NAN % number).is_nan());
        assert!((number % f64::NAN).is_nan());
    }

    // todo unit test
}
