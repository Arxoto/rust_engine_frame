use crate::{
    cores::unify_type::FixedString,
    motion::{
        behaviour::Behaviour,
        state_machine_frame_eff_impl::FrameEff,
        state_machine_param_impl::{FrameParam, PhyParam},
        state_machine_phy_eff_impl::{MovementData, PhyEff},
    },
};

/// 抽象模板 最小实现
#[derive(Debug, Default)]
pub struct AbstractBehaviour;

impl<S: FixedString> Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, PhyParam<S>, PhyEff>
    for AbstractBehaviour
{
    fn will_enter(&self, _p: &PhyParam<S>) -> bool {
        true
    }

    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, _p: &FrameParam<S>) -> FrameEff<S> {
        FrameEff::default()
    }

    fn process_physics(&mut self, _p: &mut PhyParam<S>, _data: &MovementData) -> PhyEff {
        PhyEff::default()
    }
}
