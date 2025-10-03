//! 动作系统的类型声明

/// 动作关联的触发事件
///
/// 一般基于事件或信号机制去响应
pub trait ActionEvent: Clone + std::fmt::Debug + Eq + std::hash::Hash + PartialEq {}

/// 动作关联的退出逻辑
///
/// 一般用于每帧检查
pub trait ActionExitLogic: Clone + std::fmt::Debug {
    type ExitParam;
    fn should_exit(&self, p: &Self::ExitParam) -> bool;
}

/// 动作能否覆盖
#[derive(Clone, Copy, Debug)]
pub struct ActionCanCover(pub(crate) bool);

impl From<bool> for ActionCanCover {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
