/// 运动模式
///
/// 每一套运动模式都应实现对应行为 用于动作之外的默认效果
///
/// - 行为用于基础操作、复杂逻辑，如带容错时间的移动跳跃
/// - 动作用于复杂操作、简单逻辑，如 combo 的依赖路径
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MovementMode {
    OnFloor,
    InAir,
    UnderWater,
    ClimbWall,
}
