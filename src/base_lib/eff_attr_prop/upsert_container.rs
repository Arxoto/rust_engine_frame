//! 效果存储使用数组结构，在 20-50 的数量以内，性能比 FxHashMap 优秀（预估）

use std::{fmt::Debug, hash::Hash};

/// 可合并类型
pub trait Upsert {
    type Id: Eq + Hash + Clone + Debug;

    /// 获取 id ，为能够自由组合字段，返回克隆后的所有权
    fn get_id(&self) -> Self::Id;

    /// 为快速比较，避免多次获取 id 引起不必要的克隆
    fn matched_id(&self, id: &Self::Id) -> bool;

    /// 为快速比较，避免多次获取 id 引起不必要的克隆
    fn has_same_id(&self, other: &Self) -> bool;
}

/// 持久效果的容器
/// - 根据插入顺序排序，不是更新顺序
/// - 【重要】需要手动遍历判断效果的 Timer 是否过期
/// - 【重要】外部手动定时刷新
#[derive(Debug, Default)]
pub struct UpsertContainer<E: Upsert> {
    /// 实际持有的效果
    ll: Vec<Option<E>>,
    /// 空值数量
    hole_count: usize,
}

impl<E: Upsert> UpsertContainer<E> {
    /// 遍历效果
    pub fn iter_ele(&self) -> impl Iterator<Item = &E> {
        self.ll.iter().filter_map(|e| e.as_ref())
    }

    /// 查询效果，返回 opt 包裹的效果槽位，槽位逻辑上不可能为空
    fn locate_slot<F>(&mut self, find_logic: F) -> Option<&mut Option<E>>
    where
        F: Fn(&E) -> bool,
    {
        self.ll
            .iter_mut()
            .find(|e| e.as_ref().is_some_and(&find_logic))
    }

    /// 尝试添加效果，若已有效果则对其进行修改，如进行堆叠操作等，修改后的效果会被排到最后
    pub fn upsert_ele<F>(&mut self, new_ele: E, update_logic: F)
    where
        F: Fn(&mut E, &E),
    {
        if let Some(old_ele_slot) = self.locate_slot(|ele| ele.has_same_id(&new_ele)) {
            // 槽位逻辑上不可能为空
            if let Some(old_ele) = old_ele_slot {
                update_logic(old_ele, &new_ele);

                // // 若需要保证更新顺序：刷新后后置，旧槽位置空
                // let merged_ele = old_ele_slot.take();
                // self.ll.push(merged_ele);
                // self.hole_count += 1;
            }
        } else {
            // do put
            self.ll.push(Some(new_ele));
        }
    }

    /// 查询以更新
    pub fn select_mut_ele<F>(&mut self, find_logic: F) -> Option<&mut E>
    where
        F: Fn(&E) -> bool,
    {
        if let Some(ele_slot) = self.locate_slot(find_logic) {
            ele_slot.as_mut()
        } else {
            None
        }
    }

    /// 删除效果（幂等：重复删除无副作用）
    pub fn delete_ele<F>(&mut self, find_logic: F) -> bool
    where
        F: Fn(&E) -> bool,
    {
        if let Some(ele_slot) = self.locate_slot(find_logic) {
            *ele_slot = None;
            self.hole_count += 1;
            true
        } else {
            false
        }
    }

    /// 获取当前效果个数
    pub fn ele_len(&self) -> usize {
        self.ll.len() - self.hole_count
    }

    pub fn ele_empty(&self) -> bool {
        self.ele_len() == 0
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
        if self.hole_count * 4 >= self.ll.len() {
            self.do_clean();
        }
    }

    /// 保留非空值，不修改数组容量
    fn do_clean(&mut self) {
        self.ll.retain(|e| e.is_some());
        self.hole_count = 0;
    }
}

#[derive(Debug, Default)]
pub struct UpsertContainerCleaner {
    do_clean_time: f64,
}

impl UpsertContainerCleaner {
    /// 默认 5s 刷新一次
    pub const fn get_default_period() -> f64 {
        5.0
    }

    pub fn check_clean<E: Upsert>(
        &mut self,
        container: &mut UpsertContainer<E>,
        current_time: f64,
        period: f64,
    ) {
        if self.do_clean_time < current_time {
            self.do_clean_time += period;
            container.try_clean();
        }
    }
}

// todo test clean, test hole_count
