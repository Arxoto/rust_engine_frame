use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motion::{player_operation::PlayerOperation, state_machine_types_impl::PhyParam},
};

/// 玩家控制器 实例化后对应一个玩家（本地或远端）
///
/// 其属性都是玩家操作 [`PlayerOperation`]
#[derive(Default)]
pub struct PlayerController {
    pub look_angle: f64,

    pub move_direction: f64,

    pub jump_once: TinyTimer,
    pub jump_keep: bool,

    pub dodge_once: TinyTimer,

    pub block_keep: bool,

    pub attack_once: bool,
    pub attack_keep: bool,
}

impl PlayerController {
    /// 对于 TinyTimer 类型的字段，由于其具备帧残留的副作用，因此需要手动回响关闭
    pub fn op_echo_with<S: FixedString>(&mut self, other: &PhyParam<S>) {
        self.jump_once.op_echo_with(&other.want_jump_once);
        self.dodge_once.op_echo_with(&other.want_dodge_once);
    }
}

impl<S: FixedString> PhyParam<S> {
    /// 对于 TinyTimer 类型的字段，由于其具备帧残留的副作用，因此需要手动回响关闭
    pub fn op_echo_with(&mut self, other: &Self) {
        self.want_jump_once.op_echo_with(&other.want_jump_once);
        self.want_dodge_once.op_echo_with(&other.want_dodge_once);
    }
}

impl<S: FixedString> From<&PlayerController> for PhyParam<S> {
    fn from(value: &PlayerController) -> Self {
        Self {
            want_look_angle: value.look_angle,
            want_move_direction: value.move_direction,
            want_jump_once: value.jump_once.op_active(),
            want_jump_keep: value.jump_keep,
            want_dodge_once: value.dodge_once.op_active(),
            want_block_keep: value.block_keep,
            want_attack_once: value.attack_once,
            want_attack_keep: value.attack_keep,
            ..Default::default()
        }
    }
}
