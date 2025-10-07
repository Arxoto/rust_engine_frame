//! 动作系统的类型声明

/// 动作关联的触发事件
///
/// 一般基于事件或信号机制去响应
pub trait ActionEvent: Clone + std::fmt::Debug + Eq + std::hash::Hash + PartialEq {}

/// 动作关联的退出逻辑
///
/// 一般用于每帧检查
pub trait ActionExitLogic<ExitParam>: Clone + std::fmt::Debug {
    fn should_exit(&self, exit_param: &ExitParam) -> bool;
}
