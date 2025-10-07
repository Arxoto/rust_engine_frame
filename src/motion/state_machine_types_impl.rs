//! 【动作】和【行为】和【玩家角色】的状态机的依赖类定义
//!
//! 实现为2D游戏，因为输入参数和输出效果根据2D3D均有差别，索性一起抽象
//!
//! （仅行为系统的实现与2D3D有关，但是暂时先不独立抽象一层，因为输入参数需要抽象一层特征）

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action::Action,
        action_impl::ActionBaseEvent,
        behaviour::Behaviour,
        movement_action_impl::{MovementActionEvent, MovementActionExitLogic},
        movement_impl::MovementMode,
    },
};

#[derive(Clone, Default)]
pub struct FrameParam<S: FixedString> {
    // 客观条件
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
}

#[derive(Clone, Default)]
pub struct PhyParam<S: FixedString> {
    // 客观条件
    pub delta: f64,
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
    // 事件信号标志
    pub hit_signal: bool,
    pub behit_signal: bool,
    // 主观意图
    pub want_look_angle: f64,
    pub want_move_direction: f64,
    pub want_jump_once: bool,
    pub want_jump_keep: bool,
    pub want_dodge_once: bool,
    pub want_block_keep: bool,
    pub want_attack_once: bool,
    pub want_attack_keep: bool,
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
        if self.want_jump_once {
            list.push(ActionBaseEvent::JumpInstruction);
        }
        if self.want_jump_keep {
            list.push(ActionBaseEvent::JumpHigherInstruction);
        }
        if self.want_dodge_once {
            list.push(ActionBaseEvent::DodgeInstruction);
        }
        if self.want_block_keep {
            list.push(ActionBaseEvent::BlockInstruction);
        }
        if self.want_attack_once {
            list.push(ActionBaseEvent::AttackInstruction);
        }
        if self.want_attack_keep {
            list.push(ActionBaseEvent::AttackHeavierInstruction);
        }
        list
    }
}

/// ExitParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub type MovementAction<S, PhyEff> =
    Action<S, MovementActionEvent, PhyParam<S>, MovementActionExitLogic<S>, PhyEff>;

/// EnterParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub trait MovementBehaviour<S: FixedString, FrameEff, PhyEff>:
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff, PhyParam<S>, PhyEff>
{
    fn get_movement_mode(&self) -> MovementMode;
}

/// 若有必要可将角色动画分层（如上半身下半身组合动画），动作系统的逻辑保持单一仍然只返回一个动画
#[derive(Debug, Default)]
pub struct FrameEff<S: FixedString> {
    pub anim_name: S,
}

// 由于 S 是泛型，所以无法实现 TryFrom （具体原因存疑，反正就是有冲突，怀疑可能是编译器太过于严格）
impl<S: FixedString> From<S> for FrameEff<S> {
    fn from(value: S) -> Self {
        Self { anim_name: value }
    }
}

impl<S: FixedString> FrameEff<S> {
    pub fn is_legal(&self) -> bool {
        self.anim_name.is_legal()
    }

    pub fn try_new(s: S) -> Option<Self> {
        let frame_eff = FrameEff::from(s);
        if frame_eff.is_legal() {
            Some(frame_eff)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PhyDirection {
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum PhyMode {
    // Idle,
    Run,
    Dodging,
    Jumping,
    Falling,
    Flying,
    Climbing,
}

#[derive(Clone, Copy, Debug)]
pub struct PhyEff {
    pub mode: PhyMode,
    pub direction: PhyDirection,
}

// 没有这个实现就无法对 [`PlayerMachine`] 使用宏实现 Default （不知道为什么）
impl Default for PhyEff {
    fn default() -> Self {
        Self {
            mode: PhyMode::Run,
            direction: PhyDirection::None,
        }
    }
}

pub trait EffGenerator<S: FixedString, FE, PE> {
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> FE;
    fn gen_phy_eff(by_action: Option<PhyEff>, by_behaviour: Option<PhyEff>) -> PE;
}

pub struct CommonEffGenerator;
impl<S: FixedString>
    EffGenerator<S, (Option<FrameEff<S>>, Option<FrameEff<S>>), (Option<PhyEff>, Option<PhyEff>)>
    for CommonEffGenerator
{
    fn gen_frame_eff(
        by_action: &S,
        by_behaviour: Option<FrameEff<S>>,
    ) -> (Option<FrameEff<S>>, Option<FrameEff<S>>) {
        (FrameEff::try_new(by_action.clone()), by_behaviour)
    }

    fn gen_phy_eff(
        by_action: Option<PhyEff>,
        by_behaviour: Option<PhyEff>,
    ) -> (Option<PhyEff>, Option<PhyEff>) {
        (by_action, by_behaviour)
    }
}

/// by_action first, and then by_behaviour
pub struct ActionBehaviourGenerator;
impl<S: FixedString> EffGenerator<S, Option<FrameEff<S>>, Option<PhyEff>> for CommonEffGenerator {
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> Option<FrameEff<S>> {
        FrameEff::try_new(by_action.clone()).or(by_behaviour)
    }

    fn gen_phy_eff(by_action: Option<PhyEff>, by_behaviour: Option<PhyEff>) -> Option<PhyEff> {
        by_action.or(by_behaviour)
    }
}
