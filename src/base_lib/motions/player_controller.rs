//! 玩家角色控制器
//!
//! 给一个基础的 2D 操作控制实现

use crate::base_lib::{
    cores::unify_types::time_type,
    motions::controllers::{
        AbstractInstruction, ActiveInput, InputOperation, InstructionBufferedJustOn,
        InstructionStateOn,
    },
};

/// 玩家操作输入，对应按键、摇杆等原始输入，表达玩家意图
///
/// 每帧都会去获取生成
pub struct PlayerInput {
    /// up_down
    pub look_angle: InputOperation<f64>,
    /// left_right
    pub move_direction: InputOperation<f64>,
    /// 按键 J
    pub attack_btn: InputOperation<bool>,
    /// 按键 I
    pub block_btn: InputOperation<bool>,
    /// 按键 K
    pub jump_btn: InputOperation<bool>,
    /// 按键 L
    pub dodge_btn: InputOperation<bool>,
}

/// 玩家角色控制器，从操作意图翻译而来的控制指令
///
/// 长期存在，每帧更新
pub struct PlayerCharacterController {
    look_angle: InputOperation<f64>,
    move_direction: InputOperation<f64>,

    attack_just_down: InstructionBufferedJustOn,
    attack_hold_down: InstructionStateOn,

    block_just_down: InstructionBufferedJustOn,
    block_hold_down: InstructionStateOn,

    jump_just_down: InstructionBufferedJustOn,
    jump_hold_down: InstructionStateOn,

    dodge_just_down: InstructionBufferedJustOn,
    dodge_hold_down: InstructionStateOn,
}

impl PlayerCharacterController {
    /// 公开构造器：基于玩家输入的初始状态
    ///
    /// `input_buffer_window` 为预输入缓冲时长（键缓冲窗口），
    /// 参考 [`InstructionBufferedJustOn::new`]。
    pub fn new(player_input: PlayerInput, input_buffer_window: time_type::T) -> Self {
        Self {
            look_angle: player_input.look_angle,
            move_direction: player_input.move_direction,
            attack_just_down: InstructionBufferedJustOn::new(input_buffer_window),
            attack_hold_down: InstructionStateOn::new(),
            block_just_down: InstructionBufferedJustOn::new(input_buffer_window),
            block_hold_down: InstructionStateOn::new(),
            jump_just_down: InstructionBufferedJustOn::new(input_buffer_window),
            jump_hold_down: InstructionStateOn::new(),
            dodge_just_down: InstructionBufferedJustOn::new(input_buffer_window),
            dodge_hold_down: InstructionStateOn::new(),
        }
    }

    // region: 只读访问 翻译后的控制信号

    pub fn attack_just_down(&self) -> bool {
        self.attack_just_down.is_on()
    }

    pub fn attack_hold_down(&self) -> bool {
        self.attack_hold_down.is_on()
    }

    pub fn block_just_down(&self) -> bool {
        self.block_just_down.is_on()
    }

    pub fn block_hold_down(&self) -> bool {
        self.block_hold_down.is_on()
    }

    pub fn jump_just_down(&self) -> bool {
        self.jump_just_down.is_on()
    }

    pub fn jump_hold_down(&self) -> bool {
        self.jump_hold_down.is_on()
    }

    pub fn dodge_just_down(&self) -> bool {
        self.dodge_just_down.is_on()
    }

    pub fn dodge_hold_down(&self) -> bool {
        self.dodge_hold_down.is_on()
    }

    // endregion

    pub fn update(&mut self, player_input: PlayerInput) {
        self.look_angle = player_input.look_angle;
        self.move_direction = player_input.move_direction;

        self.attack_just_down.update_by_op(&player_input.attack_btn);
        self.attack_hold_down.update_by_op(&player_input.attack_btn);

        self.block_just_down.update_by_op(&player_input.block_btn);
        self.block_hold_down.update_by_op(&player_input.block_btn);

        self.jump_just_down.update_by_op(&player_input.jump_btn);
        self.jump_hold_down.update_by_op(&player_input.jump_btn);

        self.dodge_just_down.update_by_op(&player_input.dodge_btn);
        self.dodge_hold_down.update_by_op(&player_input.dodge_btn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::cores::unify_types::time_type;

    fn make_input(attack: bool, block: bool, jump: bool, dodge: bool) -> PlayerInput {
        PlayerInput {
            look_angle: InputOperation::new(1.0),
            move_direction: InputOperation::new(0.5),
            attack_btn: InputOperation::new(attack),
            block_btn: InputOperation::new(block),
            jump_btn: InputOperation::new(jump),
            dodge_btn: InputOperation::new(dodge),
        }
    }

    /// 等级 A 演示：构造 → update → 观察翻译结果（覆盖全部 8 个只读访问）
    #[test]
    fn construct_update_translates_input() {
        let mut controller = PlayerCharacterController::new(
            make_input(false, false, false, false),
            time_type::unit::<4>(),
        );

        // 第一帧：四键按下 → just_down 与 hold 都开启
        controller.update(make_input(true, true, true, true));
        assert!(controller.attack_just_down());
        assert!(controller.attack_hold_down());
        assert!(controller.block_just_down());
        assert!(controller.block_hold_down());
        assert!(controller.jump_just_down());
        assert!(controller.jump_hold_down());
        assert!(controller.dodge_just_down());
        assert!(controller.dodge_hold_down());

        // 第二帧：持续按住 → hold 保持
        controller.update(make_input(true, true, true, true));
        assert!(controller.attack_hold_down());
        assert!(controller.block_hold_down());
        assert!(controller.jump_hold_down());
        assert!(controller.dodge_hold_down());

        // 第三帧：全部松开 → hold 关闭
        controller.update(make_input(false, false, false, false));
        assert!(!controller.attack_hold_down());
        assert!(!controller.block_hold_down());
        assert!(!controller.jump_hold_down());
        assert!(!controller.dodge_hold_down());
    }
}
