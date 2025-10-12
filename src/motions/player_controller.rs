//! 玩家角色控制器 实现了指令预输入功能
//!
//! 代码检视
//! - [x] （已通过互相实现 [`From`] 来保证）结构体 [`PlayerController`] 中的所有字段在转换至 [`PlayerInstructionCollection`] 时都应存在对应映射
//! - [ ] （新增字段后有编译错误，修复时注意即可） [`TinyTimer`] 类型的字段在 [`PlayerController::op_echo_with`] 和 [`PlayerInstructionCollection::op_echo_with`] 中都应存在
//! - [ ] （新增字段后有编译错误，修复时注意即可）结构体 [`PlayerInstructionCollection`] 中的字段语义与覆写函数逻辑一致 [`PlayerInstructionCollection::overwrite_with`]
//! - [ ] 结构体 [`PlayerInstructionCollection`] 中的【大部分】字段在 [`PlayerInstructionCollection::push_instruction`] 中转换成事件

use crate::{
    cores::tiny_timer::TinyTimer,
    motions::{
        abstracts::{
            player_input::{PlayerInstruction, PlayerOperation},
            player_pre_input::{PreInputInstruction, PreInputOperation},
        }, motion_action::ActionBaseEvent,
    },
};

/// 玩家控制器 实例化后对应一个玩家（本地或远端）
///
/// 其属性都是玩家操作 [`PlayerOperation`] 或 [`PreInputOperation`] （注：编码时由于没有引用所以没法跳转）
#[derive(Debug, Default)]
pub struct PlayerController {
    // pub(crate) look_angle: f64,
    pub(crate) move_direction: f64,

    pub(crate) jump_once: TinyTimer,
    pub(crate) jump_keep: bool,

    pub(crate) dodge_once: TinyTimer,

    pub(crate) block_hold: bool,

    pub(crate) attack_once: bool,
    pub(crate) attack_keep: bool,
}

/// 玩家指令 由玩家控制器直接转换而来
///
/// 支持持久化为状态后逐帧叠加刷新（由于不同状态的指令初始化时机不同，由调用方主动维护，如动作状态机内部、每个行为状态内部）
///
/// 字段语义：
/// - hold 按键处于按下状态
/// - once 按键刚刚被按下（一帧内）
/// - keep 按键按下后从未松开（指令初始化之后）
#[derive(Clone, Debug, Default)]
pub struct PlayerInstructionCollection {
    // pub(crate) look_angle: PlayerInstruction<f64>,
    pub(crate) move_direction: PlayerInstruction<f64>,
    pub(crate) jump_once: PreInputInstruction<TinyTimer>,
    pub(crate) jump_keep: PlayerInstruction<bool>,
    pub(crate) dodge_once: PreInputInstruction<TinyTimer>,
    pub(crate) block_hold: PlayerInstruction<bool>,
    pub(crate) attack_once: PlayerInstruction<bool>,
    pub(crate) attack_keep: PlayerInstruction<bool>,
}

impl From<PlayerInstructionCollection> for PlayerController {
    /// 没有用 仅仅为了强制要求两者字段一一对应
    fn from(value: PlayerInstructionCollection) -> Self {
        Self {
            move_direction: value.move_direction.0,
            jump_once: TinyTimer::new(0.0),
            jump_keep: value.jump_keep.0,
            dodge_once: TinyTimer::new(0.0),
            block_hold: value.block_hold.0,
            attack_once: value.attack_once.0,
            attack_keep: value.attack_keep.0,
        }
    }
}

impl From<&PlayerController> for PlayerInstructionCollection {
    /// 将玩家控制输入转换为指令
    fn from(value: &PlayerController) -> Self {
        Self {
            // look_angle: value.look_angle.into(),
            move_direction: value.move_direction.into(),
            jump_once: (&value.jump_once).into(),
            jump_keep: value.jump_keep.into(),
            dodge_once: (&value.dodge_once).into(),
            block_hold: value.block_hold.into(),
            attack_once: value.attack_once.into(),
            attack_keep: value.attack_keep.into(),
        }
    }
}

