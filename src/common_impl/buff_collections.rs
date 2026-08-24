//! Buff 效果集合（包含计时器）
//! 
//! 设计
//! - Buff 附带的效果由其生命周期方法进行管理（因此需要持有 SlotMap 的 Key 数据）
//! - Buff 需要每帧判断是否过期，每帧进行遍历，因此数据结构选择 DenseSlotMap
//! - Buff 允许堆叠，因此需要根据已知类型去搜索到已有的 Buff ，因此选择 FxHashMap 维护索引
//! - Timer 选择被 Buff 持有，防止二级索引
//!   - 思考是否可以拿 Timer 做 DenseSlotMap ，然后使用 Buff 做 SecondaryMap
//!   - 每帧遍历检查过期时无需再次查询导致缓存未命中
//!   - 显示全量 Buff 时是否要遍历 Timer 并查询 Buff
//!   - 处理计时器过期时，若需要进行堆叠，则需要查询 Buff 导致缓存失效
//! 
//! todo 合并逻辑 堆叠或替换 ； 过期逻辑 逆堆叠或移除

use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, DenseSlotMap};

#[derive(Debug)]
pub struct BuffCollection<K, Buff> {
    pool: DenseSlotMap<DefaultKey, Buff>,
    lookup: FxHashMap<K, DefaultKey>,
}

impl<K, Buff> Default for BuffCollection<K, Buff> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, Buff> BuffCollection<K, Buff> {
    pub fn new() -> Self {
        Self {
            pool: DenseSlotMap::new(),
            lookup: FxHashMap::default(),
        }
    }
}