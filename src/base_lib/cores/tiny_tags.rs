//! 标签系统，用标签替代逻辑，实现框架和业务解耦合

use crate::base_lib::cores::unify_types::FixedName;

/// 包装任意的自定义标签 `PureTag` 赋予其逻辑判断能力
pub enum TinyTag<T: FixedName> {
    Always,
    Never,
    Has(T),
    Not(T),
    Or(T, T),
    And(T, T),
    And3(T, T, T),
    // // 嵌套结构
    // AbstractNot(Box<Self>),
    // AbstractOr(Box<Self>, Box<Self>),
    // AbstractAnd(Box<Self>, Box<Self>),
}

impl<T: FixedName> TinyTag<T> {
    pub fn check_condition(&self, ll: &impl PureTagContainer<PureTag = T>) -> bool {
        match self {
            TinyTag::Always => true,
            TinyTag::Never => false,
            TinyTag::Has(t) => ll.check_condition(t),
            TinyTag::Not(t) => !ll.check_condition(t),
            TinyTag::Or(t1, t2) => ll.check_condition(t1) || ll.check_condition(t2),
            TinyTag::And(t1, t2) => ll.check_condition(t1) && ll.check_condition(t2),
            TinyTag::And3(t1, t2, t3) => {
                ll.check_condition(t1) && ll.check_condition(t2) && ll.check_condition(t3)
            }
        }
    }
}

/// `PureTag` 的容器
pub trait PureTagContainer {
    type PureTag: FixedName;

    /// 使用 [`TinyTag::check_condition`] 进行代理
    fn check_condition(&self, pure_tag: &Self::PureTag) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func() {
        struct PureTagVec(Vec<i32>);

        impl PureTagContainer for PureTagVec {
            type PureTag = i32;

            fn check_condition(&self, pure_tag: &Self::PureTag) -> bool {
                self.0.contains(pure_tag)
            }
        }

        let ll = PureTagVec(vec![1, 2, 3]);

        assert!(TinyTag::Always.check_condition(&ll));
        assert!(!TinyTag::Never.check_condition(&ll));
        assert!(TinyTag::Has(1).check_condition(&ll));
        assert!(TinyTag::Not(9).check_condition(&ll));
        assert!(TinyTag::Or(2, 9).check_condition(&ll));
        assert!(TinyTag::And(1, 3).check_condition(&ll));
        assert!(TinyTag::And3(2, 3, 1).check_condition(&ll));
    }
}
