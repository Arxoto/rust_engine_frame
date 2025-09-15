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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionExitLogic {}

/// Instruction or Signal
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionTrigger {}

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
}
