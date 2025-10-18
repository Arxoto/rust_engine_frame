use crate::{cores::unify_type::FixedName, effects::duration_effect::ProxyDurationEffect};

/// 持久效果的容器
/// - 保证顺序，后插入的在后面
#[derive(Debug)]
pub struct EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    effects: Vec<Option<E>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S, E> Default for EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            _marker: Default::default(),
        }
    }
}

impl<S, E> EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    pub fn new() -> Self {
        Default::default()
    }

    /// 更新排序
    ///
    /// 至少需要定时刷新（否则会有内存泄漏问题），不刷新也不会有逻辑问题
    pub fn refresh_capacity(&mut self) {
        // 提前确定容量
        let min_cap = self.effects.iter().filter(|e| e.is_some()).count();
        let mut new_effects = Vec::with_capacity(min_cap.next_power_of_two());

        std::mem::swap(&mut new_effects, &mut self.effects); // 先交换，因为后面会获取所有权
        let old_effects = new_effects; // 修正名称符合语义

        for ele in old_effects.into_iter() {
            if ele.is_some() {
                self.effects.push(ele);
            }
        }
    }

    /// 返回效果名称数组 根据刷新时间排序 新装载的在后面
    pub fn keys(&self) -> Vec<S> {
        self.effects
            .iter()
            .filter_map(|e| e.as_ref())
            .map(|e| e.get_effect_name().clone())
            .collect()
    }

    pub fn get_effect(&self, s: &S) -> Option<&E> {
        self.effects
            .iter()
            .filter_map(|e| e.as_ref())
            .find(|e| e.get_effect_name() == s)
    }

    pub fn get_effect_mut(&mut self, s: &S) -> Option<&mut E> {
        self.effects
            .iter_mut()
            .filter_map(|e| e.as_mut())
            .find(|e| e.get_effect_name() == s)
    }

    /// - 返回 None 说明没找到
    /// - 返回 Some(Some) 说明找到了并且可以对其进行修改
    /// - 返回 Some(None) 不可能，因为 find 条件中暗含了必须是 Some(Some)
    fn get_effect_mut_inner(&mut self, s: &S) -> Option<&mut Option<E>> {
        self.effects
            .iter_mut()
            .find(|e| e.as_ref().map(|e| e.get_effect_name()) == Some(s))
    }

    /// 卸载一个效果
    ///
    /// 之后需手动调用 [`Self::refresh_order_keys`] 以刷新 [`Self::sorted_keys`]
    pub fn del_effect(&mut self, s: &S) {
        let the_eff = self.get_effect_mut_inner(s);
        if let Some(the_eff) = the_eff {
            *the_eff = None;
        }
    }

    /// 装载一个效果
    ///
    /// 之后需手动调用 [`Self::refresh_order_keys`] 以刷新 [`Self::sorted_keys`]
    pub fn put_or_stack_effect(&mut self, eff: E) {
        let the_eff = self.get_effect_mut_inner(eff.get_effect_name());
        if let Some(the_eff) = the_eff {
            // 找到了 堆叠并后置
            if let Some(the_eff) = the_eff {
                // 始终进入
                the_eff.refresh_with_name_value_stack(&eff);
            }
            let new_eff = the_eff.take();
            self.effects.push(new_eff);
        } else {
            // 没找到 直接新增
            self.effects.push(Some(eff));
        }
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use crate::effects::{
        duration_effect::EffectBuilder, native_duration::ProxyDuration, native_effect::ProxyEffect,
    };

    use super::*;

    #[test]
    fn refresh_order_keys() {
        let mut container = EffectContainer::new();

        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "1", 1.0));
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "2", 1.0));
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "a", 1.0));
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "b", 1.0));
        assert_eq!(container.keys(), ["1", "2", "a", "b"]);
        assert_eq!(container.effects.capacity(), 4);

        // 触发重新排序
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "1", 1.0));
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "b", 1.0));
        assert_eq!(container.keys(), ["2", "a", "1", "b"]);
        assert_eq!(container.effects.capacity(), 8); // 翻倍扩容

        // 以上 不使用 refresh_order_keys 逻辑上也不会出错
        // 下面 验证刷新的准确性
        container.refresh_capacity();
        assert_eq!(container.keys(), ["2", "a", "1", "b"]);
        assert_eq!(container.effects.capacity(), 4);

        // 卸载效果
        container.del_effect(&"2");
        container.del_effect(&"1");
        assert_eq!(container.keys(), ["a", "b"]);
        assert_eq!(container.effects.capacity(), 4); // 容量没变

        container.refresh_capacity();
        assert_eq!(container.keys(), ["a", "b"]);
        assert_eq!(container.effects.capacity(), 2);
    }

    #[test]
    fn test_func() {
        let mut effect_container = EffectContainer::new();
        let _ = effect_container.get_effect(&""); // 提前推理类型

        // put
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "1", 1.0));
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "2", 1.0));
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "3", 1.0));
        effect_container.refresh_capacity();
        let mut effect_names = effect_container.keys();
        effect_names.sort();
        assert_eq!(effect_names, ["1", "2", "3"]);

        // del
        effect_container.del_effect(&"2");
        effect_container.refresh_capacity();
        let mut effect_names = effect_container.keys();
        effect_names.sort();
        assert_eq!(effect_names, ["1", "3"]);

        // get
        let effect = effect_container.get_effect_mut(&"1").unwrap();
        assert_eq!(effect.get_stack(), 1);

        // mut max_stack
        effect.set_max_stack(2);

        // stack
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("bbb", "1", 2.0));
        effect_container.refresh_capacity();

        let effect = effect_container.get_effect_mut(&"1").unwrap();
        assert_eq!(effect.get_from_name(), &"bbb");
        assert_eq!(effect.get_value(), 2.0);
        assert_eq!(effect.get_stack(), 2);
    }
}
