use crate::movements::action::ActionExitLogic;

/// 动作的触发（指令 Instruction or 信号 Signal）
///
/// 若有必要可以和 [`MovementMode`] 组合去实现 ActionEvent 来通过触发条件判断运动状态
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

/// 动作的退出逻辑
/// 
/// 若有必要可以包含 [`MovementMode`] 来实现运动状态切换导致动作退出
#[derive(Clone, Copy, Debug)]
pub enum ActionBaseExitLogic {
    /// 动画结束播放
    AnimFinished,
    /// 多长时间后，移动可取消后摇
    WantMove(f64),
    /// 多长时间后，跳跃可打断
    WantJump(f64),
}

impl ActionExitLogic<bool> for ActionBaseExitLogic {
    fn should_exit(p: &bool) -> bool {
        *p
    }
}
