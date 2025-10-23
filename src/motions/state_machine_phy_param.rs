//! 状态机通用物理帧参数

use crate::{
    cores::unify_type::FixedString,
    motions::{
        motion_action::ActionBaseEvent, motion_mode::MotionMode,
        player_controller::PlayerInstructionCollection,
    },
};

#[derive(Clone, Debug, Default)]
pub struct PhyParam<S: FixedString> {
    // =========
    // 客观条件
    // =========
    pub delta: f64,          // pub-external
    pub anim_finished: bool, // pub-external
    /// 当前正在播放的动画名称（主动画） 外部传入 因为考虑到动画不一定完全由框架控制
    pub anim_name: S, // pub-external
    /// 自由移动模式 一般用于测试
    pub behaviour_to_free: bool, // pub-external
    /// 角色当前x轴移动方向
    pub character_x_velocity: f64, // pub-external
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub character_y_fly_up: bool, // pub-external
    /// 角色能否蹬墙跳（脚部碰撞墙体）
    pub character_can_jump_on_wall: bool, // pub-external
    /// 角色正站在地面
    pub character_is_on_floor: bool, // pub-external
    /// 角色能否攀爬（脚部手部都碰撞可攀爬墙体）
    pub character_should_climb: bool, // pub-external
    /// 角色是否刚刚着陆（下落速度超过阈值后标记，速度为零时消耗标记）
    pub character_landing: bool, // pub-external
    // =========
    // 事件信号标志
    // =========
    pub signals: GameSignalCollection, // pub-external
    // =========
    // 主观意图
    // =========
    /// 玩家指令
    pub instructions: PlayerInstructionCollection, // pub-external
    // =========
    // Option 框架内部维护
    // =========
    pub inner_param: PhyInnerParam, // pub-external
}

impl<S: FixedString> PhyParam<S> {
    /// 转身判断  当前速度大于阈值 && 意图方向与速度方向相反
    pub(crate) fn want_turn_back(&self, velocity_threshold: f64) -> bool {
        self.character_x_velocity.abs() >= velocity_threshold
            && self.character_x_velocity * self.instructions.move_direction.0 < 0.0
    }
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
