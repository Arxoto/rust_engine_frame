# merge_damages 的 6 元数组顺序是解析优先级契约

`damage_system::merge_damages` 返回固定的 `[(DamageType, Option<Effect<S>>); 6]`,顺序为 `[破防护盾, 破奥术盾, 奥术, 剪切, 冲击, 真实]`(破盾优先 → 有防护 → 真实)。这个顺序**不是实现细节**:`apply_damages` 按它结算护盾命中次序,并决定 `DamageInfo` 死因记录的是"数组解析序中第一个 hurt 类型"。等级 B 测试将其钉为契约(一个顺序契约测试 + 行为断言按类型查找)。

未来若把数组改为 HashMap/Vec 等灵活结构,必须保留该解析优先级,否则护盾命中次序与死因判定会悄悄改变。等级 C 加深 `UpsertContainer` 语义时同样不可改动该契约。
