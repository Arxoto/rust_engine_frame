use std::collections::HashMap;

use crate::{cores::unify_type::FixedName, effects::duration_effect::ProxyDurationEffect};

/// 持久效果的容器 key是效果名称
#[derive(Debug)]
pub struct EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    effects: HashMap<S, E>,
    key_order_map: HashMap<S, usize>,
    sorted_keys: Vec<S>,
    next_order: usize,
}

impl<S, E> Default for EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    fn default() -> Self {
        Self {
            effects: HashMap::new(),
            key_order_map: HashMap::new(),
            next_order: 0,
            sorted_keys: Vec::new(),
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

    /// put 时更新 [`Self::key_order_map`] 和 [`Self::next_order`]
    fn refresh_when_put(&mut self, k: &S) {
        if let Some(order) = self.key_order_map.get_mut(k) {
            *order = self.next_order;
        } else {
            self.key_order_map.insert(k.clone(), self.next_order);
        }
        self.next_order += 1;
    }

    /// del 时更新 [`Self::key_order_map`]
    fn refresh_when_del(&mut self, k: &S) {
        self.key_order_map.remove(k);
    }

    /// 更新排序值 每次变更后都需要刷新（否则 [`Self::sorted_keys`] 出错）
    ///
    /// 更新 [`Self::key_order_map`] [`Self::next_order`] [`Self::sorted_keys`]
    pub fn refresh_order_keys(&mut self) {
        // 根据现存的 key_order_map 生成新的 sorted_keys
        let mut new_keys: Vec<(S, usize)> = self
            .key_order_map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        new_keys.sort_by(|a, b| a.1.cmp(&b.1));
        self.sorted_keys = new_keys.into_iter().map(|(k, _v)| k).collect();

        // 刷新 key_order_map 中的 order
        for (order, k) in self.sorted_keys.iter().enumerate() {
            if let Some(ele) = self.key_order_map.get_mut(k) {
                *ele = order;
            }
        }

        // 更新 next_order
        self.next_order = self.sorted_keys.iter().count();
    }

    /// 返回效果名称数组 根据刷新时间排序 新装载的在后面
    pub fn keys(&self) -> Vec<S> {
        // self.effects.keys().map(Clone::clone).collect::<Vec<S>>()
        // v.sort_unstable();
        self.sorted_keys.clone()
    }

    /// 装载一个效果
    ///
    /// 之后需手动调用 [`Self::refresh_order_keys`] 以刷新 [`Self::sorted_keys`]
    pub fn put_or_stack_effect(&mut self, mut eff: E) {
        let eff_name = eff.get_effect_name();
        self.refresh_when_put(eff_name);

        if let Some(the_eff) = self.effects.get_mut(eff_name) {
            the_eff.refresh_with_name_value_stack(&eff);
            return;
        } else {
            let eff_name = eff_name.clone(); // 在这里结束不可变引用 允许下面使用可变引用
            eff.restart_life();
            self.effects.insert(eff_name, eff);
        }
    }

    pub fn get_effect(&self, s: &S) -> Option<&E> {
        self.effects.get(s)
    }

    pub fn get_effect_mut(&mut self, s: &S) -> Option<&mut E> {
        self.effects.get_mut(s)
    }

    /// 卸载一个效果
    ///
    /// 之后需手动调用 [`Self::refresh_order_keys`] 以刷新 [`Self::sorted_keys`]
    pub fn del_effect(&mut self, s: &S) {
        self.refresh_when_del(s);
        self.effects.remove(s);
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
        container.refresh_when_put(&"a");
        container.refresh_when_put(&"b");
        container.refresh_order_keys();
        assert_eq!(container.keys(), ["1", "2", "a", "b"]);

        // 触发重新排序
        container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "1", 1.0));
        container.refresh_when_put(&"b");
        container.refresh_order_keys();
        assert_eq!(container.keys(), ["2", "a", "1", "b"]);

        // 卸载效果
        container.del_effect(&"2");
        container.refresh_when_del(&"1");
        container.refresh_order_keys();
        assert_eq!(container.keys(), ["a", "b"]);
    }

    #[test]
    fn test_func() {
        let mut effect_container = EffectContainer::new();
        let _ = effect_container.get_effect(&""); // 提前推理类型

        // put
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "1", 1.0));
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "2", 1.0));
        effect_container.put_or_stack_effect(EffectBuilder::new_infinite("aaa", "3", 1.0));
        effect_container.refresh_order_keys();
        let mut effect_names = effect_container.keys();
        effect_names.sort();
        assert_eq!(effect_names, ["1", "2", "3"]);

        // del
        effect_container.del_effect(&"2");
        effect_container.refresh_order_keys();
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
        effect_container.refresh_order_keys();

        let effect = effect_container.get_effect_mut(&"1").unwrap();
        assert_eq!(effect.get_from_name(), &"bbb");
        assert_eq!(effect.get_value(), 2.0);
        assert_eq!(effect.get_stack(), 2);
    }
}
