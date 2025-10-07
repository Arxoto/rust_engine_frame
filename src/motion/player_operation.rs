use crate::cores::tiny_timer::TinyTimer;

/// 玩家操作 直接对应玩家意图
pub trait PlayerOperation {
    /// 指令是否应该激活对应操作
    fn op_active(&self) -> bool;
    /// 操作成功时触发回显 取消对应操作（预输入时用）
    fn op_echo(&mut self);
    /// 若相同属性已触发回显 则也取消对应操作
    fn op_echo_with<T: PlayerOperation>(&mut self, value: &T) {
        if !value.op_active() {
            self.op_echo();
        }
    }
}

impl PlayerOperation for bool {
    fn op_active(&self) -> bool {
        *self
    }

    fn op_echo(&mut self) {
        *self = false;
    }
}

impl<T: PlayerOperation + Clone> PlayerOperation for Option<T> {
    fn op_active(&self) -> bool {
        self.clone().is_some_and(|o| o.op_active())
    }

    fn op_echo(&mut self) {
        *self = None;
    }
}

/// 对于 f64 容错值一般使用 1e-9 （防止加减乘除累积误差） 但是这里用作输入 没必要那么精确
const DEAD_ZONE: f64 = 1e-4;

// for direction or angle
impl PlayerOperation for f64 {
    fn op_active(&self) -> bool {
        self.abs() > DEAD_ZONE
    }

    fn op_echo(&mut self) {
        *self = 0.0;
    }
}

// 有容错时间的操作指令
impl PlayerOperation for TinyTimer {
    fn op_active(&self) -> bool {
        self.in_time()
    }

    fn op_echo(&mut self) {
        self.final_time();
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert!(true.op_active());
        assert!(!false.op_active());

        let mut o = true;
        assert!(o.op_active());
        o.op_echo();
        assert!(!o.op_active());
    }

    #[test]
    fn test_option() {
        assert!(!None::<bool>.op_active());
        assert!(!Some(false).op_active());
        assert!(Some(true).op_active());

        let mut o = Some(true);
        assert!(o.op_active());
        o.op_echo();
        assert!(!o.op_active());
    }

    #[test]
    fn test_f64() {
        assert!(!0.0000_0000_1.op_active());
        assert!(0.1.op_active());

        let mut o = 0.1;
        assert!(o.op_active());
        o.op_echo();
        assert!(!o.op_active());
    }

    #[test]
    fn test_timer() {
        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.op_active());
        timer.add_time(0.5);
        assert!(timer.op_active());
        timer.add_time(0.5);
        assert!(!timer.op_active());
        timer.add_time(0.5);
        assert!(!timer.op_active());

        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.op_active());
        timer.add_time(0.5);
        assert!(timer.op_active());
        timer.final_time();
        assert!(!timer.op_active());

        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.op_active());
        timer.add_time(0.5);
        assert!(timer.op_active());
        timer.op_echo();
        assert!(!timer.op_active());
    }
}
