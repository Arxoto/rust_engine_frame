use crate::{
    cores::unify_type::FixedString,
    motion::{
        behaviour::Behaviour,
        player_input::PlayerOperation,
        state_machine_types_impl::{FrameEff, FrameParam, PhyDirection, PhyEff, PhyMode, PhyParam},
    },
};

/// 行为系统的基础实现 无论如何都保证可以自由移动
pub struct BaseBehaviour;

impl BaseBehaviour {
    fn tick_physics<S: FixedString>(&mut self, p: &PhyParam<S>) -> PhyEff {
        // 对任意移动输入均做出反应

        let mode = if p.jump_keep.op_active() {
            PhyMode::Jumping
        } else {
            PhyMode::Falling
        };

        let direction = if p.move_direction.op_active() {
            if p.move_direction.0 > 0.0 {
                PhyDirection::Right
            } else {
                PhyDirection::Left
            }
        } else {
            PhyDirection::None
        };

        PhyEff { mode, direction }
    }
}

impl<S: FixedString> Behaviour<FrameParam<S>, FrameParam<S>, FrameEff<S>, PhyParam<S>, PhyEff>
    for BaseBehaviour
{
    fn will_enter(&self, _p: &FrameParam<S>) -> bool {
        true
    }

    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, _p: &FrameParam<S>) -> FrameEff<S> {
        // 不对视觉效果做修改
        FrameEff::default()
    }

    fn process_physics(&mut self, p: &mut PhyParam<S>) -> PhyEff {
        self.tick_physics(p)
    }
}
