/// 统一上下文包装
pub struct ContextWrapper<I, Ctx> {
    pub inner: I,
    pub ctx: Ctx,
}

pub trait WithContext {
    fn with_ctx<'a, 'b, Ctx>(&'a self, ctx: &'b Ctx) -> ContextWrapper<&'a Self, &'b Ctx> {
        ContextWrapper { inner: self, ctx }
    }

    fn with_ctx_mut<'a, 'b, Ctx>(
        &'a mut self,
        ctx: &'b mut Ctx,
    ) -> ContextWrapper<&'a mut Self, &'b mut Ctx> {
        ContextWrapper { inner: self, ctx }
    }
}

// 为所有类型 Blanket impl
impl<T: ?Sized> WithContext for T {}

// 可变自动转换至只读
impl<'a, I, Ctx> From<&'a ContextWrapper<&'a mut I, &'a mut Ctx>>
    for ContextWrapper<&'a I, &'a Ctx>
{
    fn from(value: &'a ContextWrapper<&'a mut I, &'a mut Ctx>) -> Self {
        Self {
            inner: value.inner,
            ctx: value.ctx,
        }
    }
}

impl<'a, I, Ctx> ContextWrapper<&'a mut I, &'a mut Ctx> {
    /// 可变引用转换为只读引用
    pub fn readonly(&'a self) -> ContextWrapper<&'a I, &'a Ctx> {
        Into::<ContextWrapper<&'a I, &'a Ctx>>::into(self)
    }
}

/// 明确上下文与目标类型，主要用于函数声明
///
/// 通过 Blanket impl 自动为只读和可变 [`ContextWrapper`] 实现本特征
///
/// todo 确认用到 ContextWrapper 的地方都有哪些，是否可以删除 ContextWrapper
pub trait WithInto<Ctx, Target> {
    fn with_into(self, ctx: Ctx) -> Target;
}

impl<'a, 'b, I, Ctx> WithInto<&'b Ctx, ContextWrapper<&'a I, &'b Ctx>> for &'a I {
    fn with_into(self, ctx: &'b Ctx) -> ContextWrapper<&'a I, &'b Ctx> {
        self.with_ctx(ctx)
    }
}

impl<'a, 'b, I, Ctx> WithInto<&'b mut Ctx, ContextWrapper<&'a mut I, &'b mut Ctx>> for &'a mut I {
    fn with_into(self, ctx: &'b mut Ctx) -> ContextWrapper<&'a mut I, &'b mut Ctx> {
        self.with_ctx_mut(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_ctx_into() {
        // 任意类型
        struct Foo;
        struct Bar;

        // 只读特征
        trait Baz {
            fn baz(&self);
        }

        // 只读包装实现只读特征
        impl Baz for ContextWrapper<&Foo, &Bar> {
            fn baz(&self) {}
        }

        // 要求能够生成只读特征
        fn need_baz<Source, Ctx, Target>(source: Source, ctx: Ctx)
        where
            Source: WithInto<Ctx, Target>,
            Target: Baz,
        {
            source.with_into(ctx).baz();
        }

        let foo = Foo;
        let bar = Bar;

        need_baz(&foo, &bar);

        // 可变特征
        trait Qux {
            fn qux(&mut self);
        }

        // 可变包装实现可变特征
        impl Qux for ContextWrapper<&mut Foo, &mut Bar> {
            fn qux(&mut self) {}
        }

        // 要求能生成可变特征
        fn need_qux<Source, Ctx, Target>(source: Source, ctx: Ctx)
        where
            Source: WithInto<Ctx, Target>,
            Target: Qux,
        {
            source.with_into(ctx).qux();
        }

        let mut foo = Foo;
        let mut bar = Bar;

        need_qux(&mut foo, &mut bar);
    }
}
