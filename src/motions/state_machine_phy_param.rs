//! 状态机通用物理帧参数

use crate::{
    cores::unify_type::FixedString,
    motions::{
        motion_action::ActionBaseEvent, motion_mode::MotionMode, player_controller::PlayerInstructionCollection
    },
};

#[derive(Clone, Debug, Default)]
pub struct PhyParam<S: FixedString> {
    // =========
    // 客观条件
    // =========
    pub(crate) delta: f64,
    pub(crate) anim_finished: bool,
    /// 当前正在播放的动画名称 外部传入 因为考虑到动画不一定完全由框架控制
    pub(crate) anim_name: S,
    /// 自由移动模式 一般用于测试
    pub(crate) behaviour_to_free: bool,
    /// 角色当前x轴移动方向
    pub(crate) character_x_velocity: f64,
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub(crate) character_y_fly_up: bool,
    /// 角色能否蹬墙跳（脚部碰撞墙体）
    pub(crate) character_can_jump_on_wall: bool,
    /// 角色正站在地面
    pub(crate) character_is_on_floor: bool,
    /// 角色能否攀爬（脚部手部都碰撞可攀爬墙体）
    pub(crate) character_should_climb: bool,
    /// 角色是否刚刚着陆（下落速度超过阈值后标记，速度为零时消耗标记）
    pub(crate) character_landing: bool,
    // =========
    // 事件信号标志
    // =========
    pub(crate) signals: GameSignalCollection,
    // =========
    // 主观意图
    // =========
    /// 玩家指令
    pub(crate) instructions: PlayerInstructionCollection,
    // =========
    // Option 框架内部维护
    // =========
    pub(crate) inner_param: PhyInnerParam,
}

#[derive(Clone, Debug, Default)]
pub struct GameSignalCollection {
    pub(crate) hit_signal: bool,
    pub(crate) behit_signal: bool,
}

impl GameSignalCollection {
    /// 将对应信号映射成事件推入列表
    pub fn push_instruction(&self, list: &mut Vec<ActionBaseEvent>) {
        if self.hit_signal {
            list.push(ActionBaseEvent::HitSignal);
        }
        if self.behit_signal {
            list.push(ActionBaseEvent::BeHitSignal);
        }
    }
}

/// 框架内部维护的状态参数
///
/// 字段均为 Option 不从外界传入、明确状态
#[derive(Clone, Debug, Default)]
pub struct PhyInnerParam {
    /// - `None` 表示内部框架还未进行判断
    /// - `Some((old, new))` 中的 old 表示旧状态 new 表示新状态
    /// - 若 old 和 new 相等，则表示状态未切换
    pub(crate) motion_changed: Option<(MotionMode, MotionMode)>,
    pub(crate) action_duration: Option<f64>,
}

/// just for test
pub fn signals_all_active() -> GameSignalCollection {
    GameSignalCollection {
        hit_signal: true,
        behit_signal: true,
    }
}
