/// 效果持续方式
#[derive(Clone, Debug)]
pub struct Duration {
    /// 存在计时
    pub(crate) life_time: f64,
    /// 持续时间 零和负数表示无限存在
    pub(crate) duration_time: f64,
    /// 触发周期【默认不触发】 零和负数表示不重复触发 如果仅表示一种状态那就不要用触发
    pub(crate) period_time: f64,
    /// 触发等待时间【默认没有】 注意每满一个周期才会触发 即第一次触发时间是等待时间加触发周期
    pub(crate) wait_time: f64,

    /// 堆叠层数【初始一层】 重新添加效果时触发
    pub(crate) stack: i64,
    /// 堆叠上限【默认上限一层】 零和负数表示无上限
    pub(crate) max_stack: i64,
}

impl Default for Duration {
    fn default() -> Self {
        Self {
            life_time: 0.0,
            duration_time: 0.0,
            period_time: 0.0,
            wait_time: 0.0,
            stack: 1,
            max_stack: 1,
        }
    }
}

impl Duration {
    pub fn new_infinite() -> Self {
        Self {
            duration_time: 0.0,
            ..Default::default()
        }
    }

    pub fn new_duration(duration_time: f64) -> Self {
        Self {
            duration_time,
            ..Default::default()
        }
    }
}

// =================================================================================

/// 代理类型 效果持续方式
pub trait ProxyDuration {
    fn as_duration(&self) -> &Duration;
    fn as_mut_duration(&mut self) -> &mut Duration;

    fn get_life_time(&self) -> f64 {
        self.as_duration().life_time
    }

    fn set_life_time(&mut self, v: f64) {
        self.as_mut_duration().life_time = v
    }

    fn get_duration_time(&self) -> f64 {
        self.as_duration().duration_time
    }

    fn set_duration_time(&mut self, v: f64) {
        self.as_mut_duration().duration_time = v
    }

    fn get_period_time(&self) -> f64 {
        self.as_duration().period_time
    }

    fn set_period_time(&mut self, v: f64) {
        self.as_mut_duration().period_time = v
    }

    fn get_wait_time(&self) -> f64 {
        self.as_duration().wait_time
    }

    fn set_wait_time(&mut self, v: f64) {
        self.as_mut_duration().wait_time = v
    }

    fn get_stack(&self) -> i64 {
        self.as_duration().stack
    }

    fn set_stack(&mut self, v: i64) {
        self.as_mut_duration().stack = v
    }

    fn get_max_stack(&self) -> i64 {
        self.as_duration().max_stack
    }

    fn set_max_stack(&mut self, v: i64) {
        self.as_mut_duration().max_stack = v
    }

    // ===========================
    // 业务逻辑
    // ===========================

    /// 是否无限存在
    fn is_infinite(&self) -> bool {
        self.get_duration_time() <= 0.0
    }

    /// 是否持续一段时间
    fn is_duration(&self) -> bool {
        self.get_duration_time() > 0.0
    }

    /// 是否周期触发
    fn is_period(&self) -> bool {
        self.get_period_time() > 0.0
    }

    /// 是否限制堆叠层数
    fn is_limit_stack(&self) -> bool {
        self.get_max_stack() > 0
    }

    /// 重新开始计时
    fn restart_life(&mut self) {
        self.set_life_time(0.0);
    }

    /// 是否过期
    fn is_expired(&self) -> bool {
        if self.is_infinite() {
            false
        } else {
            self.get_life_time() >= self.get_duration_time()
        }
    }

    /// 获取存在时间（直接获取存在计时有可能略微超过持续时间）
    fn fetch_life_time(&self) -> f64 {
        if self.is_infinite() {
            self.get_life_time()
        } else {
            self.get_duration_time().min(self.get_life_time())
        }
    }

    /// 当前时间的总共应触发次数
    fn period_counts(&self) -> i64 {
        if !self.is_period() {
            return 0;
        }

        let life_t = self.fetch_life_time();

        if life_t > self.get_wait_time() {
            ((life_t - self.get_wait_time()) / self.get_period_time()) as i64
        } else {
            0
        }
    }

