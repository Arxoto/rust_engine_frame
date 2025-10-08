//! 玩家角色控制器 实现了指令预输入功能
//!
//! 代码检视
//! - 结构体 [`PlayerController`] 中的所有字段在转换至 [`PhyParam`] 时都应存在响应映射
//! - [`TinyTimer`] 类型的字段在 [`PlayerController::op_echo_with`] 和 [`PhyParam::op_echo_with`] 中都应存在

use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{abstracts::player_pre_input::PreInputOperation, state_machine_param::PhyParam},
};

/// 玩家控制器 实例化后对应一个玩家（本地或远端）
///
/// 其属性都是玩家操作 [`PlayerOperation`] 或 [`PreInputOperation`] （注：编码时由于没有引用所以没法跳转）
#[derive(Debug, Default)]
pub struct PlayerController {
    // pub(crate) look_angle: f64,
    pub(crate) move_direction: f64,

    pub(crate) jump_once: TinyTimer,
    pub(crate) jump_keep: bool,

    pub(crate) dodge_once: TinyTimer,

    pub(crate) block_keep: bool,

    pub(crate) attack_once: bool,
    pub(crate) attack_keep: bool,
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
            // look_angle: value.look_angle.into(),
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
