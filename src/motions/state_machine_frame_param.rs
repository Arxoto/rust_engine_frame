//! 状态机通用渲染帧参数

use crate::cores::unify_type::FixedString;

#[derive(Clone, Debug, Default)]
pub struct FrameParam<S: FixedString> {
    // =========
    // 客观条件
    // =========
    pub delta: f64, // pub-external
    /// 当前动画播放结束
    pub anim_finished: bool, // pub-external
    /// 当前正在播放的动画名称 外部传入 因为考虑到动画不一定完全由框架控制
    pub anim_name: S, // pub-external
    /// 角色当前x轴移动方向
    pub character_x_velocity: f64, // pub-external
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub character_y_fly_up: bool, // pub-external

                    // =========
                    // 这里不应包含主观意图
                    // =========
}

impl<S: FixedString> FrameParam<S> {
    /// 正在播放当前动画
    pub(crate) fn anim_playing(&self, anim_name: &S) -> bool {
        !self.anim_finished && self.anim_name == *anim_name
    }
}
