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

    /// 确定性伪随机数生成器（LCG），保证压力测试可复现
    fn lcg_next(state: &mut u64) -> u64 {
        // Knuth 推荐的数值稳定、周期足够长的 LCG 参数
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    /// 压力测试：对容器的每个方法随机反复执行，每步之后校验
    /// - `ele_len` 与底层数组实际存活数一致
    /// - `hole_count` 与底层数组实际空洞数一致（即 `ele_len` 的准确性）
    /// - 迭代内容 / `ele_empty` / `is_changed` 与对照模型一致
    #[test]
    fn test_stress_random_ops_ele_len_accuracy() {
        const ITERS: u32 = 2000;
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;

        let mut c = UpsertContainer::<TestEff>::default();
        // 对照模型：当前存活 id 集合（id 唯一，用 Vec 保存）
        let mut live: Vec<u32> = Vec::new();
        let mut next_id: u32 = 0;
        // 预期 changed_flag：仅成功修改容器的操作会置位
        let mut changed: bool = false;

        /// upsert：约 1/3 概率更新已有 id，否则插入新 id
        fn do_upsert(
            c: &mut UpsertContainer<TestEff>,
            live: &mut Vec<u32>,
            next_id: &mut u32,
            rng: &mut u64,
        ) {
            let pick_existing = !live.is_empty() && lcg_next(rng) % 3 == 0;
            let id = if pick_existing {
                live[(lcg_next(rng) % live.len() as u64) as usize]
            } else {
                let id = *next_id;
                *next_id += 1;
                live.push(id);
                id
            };
            c.upsert_ele(TestEff::new(id), |old, new| old.val += new.val);
        }

        /// delete：优先删存活的 id，偶尔删不存在的 id（校验幂等返回 false）
        /// 返回是否真正发生了删除
        fn do_delete(c: &mut UpsertContainer<TestEff>, live: &mut Vec<u32>, rng: &mut u64) -> bool {
            if live.is_empty() || lcg_next(rng) % 10 == 0 {
                // 删除不存在的 id：应返回 false 且无副作用
                assert!(!delete_by_id(c, 999_999_999));
                return false;
            }
            let idx = (lcg_next(rng) % live.len() as u64) as usize;
            let id = live[idx];
            assert!(delete_by_id(c, id), "删除存活的 id 应成功");
            live.swap_remove(idx);
            true
        }

        /// select_mut：查存活的 id 应命中，查不存在的应返回 None
        /// 返回是否真正命中了元素
        fn do_select_mut(c: &mut UpsertContainer<TestEff>, live: &[u32], rng: &mut u64) -> bool {
            if live.is_empty() {
                assert!(c.select_mut_ele(|e| e.gen_id() == 999_999_999).is_none());
                return false;
            }
            if lcg_next(rng) % 5 == 0 {
                assert!(c.select_mut_ele(|e| e.gen_id() == 999_999_999).is_none());
                return false;
            }
            let id = live[(lcg_next(rng) % live.len() as u64) as usize];
            let found = c.select_mut_ele(|e| e.gen_id() == id).map(|e| {
                e.val += 1.0;
                e.id
            });
            assert_eq!(found, Some(id));
            true
        }

        /// iter_mut：对每个元素自增一次，不影响计数，校验迭代到的个数与 ele_len 一致
        fn do_iter_mut(c: &mut UpsertContainer<TestEff>) {
            let mut visited = 0usize;
            for e in c.iter_mut() {
                e.val += 0.5;
                visited += 1;
            }
            assert_eq!(visited, c.ele_len());
        }

        /// 每步之后校验核心不变量
        fn check_invariants(c: &UpsertContainer<TestEff>, live: &[u32]) {
            let real_len = c.ll.iter().filter(|e| e.is_some()).count();
            let real_holes = c.ll.iter().filter(|e| e.is_none()).count();

            // 核心：ele_len 与底层存活数一致，hole_count 与底层空洞数一致
            assert_eq!(c.ele_len(), real_len, "ele_len 与底层实际存活数不一致");
            assert_eq!(
                c.hole_count, real_holes,
                "hole_count 与底层实际空洞数不一致"
            );
            assert_eq!(c.ele_len(), live.len(), "ele_len 与对照模型不一致");
            assert_eq!(c.ele_empty(), live.is_empty(), "ele_empty 与对照模型不一致");

            // 迭代内容（id 集合）与对照模型一致
            let iter_ids: std::collections::HashSet<u32> = c.iter_ele().map(|e| e.id).collect();
            let live_set: std::collections::HashSet<u32> = live.iter().copied().collect();
            assert_eq!(iter_ids, live_set, "迭代出的元素集合与对照模型不一致");
        }

        for _ in 0..ITERS {
            match lcg_next(&mut rng) % 100 {
                0..=44 => {
                    do_upsert(&mut c, &mut live, &mut next_id, &mut rng);
                    changed = true;
                }
                45..=69 => {
                    if do_delete(&mut c, &mut live, &mut rng) {
                        changed = true;
                    }
                }
                70..=84 => {
                    if do_select_mut(&mut c, &live, &mut rng) {
                        changed = true;
                    }
                }
                85..=89 => do_iter_mut(&mut c),
                90..=96 => c.try_clean_hole(),
                // 97..=99 => 重置脏标记
                _ => {
                    c.reset_changed_flag();
                    changed = false;
                }
            }

            assert_eq!(c.is_changed(), changed, "changed_flag 与预期不一致");
            check_invariants(&c, &live);
        }
    }
}
