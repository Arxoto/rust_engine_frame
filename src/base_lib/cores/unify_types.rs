//! 为【引擎运行时的内部标识符】抽象统一接口，兼容接入不同引擎
//! 
//! 使用 Trait Alias 简化复杂的 trait 约束，提高代码可读性和一致性
//!
//! 使用 New Type 创建有明确语义的新类型，增强类型安全性和封装性

/// 一般用作【引擎运行时的内部标识符】，需要被唯一标识、快速比较
///
/// 参考 Godot StringName 和 Unreal FName 或 Bevy Entity ID
///
/// 默认实现 i64 usize &str String 类型
/// （如 godot-rust 若仅初始化涉及转换、运行时不涉及外部交互，那么可以将 GString 转换为 String 使用）
pub trait FixedName: Eq + std::hash::Hash + Clone + std::fmt::Debug {}

impl FixedName for i64 {}
impl FixedName for usize {}
impl FixedName for &str {}
impl FixedName for String {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 假设本项目中某些地方用到了 [`FixedName`] 作为泛型
    ///
    /// 为了简化使用，入参一般为 [`Into`] 类型
    pub struct WhereverUsed<S: FixedName> {
        the_name: S,
    }

    impl<S: FixedName> WhereverUsed<S> {
        pub fn new<T: Into<S>>(name: T) -> Self {
            Self {
                the_name: name.into(),
            }
        }
    }

    /// 假设要为 [`AnyType`] 类型做适配（这个类型被第三方包定义）
    type AnyType = i8;

    fn get_value_for_type() -> AnyType {
        100
    }

    /// 包装 [`AnyType`] 以绕过孤儿原则
    ///
    /// 需要实现 [`FixedName`] 的所有特征
    ///
    /// 并且为了双向转换，实现 [`From`] 和 [`Into`]
    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct FixedNameWrapper(pub AnyType);

    impl FixedName for FixedNameWrapper {}

    impl From<AnyType> for FixedNameWrapper {
        fn from(value: AnyType) -> Self {
            FixedNameWrapper(value)
        }
    }

    impl From<FixedNameWrapper> for AnyType {
        fn from(value: FixedNameWrapper) -> Self {
            value.0
        }
    }

    #[test]
    fn test_func() {
        // 使用库函数
        let used: WhereverUsed<FixedNameWrapper> = WhereverUsed::new(get_value_for_type());

        // 从中获取自己定义的类型
        let name: FixedNameWrapper = used.the_name;

        // 转换为第三方类型
        let name: AnyType = name.into();
        assert_eq!(name, get_value_for_type());
    }
}
