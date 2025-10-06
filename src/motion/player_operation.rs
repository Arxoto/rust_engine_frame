use crate::cores::tiny_timer::TinyTimer;

pub trait PlayerOperation {
    /// 指令是否应该激活对应操作
    fn operation_active(&self) -> bool;
    /// 操作成功时触发回显 取消对应操作
    fn operation_echo(&mut self);
    /// 若相同属性已触发回显 则也取消对应操作
    fn operation_echo_with(&mut self, other: &Self) {
        if !other.operation_active() {
            self.operation_echo();
        }
    }
}

impl PlayerOperation for bool {
    fn operation_active(&self) -> bool {
        *self
    }

    fn operation_echo(&mut self) {
        *self = false;
    }
}

impl<T: PlayerOperation + Clone> PlayerOperation for Option<T> {
    fn operation_active(&self) -> bool {
        self.clone().is_some_and(|o| o.operation_active())
    }

    fn operation_echo(&mut self) {
        *self = None;
    }
}

const DEAD_ZONE: f64 = 0.01;

// for direction or angle
impl PlayerOperation for f64 {
    fn operation_active(&self) -> bool {
        *self < -DEAD_ZONE || DEAD_ZONE < *self
    }

    fn operation_echo(&mut self) {
        *self = 0.0;
    }
}

// 有容错时间的操作指令
impl PlayerOperation for TinyTimer {
    fn operation_active(&self) -> bool {
        self.in_time()
    }

    fn operation_echo(&mut self) {
        self.final_time();
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert!(true.operation_active());
        assert!(!false.operation_active());

        let mut o = true;
        assert!(o.operation_active());
        o.operation_echo();
        assert!(!o.operation_active());
    }

    #[test]
    fn test_option() {
        assert!(!None::<bool>.operation_active());
        assert!(!Some(false).operation_active());
        assert!(Some(true).operation_active());

        let mut o = Some(true);
        assert!(o.operation_active());
        o.operation_echo();
        assert!(!o.operation_active());
    }

    #[test]
    fn test_f64() {
        assert!(!0.000001.operation_active());
        assert!(0.1.operation_active());

        let mut o = 0.1;
        assert!(o.operation_active());
        o.operation_echo();
        assert!(!o.operation_active());
    }

    #[test]
    fn test_timer() {
        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.operation_active());
        timer.add_time(0.5);
        assert!(timer.operation_active());
        timer.add_time(0.5);
        assert!(!timer.operation_active());
        timer.add_time(0.5);
        assert!(!timer.operation_active());

        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.operation_active());
        timer.add_time(0.5);
        assert!(timer.operation_active());
        timer.final_time();
        assert!(!timer.operation_active());

        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.operation_active());
        timer.add_time(0.5);
        assert!(timer.operation_active());
        timer.operation_echo();
        assert!(!timer.operation_active());
    }
}
