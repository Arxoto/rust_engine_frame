//! 运动模式的枚举定义

/// 运动模式
///
/// 每一套运动模式都应实现对应行为 用于动作之外的默认效果
///
/// - 行为用于基础操作、复杂逻辑，如带容错时间的移动跳跃
/// - 动作用于复杂操作、简单逻辑，如 combo 的依赖路径
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MovementMode {
    FreeStat,
    OnFloor,
    InAir,
    UnderWater,
    ClimbWall,
}

impl MovementMode {
    pub fn each_mode() -> &'static [MovementMode] {
        &[
            MovementMode::FreeStat,
            MovementMode::OnFloor,
            MovementMode::InAir,
            MovementMode::UnderWater,
            MovementMode::ClimbWall,
        ]
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    /// 该单测无法确保，仅提示 [`MovementMode::each_mode`] 应该返回全量枚举值
    #[test]
    fn test_all_enum_variants_are_returned() {
        let movements = MovementMode::each_mode();

        // 通过宏获取行数
        // 由于 match 语句强制要求补全 因此每次新增枚举值会同时新增行数
        // 以此来强制要求方法返回的枚举值是全量的
        let line_start = line!();
        for ele in movements.iter() {
            match ele {
                MovementMode::FreeStat => {}
                MovementMode::OnFloor => {}
                MovementMode::InAir => {}
                MovementMode::UnderWater => {}
                MovementMode::ClimbWall => {}
            }
        }
        let line_final = line!();
        // 其他语句所占行数
        const LINE_MID: u32 = 5;
        // 枚举匹配的行数 应与个数相等
        let enum_count: usize = (line_final - line_start - LINE_MID).try_into().unwrap();
        assert_eq!(movements.len(), enum_count);
    }
}
