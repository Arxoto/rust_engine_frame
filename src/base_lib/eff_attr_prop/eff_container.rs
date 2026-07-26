//! 效果存储使用数组结构，在 20-50 的数量以内，性能比 FxHashMap 优秀（预估）

use std::{fmt::Debug, hash::Hash};

/// 身份标识
pub trait WithId {
    type Id: Eq + Hash + Clone + Debug;
    fn get_id(&self) -> &Self::Id;
}

/// 持久效果的容器
/// - 保证顺序，后插入的在后面，更新的也在最后
/// - 【重要】需要手动遍历判断效果的 Timer 是否过期
/// - 【重要】外部手动定时刷新
///
/// 架构选型问题：面向数据ECS架构 or 面向对象直接持有
/// - ECS 定理：多个同类型的东西都要在每帧做同一件事，那么就应该拆成 Entity + System
/// - 参考 DDD 主要从聚合的角度考虑，一系列足够内聚的数据才应成为 Component 组件，否则不应被拆分
///   - 是否有父级之外的其他 system 去访问？
///   - 大部分 system 是否总是访问他和父级的其他字段？
/// - 是否需要 ECS 的变更检测？若只影响父级则不应独立拆开。
/// - 考虑 archetype 碎片成本，若拆出来的组件只被少量实体拥有，那么就不应该拆。
///
/// 综上所述，设计如下：
/// - 角色作为实体
/// - 属性和属性效果容器作为组件平铺（关键）
/// - 属性效果作为结构体，被属性效果容器持有
/// - 角色效果作为实体被关联，或者放在角色效果容器中，这点另论
///
/// 设计原因：
/// - 属性效果为什么设计为结构体，属性效果容器为什么设计为组件？
///   - 考虑内聚，因为属性效果与属性是强关联的，把属性效果作为实体会存在大量的关联查询，缓存不友好
///   - 考虑架构鲁棒性，属性里面的 Timer 适合作为 Component 被 System 每帧访问，嵌套太深容易遗漏（容器作为组件嵌套不深）
///   - 很少存在“按照效果类型跨角色查询”的场景（适合把效果作为实体），大部分是“按照角色查”的场景（适合结构体）
///   - 效果相对来说是短生命周期的，频繁增删实体存在一定的性能开销
/// - 属性、效果容器等等平铺，而不是嵌套多层，如抽象一层“战斗单元”放所有战斗相关的数据
///   - 贴近 ECS 设计理念
/// - 角色效果作为实体还是结构体被容器管理？
///   - 角色效果举例为：燃烧状态，每秒造成伤害
///   - 效果逻辑适合作为 System ，新增效果解耦合
///   - UI 展示角色所有效果，适合容器管理，作为实体是否合适，待 ECS 熟练后确认 todo
#[derive(Debug, Default)]
pub struct EffectContainer<E: WithId> {
    /// 实际持有的效果
    effects: Vec<Option<E>>,
    /// 空值数量
    hole_count: usize,
}

impl<E: WithId> EffectContainer<E> {
    /// 遍历效果
    pub fn iter_eff(&self) -> impl Iterator<Item = &E> {
        self.effects.iter().filter_map(|e| e.as_ref())
    }

    /// 遍历效果（可变）
    pub fn iter_eff_mut(&mut self) -> impl Iterator<Item = &mut E> {
        self.effects.iter_mut().filter_map(|e| e.as_mut())
    }

    /// 查询效果
    pub fn find_eff(&self, id: &E::Id) -> Option<&E> {
        self.effects
            .iter()
            .filter_map(|e| e.as_ref())
            .find(|e| e.get_id() == id)
    }

    /// 查询效果（可变）
    pub fn find_eff_mut(&mut self, id: &E::Id) -> Option<&mut E> {
        self.effects
            .iter_mut()
            .filter_map(|e| e.as_mut())
            .find(|e| e.get_id() == id)
    }

    /// 查询效果，返回 opt 包裹的效果槽位，槽位逻辑上不可能为空
    fn locate_eff_slot(&mut self, id: &E::Id) -> Option<&mut Option<E>> {
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
    pub fn del_eff_by(&mut self, id: &E::Id) {
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
