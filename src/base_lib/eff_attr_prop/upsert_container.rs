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
#[derive(Debug)]
pub struct UpsertContainer<E: Upsert> {
    /// 实际持有的效果
    ll: Vec<Option<E>>,
    /// 空值数量
    hole_count: usize,
    /// 脏标记，记录是否被修改
    changed_flag: bool,
}

// 为了支持没有实现 Default 的 E ，手动实现
impl<E: Upsert> Default for UpsertContainer<E> {
    fn default() -> Self {
        Self {
            ll: Default::default(),
            hole_count: Default::default(),
            changed_flag: false,
        }
    }
}

impl<E: Upsert> UpsertContainer<E> {
    /// 遍历效果
    pub fn iter_ele(&self) -> impl Iterator<Item = &E> {
        self.ll.iter().filter_map(|e| e.as_ref())
    }

    /// 查询效果，返回 opt 包裹的效果槽位，槽位逻辑上不可能为空
    fn locate_slot<F>(ll: &mut Vec<Option<E>>, find_logic: F) -> Option<&mut Option<E>>
    where
        F: Fn(&E) -> bool,
    {
        ll.iter_mut().find(|e| e.as_ref().is_some_and(&find_logic))
    }

    /// 尝试添加效果，若已有效果则对其进行修改，如进行堆叠操作等，修改后的效果会被排到最后
    pub fn upsert_ele<F>(&mut self, new_ele: E, update_logic: F)
    where
        F: Fn(&mut E, &E),
    {
        let located_slot = Self::locate_slot(&mut self.ll, |ele| ele.has_same_id(&new_ele));
        if let Some(old_ele_slot) = located_slot {
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
        self.changed_flag = true;
    }

    /// 删除效果（幂等：重复删除无副作用）
    pub fn delete_ele<F>(&mut self, find_logic: F) -> bool
    where
        F: Fn(&E) -> bool,
    {
        if let Some(ele_slot) = Self::locate_slot(&mut self.ll, find_logic) {
            *ele_slot = None;
            self.hole_count += 1;
            self.changed_flag = true;
            true
        } else {
            false
        }
    }

    /// 查询以更新（无论是否修改都会标记容器内元素被修改，因此应避免只读查询，尽量通过容器实现，或单独实现只读查询）
    pub fn select_mut_ele<F>(&mut self, find_logic: F) -> Option<&mut E>
    where
        F: Fn(&E) -> bool,
    {
        if let Some(ele_slot) = Self::locate_slot(&mut self.ll, find_logic) {
            self.changed_flag = true;
            ele_slot.as_mut()
        } else {
            None
        }
    }

    /// 获取当前效果个数
    pub fn ele_len(&self) -> usize {
        self.ll.len() - self.hole_count
    }

    pub fn ele_empty(&self) -> bool {
        self.ele_len() == 0
    }

    /// 清理空洞，让元素排列紧凑，不影响容量
    ///
    /// 阈值触发，若空洞数过少或空洞率很低，则不进行任何操作
    ///
    /// 应该定时去执行，如 5s 清理一次
    pub fn try_clean_hole(&mut self) {
        if self.hole_count < 3 {
            // 空洞数过少则不回收
            return;
        } else if self.hole_count > 50 {
            // 异常膨胀，无论比率如何，都进行压缩
            self.do_clean_hole();
            return;
        }

        // 空洞率达到 25% 才压缩
        if self.hole_count * 4 >= self.ll.len() {
            self.do_clean_hole();
        }
    }

    /// 保留非空值，不修改数组容量
    fn do_clean_hole(&mut self) {
        self.ll.retain(|e| e.is_some());
        self.hole_count = 0;
    }

    pub fn reset_changed_flag(&mut self) {
        self.changed_flag = false;
    }

    pub fn is_changed(&self) -> bool {
        self.changed_flag
    }
}

/// 集合定时清理工具
#[derive(Debug, Default)]
pub struct UpsertContainerCleaner {
    do_clean_time: f64,
}

impl UpsertContainerCleaner {
    /// 默认 5s 刷新一次
    pub const fn get_default_period() -> f64 {
        5.0
    }

    pub fn should_clean_hole(&mut self, delta: f64, period: f64) -> bool {
        self.do_clean_time += delta;
        if self.do_clean_time > period {
            self.do_clean_time = 0.0;
            true
        } else {
            false
        }
    }

    pub fn do_clean_hole<E: Upsert>(&mut self, container: &mut UpsertContainer<E>) {
        container.try_clean_hole();
    }
}

// todo test clean, test hole_count
