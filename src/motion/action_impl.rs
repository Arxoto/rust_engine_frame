//! 动作系统的【事件】和【退出逻辑】实现案例

use crate::{cores::unify_type::FixedString, motion::action_types::ActionEvent};

/// 动作的触发
/// - 指令 Instruction
/// - 信号 Signal
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
    ///
    /// - 若想实现【命中后自动连击】或者【命中后才能衔接】，则在接收该信号后转换状态，状态的第一个动画为同名动画，因此不会导致动画提前取消
    HitSignal,
    /// 被命中
    ///
    /// - 若想实现【受击后自动格挡】，则立刻切换状态
    BeHitSignal,
}

impl ActionEvent for ActionBaseEvent {}

/// 动作的退出逻辑
#[derive(Clone, Copy, Debug)]
pub enum ActionBaseExitLogic<S: FixedString> {
    /// 动画结束播放
    AnimFinished(S),
    /// 多长时间后，移动可取消后摇
    MoveAfter(f64),
    /// 多长时间后，跳跃可打断
    JumpAfter(f64),
    /// 当播放动画为S时，攻击可打断
    AttackWhen(S),
}
