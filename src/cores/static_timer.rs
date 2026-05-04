/// 基于绝对时间戳实现的计时器
/// - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
/// - 适用于服务端验证、长期计时、数量规模庞大等场景
#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 生命周期持续时间 固定值
    duration: f64,
    /// 过期时间 静态记录
    expire_at: f64,
    /// 特殊的 为了内存优化 负数认为是没有暂停
    pause_at: f64,
}

impl StaticTimer {
    pub fn new(current_time: f64, duration: f64) -> Self {
        Self {
            duration,
            expire_at: current_time + duration,
            pause_at: -1.0,
        }
    }

    /// 重新计时
    ///
    /// ```
    /// # use rust_engine_frame::cores::static_timer::StaticTimer;
    /// let mut timer = StaticTimer::new(0.0, 5.0);
    /// timer.restart(3.0);
    /// assert!(!timer.is_expired(7.0));
    /// assert!(timer.is_expired(8.0));
    /// ```
    pub fn restart(&mut self, current_time: f64) {
        self.expire_at = current_time + self.duration;
        self.pause_at = -1.0;
    }

    /// 暂停
    ///
    /// ```
    /// # use rust_engine_frame::cores::static_timer::StaticTimer;
    /// let mut timer = StaticTimer::new(0.0, 5.0);
    /// assert!(timer.is_not_paused());
    /// timer.pause(3.0);
    /// assert!(timer.is_paused());
    /// ```
    pub fn pause(&mut self, current_time: f64) {
        if self.pause_at < 0.0 {
            self.pause_at = current_time;
        }
    }

    /// 继续计时
    ///
    /// ```
    /// # use rust_engine_frame::cores::static_timer::StaticTimer;
    /// let mut timer = StaticTimer::new(0.0, 5.0);
    /// assert!(timer.is_not_paused());
    /// timer.pause(3.0);
    /// assert!(timer.is_paused());
    /// timer.resume(5.0);
    /// assert!(timer.is_not_paused());
    /// ```
    pub fn resume(&mut self, current_time: f64) {
        if self.pause_at >= 0.0 {
            let pause_duration = current_time - self.pause_at;
            self.expire_at += pause_duration;
            self.pause_at = -1.0;
        }
    }

    pub fn is_paused(&self) -> bool {
        self.pause_at >= 0.0
    }

    pub fn is_not_paused(&self) -> bool {
        self.pause_at < 0.0
    }

    pub fn is_expired(&self, current_time: f64) -> bool {
        current_time >= self.expire_at
    }

    pub fn is_not_expired(&self, current_time: f64) -> bool {
        current_time < self.expire_at
    }

    pub fn is_timing(&self, current_time: f64) -> bool {
        self.is_not_paused() && self.is_not_expired(current_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== creation ==========

    #[test]
    fn new_sets_lifetime_and_expire() {
        let timer = StaticTimer::new(10.0, 3.0);
        assert!(timer.is_not_paused());
        assert!(timer.is_not_expired(12.0));
        assert!(timer.is_expired(13.0));
        assert!(timer.is_expired(13.0)); // exactly at expire_at
        assert!(timer.is_timing(12.5));
    }

    #[test]
    fn zero_duration_expires_immediately() {
        let timer = StaticTimer::new(0.0, 0.0);
        assert!(timer.is_expired(0.0));
        assert!(timer.is_not_expired(-0.001));
    }

    // ========== restart ==========

    #[test]
    fn restart_resets_expiry() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        assert!(timer.is_not_expired(4.0));
        timer.restart(4.0);
        assert!(timer.is_not_expired(8.0));
        assert!(timer.is_expired(9.0));
    }

    #[test]
    fn restart_clears_pause() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        timer.pause(2.0);
        assert!(timer.is_paused());
        timer.restart(10.0);
        assert!(timer.is_not_paused());
        assert!(timer.is_not_expired(14.0));
        assert!(timer.is_expired(15.0));
    }

