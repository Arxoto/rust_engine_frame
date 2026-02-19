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
        },
        motion_action::ActionBaseEvent,
    },
};

/// 玩家控制器 对应玩家的原始输入（如按键、摇杆）
///
/// 每帧根据玩家操作实时生成（客户端）
///
/// 其属性直接对应玩家的直接操作结果
pub struct PlayerController {
    pub look_angle: f64,
    pub move_direction: f64,

    pub jump_hold: bool,
    pub dodge_hold: bool,
    pub block_hold: bool,
    pub attack_hold: bool,
}

/// 玩家操作集 对应某个玩家的行为意图（预输入冗余）
///
/// 实例化后长期存在在【客户端】 每帧根据玩家控制器更新
///
/// 其属性都是玩家操作 [`PlayerOperation`] 或 [`PreInputOperation`] todo 类型封装 不要直接实现
#[derive(Debug, Default)]
pub struct PlayerOperationCollection {
    pub(crate) look_angle: f64,
    pub(crate) move_direction: f64,

    pub(crate) jump_once: TinyTimer,
    pub(crate) jump_hold: bool,

    pub(crate) dodge_once: TinyTimer,
    pub(crate) dodge_hold: bool,

    pub(crate) block_once: TinyTimer,
    pub(crate) block_hold: bool,

    pub(crate) attack_once: TinyTimer,
    pub(crate) attack_hold: bool,
}

/// 玩家指令集 对应某个玩家角色的控制指令（与角色状态强相关）
///
/// 实例化后长期存在在【服务端】 客户端中每帧根据玩家操作集生成指令集中间态 服务端中每帧更新指令集最终态（状态机可能会改变或重置指令、连接超时应保持不变）
///
/// 其属性都是玩家操作 [`PlayerInstruction`] 或 [`PreInputInstruction`]
///
/// 字段语义：
/// - once 按键刚刚被按下（一帧内/预输入冗余）
/// - keep 按键按下后从未松开（指令初始化之后）
/// - hold 按键处于按下状态
#[derive(Clone, Debug, Default)]
pub struct PlayerInstructionCollection {
    pub(crate) look_angle: PlayerInstruction<f64>,
    pub(crate) move_direction: PlayerInstruction<f64>,

    pub(crate) jump_once: PreInputInstruction<TinyTimer>,
    pub(crate) jump_keep: PlayerInstruction<bool>,
    pub(crate) jump_hold: PlayerInstruction<bool>,

    pub(crate) dodge_once: PreInputInstruction<TinyTimer>,
    pub(crate) dodge_hold: PlayerInstruction<bool>,

    pub(crate) block_once: PreInputInstruction<TinyTimer>,
    pub(crate) block_hold: PlayerInstruction<bool>,

    pub(crate) attack_once: PreInputInstruction<TinyTimer>,
    pub(crate) attack_keep: PlayerInstruction<bool>,
    pub(crate) attack_hold: PlayerInstruction<bool>,
}

/// 专门用于客户端到服务端传输的包装类型，表示中间态，借助强类型系统保证业务逻辑正确性
#[derive(Clone, Debug, Default)]
pub struct PlayerInstructionCollectionRaw(pub PlayerInstructionCollection);

/// 专门用于服务端到客户端传输的包装类型，表示最终态，借助强类型系统保证业务逻辑正确性
#[derive(Clone, Debug, Default)]
pub struct PlayerInstructionCollectionFinal(pub PlayerInstructionCollection);

impl From<PlayerInstructionCollection> for PlayerOperationCollection {
    /// 没有用 仅仅为了强制要求两者字段一一对应
    ///
    /// 注意：里面的字段 TinyTimer 都没有启动
    fn from(value: PlayerInstructionCollection) -> Self {
        Self {
            look_angle: value.look_angle.0,
            move_direction: value.move_direction.0,

            jump_once: TinyTimer::new(0.0),
            jump_hold: value.jump_hold.0,

            dodge_once: TinyTimer::new(0.0),
            dodge_hold: value.dodge_hold.0,

            block_once: TinyTimer::new(0.0),
            block_hold: value.block_hold.0,

            attack_once: TinyTimer::new(0.0),
            attack_hold: value.attack_hold.0,
        }
    }
}

