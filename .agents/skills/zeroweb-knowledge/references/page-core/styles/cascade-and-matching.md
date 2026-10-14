# 层叠、匹配、继承与伪元素

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖匹配索引、声明合法性、origin/layer/style context、继承、UA 与伪元素。变量和属性转换见 [值与属性](values-and-properties.md)，生成节点接入见 [页面管线](../pipeline.md)。

### User Task Navigation
“合法旧声明被非法新声明覆盖”查 cascade；伪元素污染子树查 compute_styles_recursive 的 custom 快照。索引差分判据在 `crates/style-system/src/matcher/tests/index_buckets.rs`，层叠/继承另有内联测试；本次未运行。

## Gotchas
- CascadeOrder 的 important 会反转 origin 优先级为 UA>user>author，style_attribute 是独立上下文而不是仅靠高 specificity 模拟。 (`crates/style-system/src/cascade.rs`)
- 普通 fast path 仍按 unlayered>layered、晚层>早层排序，important 也沿用该旧层序；只有出现合法 revert-layer 的 slow path 使用 important 早层优先，不能宣称两路径已完全等价或完整实现 Cascade 5。 (`crates/style-system/src/cascade.rs`)
- revert-layer 的慢路径回退按 origin 与 CascadeCtx 截止，排除同层及更高上下文且不区分 importance；style 属性回退可重新进入 author sheet，孤立关键字留给继承层处理。 (`crates/style-system/src/cascade.rs`)
- inheritance 的残余 revert/revert-layer 仍是简化实现：继承属性取父值，非继承属性取初始值；不能用它重建完整 user/UA 回退链。 (`crates/style-system/src/inheritance.rs`)
- 无效负长度和枚举值必须在挑胜者前丢弃，apply 阶段才拒绝会失去较低优先级合法声明；合法性探测使用真实 apply-on-dummy 而非支持声明列表。 (`crates/style-system/src/cascade.rs`)
- custom property、var/env 和 CSS-wide 值必须穿过级联有效性预检，实际语法合法性要等计算值阶段；未知普通属性则会被 apply-on-dummy 拒绝。 (`crates/style-system/src/cascade.rs`)
- logical sizing 与物理 sizing、word-wrap 与 overflow-wrap 在分组前归一为同一槽位，否则 HashMap 应用顺序会覆盖 specificity；并非所有 vendor 前缀都可通用剥离。 (`crates/style-system/src/cascade.rs`)
- legacy -webkit-line-clamp 需要保留来源并检查 -webkit-box/-webkit-inline-box 上下文，胜出的 prefixed 但不适用声明会 no-op，不能简单 canonical 成标准 line-clamp。 (`crates/style-system/src/cascade.rs`)
- hidden input 的 display:none 被作为 UA important 注入，作者 important 不能覆盖；这是强制无盒的当前实现，不是普通 UA normal display 默认。 (`crates/style-system/src/lib.rs`)
- presentational hints 属于 author 来源、specificity 零且最早位置，处在 UA 默认之上但可被真实选择器/inline 覆盖；伪元素不接入标签 UA 和 hints。 (`crates/style-system/src/lib.rs`)
- 伪元素计算前后保存并恢复 custom_properties，防止 before/after/selection 等规则内变量污染后续子元素继承；普通子递归也要先捕获当前元素变量快照。 (`crates/style-system/src/lib.rs`)
- 当前属性选择器缺省大小写按 content_is_xml 分流，HTML 走 ASCII-ci、XHTML 走 exact，i/s 显式修饰覆盖默认；不要将此实现概括成所有 HTML 属性均符合规范的逐属性语义。 (`crates/style-system/src/matcher/mod.rs`)

## Architecture
- 匹配从右向左，兄弟组合器跳过非元素节点；桶索引只筛候选而不替代真实链匹配与源码序，跨桶同规则多匹配必须去重。 (`crates/style-system/src/matcher/mod.rs`, `crates/style-system/src/matcher/tests/index_buckets.rs`)
- 元素的 placeholder/selection/marker 等伪样式存回 ComputedStyle，before/after 再由管线生成节点；伪元素样式表存在性与具体元素匹配成功不是同一条件。 (`crates/style-system/src/lib.rs`, `crates/engine/src/pipeline/mod.rs`)
- color-scheme 在其他颜色属性前确定，font-size 也需提前归一供相对长度使用；不应把继承计算改成无序遍历所有键。 (`crates/style-system/src/inheritance.rs`)

## Patterns
- 索引修改用全量扫描参考作逐声明差分，覆盖多 sheet、media/container 上下文、大小写 tag、跨 class 桶双命中和伪元素；只比较最终颜色会遗漏重复声明或排序错误。 (`crates/style-system/src/matcher/tests/index_buckets.rs`)
