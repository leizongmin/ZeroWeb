# 属性、简写、计算值与自定义属性

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 CSS values、PropertyRegistry、apply/inherit、简写延迟替换与 CSS 变量两层实现。字体物理度量来源见 [字体加载与匹配](../../graphics/render-foundation/fonts/loading-and-matching.md)；这里记录计算值输入约束。

### User Task Navigation
新增属性需核对 registry/default/apply/inherit 和 shorthand；变量案例入口为 `crates/style-system/src/shorthand/tests/var_pending.rs`、`crates/engine/src/pipeline/css_vars.rs` 内测试，本次未运行。

## Gotchas
- 简写先展开再层叠，含 var 的受支持简写以每个 longhand 的 pending sentinel 参与竞争；替换后仅重写仍为 sentinel 的槽位，不能覆盖已经胜出的显式 longhand。 (`crates/style-system/src/shorthand/mod.rs`, `crates/style-system/src/shorthand/tests/var_pending.rs`)
- pending_shorthand_longhands 只列部分简写，未列出的含 var 简写仍可能在初次展开失败后丢弃；不能由 font/background 的测试外推全部简写。 (`crates/style-system/src/shorthand/mod.rs`)
- 逐元素 resolve_var 使用 visiting 集检测直接/间接环，嵌入值中任一无法解析引用使整值失效；无合法替代时保留原值交下游 IACVT 处理，不是回到级联前较低声明。 (`crates/style-system/src/computed.rs`)
- engine 的文本预替换最多迭代 8 层，且不承担完整选择器级联；同名多值列入 skip，连带空 fallback 一起保留原 var 文本交逐元素解析。 (`crates/engine/src/pipeline/css_vars.rs`)
- 文本收集对同名声明的 important 与后出现顺序有最小竞争处理，但不能替代 origin、selector specificity 和元素继承；是否跳过冲突名比最终收集值更关键。 (`crates/engine/src/pipeline/css_vars.rs`)
- 文本预替换使用 str::find 和 UTF-8 切片，CSS 的 // 不当作注释；字节转 char 重建会损坏中文，错误剥 // 会吞 URL，两者有专门回归。 (`crates/engine/src/pipeline/css_vars.rs`)
- env 的 safe-area 等已知值与未知 fallback 分开，未知且无 fallback 保留字面量供下游失效处理，不应该默认为任意环境变量都为零。 (`crates/style-system/src/computed.rs`)
- 容器相对单位在没有查询容器时回退视口；有容器但需要的轴 unknown 时 fail-closed 为零，不能混淆这两种 None 来源。 (`crates/style-system/src/computed.rs`)
- font-size larger/smaller 可走绝对字号表步进而不总是乘 1.2；相对长度解析要使用提前确定的本元素/根字体基准及字体度量。 (`crates/style-system/src/inheritance.rs`, `crates/style-system/src/computed.rs`)
- CSS math 与 quirks 长度由 apply 的解析路径处理，简化 parse_length 不等价于最终可消费值；声明合法性检查必须复用生产 apply。 (`crates/style-system/src/property/apply.rs`, `crates/style-system/src/cascade.rs`)

## Architecture
- PropertyRegistry 同时提供初始值、继承属性和已知 longhand 集合，all 展开及隐式继承依赖它；只增加 ComputedStyle 字段与 apply 分支不足以形成完整属性契约。 (`crates/style-system/src/property/registry.rs`, `crates/style-system/src/inheritance.rs`, `crates/style-system/src/cascade.rs`)
- 自定义属性注册信息与每元素 parent_custom 分开存储，递归前捕获快照；不能复用可变 StyleSystem.custom_properties 作为多个兄弟共同的继承源。 (`crates/style-system/src/lib.rs`)

## Decisions
- 文本预替换是补静态网站主题/字体的兼容路径，R5106 增加冲突门后将同名不同值交回逐元素级联；历史“全局取最后一个值”不能继续当作现行完整变量模型。 (`crates/engine/src/pipeline/css_vars.rs`, `git:acdd18ed35393c814618026244c11e11082e019a`)