impl PlayerController {
    /// 对于 [`TinyTimer`] 类型的字段，由于其具备帧残留的副作用，因此需要手动回响关闭
    pub fn op_echo_with(&mut self, other: &PlayerInstructionCollection) {
        *self = Self {
            move_direction: self.move_direction,
            jump_once: self.jump_once.op_echo_with_pure(&other.jump_once),
            jump_keep: self.jump_keep,
            dodge_once: self.dodge_once.op_echo_with_pure(&other.dodge_once),
            block_hold: self.block_hold,
            attack_once: self.attack_once,
            attack_keep: self.attack_keep,
        }
    }
}

impl PlayerInstructionCollection {
    /// 对于 [`TinyTimer`] 类型的字段，由于其具备帧残留的副作用，因此需要手动回响关闭
    pub fn op_echo_with(&mut self, other: &Self) {
        *self = Self {
            move_direction: self.move_direction,
            jump_once: self.jump_once.op_echo_with_pure(&other.jump_once),
            jump_keep: self.jump_keep,
            dodge_once: self.dodge_once.op_echo_with_pure(&other.dodge_once),
            block_hold: self.block_hold,
            attack_once: self.attack_once,
            attack_keep: self.attack_keep,
        }
    }

    /// 覆写 用于持久化指令的状态刷新
    ///
    /// 修改时确认是否同步修改 [`PlayerInstructionCollection::push_instruction`]
    pub fn overwrite_with(&mut self, other: &Self) {
        *self = Self {
            move_direction: other.move_direction,
            jump_once: other.jump_once.clone(),
            jump_keep: PlayerInstruction::from(self.jump_keep.0 && other.jump_keep.0),
            dodge_once: other.dodge_once.clone(),
            block_hold: other.block_hold,
            attack_once: other.attack_once,
            attack_keep: PlayerInstruction::from(self.attack_keep.0 && other.attack_keep.0),
        }
    }

    /// 将对应指令映射成事件推入列表
    pub fn push_instruction(&self, list: &mut Vec<ActionBaseEvent>) {
        if self.jump_once.op_active() {
            list.push(ActionBaseEvent::JumpInstruction);
        }
        if self.jump_keep.op_active() {
            list.push(ActionBaseEvent::JumpHigherInstruction);
        }
        if self.dodge_once.op_active() {
            list.push(ActionBaseEvent::DodgeInstruction);
        }
        if self.block_hold.op_active() {
            list.push(ActionBaseEvent::BlockInstruction);
        }
        if self.attack_once.op_active() {
            list.push(ActionBaseEvent::AttackInstruction);
        }
        if self.attack_keep.op_active() {
            list.push(ActionBaseEvent::AttackHeavierInstruction);
        }
    }
}

/// just for test
pub fn instructions_all_active() -> PlayerInstructionCollection {
    PlayerInstructionCollection {
        move_direction: PlayerInstruction::from(1.0),
        jump_once: PreInputInstruction(true, Default::default()),
        jump_keep: PlayerInstruction::from(true),
        dodge_once: PreInputInstruction(true, Default::default()),
        block_hold: PlayerInstruction::from(true),
        attack_once: PlayerInstruction::from(true),
        attack_keep: PlayerInstruction::from(true),
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn overwrite_with() {
        let mut old_instructions = PlayerInstructionCollection {
            move_direction: PlayerInstruction::from(1.0),
            jump_once: PreInputInstruction(false, Default::default()),
            jump_keep: PlayerInstruction::from(false),
            dodge_once: PreInputInstruction(false, Default::default()),
            block_hold: PlayerInstruction::from(false),
            attack_once: PlayerInstruction::from(false),
            attack_keep: PlayerInstruction::from(false),
        };

        let new_instructions = instructions_all_active();
        old_instructions.overwrite_with(&new_instructions);

        // hold
        assert!(old_instructions.block_hold.op_active());

        // once
        assert!(old_instructions.jump_once.op_active());
        assert!(old_instructions.dodge_once.op_active());
        assert!(old_instructions.attack_once.op_active());

        // keep not active
        assert!(!old_instructions.jump_keep.op_active());
        assert!(!old_instructions.attack_keep.op_active());
    }
}
