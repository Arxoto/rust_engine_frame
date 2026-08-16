# Spec: 接口加深(等级 C)

Status: ready-for-agent

## Problem Statement

作为库,接口质量直接决定上层接入成本与未来贡献者的上手成本。当前存在三处"浅接口"或语义缺陷:

1. **计时器模块接口 ≈ 实现**:支撑一个 30–80 行具体计时器,需要读者同时理解约 11 个概念(`Tickable`/`TimerProgress`/`TimerView`/`TimerControl`/`TimerPauseView`/`TimerPauseControl`/`CyclicalTrigger`/`HasTimer` + `DependCtx` GAT + `Union` + 特性切换的 `time_type::T`)。签名族三套不一致(部分 trait 传 `ctx`,部分不传);无上下文类型需要显式传 `()`(约 22 处调用税);新增一个 prefab 需约 10 个复制粘贴的 impl 块;`WithInto`(方案一残留)是公开 trait 却零生产调用。
2. **`UpsertContainer` 语义矛盾与状态机泄漏**:两条修改路径 dirty 语义互相矛盾 —— `iter_mut` 静默绕过脏检查,`select_mut_ele` 无条件置脏;三个状态机(脏标志生命周期、清洞调度、合并策略)泄漏给调用方;清洞逻辑散落在 4 个文件(`time_type` 常量 + `UpsertContainerCleaner` + `upsert_container::try_clean_hole` + `attr_systems::try_clean_hole`)。
3. **孤儿文件**:`base_lib/motions/animations.rs` 只有模块文档、未被 `motions.rs` 声明,是死重。

## Solution

- **计时器**:收拢 trait 面 —— 合并过度拆分(暂停 View/Control 围绕单布尔;`TimerProgress`+`TimerView`;单方法 `CyclicalTrigger`),为 `Ctx = ()` 提供 blanket impl 消除 `(())` 税,移除或私有化零调用 `WithInto`;新 prefab 的 impl 块从约 10 个降至少量。**保留 `StaticTimeline`/`StaticTimer` 上下文机制** —— 它是"无引擎依赖 + 长期效果"的承重结构(与 README 项目介绍一致)。
- **`UpsertContainer`**:统一 dirty 语义(或提供显式批量 API),将清洞调度收进容器,提供默认合并策略方法,并用测试钉住契约。
- **`animations.rs`**:并入 `motions.rs`(保留动画过渡设计文档)或删除,并同步更新 CLAUDE.md。

## User Stories

1. As 一个游戏上层调用方, I want 使用计时器只需理解少量概念, so that 接入成本降低。
2. As 一个游戏上层调用方, I want 无上下文类型无需显式传 `(())`, so that 调用更自然。
3. As 一个框架贡献者, I want 新增一个 prefab 只需少量 impl 块, so that 扩展成本降低。
4. As 一个框架贡献者, I want 不再需要为 `WithInto`(方案一残留)维护公开接口, so that 概念面收敛。
5. As 一个游戏上层调用方, I want `UpsertContainer` 的脏标志语义一致, so that 我不会经 `iter_mut` 静默绕过属性刷新。
6. As 一个游戏上层调用方, I want 清洞调度与合并策略有容器内统一入口, so that 我不需要跨 4 个文件拼装。
7. As 一个维护者, I want 计时器加深后仍保留 `StaticTimeline`/`time_type` 机制, so that 长期效果与引擎无关性不被破坏。
8. As 一个维护者, I want `animations.rs` 被并入或删除, so that 模块树干净、CLAUDE.md 与实际一致。
9. As 一个维护者, I want 加深后原有测试(等级 B 的演示测试)继续通过, so that 外部行为不被破坏。

## Implementation Decisions

- **计时器加深,保留方案二**:`DependCtx` 关联类型方案是既定选型(见 `design_patterns.rs`),不加反转。仅对 `Ctx = ()` 提供 blanket impl 消除 `(())` 税。
- **合并 trait**:暂停 View/Control 围绕单布尔过度拆分,合并为一对或默认方法;`TimerView::is_completed` 与 `TimerProgress::progress` 视删除测试考量合并或提供默认实现(需论证借用冲突的原有理由是否仍成立)。
- **`WithInto`**:降为私有或删除(若 `motions/actions.rs` 仍用 `Union::new`,保留 `Union` 本身)。
- **删除测试**:加深的验收标准是"删除过度拆分后,复杂度集中在计时器模块内部,而非散落调用方"(删除测试通过)。
- **`UpsertContainer` 语义统一**:倾向"任何修改都置脏";若代价过大,则提供显式 `begin_batch`/`end_batch` API 使批量修改也纳入脏跟踪。`UpsertContainerCleaner` 与清理周期常量收进容器或容器模块;新增默认合并策略方法(如 `upsert_replace`)。
- **`animations.rs`**:并入 `motions.rs` 时把"动画播放器结束信号自动衔接下一段动画"的设计保留为模块文档;删除时同步移除 CLAUDE.md 对应条目。
- 所有改动保持引擎无关;不引入新 trait/宏生成方案(宏方案已在 `tiny_timer.rs` 模块文档中否决)。

## Testing Decisions

- **好测试的标准**:通过公开接口断言行为变化;加深不得破坏外部行为 —— 等级 B 的演示测试是回归基线,加深后必须全部继续通过。
- **被测模块**:`base_lib/cores/timers/`、`base_lib/cores/design_patterns.rs`、`base_lib/eff_attr_prop/upsert_container.rs`。
- **现有先例**:`eff_attr_prop` 内联测试;`design_patterns.rs` 的类型推断测试;`upsert_container.rs` 的随机操作压力测试(用于钉住 dirty/长度不变量)。
- **硬性要求**:加深后测试数量只增不减;"接口即测试面" —— 每收拢一个 trait,对应的行为测试要迁移到新接口而非删除。

## Out of Scope

- 不推翻方案二,不回到 `WithInto` 作为默认组合机制。
- 不重写计时器实现语义(先业务后 tick 的次序、few-shot 边界行为、暂停语义)。
- 不引入宏生成计时器实现(已否决)。
- 等级 A 的构造路径与等级 B 的演示测试本身。
- 不触碰 `time_type` 特性切换机制(它是承重结构)。

## Further Notes

- 建议顺序:**等级 A → 等级 B → 等级 C**。先打通入口,再钉住行为(回归基线),最后加深接口。
- `StaticTimeline`/`StaticTimer` 生命周期、`time_type` 特性切换、`FixedName` 适配器接缝是"无引擎依赖"目标的承重结构,任何加深必须保留。
- 加深过程中发现的文档与实现偏差,随手修正并记录到 Comments。
