#[derive(Debug, Default)]
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
            ..Default::default()
        }
    }

    /// 开始计时
    pub fn start_time(&mut self) {
        self.flow = true;
        self.time = 0.0;
    }

    /// 强制结束 pause/freeze
    pub fn final_time(&mut self) {
        self.flow = false;
    }

    /// 时间流逝
    pub fn add_time(&mut self, delta: f64) {
        if self.flow {
            self.time = self.time_limit.min(self.time + delta);
        }
    }

    /// 计时进行中 未结束
    pub fn in_time(&self) -> bool {
        self.flow && self.time < self.time_limit
    }

    /// 时间自然结束
    pub fn is_end(&self) -> bool {
        self.flow && self.time >= self.time_limit
    }

    /// 时间强制冻结
    pub fn is_forced_final(&self) -> bool {
        !self.flow
    }
}
