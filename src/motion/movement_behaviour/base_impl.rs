use crate::{
    cores::unify_type::FixedString,
    motion::{
        abstracts::behaviour::Behaviour,
        abstracts::player_input::PlayerOperation,
        movement::MovementMode,
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::{MovementData, PhyEff},
        state_machine_types::MovementBehaviour,
    },
};

/// 行为系统的基础实现 无论如何都保证可以自由移动
///
/// 也可作为最小实现 用作模板创建新行为
#[derive(Debug, Default)]
pub struct BaseBehaviour;

impl BaseBehaviour {
    pub fn new() -> Self {
        Self
    }

    fn tick_physics<S: FixedString>(&mut self, p: &PhyParam<S>, data: &MovementData) -> PhyEff {
        // 摁住螺旋升天
        if p.jump_keep.op_active() {
            PhyEff::create_jump(data, p.move_direction.0)
        } else {
            PhyEff::create_falling(data, p.move_direction.0)
        }
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&PhyParam<S>, &MovementData), PhyEff>
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

    fn process_physics(&mut self, (p, data): &mut (&PhyParam<S>, &MovementData)) -> PhyEff {
        // 对任意移动输入均做出反应
        self.tick_physics(p, data)
    }
}

impl<S: FixedString> MovementBehaviour<S, FrameEff<S>, PhyEff> for BaseBehaviour {
    fn get_movement_mode(&self) -> MovementMode {
        MovementMode::FreeStat
    }
}
