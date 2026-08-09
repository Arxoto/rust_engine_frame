//! 玩家角色控制器
//!
//! 给一个基础的 2D 操作控制实现

use crate::base_lib::motions::controllers::{
    AbstractInstruction, InputOperation, InstructionBufferedJustOn, InstructionStateOn,
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
