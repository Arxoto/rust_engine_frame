use crate::cores::tiny_timer::TinyTimer;

pub struct PlayerController {
    pub move_direction: f64,

    pub look_angle: f64,

    pub block_keep: bool,

    pub attack_once: bool,
    pub attack_keep: bool,

    pub jump_once: TinyTimer,
    pub jump_keep: bool,

    pub dodge_once: TinyTimer,
}
