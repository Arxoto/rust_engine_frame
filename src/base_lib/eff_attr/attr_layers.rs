//! 复合属性
//!
//! ## 设计原因
//!
//! 需求描述
//! - 处理【伤害】效果时，可能影响【血量】【护盾】等多个不同的属性， [`super::bounded_attrs`] 里的方法难以支撑业务复杂性
//! - 为统一抽象，专门封装此模块
//!
//! 需求分析
//! - 效果有多种类型，不同类型可能针对不同的属性；如【物理切割】针对【防护护盾】和【血量】，【奥术】针对【奥术护盾】和【血量】
//! - 不是每一种效果都会针对最底层的【血量】，如【护盾特攻伤害】
//! - 同一帧的处理中，不同类型伤害应用的顺序可能会影响最终伤害体现，应该寻找一个确定的、伤害总量最大的伤害应用方式
//!
//! 原理分析（理论支撑）
//! - 什么情况下会造成伤害浪费？
//!   - 当【物理切割】先计算，把【防护护盾】打空后，应用【防护特攻】，此时因为护盾是空的，且此伤害只针对这个护盾，因此实际伤害为零
//!   - 同时一开始的【物理切割】因为把伤害浪费在【防护护盾】上，也导致对下层的属性伤害减少
//! - “浪费”出现的原因
//!   - 一个效果无法对下层属性生效，并且存在另一个效果能够对下层属性生效，他们总体的伤害溢出了本层，而能对下层属性生效的效果先一步生效了
//! - 避免“浪费”
//!   - 系统可控的只有效果执行的先后顺序，因此必须调整下层属性效果后生效
//! - 如何定义“下层属性”
//!   - 增加约束条件，将伤害的传递顺序转换为属性之间的关系，以确定传导路径，要求必须为“树状结构”
//! - 因此可以推出
//!   - 若两个效果的伤害传递路径之间存在同节点，那么这个节点后面的路径必定重合
//!   - 因此对于任意交汇的两条路径，可以将中间节点合并为一个节点，抽象为 `A1-B-C && A2-B` 或者 `A1-B && A2-B-C`
//!   - 这两条路径的计算优先级，即最下层节点 C 所在的路径必须靠后运算，这是两两效果计算的最佳策略
//! - 效果生效顺序
//!   - 根据能够生效的最底层属性进行排序，底层低的排在后面
//!   - 若底层相同，则根据顶层排序，顶层低的排在后面
//! - 为方便实现，设定如下
//!   - 定义属性层级为“无符号整型”，下层属性数值小
//!   - 每一层都比上一层 -1 ，同时能避免没有环结构

use std::fmt::Debug;

/// 自定义复合属性的层级类型
pub trait AttrLayerType: Debug + Copy + Eq {
    /// 指向自己说明是底层
    fn get_next(&self) -> Self;
    /// 获取当前层级，下一层级必须是上一层级的数值 -1
    fn get_layer(&self) -> u8;
}

/// 对复合属性的效果定义起始和结束标志
pub trait AttrLayerEffType: Debug + Copy + Eq {
    type LayerType: AttrLayerType;

    fn start_at(&self) -> Self::LayerType;
    fn stop_at(&self) -> Self::LayerType;
}

#[derive(Debug, Clone)]
pub struct AttrLayerTypeIter<T: AttrLayerType> {
    current: Option<T>,
}

#[derive(Debug, Clone)]
pub struct AttrLayerEffTypeIter<T: AttrLayerType> {
    current: Option<T>,
    stop_at: T,
}

impl<T: AttrLayerType> Iterator for AttrLayerTypeIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            let next = current.get_next();
            self.current = if next == current { None } else { Some(next) };
            Some(current)
        } else {
            None
        }
    }
}

impl<T: AttrLayerType> Iterator for AttrLayerEffTypeIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            if current == self.stop_at {
                self.current = None;
            } else {
                let next = current.get_next();
                self.current = if next == current { None } else { Some(next) };
            }
            Some(current)
        } else {
            None
        }
    }
}

impl<E: AttrLayerType> From<E> for AttrLayerTypeIter<E> {
    fn from(value: E) -> Self {
        Self {
            current: Some(value),
        }
    }
}

impl<E: AttrLayerEffType> From<E> for AttrLayerEffTypeIter<E::LayerType> {
    fn from(value: E) -> Self {
        Self {
            current: Some(value.start_at()),
            stop_at: value.stop_at(),
        }
    }
}

pub mod attr_layer_system {
    use std::cmp::Ordering;

    use super::*;

    /// 检查属性层级是否合规
    pub fn check_attr_layer<E: AttrLayerType>(start_at: E) {
        let mut current_node = start_at;
        let mut next_node = current_node.get_next();
        while current_node != next_node {
            // 下一层级必须恰好 -1 同时检查无环
            if current_node.get_layer() - 1 != next_node.get_layer() {
                panic!(
                    "the next layer must be one number smaller, at {:?} -> {:?}",
                    current_node, next_node
                );
            }
            current_node = next_node;
            next_node = next_node.get_next();
        }
    }

    /// 检查复合属性效果定义是否合法（必须可达 stop_at ）
    pub fn check_attr_layer_eff_type<E: AttrLayerEffType>(eff_type: E) {
        let stop_at = eff_type.stop_at();
        let ll = AttrLayerEffTypeIter::from(eff_type);
        let mut visited = false;
        for current in ll {
            if current == stop_at {
                visited = true;
            }
        }
        if !visited {
            panic!("the stop_at not visited");
        }
    }

    /// 属性效果排序
    ///
    /// 优先根据底层排序，低的在后面；其次根据顶层排序，低的在后面
    pub fn rank_attr_layer_eff<E: AttrLayerEffType>(a: &E, b: &E) -> Ordering {
        let a_stop_layer = a.stop_at().get_layer();
        let b_stop_layer = b.stop_at().get_layer();
        if a_stop_layer == b_stop_layer {
            let a_start_layer = a.start_at().get_layer();
            let b_start_layer = b.start_at().get_layer();
            b_start_layer.cmp(&a_start_layer)
        } else {
            b_stop_layer.cmp(&a_stop_layer)
        }
    }
}
