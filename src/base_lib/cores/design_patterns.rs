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
