//! 存储效果的集合（不应包含计时器）
//!
//! 设计
//! - 实际效果的生命周期由 Buff 控制，不自带计时器管理（支持强制清除某 Buff ）
//! - 修改集合后将集合标记为“脏”，后续读属性值的时候发现脏标记再遍历效果刷新属性
//!   - 细节待定：是读时触发刷新（读属性时需要获取属性的可变引用和集合的共享引用）还是每帧固定刷新（一帧内没有读任然会刷新）
//! - 考虑到脏标记的设计，只有修改集合才会触发遍历，因此认为插入和删除的频率比遍历多，数据结构选择 `SlotMap` 而非 `DenseSlotMap` （修改代价太大）

use slotmap::{DefaultKey, SlotMap};

/// todo Attr Eff 使用这个存储
#[derive(Debug)]
pub struct EffCollection<Eff> {
    ll: SlotMap<DefaultKey, Eff>,
    /// 脏标记，记录是否被修改
    changed_flag: bool,
}

impl<Eff> Default for EffCollection<Eff> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Eff> EffCollection<Eff> {
    pub fn new() -> Self {
        Self {
            ll: SlotMap::new(),
            changed_flag: false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Eff> {
        self.ll.iter().map(|(_, v)| v)
    }

    pub fn insert(&mut self, value: Eff) -> DefaultKey {
        self.changed_flag = true;
        self.ll.insert(value)
    }

    pub fn remove(&mut self, key: DefaultKey) -> Option<Eff> {
        let value_opt = self.ll.remove(key);
        if value_opt.is_some() {
            self.changed_flag = true;
        }
        value_opt
    }

    pub fn get_mut(&mut self, key: DefaultKey) -> Option<&mut Eff> {
        let value_opt = self.ll.get_mut(key);
        if value_opt.is_some() {
            self.changed_flag = true;
        }
        value_opt
    }

    pub fn reset_flag(&mut self) {
        self.changed_flag = false;
    }
}
