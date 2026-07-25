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
