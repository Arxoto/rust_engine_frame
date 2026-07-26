//! 效果存储使用数组结构，在 20-50 的数量以内，性能比 FxHashMap 优秀（预估）

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// 身份标识
pub trait WithId<T: Eq + Hash + Clone + Debug> {
    fn get_id(&self) -> &T;
}

/// 持久效果的容器
/// - 保证顺序，后插入的在后面
#[derive(Debug, Default)]
pub struct EffectContainer<T: Eq + Hash + Clone + Debug, E: WithId<T>> {
    /// 实际持有的效果
    effects: Vec<Option<E>>,
    /// 空值数量
    hole_count: usize,
    /// 幽灵数据
    phantom: PhantomData<T>,
}

impl<T: Eq + Hash + Clone + Debug, E: WithId<T>> EffectContainer<T, E> {
    /// 遍历效果
    pub fn iter_eff(&self) -> impl Iterator<Item = &E> {
        self.effects.iter().filter_map(|e| e.as_ref())
    }

    /// 遍历效果（可变）
    pub fn iter_eff_mut(&mut self) -> impl Iterator<Item = &mut E> {
        self.effects.iter_mut().filter_map(|e| e.as_mut())
    }

    /// 查询效果
    pub fn find_eff(&self, id: &T) -> Option<&E> {
        self.effects
            .iter()
            .filter_map(|e| e.as_ref())
            .find(|e| e.get_id() == id)
    }

    /// 查询效果
    pub fn find_eff_mut(&mut self, id: &T) -> Option<&mut E> {
        self.effects
            .iter_mut()
            .filter_map(|e| e.as_mut())
            .find(|e| e.get_id() == id)
    }

    /// 查询效果，返回 opt 包裹的效果槽位，槽位逻辑上不可能为空
    fn locate_eff_slot(&mut self, id: &T) -> Option<&mut Option<E>> {
        self.effects
            .iter_mut()
            .find(|e| e.as_ref().is_some_and(|inner| inner.get_id() == id))
    }

    /// 尝试添加效果，若已有效果则对其进行修改，如进行堆叠操作等
    pub fn upsert_eff<F>(&mut self, new_eff: E, mut f: F)
    where
        F: FnMut(&mut E, E),
    {
        let id = new_eff.get_id();
        if let Some(old_eff_slot) = self.locate_eff_slot(id) {
            // 槽位逻辑上不可能为空
            if let Some(old_eff) = old_eff_slot {
                f(old_eff, new_eff);

                // 刷新后后置，旧槽位置空
                let merged_eff = old_eff_slot.take();
                self.effects.push(merged_eff);
                self.hole_count += 1;
            }
        } else {
            // do put
            self.effects.push(Some(new_eff));
        }
    }

    /// 删除效果（幂等：重复删除无副作用）
    pub fn del_eff_by(&mut self, id: &T) {
        if let Some(eff_slot) = self.locate_eff_slot(id) {
            *eff_slot = None;
            self.hole_count += 1;
        }
    }

    /// 清理容器，去除空值
    ///
    /// 阈值触发，若空洞数过少或空洞率很低，则不进行任何操作
    ///
    /// 应该定时去执行，如 5s 清理一次
    pub fn try_clean(&mut self) {
        if self.hole_count < 3 {
            // 空洞数过少则不回收
            return;
        } else if self.hole_count > 50 {
            // 异常膨胀，无论比率如何，都进行压缩
            self.do_clean();
            return;
        }

        // 空洞率达到 25% 才压缩
        if self.hole_count * 4 >= self.effects.len() {
            self.do_clean();
        }
    }

    /// 保留非空值，不修改数组容量
    fn do_clean(&mut self) {
        self.effects.retain(|e| e.is_some());
        self.hole_count = 0;
    }
}

// todo test clean, test hole_count
