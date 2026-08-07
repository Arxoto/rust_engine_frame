//! 标签系统，用标签替代逻辑，实现框架和业务解耦合

use crate::base_lib::cores::unify_types::FixedName;

pub enum TinyTag<T: FixedName> {
    Has(T),
    Not(T),
    And(T, T),
    Or(T, T),
    // // 嵌套结构
    // AbstractNot(Box<Self>),
    // AbstractAnd(Box<Self>, Box<Self>),
    // AbstractOr(Box<Self>, Box<Self>),
}

impl<T: FixedName> TinyTag<T> {
    pub fn check_condition(&self, ll: &impl TinyTagContainer<Element = T>) -> bool {
        match self {
            TinyTag::Has(t) => ll.check_condition(t),
            TinyTag::Not(t) => !ll.check_condition(t),
            TinyTag::And(t1, t2) => ll.check_condition(t1) && ll.check_condition(t2),
            TinyTag::Or(t1, t2) => ll.check_condition(t1) || ll.check_condition(t2),
            // // 嵌套结构
            // TinyTag::AbstractNot(t) => !t.check_condition(ll),
            // TinyTag::AbstractAnd(t1, t2) => t1.check_condition(ll) && t2.check_condition(ll),
            // TinyTag::AbstractOr(t1, t2) => t1.check_condition(ll) || t2.check_condition(ll),
        }
    }
}

pub trait TinyTagContainer {
    type Element: FixedName;

    /// 使用 [`TinyTag::check_condition`] 进行代理
    fn check_condition(&self, pure_tag: &Self::Element) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func() {
        struct TagVec(Vec<i32>);

        impl TinyTagContainer for TagVec {
            type Element = i32;

            fn check_condition(&self, pure_tag: &Self::Element) -> bool {
                self.0.contains(pure_tag)
            }
        }

        let ll = TagVec(vec![1, 2, 3]);

        assert!(TinyTag::Has(1).check_condition(&ll));
        assert!(TinyTag::Not(9).check_condition(&ll));
        assert!(TinyTag::And(1, 3).check_condition(&ll));
        assert!(TinyTag::Or(2, 9).check_condition(&ll));
    }
}
