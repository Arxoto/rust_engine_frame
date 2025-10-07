use crate::{cores::{tiny_timer::TinyTimer, unify_type::FixedString}, motion::{action_impl::ActionBaseEvent, movement_impl::MovementMode, player_input::{PlayerInstruction, PlayerOperation}, player_pre_input::PreInputInstruction}};


#[derive(Clone, Debug, Default)]
pub struct FrameParam<S: FixedString> {
    // 客观条件
    pub delta: f64,
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
}

#[derive(Clone, Debug, Default)]
pub struct PhyParam<S: FixedString> {
    // 客观条件
    pub delta: f64,
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
    pub character_x_velocity: f64,
    pub character_y_velocity: f64,
    pub character_can_climb: bool,
    // 事件信号标志
    pub hit_signal: bool,
    pub behit_signal: bool,
    // 主观意图
    pub look_angle: PlayerInstruction<f64>,
    pub move_direction: PlayerInstruction<f64>,
    pub jump_once: PreInputInstruction<TinyTimer>,
    pub jump_keep: PlayerInstruction<bool>,
    pub dodge_once: PreInputInstruction<TinyTimer>,
    pub block_keep: PlayerInstruction<bool>,
    pub attack_once: PlayerInstruction<bool>,
    pub attack_keep: PlayerInstruction<bool>,
    // Option 框架内部维护 不从外界传入、明确状态
    /// None 时表示没有发生模式的切换
    pub(crate) movement_changed: Option<(Option<MovementMode>, Option<MovementMode>)>,
    pub(crate) action_duration: Option<f64>,
}

impl<S: FixedString> PhyParam<S> {
    pub fn to_instructions(&self) -> Vec<ActionBaseEvent> {
        // 为性能考虑给予必要的空间防止后续扩容
        let mut list = Vec::with_capacity(10);
        // todo more and more
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