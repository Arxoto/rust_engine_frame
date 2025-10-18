//! 动作系统的类型声明

/// 动作关联的触发事件
///
/// 直接基于类型去判断
pub trait ActionEvent: Clone + std::fmt::Debug + Eq + PartialEq {}

/// 动作关联的退出逻辑
///
/// 
/// 能够实现较为复杂的逻辑判断
pub trait ActionExitLogic<ExitParam>: Clone + std::fmt::Debug {
    fn should_exit(&self, exit_param: &ExitParam) -> bool;
}
