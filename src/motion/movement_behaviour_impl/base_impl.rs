use crate::{
    cores::unify_type::FixedString,
    motion::{
        behaviour::Behaviour,
        player_input::PlayerOperation,
        state_machine_frame_eff_impl::FrameEff,
        state_machine_param_impl::{FrameParam, PhyParam},
        state_machine_phy_eff_impl::{MovementData, PhyEff},
    },
};

/// 行为系统的基础实现 无论如何都保证可以自由移动
#[derive(Debug, Default)]
pub struct BaseBehaviour;

impl BaseBehaviour {
    fn tick_physics<S: FixedString>(&mut self, p: &PhyParam<S>, data: &MovementData) -> PhyEff {
        // 对任意移动输入均做出反应

        if p.jump_keep.op_active() {
            PhyEff::create_jump(data, p.move_direction.0)
        } else {
            PhyEff::create_falling(data, p.move_direction.0)
        }
    }
}

impl<S: FixedString> Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, PhyParam<S>, PhyEff>
    for BaseBehaviour
{
    fn will_enter(&self, _p: &PhyParam<S>) -> bool {
        true
    }

    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, _p: &FrameParam<S>) -> FrameEff<S> {
        // 不对视觉效果做修改
        FrameEff::default()
    }

    fn process_physics(&mut self, p: &mut PhyParam<S>, data: &MovementData) -> PhyEff {
        self.tick_physics(p, data)
    }
}
