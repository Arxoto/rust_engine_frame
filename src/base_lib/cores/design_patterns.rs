//! 设计模式抽象

/// 联合体
pub struct Union<T, U>(pub(super) T, pub(super) U);

impl<T, U> Union<T, U> {
    #[inline]
    pub fn new(t: T, u: U) -> Self {
        Union(t, u)
    }
}

/// 可转换目标类型
pub trait UnitedInto<With> {
    type Target;

    fn unite_into(self, w: With) -> Self::Target;
}

// Blanket impl 为所有类型默认实现 伪联合体
impl<T> UnitedInto<()> for T {
    type Target = Union<T, ()>;
    
    fn unite_into(self, w: ()) -> Union<T, ()> {
        Union(self, w)
    }
}
