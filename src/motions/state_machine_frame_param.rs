//! 状态机通用渲染帧参数

use crate::cores::unify_type::FixedString;

#[derive(Clone, Debug, Default)]
pub struct FrameParam<S: FixedString> {
    // =========
    // 客观条件
    // =========
    pub(crate) delta: f64,
    pub(crate) anim_finished: bool,
    /// 当前正在播放的动画名称 外部传入 因为考虑到动画不一定完全由框架控制
    pub(crate) anim_name: S,
    /// 角色当前x轴移动方向
    pub(crate) character_x_velocity: f64,
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub(crate) character_y_fly_up: bool,
    // =========
    // 这里不应包含主观意图
    // =========
}
