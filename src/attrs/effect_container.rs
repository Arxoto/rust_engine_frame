use std::collections::HashMap;

use crate::{cores::unify_type::FixedName, effects::duration_effect::ProxyDurationEffect};

/// 持久效果的容器 key是效果名称
pub struct EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    effects: HashMap<S, E>,
}

impl<S, E> EffectContainer<S, E>
where
    S: FixedName,
    E: ProxyDurationEffect<S>,
{
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    /// 返回效果名称数组 根据字典排序
    pub fn each_effect_names(&self) -> Vec<S> {
        let v = self.effects.keys().map(Clone::clone).collect::<Vec<S>>();
        // todo sorted
        // v.sort_unstable();
        v
    }

    pub fn put_or_stack_effect(&mut self, mut eff: E) {
        let eff_name = eff.get_effect_name();
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

    pub fn del_effect(&mut self, s: &S) {
        self.effects.remove(s);
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use crate::effects::{
        duration_effect::DurationEffect, native_duration::ProxyDuration, native_effect::ProxyEffect,
    };

    use super::*;

    #[test]
    fn test_func() {
        let mut effect_container = EffectContainer::new();

        effect_container.put_or_stack_effect(DurationEffect::new_infinite("aaa", "1", 1.0));
        effect_container.put_or_stack_effect(DurationEffect::new_infinite("aaa", "2", 1.0));
        effect_container.put_or_stack_effect(DurationEffect::new_infinite("aaa", "3", 1.0));
        let mut effect_names = effect_container.each_effect_names();
        effect_names.sort();
        assert_eq!(effect_names, ["1", "2", "3"]);

        effect_container.del_effect(&"2");
        let mut effect_names = effect_container.each_effect_names();
        effect_names.sort();
        assert_eq!(effect_names, ["1", "3"]);

        let effect = effect_container.get_effect_mut(&"1").unwrap();
        assert_eq!(effect.get_stack(), 1);

        effect.set_max_stack(2);
        effect_container.put_or_stack_effect(DurationEffect::new_infinite("bbb", "1", 2.0));

        let effect = effect_container.get_effect_mut(&"1").unwrap();
        assert_eq!(effect.get_from_name(), &"bbb");
        assert_eq!(effect.get_value(), 2.0);
        assert_eq!(effect.get_stack(), 2);
    }
}
