//! 设计模式抽象
//!
//! ## 需求描述
//!
//! - 对于 TickTimer 和 StaticTimer ，他们都需要实现相同的抽象
//!   - TickTimer 本身即拥有完备的信息，自身就能够实现特征
//!   - StaticTimer 需要依赖 StaticTimeline 信息才能够实现特征
//! - 为防止代码重复逻辑（后续修改需要同时改动两处地方），需要实现一种方案兼容【依赖上下文】和【无需上下文】的类型实现
//!
//! ## 方案一
//!
//! 基于联合体 [`Union`] 和转换目标类型 [`WithInto`] 实现，当无需依赖上下文时，实现 [`WithInto<()>`] 即可
//!
//! 缺陷，也即 Rust 传统类型求解器 (Old Trait Solver) 中最经典的痛点之一
//! - 高阶生命周期 (HRTB) 与关联类型规范化 (Projection Normalization) 的耦合死锁
//! - 旧求解器缺乏“延迟承诺”能力。当遇到高阶生命周期 for<'a> 时，它必须“提前”把关联类型 <...>::Target 展开
//! - 但此时类型推断变量（如 With ）还没确定，导致它去盲目匹配 Blanket Implementation 时触发了歧义，进而编译报错
//!
//! 解决
//! - 使用函数时显式标注类型（部分类型即可）协助类型推断（详见 tests ）
//! - 使用 `Next-gen trait solving` （但是怕出问题）
//!
//! ## 方案二
//!
//! 直接在特征中定义关联类型
//!
//! 优点：语义明确，无需 Blanket impl 隐式实现
//!
//! 缺点：当无需依赖上下文时，需要 `call_fn(())` 形式传入 unit ，不是很优雅
//!
//! 介于方案一的“下一代求解器缺”始终没有默认采用，优先选用方案二

/// 依赖上下文
pub trait DependCtx {
    type Ctx<'a>: Copy;
}

/// 联合体
pub struct Union<T, U>(pub(super) T, pub(super) U);

impl<T, U> Union<T, U> {
    #[inline]
    pub fn new(t: T, u: U) -> Self {
        Union(t, u)
    }
}

/// 可转换目标类型
pub trait WithInto<Ctx> {
    type Target;

    fn with_into(self, ctx: Ctx) -> Self::Target;
}

/// 案例，揭示如何通过 [`WithInto`] 兼容【依赖上下文】和【无需上下文】的类型
/// - 【依赖上下文】 [`WithInto<Ctx>`]
/// - 【无需上下文】 [`WithInto<()>`]
#[cfg(test)]
mod tests {
    use super::*;

    // 密封，限制作用域
    mod private {
        pub trait Sealed {}
    }

    // Blanket impl 为所有类型默认实现 伪联合体
    impl<T: private::Sealed> WithInto<()> for T {
        type Target = Union<T, ()>;

        fn with_into(self, w: ()) -> Union<T, ()> {
            Union(self, w)
        }
    }

    /// 任意特征，只读和可变方法分离
    trait AnyTrait {
        fn do_something(&self);
    }

    /// 任意特征，只读和可变方法分离
    trait AnyTraitMut {
        fn do_something_mut(&mut self);
    }

    // region: Blanket impl 为所有伪联合体类型默认实现 需要的特征

    impl<T: AnyTrait> AnyTrait for Union<&T, ()> {
        fn do_something(&self) {
            self.0.do_something();
        }
    }

    impl<T: AnyTraitMut> AnyTraitMut for Union<&mut T, ()> {
        fn do_something_mut(&mut self) {
            self.0.do_something_mut();
        }
    }

    // endregion

    /// 持有特征，基于组合思想
    trait HasInner {
        type Inner;

        fn get_inner(&self) -> &Self::Inner;

        fn get_inner_mut(&mut self) -> &mut Self::Inner;
    }

    /// 只读函数定义
    fn auto_do_something<E, With>(anything: &E, with: With)
    where
        With: Copy,
        E: HasInner,
        for<'a> &'a <E as HasInner>::Inner: WithInto<With>,
        for<'a> <&'a <E as HasInner>::Inner as WithInto<With>>::Target: AnyTrait,
    {
        anything.get_inner().with_into(with).do_something();
    }

    /// 可变函数定义
    fn auto_do_something_mut<E, With>(anything: &mut E, with: With)
    where
        With: Copy,
        E: HasInner,
        for<'a> &'a mut <E as HasInner>::Inner: WithInto<With>,
        for<'a> <&'a mut <E as HasInner>::Inner as WithInto<With>>::Target: AnyTraitMut,
    {
        anything.get_inner_mut().with_into(with).do_something_mut();
    }

    #[test]
    fn test_foo() {
        /// 举例类型，该类型必须需要一个上下文才能够实现需要的特征
        struct Foo;

        #[derive(Debug, Clone, Copy)]
        struct FooWith;

        // region: impl Union<&T, &With>

        impl AnyTrait for Union<&Foo, &FooWith> {
            fn do_something(&self) {}
        }

        impl AnyTraitMut for Union<&mut Foo, &FooWith> {
            fn do_something_mut(&mut self) {}
        }

        // endregion

        // region: impl UnitedInto<With>

        impl<'a, 'b> WithInto<&'b FooWith> for &'a Foo {
            type Target = Union<&'a Foo, &'b FooWith>;

            fn with_into(self, w: &'b FooWith) -> Union<&'a Foo, &'b FooWith> {
                Union(self, w)
            }
        }

        impl<'a, 'b> WithInto<&'b FooWith> for &'a mut Foo {
            type Target = Union<&'a mut Foo, &'b FooWith>;

            fn with_into(self, w: &'b FooWith) -> Union<&'a mut Foo, &'b FooWith> {
                Union(self, w)
            }
        }

        // endregion

        // region: Wrapper

        struct HasFoo(Foo);

        impl HasInner for HasFoo {
            type Inner = Foo;

            fn get_inner(&self) -> &Self::Inner {
                &self.0
            }

            fn get_inner_mut(&mut self) -> &mut Self::Inner {
                &mut self.0
            }
        }

        // endregion

        auto_do_something::<_, &FooWith>(&HasFoo(Foo), &FooWith);
        auto_do_something_mut::<_, &FooWith>(&mut HasFoo(Foo), &FooWith);
    }

    #[test]
    fn test_bar() {
        /// 举例类型，该类型无需上下文就能够实现需要的特征
        struct Bar;
        // endregion

        // region: impl itself

        // 由于本身实现该特征，因此自动通过 Blanket impl 为该类型的伪联合体类型实现同特征

        impl AnyTrait for Bar {
            fn do_something(&self) {}
        }

        impl AnyTraitMut for Bar {
            fn do_something_mut(&mut self) {}
        }

        // endregion

        // region: Blanket impl UnitedInto<()>

        // 赋予密封特征，因此自动通过 Blanket impl 实现转换为伪联合体

        impl private::Sealed for &Bar {}
        impl private::Sealed for &mut Bar {}

        // region: Wrapper

        struct HasBar(Bar);

        impl HasInner for HasBar {
            type Inner = Bar;

            fn get_inner(&self) -> &Self::Inner {
                &self.0
            }

            fn get_inner_mut(&mut self) -> &mut Self::Inner {
                &mut self.0
            }
        }

        // endregion

        auto_do_something::<HasBar, ()>(&HasBar(Bar), ());
        auto_do_something_mut::<HasBar, ()>(&mut HasBar(Bar), ());
    }
}
