use std::collections::HashSet;

/// 运动模式
///
/// 每一套运动模式都应实现对应行为 用于动作之外的默认效果
///
/// 行为用于简单操作，如基础移动；动作用于复杂操作，如 combo
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MovementMode {
    OnFloor,
    InAir,
    UnderWater,
    ClimbWall,
}

impl MovementMode {
    pub fn gen_set() -> HashSet<MovementMode> {
        let mut the_set = HashSet::new();
        the_set.insert(MovementMode::OnFloor);
        the_set.insert(MovementMode::InAir);
        the_set.insert(MovementMode::UnderWater);
        the_set.insert(MovementMode::ClimbWall);
        the_set
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ActionExitLogic {
    /// 动画结束播放
    AnimFinished,
    /// 移动 多长时间后才可取消后摇
    WantMove(f64),
}

impl ActionExitLogic {
    pub fn gen_list() -> &'static [ActionExitLogic] {
        &[ActionExitLogic::AnimFinished]
    }
}

/// Instruction or Signal
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionTrigger {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 什么都不做 仅用于确保生成的集合附带全量类型
    #[test]
    fn test_movement_set() {
        let mut count = 0;
        let the_set = MovementMode::gen_set();
        for ele in the_set.iter() {
            match ele {
                MovementMode::OnFloor => count += 1,
                MovementMode::InAir => count += 1,
                MovementMode::UnderWater => count += 1,
                MovementMode::ClimbWall => count += 1,
            }
        }
        assert_eq!(count, the_set.iter().count());
    }

    /// 什么都不做 仅用于确保生成的集合附带全量类型
    #[test]
    fn test_exit_logic() {
        let mut count = 0;
        let exit_logic_list = ActionExitLogic::gen_list();
        for ele in exit_logic_list.iter() {
            match ele {
                ActionExitLogic::AnimFinished => count += 1,
                ActionExitLogic::WantMove(_) => count += 1,
            }
        }

        assert_eq!(count, exit_logic_list.iter().count());
    }
}
