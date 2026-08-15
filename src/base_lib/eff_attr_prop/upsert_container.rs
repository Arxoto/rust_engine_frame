//! 效果存储使用数组结构，在 20-50 的数量以内，性能比 FxHashMap 优秀（预估）

use std::{fmt::Debug, hash::Hash};

use crate::base_lib::cores::unify_types::time_type;

/// 可合并类型
pub trait Upsert {
    type Id: Eq + Hash + Clone + Debug;

    /// 获取 id ，为能够自由组合字段，返回克隆后的所有权
    fn gen_id(&self) -> Self::Id;

    /// 为快速比较，避免多次获取 id 引起不必要的克隆
    fn matched_id(&self, id: &Self::Id) -> bool;

    /// 为快速比较，避免多次获取 id 引起不必要的克隆
    fn has_same_id(&self, other: &Self) -> bool;

    /// 直接替换，若想实现特殊效果（效果堆叠等）自己实现
    #[inline]
    fn replace(old: &mut Self, new: Self)
    where
        Self: Sized,
    {
        *old = new;
    }
}

/// 持久效果的容器
/// - 根据插入顺序排序，不是更新顺序
/// - 【重要】为防止空洞数过多，建议手动定时刷新
/// - 【重要】若元素有过期老化，需要手动遍历判断
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
    /// 只读遍历
    pub fn iter_ele(&self) -> impl Iterator<Item = &E> {
        self.ll.iter().filter_map(|e| e.as_ref())
    }

    /// 可变遍历
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut E> {
        // 无法保证在可变遍历的情况下保证更新顺序排序
        self.ll.iter_mut().filter_map(|e| e.as_mut())
    }

    /// 定位插槽，返回 opt 包裹的效果槽位，槽位逻辑上不可能为空
    fn locate_slot<F>(ll: &mut [Option<E>], find_logic: F) -> Option<&mut Option<E>>
    where
        F: Fn(&E) -> bool,
    {
        ll.iter_mut().find(|e| e.as_ref().is_some_and(&find_logic))
    }

    /// 添加或更新，若已有效果则对其进行修改，如进行堆叠操作等，修改后的效果会被排到最后
    pub fn upsert_ele<F>(&mut self, new_ele: E, update_logic: F)
    where
        F: Fn(&mut E, E),
    {
        let located_slot = Self::locate_slot(&mut self.ll, |ele| ele.has_same_id(&new_ele));
        if let Some(old_ele_slot) = located_slot {
            // 槽位逻辑上不可能为空
            if let Some(old_ele) = old_ele_slot {
                update_logic(old_ele, new_ele);

                // // 若实现更新顺序排序：刷新后后置，旧槽位置空
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

    /// 删除（幂等：重复删除无副作用）
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
        // 若实现更新顺序排序，需要先 take 然后后置
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
    do_clean_time: time_type::T,
}

impl UpsertContainerCleaner {
    pub fn should_clean_hole(&mut self, delta: time_type::T, period: time_type::T) -> bool {
        self.do_clean_time += delta;
        if self.do_clean_time > period {
            self.do_clean_time = time_type::ZERO;
            true
        } else {
            false
        }
    }

    pub fn do_clean_hole<E: Upsert>(&mut self, container: &mut UpsertContainer<E>) {
        container.try_clean_hole();
    }
}

#[cfg(test)]
mod tests {
    use crate::base_lib::cores::unify_types::time_type;

    use super::*;

    /// 测试用的最小效果类型
    #[derive(Debug)]
    struct TestEff {
        id: u32,
        val: f64,
    }

    impl TestEff {
        fn new(id: u32) -> Self {
            Self { id, val: 0.0 }
        }
    }

    impl Upsert for TestEff {
        type Id = u32;

        fn gen_id(&self) -> Self::Id {
            self.id
        }

        fn matched_id(&self, id: &Self::Id) -> bool {
            self.id == *id
        }

        fn has_same_id(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    /// 删除指定 id 的辅助函数
    fn delete_by_id<E: Upsert<Id = u32>>(container: &mut UpsertContainer<E>, id: u32) -> bool {
        container.delete_ele(|e| e.gen_id() == id)
    }

    /// upsert：新增元素并标记脏
    #[test]
    fn test_upsert_add() {
        let mut c = UpsertContainer::<TestEff>::default();
        assert!(c.ele_empty());
        assert!(!c.is_changed());

        c.upsert_ele(TestEff::new(1), |_, _| {});
        c.upsert_ele(TestEff::new(2), |_, _| {});
        assert_eq!(c.ele_len(), 2);
        assert!(!c.ele_empty());
        assert!(c.is_changed());
    }

    /// upsert：同 id 更新而非新增，且更新逻辑生效
    #[test]
    fn test_upsert_update_same_id() {
        let mut c = UpsertContainer::<TestEff>::default();
        c.upsert_ele(TestEff::new(1), |_, _| {});
        c.upsert_ele(TestEff { id: 1, val: 3.0 }, |old, new| old.val += new.val);

        assert_eq!(c.ele_len(), 1);
        let eff = c.iter_ele().next().unwrap();
        assert_eq!(eff.val, 3.0);
    }

    /// upsert：同 id 重复插入不会增加个数
    #[test]
    fn test_upsert_idempotent_count() {
        let mut c = UpsertContainer::<TestEff>::default();
        c.upsert_ele(TestEff::new(1), |_, _| {});
        c.upsert_ele(TestEff::new(1), |_, _| {});
        c.upsert_ele(TestEff::new(1), |_, _| {});
        assert_eq!(c.ele_len(), 1);
    }

    /// delete：删除成功返回 true 并产生空洞
    #[test]
    fn test_delete_marks_hole() {
        let mut c = UpsertContainer::<TestEff>::default();
        c.upsert_ele(TestEff::new(1), |_, _| {});
        c.upsert_ele(TestEff::new(2), |_, _| {});

        assert!(delete_by_id(&mut c, 1));
        assert_eq!(c.ele_len(), 1);
        assert_eq!(c.hole_count, 1);
        // 底层数组长度不变，产生空洞
        assert_eq!(c.ll.len(), 2);
    }

    /// delete：重复删除幂等，返回 false
    #[test]
    fn test_delete_idempotent() {
        let mut c = UpsertContainer::<TestEff>::default();
        c.upsert_ele(TestEff::new(1), |_, _| {});

        assert!(delete_by_id(&mut c, 1));
        assert!(!delete_by_id(&mut c, 1));
        assert_eq!(c.hole_count, 1);
        assert_eq!(c.ele_len(), 0);
    }

    /// 空洞数过少（< 3）不回收
    #[test]
    fn test_clean_hole_too_few_no_op() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..8 {
            c.upsert_ele(TestEff::new(id), |_, _| {});
        }
        delete_by_id(&mut c, 0);
        delete_by_id(&mut c, 1);
        assert_eq!(c.hole_count, 2);

        c.try_clean_hole();
        assert_eq!(c.hole_count, 2);
        assert_eq!(c.ll.len(), 8);
        assert_eq!(c.ele_len(), 6);
    }

    /// 空洞率达到 25% 时回收
    #[test]
    fn test_clean_hole_at_ratio() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..8 {
            c.upsert_ele(TestEff::new(id), |_, _| {});
        }
        delete_by_id(&mut c, 0);
        delete_by_id(&mut c, 1);
        delete_by_id(&mut c, 2);
        assert_eq!(c.hole_count, 3);

        c.try_clean_hole();
        assert_eq!(c.hole_count, 0);
        assert_eq!(c.ll.len(), 5);
        assert_eq!(c.ele_len(), 5);
    }

    /// 空洞率不足 25% 时不回收
    #[test]
    fn test_clean_hole_below_ratio_no_op() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..20 {
            c.upsert_ele(TestEff::new(id), |_, _| {});
        }
        delete_by_id(&mut c, 0);
        delete_by_id(&mut c, 1);
        delete_by_id(&mut c, 2);
        assert_eq!(c.hole_count, 3);

        c.try_clean_hole();
        // 3 * 4 = 12 < 20，不回收
        assert_eq!(c.hole_count, 3);
        assert_eq!(c.ll.len(), 20);
    }

    /// 空洞数超过 50 时无条件回收（忽略空洞率）
    #[test]
    fn test_clean_hole_overflow_clean() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..400 {
            c.upsert_ele(TestEff::new(id), |_, _| {});
        }
        for id in 0..51 {
            delete_by_id(&mut c, id);
        }
        assert_eq!(c.hole_count, 51);
        assert_eq!(c.ll.len(), 400);

        c.try_clean_hole();
        assert_eq!(c.hole_count, 0);
        assert_eq!(c.ll.len(), 349);
    }

    /// 回收不影响迭代顺序与内容
    #[test]
    fn test_clean_hole_preserves_order() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..8 {
            c.upsert_ele(TestEff { id, val: id as f64 }, |_, _| {});
        }
        delete_by_id(&mut c, 0);
        delete_by_id(&mut c, 2);
        delete_by_id(&mut c, 4);

        let order_before: Vec<u32> = c.iter_ele().map(|e| e.id).collect();
        c.try_clean_hole();
        let order_after: Vec<u32> = c.iter_ele().map(|e| e.id).collect();

        assert_eq!(order_before, order_after);
        assert_eq!(order_after, vec![1, 3, 5, 6, 7]);
    }

    /// 回收不改变数组容量
    #[test]
    fn test_clean_hole_keeps_capacity() {
        let mut c = UpsertContainer::<TestEff>::default();
        for id in 0..8 {
            c.upsert_ele(TestEff::new(id), |_, _| {});
        }
        let cap_before = c.ll.capacity();
        for id in 0..3 {
            delete_by_id(&mut c, id);
        }

        c.try_clean_hole();
        assert_eq!(c.ll.capacity(), cap_before);
    }

    /// 定时清理器：时间累积超过周期才触发并重置
    #[test]
    fn test_cleaner_should_clean_hole_period() {
        let mut cleaner = UpsertContainerCleaner::default();
        let period = time_type::unit::<5>();
        // 累积 5s 刚好等于周期，不触发
        for _ in 0..5 {
            assert!(!cleaner.should_clean_hole(time_type::unit::<1>(), period));
        }
        // 超过周期，触发并重置
        assert!(cleaner.should_clean_hole(time_type::unit::<1>(), period));
        assert!(!cleaner.should_clean_hole(time_type::unit::<1>(), period));
    }

    /// 定时清理器：自定义周期
    #[test]
    fn test_cleaner_custom_period() {
        let mut cleaner = UpsertContainerCleaner::default();
        let period = time_type::unit::<2>();
        assert!(!cleaner.should_clean_hole(time_type::unit::<1>(), period));
        assert!(!cleaner.should_clean_hole(time_type::unit::<1>(), period)); // 恰好 2s，不触发
        assert!(cleaner.should_clean_hole(time_type::unit::<1>(), period)); // 3s > 2s，触发
        assert!(!cleaner.should_clean_hole(time_type::unit::<1>(), period)); // 已重置
    }
}
