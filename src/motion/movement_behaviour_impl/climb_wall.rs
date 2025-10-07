use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motion::{
        behaviour::Behaviour, state_machine_frame_eff_impl::FrameEff, state_machine_param_impl::{FrameParam, PhyParam}, state_machine_phy_eff_impl::{MovementData, PhyEff}
    },
};

const CLIMB_BEGIN_TIME: f64 = 0.2;

/// 行为系统的基础实现 无论如何都保证可以自由移动
#[derive(Debug, Default)]
pub struct ClimbWallBehaviour<S: FixedString> {
    pub climb_begin_anim: S,
    pub climbing_anim: S,
    beginning: TinyTimer,
}

impl<S: FixedString> ClimbWallBehaviour<S> {
    pub fn new(climb_begin_anim: S, climbing_anim: S) -> ClimbWallBehaviour<S> {
        Self {
            climb_begin_anim,
            climbing_anim,
            beginning: TinyTimer::new(CLIMB_BEGIN_TIME),
        }
    }
}

impl<S: FixedString> Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, PhyParam<S>, PhyEff>
    for ClimbWallBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        p.character_can_climb // 判断条件需要用到向量运算 为保证项目纯净 交由外部判断输入
    }

    fn on_enter(&mut self) {
        self.beginning.start_time();
    }

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        self.beginning.add_time(p.delta);
        if self.beginning.in_time() {
            FrameEff::from(self.climb_begin_anim.clone())
        } else {
            FrameEff::from(self.climbing_anim.clone())
        }
    }

    fn process_physics(&mut self, p: &mut PhyParam<S>, data: &MovementData) -> PhyEff {
        PhyEff::create_climb(data, p.move_direction.0)
    }
}