impl From<&PlayerOperationCollection> for PlayerInstructionCollectionRaw {
    /// 将玩家控制输入转换为指令
    ///
    /// 注意 keep 字段的值直接取自 hold 字段 这仅仅是用于客户端服务端传输的中间值 后续应与实例化的变量聚合得到最终值
    fn from(value: &PlayerOperationCollection) -> Self {
        Self(PlayerInstructionCollection {
            look_angle: value.look_angle.into(),
            move_direction: value.move_direction.into(),

            jump_once: PreInputInstruction::from(&value.jump_once),
            jump_keep: value.jump_hold.into(),
            jump_hold: value.jump_hold.into(),

            dodge_once: PreInputInstruction::from(&value.dodge_once),
            dodge_hold: value.dodge_hold.into(),

            block_once: PreInputInstruction::from(&value.block_once),
            block_hold: value.block_hold.into(),

            attack_once: PreInputInstruction::from(&value.attack_once),
            attack_keep: value.attack_hold.into(),
            attack_hold: value.attack_hold.into(),
        })
    }
}

impl PlayerOperationCollection {
    /// 按下则激活/重置时间 否则保持原样
    #[inline]
    fn fix_once_by_ctrl<T>(mut once_value: T, controller_value: bool) -> T
    where
        T: PreInputOperation,
    {
        if controller_value {
            once_value.op_do_reactivate();
        }
        once_value
    }

    /// 根据玩家控制器更新操作集
    ///
    /// 其中 once 字段因为具备预输入能力 因此是有状态的 需要更新
    ///
    /// P.S. 为了不必要的 clone 获取了所有权
    pub fn op_update_by_controller(self, controller: &PlayerController) -> Self {
        Self {
            look_angle: controller.look_angle,
            move_direction: controller.move_direction,

            jump_once: Self::fix_once_by_ctrl(self.jump_once, controller.jump_hold),
            jump_hold: controller.jump_hold,

            dodge_once: Self::fix_once_by_ctrl(self.dodge_once, controller.dodge_hold),
            dodge_hold: controller.dodge_hold,

            block_once: Self::fix_once_by_ctrl(self.block_once, controller.block_hold),
            block_hold: controller.block_hold,

            attack_once: Self::fix_once_by_ctrl(self.attack_once, controller.attack_hold),
            attack_hold: controller.attack_hold,
        }
    }

    #[inline]
    fn fix_once_by_inst<T>(mut once_value: T, instruction_value: PreInputInstruction<T>) -> T
    where
        T: PreInputOperation,
    {
        once_value.op_update(&instruction_value);
        once_value
    }

    /// 根据服务端的指令集结果更新客户端的操作集状态（操作成功取消预输入）
    ///
    /// 对 once 字段更新状态 其他字段不变（会在下一帧直接覆盖）
    ///
    /// P.S. 为了不必要的 clone 获取了所有权
    pub fn op_update_by_instruction(
        self,
        instruction_final: &PlayerInstructionCollectionFinal,
    ) -> Self {
        let instruction = &instruction_final.0;
        Self {
            look_angle: self.look_angle,
            move_direction: self.move_direction,

            jump_once: Self::fix_once_by_inst(self.jump_once, instruction.jump_once.clone()),
            jump_hold: self.jump_hold,

            dodge_once: Self::fix_once_by_inst(self.dodge_once, instruction.dodge_once.clone()),
            dodge_hold: self.dodge_hold,

            block_once: Self::fix_once_by_inst(self.block_once, instruction.block_once.clone()),
            block_hold: self.block_hold,

            attack_once: Self::fix_once_by_inst(self.attack_once, instruction.attack_once.clone()),
            attack_hold: self.attack_hold,
        }
    }
}

