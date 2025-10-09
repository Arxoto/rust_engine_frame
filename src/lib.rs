//! 编码规范：
//! - 为尽量保持项目纯净，尽量不对字段使用 `pub` ，函数由于编译期优化所以无所谓
//!   - 使用命令检查是否有属性直接暴露给外部 `grep -r 'pub ' . | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'`
//!
//! 性能优化：
//! - HashMap/HashSet 的性能瓶颈
//!   - rust 的默认哈希算法考虑到 HashDoS 攻击，使用了加密哈希算法 SipHash ，计算哈希值时会有一定的性能损耗，特别是小数据、高频率的情况下
//!     - 另一个维度上，其哈希表基于 SwissTable 实现，对 CPU 缓冲命中做了优化，是目前业界的最佳实践
//!   - 一般认为游戏中计算 Hash 的 key 都是可信输入，无需特别关心安全问题，选型如下：
//!     - 小数据集可直接使用 `Vec` 线性搜索，对缓存友好，可能比 `FxHashMap` 的恒定开销更快，可以认为性能曲线的交叉点在元素个数为 30 左右（根据环境和类型可能在 10-50 之间浮动）
//!     - 可用 rustc-hash （rust编译器使用的库）的 `FxHashMap` ，对数字类型或短字符串具有极大的性能提升，适用于游戏引擎
//!     - 仅当 key 为纯数字时， 斟酌选用 nohash-hasher 库，键本身即是哈希值，非极端场景一般用 rustc-hash 也足够了
//!     - 通用解 ahash 库，较高的安全性和优秀的性能，大结构体时也会使用 SIMD 指令进行性能优化
//!   - 另一方面，预分配容量也是一个优化点 `HashMap::with_capacity(capacity)` ，避免重新分配和重新哈希

pub mod cores;

// Ability System Component
pub mod attrs;
pub mod effects;

// Damage System Component
pub mod damage;

// Motion System Component
pub mod motions;

// todo 修改的代码格式化
// todo `cargo test`
