//! 玩家操作输入和控制指令转义，支持预输入或称键缓冲
//!
//! - Operation 操作，对应玩家意图
//! - Instruction 指令，对应角色控制，从意图中翻译而来

use crate::base_lib::cores::{
    tick_timer::TickTimerFinite,
    tiny_timer::{FlowingTimer, FlowingTimerReadonly, TickTimer},
};

/// 操作 直接对应玩家意图
pub struct InputOperation<T>(T);

/// 指令 直接控制玩家角色 按键刚刚被按下，严格前一帧刚刚按键按下
pub struct InstructionStrictJustOn(bool);

/// 指令 直接控制玩家角色 按键刚刚被按下，预输入缓冲，有容错时间
pub struct InstructionBufferedJustOn(TickTimerFinite, bool);

/// 指令 直接控制玩家角色 按键处于按下状态（仅表示当前状态，不关心是否刚刚被摁下）
pub struct InstructionStateOn(bool);

/// 指令 直接控制玩家角色 按键一直被按下从未释放 不应放在全局控制器中 这里提供实现供状态逻辑中去调用
pub struct InstructionStillKeep(bool);

// region: trait

pub trait ActiveInput {
    /// 输入是否激活
    fn is_on(&self) -> bool;
}

pub trait AbstractInstruction {
    fn update_by_op(&mut self, operation: &impl ActiveInput);
}

// endregion

// region: impl InputOperation

mod private {
    // 密封特质：外部无法访问，因此不能为其他类型实现
    pub trait Sealed {}
}

// 为允许的类型实现密封特质
impl private::Sealed for bool {}
impl private::Sealed for f64 {}

impl<T: private::Sealed> InputOperation<T> {
    pub fn new(t: T) -> Self {
        Self(t)
    }
}

impl ActiveInput for InputOperation<bool> {
    fn is_on(&self) -> bool {
        self.0
    }
}

const DEAD_ZONE: f64 = 1e-4;

impl ActiveInput for InputOperation<f64> {
    fn is_on(&self) -> bool {
        // 死区 防止精度过高导致漂移
        self.0.abs() > DEAD_ZONE
    }
}

// endregion

// region: impl InstructionStrictOnce

impl InstructionStrictJustOn {
    pub fn new() -> Self {
        Self(false)
    }
}

impl ActiveInput for InstructionStrictJustOn {
    fn is_on(&self) -> bool {
        self.0
    }
}

impl AbstractInstruction for InstructionStrictJustOn {
    fn update_by_op(&mut self, operation: &impl ActiveInput) {
        if self.0 {
            // 上一帧是开启状态 无论如何都关闭
            self.0 = false;
        } else {
            // 仅上一帧是关闭状态 这一帧才算是刚刚摁下
            self.0 = operation.is_on();
        }
    }
}

// endregion

// region: InstructionBufferedOnce

impl InstructionBufferedJustOn {
    pub fn new(limit: f64) -> Self {
        let mut new_one = Self(TickTimerFinite::new(limit), false);
        new_one.0.finish();
        new_one
    }

    pub fn consume_instruction(&mut self) {
        self.0.finish();
    }
}

impl ActiveInput for InstructionBufferedJustOn {
    fn is_on(&self) -> bool {
        !self.0.is_finished()
    }
}

impl AbstractInstruction for InstructionBufferedJustOn {
    fn update_by_op(&mut self, operation: &impl ActiveInput) {
        if !self.1 && operation.is_on() {
            // 根据上一帧关闭 这一帧开启 重置时间
            self.0.restart();
        }
        self.1 = operation.is_on();
    }
}

impl TickTimer for InstructionBufferedJustOn {
    fn tick(&mut self, delta: f64) {
        self.0.tick(delta);
    }
}

// endregion

// region: impl InstructionStateOn

impl InstructionStateOn {
    pub fn new() -> Self {
        Self(false)
    }
}

impl ActiveInput for InstructionStateOn {
    fn is_on(&self) -> bool {
        self.0
    }
}

impl AbstractInstruction for InstructionStateOn {
    fn update_by_op(&mut self, operation: &impl ActiveInput) {
        self.0 = operation.is_on()
    }
}

// endregion

// region: InstructionStillKeep

impl InstructionStillKeep {
    pub fn new() -> Self {
        Self(false)
    }

    pub fn reactivate(&mut self) {
        self.0 = true;
    }
}

impl ActiveInput for InstructionStillKeep {
    fn is_on(&self) -> bool {
        self.0
    }
}

impl AbstractInstruction for InstructionStillKeep {
    fn update_by_op(&mut self, operation: &impl ActiveInput) {
        self.0 &= operation.is_on()
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_just_on() {
        let mut instruction = InstructionStrictJustOn::new();
        // 每一帧输入
        let inputs = vec![true, false, false, true, false, true, true];
        // 这一帧第一次摁下
        let answer = vec![true, false, false, true, false, true, false];
        let mut answer_iter = answer.into_iter();
        // update
        for input in inputs {
            let operation = InputOperation::new(input);
            instruction.update_by_op(&operation);
            assert_eq!(answer_iter.next(), Some(instruction.is_on()));
        }
    }

    #[test]
    fn buffered_just_on() {
        // 图形化表示输入动作与预输入缓冲窗口
        // 缓冲和动作的组合，包含“缓冲窗口长”、“动作持续长”、“动作覆盖缓冲窗口”
        const UNIT_TIME: f64 = 1.0; // 单位时间
        const INPUT_BUFFER_WINDOWS: f64 = 4.0; // 预输入缓冲窗口 4 个单位时间
        let input_buf = "+---  +---       +--+--- "; // 输入缓冲 + 表示触发 - 表示持续
        let input_tag = "+++---++++++-----++-++---"; // 输入动作 + 表示摁下 - 表示释放

        // 类型转换方便测试
        let input_buf: Vec<char> = input_buf.chars().collect();
        let input_tag: Vec<char> = input_tag.chars().collect();
        assert_eq!(input_buf.len(), input_tag.len());
        let len = input_buf.len();

        // 自动化测试
        // 这里无论先 tick 还是后 tick 都可以
        for tick_first in [false, true] {
            let mut instruction = InstructionBufferedJustOn::new(INPUT_BUFFER_WINDOWS);
            for i in 0..len {
                let in_buf_window = input_buf[i] != ' ';
                let button_press = input_tag[i] == '+';
                let input_operation = InputOperation::new(button_press);

                if tick_first {
                    instruction.tick(UNIT_TIME);
                }

                instruction.update_by_op(&input_operation);

                assert_eq!(in_buf_window, instruction.is_on(), "at index {i}");

                if !tick_first {
                    instruction.tick(UNIT_TIME);
                }
            }
        }
    }
}