impl PlayerInstructionCollection {
    /// 在服务端根据客户端的中间态指令集更新最终态指令集（连接超时应保持不变）
    ///
    /// 其中 keep 字段因为状态管理在服务端 因此在这里更新
    pub fn op_update(&mut self, raw_value: &PlayerInstructionCollectionRaw) {
        let other = &raw_value.0;
        *self = Self {
            look_angle: other.look_angle,
            move_direction: other.move_direction,

            jump_once: other.jump_once.clone(),
            jump_keep: PlayerInstruction::from(self.jump_keep.0 && other.jump_keep.0),
            jump_hold: other.jump_hold,

            dodge_once: other.dodge_once.clone(),
            dodge_hold: other.dodge_hold,

            block_once: other.block_once.clone(),
            block_hold: other.block_hold,

            attack_once: other.attack_once.clone(),
            attack_keep: PlayerInstruction::from(self.attack_keep.0 && other.attack_keep.0),
            attack_hold: other.attack_hold,
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
        if self.jump_hold.op_active() {
            list.push(ActionBaseEvent::JumpFlyInstruction);
        }

        if self.dodge_once.op_active() {
            list.push(ActionBaseEvent::DodgeInstruction);
        }
        if self.dodge_hold.op_active() {
            list.push(ActionBaseEvent::DodgeDashInstruction);
        }

        if self.block_once.op_active() {
            list.push(ActionBaseEvent::BlockOnceInstruction);
        }
        if self.block_hold.op_active() {
            list.push(ActionBaseEvent::BlockHoldInstruction);
        }

        if self.attack_once.op_active() {
            list.push(ActionBaseEvent::AttackInstruction);
        }
        if self.attack_keep.op_active() {
            list.push(ActionBaseEvent::AttackHeavierInstruction);
        }
        if self.attack_hold.op_active() {
            list.push(ActionBaseEvent::AttackChargeInstruction);
        }
    }
}

#[cfg(test)]
pub(crate) mod unit_tests {
    use super::*;

    /// just for test
    pub(crate) fn instructions_all_active() -> (PlayerInstructionCollection, u32) {
        let line_start = line!();
        let pic = PlayerInstructionCollection {
            look_angle: PlayerInstruction::from(1.0),
            move_direction: PlayerInstruction::from(1.0),

            jump_once: PreInputInstruction(true, Default::default()),
            jump_keep: PlayerInstruction::from(true),
            jump_hold: PlayerInstruction::from(true),

            dodge_once: PreInputInstruction(true, Default::default()),
            dodge_hold: PlayerInstruction::from(true),

            block_once: PreInputInstruction(true, Default::default()),
            block_hold: PlayerInstruction::from(true),

            attack_once: PreInputInstruction(true, Default::default()),
            attack_keep: PlayerInstruction::from(true),
            attack_hold: PlayerInstruction::from(true),
        };
        let line_final = line!();
        (pic, line_final - line_start - 3 - 4) // 3 是正常语句占用 4 是空行
    }

    #[test]
    fn overwrite_with() {
        let mut old_instructions = PlayerInstructionCollection {
            look_angle: PlayerInstruction::from(1.0),
            move_direction: PlayerInstruction::from(1.0),

            jump_once: PreInputInstruction(false, Default::default()),
            jump_keep: PlayerInstruction::from(false),
            jump_hold: PlayerInstruction::from(false),

            dodge_once: PreInputInstruction(false, Default::default()),
            dodge_hold: PlayerInstruction::from(false),

            block_once: PreInputInstruction(false, Default::default()),
            block_hold: PlayerInstruction::from(false),

            attack_once: PreInputInstruction(false, Default::default()),
            attack_keep: PlayerInstruction::from(false),
            attack_hold: PlayerInstruction::from(false),
        };

        let (new_instructions, _) = instructions_all_active();
        old_instructions.op_update(&PlayerInstructionCollectionRaw(new_instructions));

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

    #[test]
    fn push_instruction() {
        let (instructions, member_num) = instructions_all_active();
        let mut list = Vec::new();
        instructions.push_instruction(&mut list);
        // 保证所有的指令都被转换成事件推入列表
        assert_eq!(list.len() + 2, member_num.try_into().unwrap()); // 2 是 look_angle & move_direction 不参与事件因此手动补齐
    }
}
