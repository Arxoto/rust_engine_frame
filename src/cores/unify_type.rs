pub trait FixedName: Eq + std::hash::Hash + Clone {}

impl FixedName for String {}
impl FixedName for &str {}

#[cfg(test)]
mod tests {
    use super::*;

    // define in lib

    pub struct FixedNameBox<S>
    where
        S: FixedName,
    {
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
    #[derive(PartialEq, Hash, Eq, Clone)]
    pub struct FixedNameWrapper(AnyType);

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
        let fixed_name_box: FixedNameBox<FixedNameWrapper> = FixedNameBox::new(get_value_for_type());
        let real_name: AnyType = fixed_name_box.get_name().into();
        assert_eq!(real_name, get_value_for_type());
    }
}
