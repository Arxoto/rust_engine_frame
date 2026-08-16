# Spec: 接口加深(等级 C)

Status: resolved

## Problem Statement

作为库,接口质量直接决定上层接入成本与未来贡献者的上手成本。当前存在三处"浅接口"或语义缺陷:

1. **计时器模块接口 ≈ 实现**:支撑一个 30–80 行具体计时器,需要读者同时理解约 11 个概念(`Tickable`/`TimerProgress`/`TimerView`/`TimerControl`/`TimerPauseView`/`TimerPauseControl`/`CyclicalTrigger`/`HasTimer` + `DependCtx` GAT + `Union` + 特性切换的 `time_type::T`)。签名族三套不一致(部分 trait 传 `ctx`,部分不传);无上下文类型需要显式传 `()`(约 22 处调用税);新增一个 prefab 需约 10 个复制粘贴的 impl 块;`WithInto`(方案一残留)是公开 trait 却零生产调用。
2. **`UpsertContainer` 语义矛盾与状态机泄漏**:两条修改路径 dirty 语义互相矛盾 —— `iter_mut` 静默绕过脏检查,`select_mut_ele` 无条件置脏;三个状态机(脏标志生命周期、清洞调度、合并策略)泄漏给调用方;清洞逻辑散落在 4 个文件(`time_type` 常量 + `UpsertContainerCleaner` + `upsert_container::try_clean_hole` + `attr_systems::try_clean_hole`)。
3. **孤儿文件**:`base_lib/motions/animations.rs` 只有模块文档、未被 `motions.rs` 声明,是死重。

## Solution

- **计时器**:**trait 面不合并(否决「收拢 trait 面」方向,决议见 issue 01)** —— 各 trait 的能力边界有真实语义:`TimerProgress`=进度模型 vs `TimerView`=完成状态(`FewShotStaticTrigger` 是「仅状态、无进度」的现成反例);`TimerControl` vs `CyclicalTrigger`(`TickTimer`/`StaticTimer` 无循环触发);暂停 View/Control 的只读可变分离。合并会把不同能力耦合、强迫仅实现单能力的类型实现无语义方法。实际交付:补全 `FewShotStaticTrigger` 的 `TimerProgress`(透传内层 `InfiniteStaticTrigger`)。`(())` 调用税**保留**(issue 02 决议:「blanket impl 消税」不可实现,`(())` 是统一抽象方案二的可见代价);`WithInto` 处理仍按 issue 03 推进。**保留 `StaticTimeline`/`StaticTimer` 上下文机制** —— 它是"无引擎依赖 + 长期效果"的承重结构(与 README 项目介绍一致)。
- **`UpsertContainer`**:统一 dirty 语义(或提供显式批量 API),将清洞调度收进容器,提供默认合并策略方法,并用测试钉住契约。
- **`animations.rs`**:并入 `motions.rs`(保留动画过渡设计文档)或删除,并同步更新 CLAUDE.md。

## User Stories

1. As 一个游戏上层调用方, I want 使用计时器只需理解少量概念, so that 接入成本降低。
2. As 一个游戏上层调用方, I want 无上下文类型无需显式传 `(())`, so that 调用更自然。**(已关闭:issue 02 决议不实施,`(())` 保留为统一抽象的可见代价。)**
3. As 一个框架贡献者, I want 新增一个 prefab 只需少量 impl 块, so that 扩展成本降低。
4. As 一个框架贡献者, I want 不再需要为 `WithInto`(方案一残留)维护公开接口, so that 概念面收敛。
5. As 一个游戏上层调用方, I want `UpsertContainer` 的脏标志语义一致, so that 我不会经 `iter_mut` 静默绕过属性刷新。
6. As 一个游戏上层调用方, I want 清洞调度与合并策略有容器内统一入口, so that 我不需要跨 4 个文件拼装。
7. As 一个维护者, I want 计时器加深后仍保留 `StaticTimeline`/`time_type` 机制, so that 长期效果与引擎无关性不被破坏。
8. As 一个维护者, I want `animations.rs` 被并入或删除, so that 模块树干净、CLAUDE.md 与实际一致。
9. As 一个维护者, I want 加深后原有测试(等级 B 的演示测试)继续通过, so that 外部行为不被破坏。

