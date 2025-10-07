use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{abstracts::player_pre_input::PreInputOperation, state_machine_param::PhyParam},
};

/// 玩家控制器 实例化后对应一个玩家（本地或远端）
///
/// 其属性都是玩家操作 [`PlayerOperation`] 或 [`PreInputOperation`] （注：编码时由于没有引用所以没法跳转）
#[derive(Debug, Default)]
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
        self.jump_once.op_echo_with(&other.jump_once);
        self.dodge_once.op_echo_with(&other.dodge_once);
    }
}

impl<S: FixedString> PhyParam<S> {
    /// 对于 TinyTimer 类型的字段，由于其具备帧残留的副作用，因此需要手动回响关闭
    pub fn op_echo_with(&mut self, other: &Self) {
        self.jump_once.op_echo_with(&other.jump_once);
        self.dodge_once.op_echo_with(&other.dodge_once);
    }
}

// 需要保证全量字段
impl<S: FixedString> From<&PlayerController> for PhyParam<S> {
    fn from(value: &PlayerController) -> Self {
        Self {
            look_angle: value.look_angle.into(),
            move_direction: value.move_direction.into(),
            jump_once: (&value.jump_once).into(),
            jump_keep: value.jump_keep.into(),
            dodge_once: (&value.dodge_once).into(),
            block_keep: value.block_keep.into(),
            attack_once: value.attack_once.into(),
            attack_keep: value.attack_keep.into(),
            ..Default::default()
        }
    }
}
