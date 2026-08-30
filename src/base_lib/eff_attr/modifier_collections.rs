//! 存储效果的集合（不应包含计时器）
//!
//! 设计
//! - 实际效果的生命周期由 Buff 控制，不自带计时器管理（支持强制清除某 Buff ）
//! - 修改集合后将集合标记为“脏”，后续读属性值的时候发现脏标记再遍历效果刷新属性
//!   - 细节待定：是读时触发刷新（读属性时需要获取属性的可变引用和集合的共享引用）还是每帧固定刷新（一帧内没有读任然会刷新）
//! - 考虑到脏标记的设计，只有修改集合才会触发遍历，因此认为插入和删除的频率比遍历多，数据结构选择 `SlotMap` 而非 `DenseSlotMap` （修改代价太大）

use slotmap::{DefaultKey, SlotMap};

use crate::base_lib::eff_attr::aggregators::InvalidModifier;

pub trait ModifiableAttr<Modifier> {
    /// 获取原始值
    fn get_origin(&self) -> f64;

    /// 获取当前值
    fn get_current(&self) -> f64;

    /// 刷新属性
    fn refresh_value<'a>(&mut self, modifiers: impl Iterator<Item = &'a Modifier>)
    where
        Modifier: 'a;
}

#[derive(Debug)]
enum ModifierOp<Modifier> {
    Remove(DefaultKey),
    Replace(DefaultKey, Modifier),
}

#[derive(Debug)]
pub struct ModifierCollection<Modifier: InvalidModifier> {
    ll: SlotMap<DefaultKey, Modifier>,
    /// 待生效修改器
    pending_list: Vec<ModifierOp<Modifier>>,
    /// 脏标记，本帧的数据是否需要更新
    changed_flag: bool,
}

impl<Modifier: InvalidModifier> Default for ModifierCollection<Modifier> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Modifier: InvalidModifier> ModifierCollection<Modifier> {
    pub fn new() -> Self {
        Self {
            ll: SlotMap::new(),
            changed_flag: false,
            pending_list: Vec::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Modifier> {
        self.ll.values()
    }

    #[must_use]
    pub fn insert(&mut self, value: Modifier) -> DefaultKey {
        // 无效修改器占位
        let default_key = self.ll.insert(Modifier::new_invalid());
        self.pending_list
            .push(ModifierOp::Replace(default_key, value));
        default_key
    }

    pub fn remove(&mut self, key: DefaultKey) {
        self.pending_list.push(ModifierOp::Remove(key));
    }

    pub fn replace(&mut self, key: DefaultKey, value: Modifier) {
        self.pending_list.push(ModifierOp::Replace(key, value));
    }

    pub fn commit_pending(&mut self) {
        if self.pending_list.is_empty() {
            return;
        }

        for ele in self.pending_list.drain(..) {
            match ele {
                ModifierOp::Remove(key) => {
                    self.ll.remove(key);
                }
                ModifierOp::Replace(key, value) => {
                    if let Some(value_origin) = self.ll.get_mut(key) {
                        *value_origin = value;
                    }
                }
            }
        }

        self.changed_flag = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.changed_flag
    }

    pub fn reset_flag(&mut self) {
        self.changed_flag = false;
    }
}