## Implementation Decisions

- **计时器加深,保留方案二**:`DependCtx` 关联类型方案是既定选型(见 `design_patterns.rs`),不加反转。~~为 `Ctx = ()` 提供 blanket impl 消除 `(())` 税~~(**已否决,issue 02 决议**):最小样例实证(rustc 1.94/edition 2024)显示 blanket impl 无法改变方法签名;扩展 trait/core-trait 同名方法触发 E0034 歧义(方法解析先按名后按 arity),内禀遮蔽触发 E0061 且要求全量迁移+方法写两份,改名(`elapsed_with`)或拆互斥 trait 会加概念/破坏统一抽象。结论:`(())` 调用点保留,不再追求消除。
- **合并 trait:已否决(issue 01 决议)**。理由:(1) `TimerProgress`/`TimerView` 是「进度模型」与「完成状态」两种能力,`FewShotStaticTrigger` 只实现后者,证明拆分必要;(2) `TimerControl`/`CyclicalTrigger` 同理,`TickTimer`/`StaticTimer` 无循环触发能力;(3) 暂停 View/Control 拆分符合「只读可变分离」原则。保留 8 个 trait 拆分。可选的清理(未纳入决议):删除暂停 Union 死代理。
- **`WithInto`**:定义移入 `design_patterns.rs` 的单元测试 mod,不再作为 crate 公开 trait;原有测试脚手架(`test_foo`/`test_bar`、`auto_do_something*`、blanket impl)保留为「方案一曾如何实现」的可运行参考。理由:方案一是未采用的备选,其 old-solver 缺陷与「显式标注类型」workaround 是历史记录,无需活代码承载;但保留为可运行参考比删除更稳妥,支持未来(solver 再进化)重估。`Union` 保留(`motions/actions.rs` 生产用 `Union::new`),构造统一为 `Union::new` 单写法:元组构造(`Union(a, b)`)在 few_shot_times.rs / pause_prefab.rs / design_patterns.rs 测试内全部替换;字段保持 `pub(super)`(prefab 代理直接访问 `.0`/`.1`,如 pause_prefab.rs,不可收私有)。
- **删除测试**:加深的验收标准是"删除过度拆分后,复杂度集中在计时器模块内部,而非散落调用方"(删除测试通过)。
- **`UpsertContainer` 语义统一**:**任何可能的修改都置脏,不论实际是否改动**(决议)。`iter_mut` 创建即置脏;`select_mut_ele` 命中即置脏;`upsert_ele`/`delete_ele` 成功即置脏;`delete_ele` miss(无匹配元素)不置脏。理由:唯一「改了但无需重算」的场景是重置时间线(经 `iter_mut` 改计时器、效果数值不变),按年一次,为此付一次重算可接受;换取语义简单一致、杜绝经 `iter_mut` 静默绕过 `try_refresh_dirty_attr`。压力测试 `changed` 模型同步更新。`UpsertContainerCleaner` **保留独立类型**(ECS 一实体多容器、一个 cleaner 触发多个容器是真实需求,不并入容器),自持默认周期(引用 `time_type::DEFAULT_REFRESH_PERIOD`,可构造覆盖),新增 `clean_holes(delta, containers_iter)` 单一编排入口;`attr_systems::try_clean_hole` 删除或退化为薄委托。新增 `UpsertContainer::upsert_replace(new_ele)` 默认合并策略方法,`equips.rs` 改用它。
- **`animations.rs`**:并入 `motions.rs`(决议)——声明 `pub mod animations;`,动画过渡设计保留为该模块文档;同步更新 CLAUDE.md 对应段落,消除「Re-integrate or delete」悬置。
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
