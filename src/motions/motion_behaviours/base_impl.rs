use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::behaviour::Behaviour,
        abstracts::player_input::PlayerOperation,
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_types::MotionBehaviour,
    },
};

/// 行为系统的基础实现 无论如何都保证可以自由移动 测试时使用
#[derive(Debug, Default)]
pub struct BaseBehaviour;

impl BaseBehaviour {
    pub fn new() -> Self {
        Self
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for BaseBehaviour
{
    fn will_enter(&self, _p: &PhyParam<S>) -> bool {
        true
    }

    fn on_enter(&mut self, _p: &PhyParam<S>) {}

    fn tick_frame(&mut self, _p: &FrameParam<S>) -> FrameEff<S> {
        // 不对视觉效果做修改
        FrameEff::default()
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        // 对任意移动输入均做出反应 摁住跳跃键螺旋升天
        if p.jump_keep.op_active() {
            PhyEff::create_jump(data, p.move_direction.0)
        } else {
            PhyEff::create_falling(data, p.move_direction.0)
        }
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for BaseBehaviour {
    fn get_motion_mode(&self) -> MotionMode {
        MotionMode::FreeStat
    }
}
