# Flex、Grid 与本地 Taffy 契约

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查 Flex/Grid 内容尺寸、比例传递、轨道和本地算法扩展。`computed_style_to_taffy` 转换样式，`engine.rs` 组织重算；`crates/taffy-local` 是参与生产的补丁依赖，不是仅供阅读的上游副本。

## Gotchas
- column flex 的 auto-cross aspect-ratio item 在 content-based base 阶段先量 max-content cross，再按可用 cross 做 fit-content 并扣 padding/border，最后乘 ratio；row 不能套用这个传递分支。 (`crates/taffy-local/src/compute/flexbox.rs`)
- flex base 阶段通过 `SizingMode::ContentSize` 排除 item 自身 min/max 污染，hypothetical main 才做钳制；min-height 经比例传给 min-width 后不得再反向抬高 base。 (`crates/taffy-local/src/compute/flexbox.rs`)
- ratio item 的 flex-resolved main 不应被 transferred max-cross 反钳；R5047 常驻测试固定 `flex:1 + ratio 1/2 + max-height:100` 于 100 宽容器输出 100×100，而不是 50×100。 (`crates/layout-engine/src/engine/tests/r5047_flex_ar_flexed_main_tests.rs`)
- Grid 显式 stretch 一轴而另一轴 auto 时需要按轨道尺寸钳制并传递 ratio；另一轴已 definite 时双轴约束优先，不能再强行保持比例。 (`crates/layout-engine/src/engine/tests/r3860_grid_ar_stretch_transfer_tests.rs`)
- 内容系宽度的 inline-grid 不强推 auto 轨道 STRETCH，否则容器仍收缩到内容而条目却按父宽分散；显式宽度的 inline-grid 保留 STRETCH 适配。 (`crates/layout-engine/src/converter/mod.rs`)
- Taffy ComputeSize 缓存命中只比较混合 known/available key 与 parent 的 x 轴部分，PerformLayout 则比较完整 key；不能把九个 measure 槽误认为九份完整布局。 (`crates/taffy-local/src/tree/cache.rs`)

## Architecture
- Flex/Grid 初次提取之后，intrinsic 关键字、百分比高度、padding 和 ratio 修正写回 Taffy style，统一 mark_dirty 重算并重新提取，单改最终盒宽不等价于子树重新布局。 (`crates/layout-engine/src/engine.rs`)
- `cached_baselines()` 暴露 Taffy 内部 first_baselines 给 LayoutEngine 提取，删除补丁或改缓存生命周期会影响 atomic inline 对齐，不能只验证块宽高。 (`crates/taffy-local/src/tree/taffy_tree.rs`, `crates/layout-engine/src/engine.rs`)
- 缓存区分 final layout 与 preliminary size；hidden layout 不读写这些槽，清空已空缓存返回 AlreadyEmpty，mark_dirty 用此结果停止无效祖先传播。 (`crates/taffy-local/src/tree/cache.rs`, `crates/taffy-local/src/tree/taffy_tree.rs`)

## Decisions
- 本地 Flex 算法仍保留 orthogonal flow base 分支的 TODO；高层已有竖排适配不代表 Taffy 本体完整支持该步骤。 (`crates/taffy-local/src/compute/flexbox.rs`)
- Flex base 以 padding+border 为地板是实现主动保留的 Chrome/Firefox 对齐选择，旁边 TODO 明确规范解释仍有争议，不能写成无争议的规范等式。 (`crates/taffy-local/src/compute/flexbox.rs`)

## Dependencies
- workspace 的 `[patch.crates-io]` 将 taffy 指向 `crates/taffy-local`，升级版本时必须一起核对测量 trait、baseline 访问和本地 Flex 补丁，而非仅修改版本号。 (`Cargo.toml`, `crates/taffy-local/src/tree/traits.rs`)
