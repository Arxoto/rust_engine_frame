//! 编码规范：
//! - 为尽量保持项目纯净，尽量不对字段使用 `pub` ，函数由于编译期优化所以无所谓
//!   - 使用命令检查是否有属性直接暴露给外部 `grep -r 'pub ' . | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'`
//!
//! 特性：
//! - baselib 最小依赖包，用于打包成 lib
//! - commonimpl 基础的业务实现
//! - godotext 使用 godot-rust 来生成 GDExtension
//!
//! 使用注意：
//! - 若在 Bevy 引擎中使用，则属于原生语言开发，完全没有额外的性能损耗
//! - 若在 Godot 引擎中使用，需要基于 <https://github.com/godot-rust/gdext> 的兼容层，每次调用 Rust 函数都有绑定层开销（少量开销，计算密集型逻辑能轻松弥补）
//!   - 栈和寄存器的保存与恢复(Context Switching)，跨语言调用的必要消耗，一般开销通常以纳秒计算
//!   - 参数和返回值的编组(Marshalling)，数据结构不匹配时的主要开销，对于 i64 和 f64 等类型通常是零成本编组
//!   - 绑定层和运行时检查(Binding & Runtime Overhead)，如 Godot 通过 ClassDB 定位到要调用的 Rust 函数、绑定层运行时检查、跨语言错误机制处理等
//!   - 因此：建议将大量操作打包成一个函数来一次调用，如同时大量角色的场景，则尽量不要在每个角色的 process 方法中去调用 Rust 函数
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
//!   - 另一方面，预分配容量也是一个优化点 `Vec::with_capacity(capacity.next_power_of_two())` ，避免重新分配和重新哈希
//!     - 注意处于内存对齐的目的，初始化时 capacity 尽量做 2 的幂次优化（因为后续扩容也是翻倍扩容的）
//!     - 使用命令进行检查 `grep -r 'with_capacity' . | grep -v 'next_power_of_two' | grep -v EVENT_LIST_CAPACITY`
//! - 编译优化选项 (in Cargo.toml, see <https://doc.rust-lang.org/cargo/reference/profiles.html>)
//!   - 构建命令 `cargo build --release`
//!   - 编译优化等级 `opt-level = 3`
//!     - 高等级需要更多编译时间
//!   - 链接时优化 `lto = true`
//!     - 增加链接时间为代价生成更优化代码，如允许跨模块函数内联
//!     - 默认为 false ，即 "thin" ，提供了接近 "fat" 的性能提升，但链接时间可观减少
//!     - 设置为 true ，即 "fat"
//!   - 代码生成单元 `codegen-units = 1`
//!     - 默认为 16 ，设置为 1 允许编译器将整个 crate 视为一个单元进行优化，以获得最佳运行时性能
//!   - Panic 策略 `panic = "abort"`
//!     - 发生 panic 时立即终止，减少运行时开销
//!   - 调试符号 `debug = false`
//!     - release 下默认关闭，仅显式配置

pub mod cores;

// Ability System Component
pub mod attrs;
pub mod effects;

// Motion System Component
pub mod motions;

// todo 修改的代码格式化
// todo `cargo test`
// todo `cargo clippy`

// Combat System Component
#[cfg(feature = "commonimpl")]
pub mod combat;

#[cfg(feature = "godotext")]
pub mod godot_ext_impl;
