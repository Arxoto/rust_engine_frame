pub mod abstracts;

// 业务逻辑具体实现

pub mod motion_mode;

pub mod motion_action;
pub mod motion_behaviours;

// 输入输出参数抽象

pub mod player_controller;

pub mod state_machine_frame_eff;
pub mod state_machine_frame_param;
pub mod state_machine_phy_eff;
pub mod state_machine_phy_param;

// 状态机实现

pub mod state_machine_types;

pub mod state_machine_action;
pub mod state_machine_behaviour;

pub mod state_machine;