    /// 处理时间 返回这个周期触发的次数
    fn process_period(&mut self, delta: f64) -> i64 {
        if self.is_expired() {
            return 0;
        }

        if !self.is_period() {
            self.set_life_time(self.get_life_time() + delta);
            return 0;
        }

        let old_count = self.period_counts();
        self.set_life_time(self.get_life_time() + delta);
        return self.period_counts() - old_count;
    }

    /// 尝试叠加 返回叠加后的层数
    fn try_add_stack(&mut self, c: i64) -> i64 {
        if self.is_limit_stack() {
            self.set_stack(self.get_max_stack().min(self.get_stack() + c));
        } else {
            self.set_stack(self.get_stack() + c);
        }
        return self.get_stack();
    }
}

impl ProxyDuration for Duration {
    fn as_duration(&self) -> &Duration {
        self
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        self
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite() {
        let mut ddd = Duration::new_infinite();
        assert!(ddd.is_infinite());
        assert!(!ddd.is_expired());

        ddd.set_life_time(1.0);
        assert_eq!(1.0, ddd.fetch_life_time());
    }

    #[test]
    fn test_expired_duration() {
        let mut ddd = Duration::new_duration(1.0);
        assert!(!ddd.is_infinite());
        assert!(!ddd.is_expired());

        ddd.set_life_time(0.9);
        assert!(!ddd.is_expired());
        assert_eq!(0.9, ddd.fetch_life_time());

        ddd.set_life_time(1.0);
        assert!(ddd.is_expired());
        assert_eq!(1.0, ddd.fetch_life_time());

        ddd.set_life_time(1.1);
        assert!(ddd.is_expired());
        assert_eq!(1.0, ddd.fetch_life_time());
    }

    #[test]
    fn test_period() {
        // 周期0不触发
        assert_eq!(
            0,
            Duration {
                life_time: 6.0,
                ..Default::default()
            }
            .period_counts()
        );

        // 无等待时间触发
        assert_eq!(
            6,
            Duration {
                life_time: 6.0,
                period_time: 1.0,
                ..Default::default()
            }
            .period_counts()
        );

        // 等待时间内不触发
        assert_eq!(
            0,
            Duration {
                life_time: 6.0,
                period_time: 1.0,
                wait_time: 8.0,
                ..Default::default()
            }
            .period_counts()
        );

        // 等待时间外 但没满一个周期 不触发
        assert_eq!(
            0,
            Duration {
                life_time: 8.9,
                period_time: 1.0,
                wait_time: 8.0,
                ..Default::default()
            }
            .period_counts()
        );

        // 等待时间后刚满一个周期 触发
        assert_eq!(
            1,
            Duration {
                life_time: 9.0,
                period_time: 1.0,
                wait_time: 8.0,
                ..Default::default()
            }
            .period_counts()
        );

        // 等待时间后刚满一个周期 触发多次
        assert_eq!(
            10,
            Duration {
                life_time: 9.0,
                period_time: 0.1,
                wait_time: 8.0,
                ..Default::default()
            }
            .period_counts()
        );
    }

    #[test]
    fn process_period() {
        let mut ddd = Duration {
            duration_time: 10.0,
            period_time: 1.0,
            wait_time: 3.0,
            ..Default::default()
        };
        assert_eq!(0, ddd.process_period(1.0));
        assert_eq!(0, ddd.process_period(1.0));
        assert_eq!(0, ddd.process_period(1.1)); // 3.1s
        assert_eq!(1, ddd.process_period(1.1)); // 4.2s
        assert_eq!(1, ddd.process_period(1.1)); // 5.3s
        assert_eq!(0, ddd.process_period(0.6)); // 5.9s
        assert_eq!(1, ddd.process_period(0.6)); // 6.5s
    }

    #[test]
    fn add_stack() {
        // 默认仅一层
        let mut ddd = Duration::default();
        assert_eq!(1, ddd.get_stack());

        ddd.try_add_stack(1);
        assert_eq!(1, ddd.get_stack());

        // 限制3层
        ddd.set_max_stack(3);

        ddd.try_add_stack(1);
        assert_eq!(2, ddd.get_stack());

        ddd.try_add_stack(1);
        assert_eq!(3, ddd.get_stack());

        ddd.try_add_stack(1);
        assert_eq!(3, ddd.get_stack());

        // 不限制
        ddd.set_max_stack(0);

        ddd.try_add_stack(1);
        assert_eq!(4, ddd.get_stack());
    }
}
