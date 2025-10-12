//! 运动模式的枚举定义

use crate::{cores::unify_type::FixedString, motions::state_machine_phy_param::PhyParam};

/// 运动模式
///
/// 每一套运动模式都应实现对应行为（但行为可以是运动模式的超集） 用于动作之外的默认效果
///
/// - 行为用于基础操作、复杂逻辑，如带容错时间的移动跳跃
/// - 动作用于复杂操作、简单逻辑，如 combo 的依赖路径
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MotionMode {
    /// 特殊状态 自由移动 一般用于测试
    FreeStat,
    /// 特殊状态 静止 仅初始状态用
    Motionless,
    // 具体的运动模式
    OnFloor,
    InAir,
    UnderWater,
    ClimbWall,
}

impl<S: FixedString> From<&PhyParam<S>> for MotionMode {
    fn from(value: &PhyParam<S>) -> Self {
        // 特殊状态
        if value.behaviour_to_free {
            return Self::FreeStat;
        }

        // 具体运动模式
        if value.character_should_climb {
            // 判断条件需要用到向量运算 为保证项目纯净 交由外部判断输入
            return Self::ClimbWall;
        }
        // 基础运动模式
        if value.character_is_on_floor {
            Self::OnFloor
        } else {
            Self::InAir
        }
    }
}

impl MotionMode {
    pub fn each_mode() -> &'static [MotionMode] {
        &[
            MotionMode::FreeStat,
            MotionMode::Motionless,
            MotionMode::OnFloor,
            MotionMode::InAir,
            MotionMode::UnderWater,
            MotionMode::ClimbWall,
        ]
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    /// 该单测无法确保，仅提示 [`MotionMode::each_mode`] 应该返回全量枚举值
    #[test]
    fn test_all_enum_variants_are_returned() {
        let motions = MotionMode::each_mode();

        // 通过宏获取行数
        // 由于 match 语句强制要求补全 因此每次新增枚举值会同时新增行数
        // 以此来强制要求方法返回的枚举值是全量的
        let line_start = line!();
        for ele in motions.iter() {
            match ele {
                MotionMode::FreeStat => {}
                MotionMode::Motionless => {}
                MotionMode::OnFloor => {}
                MotionMode::InAir => {}
                MotionMode::UnderWater => {}
                MotionMode::ClimbWall => {}
            }
        }
        let line_final = line!();
        // 其他语句所占行数
        const LINE_MID: u32 = 5;
        // 枚举匹配的行数 应与个数相等
        let enum_count: usize = (line_final - line_start - LINE_MID).try_into().unwrap();
        assert_eq!(motions.len(), enum_count);
    }
}
