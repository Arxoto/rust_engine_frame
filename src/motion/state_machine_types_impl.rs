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
    pub delta: f64,
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
    // 事件信号标志
    pub hit: bool,
    pub behit: bool,
    // 主观意图
    /// move direction
    pub want_move: f64,
    /// look angle
    pub want_look: f64,
    pub want_jump: bool,
    pub want_dodge: bool,
    pub want_attack: bool,
    pub want_defence: bool,
    // 框架内部维护
    // Option 类型，因为：内部维护，不从外界传入，明确状态；
    pub(crate) movement_changed: Option<(MovementMode, MovementMode)>,
    pub(crate) action_duration: Option<f64>,
}

impl<S: FixedString> FrameParam<S> {
    pub fn to_instructions(&self) -> Vec<ActionBaseEvent> {
        // 为性能考虑给予必要的空间防止后续扩容
        let mut list = Vec::with_capacity(10);
        // todo more and more
        if self.hit {
            list.push(ActionBaseEvent::HitSignal);
        }
        if self.behit {
            list.push(ActionBaseEvent::BeHitSignal);
        }
        if self.want_jump {
            list.push(ActionBaseEvent::JumpInstruction);
        }
        if self.want_dodge {
            list.push(ActionBaseEvent::DodgeInstruction);
        }
        if self.want_attack {
            list.push(ActionBaseEvent::AttackInstruction);
        }
        if self.want_defence {
            list.push(ActionBaseEvent::DefenceInstruction);
        }
        list
    }
}

pub struct PhyParam<S: FixedString> {
    pub delta: f64,
    pub anim_name: S,
}

/// ExitParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub type MovementAction<S, PhyEff> =
    Action<S, MovementActionEvent, FrameParam<S>, MovementActionExitLogic<S>, PhyEff>;

/// EnterParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub trait MovementBehaviour<S: FixedString, FrameEff, PhyEff>:
    Behaviour<FrameParam<S>, FrameParam<S>, FrameEff, PhyParam<S>, PhyEff>
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

#[derive(Clone, Copy, Debug, Default)]
pub struct PhyEff {
    pub x: f64,
    pub y: f64,
}

pub trait EffGenerator<S: FixedString, FE, PE> {
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> FE;
    fn gen_phy_eff(by_action: Option<&PhyEff>, by_behaviour: Option<PhyEff>) -> PE;
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
        by_action: Option<&PhyEff>,
        by_behaviour: Option<PhyEff>,
    ) -> (Option<PhyEff>, Option<PhyEff>) {
        (by_action.map(|p| p.clone()), by_behaviour)
    }
}

/// by_action first, and then by_behaviour
pub struct ActionBehaviourGenerator;
impl<S: FixedString> EffGenerator<S, Option<FrameEff<S>>, Option<PhyEff>> for CommonEffGenerator {
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> Option<FrameEff<S>> {
        FrameEff::try_new(by_action.clone()).or(by_behaviour)
    }

    fn gen_phy_eff(by_action: Option<&PhyEff>, by_behaviour: Option<PhyEff>) -> Option<PhyEff> {
        by_action.map(|p| p.clone()).or(by_behaviour)
    }
}
