//! 【动作】和【行为】和【玩家角色】的状态机的依赖类定义
//!
//! 实现为2D游戏，因为输入参数和输出效果根据2D3D均有差别，索性一起抽象
//!
//! （仅行为系统的实现与2D3D有关，但是暂时先不独立抽象一层，因为输入参数需要抽象一层特征）

use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::action::Action,
        abstracts::behaviour::Behaviour,
        motion_mode::MotionMode,
        motion_action::{MotionActionEvent, MotionActionExitLogic},
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::{MotionData, PhyEff},
    },
};

/// ExitParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub type MotionAction<S, PhyEff> =
    Action<S, MotionActionEvent, PhyParam<S>, MotionActionExitLogic<S>, PhyEff>;

/// EnterParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub trait MotionBehaviour<S: FixedString, FrameEff, PhyEff>:
    for<'a> Behaviour<PhyParam<S>, FrameParam<S>, FrameEff, (&'a PhyParam<S>, &'a MotionData), PhyEff>
{
    fn get_motion_mode(&self) -> MotionMode;
}

/// 最终效果聚合器 将两个状态机的结果聚合
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
