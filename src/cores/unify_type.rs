// Trait Alias 简化复杂的 trait 约束，提高代码可读性和一致性
pub trait FixedName: Eq + std::hash::Hash + Clone + std::fmt::Debug {}

pub trait FixedString: Eq + std::hash::Hash + Clone + std::fmt::Debug + Default {
    fn is_legal(&self) -> bool {
        *self != Self::default()
    }
}

// 当使用 godot-rust 时，若在一个封闭系统中（仅初始化时涉及字符串输入，运行时字符串不与外部交互），也可以直接将 GString 转换为 String 使用
impl FixedName for String {}
impl FixedName for &str {}
impl FixedString for String {}
impl FixedString for &str {}
impl FixedString for usize {}

#[cfg(test)]
mod tests {
    use super::*;

    // define in lib

    // New Type 创建有明确语义的新类型，增强类型安全性和封装性
    // 这里使用 New Type 来为其他crate的类型实现 Trait Alias 绕过孤儿原则
    pub struct FixedNameBox<S: FixedName> {
        the_name: S,
    }

    impl<S: FixedName> FixedNameBox<S> {
        pub fn new<T: Into<S>>(w: T) -> Self {
            Self { the_name: w.into() }
        }

        pub fn get_name(&self) -> &S {
            &self.the_name
        }
    }

    // adapter in project

    type AnyType = String;
    // type AnyType = i32;
    fn get_value_for_type() -> AnyType {
        "asd".to_string()
        // 123
    }

    /// same as [`FixedName`]
    #[derive(PartialEq, Hash, Eq, Clone, Debug)]
    pub struct FixedNameWrapper(pub AnyType);

    impl From<AnyType> for FixedNameWrapper {
        fn from(value: AnyType) -> Self {
            FixedNameWrapper(value)
        }
    }

    /// convert when return
    impl Into<AnyType> for &FixedNameWrapper {
        fn into(self) -> AnyType {
            self.0.clone()
        }
    }

    impl FixedName for FixedNameWrapper {}

    // use in project

    #[test]
    fn test_func() {
        let fixed_name_box: FixedNameBox<FixedNameWrapper> =
            FixedNameBox::new(get_value_for_type());
        let real_name: AnyType = fixed_name_box.get_name().into();
        assert_eq!(real_name, get_value_for_type());
    }
}
