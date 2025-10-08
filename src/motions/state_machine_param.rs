//! 状态机通用参数
//!
//! 代码检视：
//! - 所有的事件信号和大部分的主观意图都应在 [`PhyParam::to_instructions`] 中转换成事件
//! - [`PhyParam::to_instructions`] 中初始队列长度足够

use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{
        abstracts::player_input::{PlayerInstruction, PlayerOperation},
        abstracts::player_pre_input::PreInputInstruction,
        action_impl::ActionBaseEvent,
        motion_mode::MotionMode,
    },
};

#[derive(Clone, Debug, Default)]
pub struct FrameParam<S: FixedString> {
    // =========
    // 客观条件
    // =========
    pub(crate) delta: f64,
    pub(crate) anim_finished: bool,
    /// 当前正在播放的动画名称 外部传入 因为考虑到动画不一定完全由框架控制
    pub(crate) anim_name: S,
    /// 角色是否x轴移动（外部判断）
    pub(crate) character_x_moving: bool,
    /// 角色是否y轴上升（不包含静止） 不同游戏引擎2D游戏中的y轴方向不一样 因此不要自己判断上下
    pub(crate) character_y_fly_up: bool,
    // =========
    // 这里不应包含主观意图
    // =========
}

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
    // pub(crate) character_x_velocity: f64,
    // pub(crate) character_y_velocity: f64,
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
    pub(crate) hit_signal: bool,
    pub(crate) behit_signal: bool,
    // =========
    // 主观意图
    // =========
    // pub(crate) look_angle: PlayerInstruction<f64>,
    pub(crate) move_direction: PlayerInstruction<f64>,
    pub(crate) jump_once: PreInputInstruction<TinyTimer>,
    pub(crate) jump_keep: PlayerInstruction<bool>,
    pub(crate) dodge_once: PreInputInstruction<TinyTimer>,
    pub(crate) block_keep: PlayerInstruction<bool>,
    pub(crate) attack_once: PlayerInstruction<bool>,
    pub(crate) attack_keep: PlayerInstruction<bool>,
    // =========
    // Option 框架内部维护 不从外界传入、明确状态
    // =========
    /// - `None` 表示内部框架还未进行判断
    /// - `Some((Some, None))` 表示未进行切换
    /// - `Some((_, Some))` 表示进行切换（首次切换旧状态为 `None` ）
    pub(crate) motion_changed: Option<(Option<MotionMode>, Option<MotionMode>)>,
    pub(crate) action_duration: Option<f64>,
}

impl<S: FixedString> PhyParam<S> {
    pub fn to_instructions(&self) -> Vec<ActionBaseEvent> {
        // 为性能考虑给予必要的空间防止后续扩容
        let mut list = Vec::with_capacity(10);
        if self.hit_signal {
            list.push(ActionBaseEvent::HitSignal);
        }
        if self.behit_signal {
            list.push(ActionBaseEvent::BeHitSignal);
        }
        if self.jump_once.op_active() {
            list.push(ActionBaseEvent::JumpInstruction);
        }
        if self.jump_keep.op_active() {
            list.push(ActionBaseEvent::JumpHigherInstruction);
        }
        if self.dodge_once.op_active() {
            list.push(ActionBaseEvent::DodgeInstruction);
        }
        if self.block_keep.op_active() {
            list.push(ActionBaseEvent::BlockInstruction);
        }
        if self.attack_once.op_active() {
            list.push(ActionBaseEvent::AttackInstruction);
        }
        if self.attack_keep.op_active() {
            list.push(ActionBaseEvent::AttackHeavierInstruction);
        }
        list
    }
}
