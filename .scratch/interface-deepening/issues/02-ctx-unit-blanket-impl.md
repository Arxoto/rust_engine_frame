# Ctx=() blanket impl,消除 (()) 调用税

Type: task
Status: ready-for-agent
Blocked by: interface-demonstration-tests/01

`DependCtx` 方案要求无上下文类型显式传 `()`,约 22 处调用税(如 `actions.rs:94`、`controllers.rs:115-134`、`behaviours.rs:202-338`、`static_timer.rs:34,40`),实现方还要写大量 `fn elapsed(&self, _: ())` 死参数。为 `Ctx = ()` 提供 blanket impl,使无上下文类型无需显式传 `()`。spec:`interface-deepening/spec.md`。

## 验收

- 无上下文类型的 trait 方法调用不再需要 `(())`;仓库内 `(())` 调用点清零或大幅减少。
- 实现方不再写 `_: ()` 死参数。
- 不破坏 `StaticTimer`(依赖 `&StaticTimeline` 上下文)的调用方式。
- 等级 B 的计时器测试全部继续通过。