    #[test]
    fn restart_from_expired() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        assert!(timer.is_expired(6.0));
        timer.restart(6.0);
        assert!(timer.is_not_expired(10.0));
        assert!(timer.is_expired(11.0));
    }

    // ========== pause / resume ==========

    #[test]
    fn pause_resume() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        assert!(timer.is_not_paused());
        // 计时区间 0.0 - 3.0
        timer.pause(3.0);
        assert!(timer.is_paused());

        // 计时区间 0.0 - 3.0 and 5.0 - now
        timer.resume(5.0);
        assert!(timer.is_not_paused());

        // 计时区间 0.0 - 3.0 and 5.0 - 6.0, cost 4.0, < lifetime 5.0
        assert!(timer.is_not_expired(6.0));
        // 计时区间 0.0 - 3.0 and 5.0 - 7.0, cost 5.0, >= lifetime 5.0
        assert!(timer.is_expired(7.0));
    }

    #[test]
    fn pause_and_pause() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        // 计时区间 0.0 - 3.0
        timer.pause(3.0);
        // 计时区间 0.0 - 3.0, 重复暂停不影响
        timer.pause(4.0);

        // 计时区间 0.0 - 3.0 and 5.0 - now
        timer.resume(5.0);

        // 计时区间 0.0 - 3.0 and 5.0 - 6.0, cost 4.0, < lifetime 5.0
        assert!(timer.is_not_expired(6.0));
        // 计时区间 0.0 - 3.0 and 5.0 - 7.0, cost 5.0, >= lifetime 5.0
        assert!(timer.is_expired(7.0));
    }

    #[test]
    fn resume_and_resume() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        // 计时区间 0.0 - 3.0
        timer.pause(3.0);

        // 计时区间 0.0 - 3.0 and 5.0 - now
        timer.resume(5.0);
        // 计时区间 0.0 - 3.0 and 5.0 - now, 重复继续不影响
        timer.resume(6.0);

        // 计时区间 0.0 - 3.0 and 5.0 - 6.0, cost 4.0, < lifetime 5.0
        assert!(timer.is_not_expired(6.0));
        // 计时区间 0.0 - 3.0 and 5.0 - 7.0, cost 5.0, >= lifetime 5.0
        assert!(timer.is_expired(7.0));
    }

    #[test]
    fn resume_when_not_paused_is_noop() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        timer.resume(3.0);
        assert!(timer.is_not_paused());
        assert!(timer.is_expired(5.0));
    }

    #[test]
    fn pause_at_zero() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        timer.pause(0.0);
        assert!(timer.is_paused());
        timer.resume(10.0);
        // expire_at was 5.0, paused for 10.0s → expire_at = 15.0
        assert!(timer.is_not_expired(14.0));
        assert!(timer.is_expired(15.0));
    }

    // ========== multiple pause/resume cycles ==========

    #[test]
    fn multiple_pause_resume_cycles() {
        let mut timer = StaticTimer::new(0.0, 10.0);
        // run 0→2 (2s elapsed)
        timer.pause(2.0);
        timer.resume(5.0); // paused 3s, expire_at shifts from 10→13
        // run 5→8 (3s elapsed, total 5s)
        timer.pause(8.0);
        timer.resume(12.0); // paused 4s, expire_at shifts from 13→17
        // total elapsed: 5s, lifetime: 10s
        assert!(timer.is_not_expired(14.0)); // run 8→14 = 6s, total 5+6=11 > 10
        // actually: from start 0, ran 0→2 (2s), 5→8 (3s), 12→14 (2s) = 7s < 10
        assert!(timer.is_not_expired(14.0));
        assert!(timer.is_expired(17.0));
    }

    // ========== is_expired vs pause interaction ==========

    #[test]
    fn is_expired_ignores_pause_state() {
        // is_expired 只看 expire_at，不管是否暂停
        let mut timer = StaticTimer::new(0.0, 5.0);
        timer.pause(1.0);
        // expire_at 仍然是 5.0，即便在暂停中
        assert!(timer.is_paused());
        assert!(timer.is_not_expired(4.0));
        assert!(timer.is_expired(6.0));
    }

    // ========== is_timing ==========

    #[test]
    fn is_timing_false_when_paused() {
        let mut timer = StaticTimer::new(0.0, 5.0);
        assert!(timer.is_timing(1.0));
        timer.pause(2.0);
        assert!(!timer.is_timing(3.0)); // paused, so not timing
        timer.resume(4.0);
        assert!(timer.is_timing(5.0)); // resumed, still not expired
    }

    #[test]
    fn is_timing_false_when_expired() {
        let timer = StaticTimer::new(0.0, 5.0);
        assert!(timer.is_timing(3.0));
        assert!(!timer.is_timing(5.0)); // exactly at expire
    }
}
