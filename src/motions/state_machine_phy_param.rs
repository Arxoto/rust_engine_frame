//! 状态机通用物理帧参数
//!
//! 代码检视：
//! - [`PhyParam::into_instructions`] 中初始队列长度足够

use crate::{
    cores::unify_type::FixedString,
    motions::{
        action_impl::ActionBaseEvent, motion_mode::MotionMode,
        player_controller::PlayerInstructionCollection,
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
    /// 强制进行行为切换时使用 一般用于特殊逻辑
    pub(crate) behaviour_cut_out: bool,
    /// 角色当前x轴移动方向
    pub(crate) character_x_velocity: f64,
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub(crate) character_y_fly_up: bool,
    /// 角色能否蹬墙跳（脚部碰撞墙体）
    pub(crate) character_can_jump_on_wall: bool,
    /// 角色正站在地面
    pub(crate) character_is_on_floor: bool,
    /// 角色能否攀爬（脚部手部都碰撞可攀爬墙体）
    pub(crate) character_can_climb: bool,
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
    /// - `Some((Some, None))` 表示未进行切换
    /// - `Some((_, Some))` 表示进行切换（首次切换旧状态为 `None` ）
    pub(crate) motion_changed: Option<(Option<MotionMode>, Option<MotionMode>)>,
    pub(crate) action_duration: Option<f64>,
}

/// just for test
pub fn signals_all_active() -> GameSignalCollection {
    GameSignalCollection {
        hit_signal: true,
        behit_signal: true,
    }
}
