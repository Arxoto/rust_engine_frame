//! 动作系统的【事件】和【退出逻辑】实现案例

use crate::motion::action_types::ActionEvent;

/// 动作的触发（指令 Instruction or 信号 Signal）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionBaseEvent {
    /// 跳跃指令
    JumpInstruction,
    /// 闪避指令
    DodgeInstruction,
    /// 攻击指令
    AttackInstruction,
    /// 防御指令
    DefenceInstruction,

    /// 命中对方
    HitSignal,
    /// 被命中
    BeHitSignal,
}

impl ActionEvent for ActionBaseEvent {}

/// 动作的退出逻辑
#[derive(Clone, Copy, Debug)]
pub enum ActionBaseExitLogic {
    /// 动画结束播放
    AnimFinished,
    /// 多长时间后，移动可取消后摇
    WantMove(f64),
    /// 多长时间后，跳跃可打断
    WantJump(f64),
}
