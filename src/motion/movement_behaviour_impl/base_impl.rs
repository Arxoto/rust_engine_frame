use crate::{
    cores::unify_type::FixedString,
    motion::{
        behaviour::Behaviour,
        state_machine_types_impl::{FrameEff, FrameParam, PhyEff, PhyParam},
    },
};

/// 行为系统的基础实现 无论如何都保证可以自由移动
pub struct BaseBehaviour;

impl<S: FixedString> Behaviour<FrameParam<S>, FrameParam<S>, FrameEff<S>, PhyParam<S>, PhyEff>
    for BaseBehaviour
{
    fn will_enter(&self, _p: &FrameParam<S>) -> bool {
        true
    }

    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn process_frame(&mut self, _p: &FrameParam<S>) {}

    fn get_frame_eff(&mut self) -> FrameEff<S> {
        // 不对视觉效果做修改
        FrameEff::default()
    }

    fn tick_physics(&mut self, _p: &PhyParam<S>) -> PhyEff {
        // 对任意移动输入均做出反应
        // todo
        PhyEff::default()
    }
}
