# Ctx=() blanket impl,消除 (()) 调用税

Type: task
Status: resolved
Blocked by: interface-demonstration-tests/01

`DependCtx` 方案要求无上下文类型显式传 `()`,约 22 处调用税(如 `actions.rs:94`、`controllers.rs:115-134`、`behaviours.rs:202-338`、`static_timer.rs:34,40`),实现方还要写大量 `fn elapsed(&self, _: ())` 死参数。为 `Ctx = ()` 提供 blanket impl,使无上下文类型无需显式传 `()`。spec:`interface-deepening/spec.md`。

## 验收

- 无上下文类型的 trait 方法调用不再需要 `(())`;仓库内 `(())` 调用点清零或大幅减少。
- 实现方不再写 `_: ()` 死参数。
- 不破坏 `StaticTimer`(依赖 `&StaticTimeline` 上下文)的调用方式。
- 等级 B 的计时器测试全部继续通过。

## Answer

**决议:不实施「blanket impl 消除 `(())` 调用税」(决策不实施),`(())` 调用点保留。** 经最小样例编译实证(rustc 1.94 / edition 2024),issue 提议的机制不可实现:

- **blanket impl 无法改变方法签名**:`fn elapsed(&self, ctx: Self::Ctx<'_>)` 对 `Ctx = ()` 也无法免参调用(Rust 无默认/可选方法参数)。
- **扩展 trait + blanket(同名方法)**:编译失败 **E0034** —— 方法解析先按「方法名」收集候选、后查参数个数,同名即歧义(0 参扩展方法 vs 1 参 trait 方法直接冲突)。
- **core trait + blanket 委托(同名)**:同样 E0034(具体类型同时实现两个同名 trait)。
- **内禀 0 参方法遮蔽**:可编译,但内禀方法**完全遮蔽**同名 trait 方法(连 arity 不同也不回落),`t.elapsed(())` 变 E0061 错误 → 所有调用点须全量迁移;每个 tick 类型每个方法写两份(内禀 + trait impl,保持同步);trait impl 的 `_: ()` 死参数仍在;Union/泛型路径的 `(())` 清不掉。
- **改名绕歧义**(如 `elapsed_with`):可行但每能力两套 trait + 两侧命名不一致。
- **拆互斥两套 trait**:可行但放弃统一抽象,破坏 `attr_systems::clean_expired_element` 这类「泛型同时收 TickTimer 与 StaticTimer」的代码 —— 这正是 `DependCtx`(方案二)存在的理由。

**结论**:`(())` 不是设计缺陷的「税」,而是统一抽象(方案二)的**可见代价**。全仓库约 88 处 `(())`,其中约 72 处在测试(tick_timer 24 / tick_trigger 27 / few_shot 11 / pause_prefab 4),生产调用约 16 处(behaviours 11 / controllers 4 / actions 1 / static_timer 2)。为「调用更自然」这一观感收益付上述代价(概念膨胀或统一抽象破坏)不值,与 issue 01 同一模式。

**实际交付**:无代码变更。等级 C 剩余有价值内容为 issue 03(`WithInto` 公开死 trait)、04(`UpsertContainer` dirty 语义矛盾)、05(孤儿文件)。
