use crate::{cores::tiny_timer::TinyTimer, motion::player_input::PlayerOperation};

/// 玩家操作 【增强】 预输入（键缓冲）
///
/// 该类型属性一般持久化存在，根据玩家输入实时修改（因此也能实现预输入）
pub trait PreInputOperation: PlayerOperation {
    /// 操作成功时触发回显 取消对应操作（预输入时用）
    fn op_echo(&mut self);

    /// 若映射属性已触发回显 则也取消对应操作
    fn op_echo_with<T: PreInputOperation>(&mut self, value: &T) {
        if !value.op_active() {
            self.op_echo();
        }
    }

    /// 集成方法 消费获得是否激活
    ///
    /// 返回是否为【激活态】后 自动触发回显
    ///
    /// 在行为系统中主要用到
    fn op_consume_active(&mut self) -> bool {
        let b = self.op_active();
        self.op_echo();
        b
    }
}

/// 带预输入功能的操作 在其激活期间反复发送指令 直至指令下发成功
///
/// 该类型属性生命周期在一帧内，每次根据玩家操作临时生成（指令响应后及时反馈给玩家操作）
#[derive(Clone, Default)]
pub struct PreInputInstruction<T: PreInputOperation>(
    pub(crate) bool,
    pub(crate) std::marker::PhantomData<T>,
);

impl<T: PreInputOperation> PlayerOperation for PreInputInstruction<T> {
    fn op_active(&self) -> bool {
        self.0.op_active()
    }
}

impl<T: PreInputOperation> PreInputOperation for PreInputInstruction<T> {
    fn op_echo(&mut self) {
        self.0 = false;
    }
}

impl<T: PreInputOperation> From<&T> for PreInputInstruction<T> {
    fn from(value: &T) -> Self {
        Self(value.op_active(), Default::default())
    }
}

// ===========================
// impl
// ===========================

// 有容错时间的操作指令
impl PlayerOperation for TinyTimer {
    fn op_active(&self) -> bool {
        self.in_time()
    }
}

impl PreInputOperation for TinyTimer {
    fn op_echo(&mut self) {
        self.final_time();
    }
}

// ===========================
// test
// ===========================

#[cfg(test)]
mod unit_tests {
    use super::*;

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

    #[test]
    fn test_timer_echo_with() {
        let mut timer = TinyTimer::new(1.0);
        timer.start_time();
        assert!(timer.op_active());
        timer.add_time(0.5);
        assert!(timer.op_active());

        let mut timer_ins: PreInputInstruction<TinyTimer> = (&timer).into();
        assert!(timer_ins.op_active());
        assert!(timer_ins.op_consume_active()); // 消费完成后 激活态改变
        assert!(!timer_ins.op_active());
        timer.op_echo_with(&timer_ins);
        assert!(!timer.op_active());
    }
}
