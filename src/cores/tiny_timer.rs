#[derive(Clone, Debug)]
pub struct TinyTimer {
    time_limit: f64,
    time: f64,
    /// 时间流动
    flow: bool,
}

impl TinyTimer {
    pub fn new(limit: f64) -> Self {
        Self {
            time_limit: limit,
            time: 0.0,
            flow: true,
        }
    }

    /// 重新计时
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(3.0);
    /// timer.restart();
    /// timer.tick(2.0);
    /// assert_eq!(timer.cost_time(), 2.0);
    /// ```
    pub fn restart(&mut self) {
        self.flow = true;
        self.time = 0.0;
    }

    /// 暂停 pause/freeze
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(3.0);
    /// timer.pause();
    /// timer.tick(2.0);
    /// assert_eq!(timer.cost_time(), 3.0);
    /// ```
    pub fn pause(&mut self) {
        self.flow = false;
    }

    /// 继续计时
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(3.0);
    /// timer.pause();
    /// timer.tick(2.0);
    /// assert_eq!(timer.cost_time(), 3.0);
    /// timer.resume();
    /// timer.tick(1.0);
    /// assert_eq!(timer.cost_time(), 4.0);
    /// ```
    pub fn resume(&mut self) {
        self.flow = true;
    }

    /// 时间流逝
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(3.0);
    /// assert_eq!(timer.cost_time(), 3.0);
    /// ```
    pub fn tick(&mut self, delta: f64) {
        if self.flow {
            self.time = self.time_limit.min(self.time + delta);
        }
    }

    /// 获得计时器经过了多少时间
    pub fn cost_time(&self) -> f64 {
        self.time
    }

    /// 计时器是否暂停
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// assert!(timer.is_not_paused());
    /// timer.pause();
    /// assert!(timer.is_paused());
    /// ```
    pub fn is_paused(&self) -> bool {
        !self.flow
    }

    /// 计时器是否正在计时
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// assert!(timer.is_not_paused());
    /// timer.pause();
    /// assert!(timer.is_paused());
    /// ```
    pub fn is_not_paused(&self) -> bool {
        self.flow
    }

    /// 计时进行中 未结束
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(1.0);
    /// assert!(timer.is_timing());
    /// timer.pause();
    /// assert!(!timer.is_timing());
    /// timer.resume();
    /// timer.tick(5.0);
    /// assert!(!timer.is_timing());
    /// ```
    pub fn is_timing(&self) -> bool {
        self.flow && self.time < self.time_limit
    }

    /// 计时结束
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(1.0);
    /// assert!(timer.is_not_finished());
    /// timer.tick(5.0);
    /// assert!(timer.is_finished());
    /// ```
    pub fn is_finished(&self) -> bool {
        self.time >= self.time_limit
    }

    /// 计时未结束
    /// 
    /// ```
    /// # use rust_engine_frame::cores::tiny_timer::TinyTimer;
    /// let mut timer = TinyTimer::new(5.0);
    /// timer.tick(1.0);
    /// assert!(timer.is_not_finished());
    /// timer.tick(5.0);
    /// assert!(timer.is_finished());
    /// ```
    pub fn is_not_finished(&self) -> bool {
        self.time < self.time_limit
    }
}
