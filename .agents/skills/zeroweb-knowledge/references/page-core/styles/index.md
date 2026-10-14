# CSS 解析与样式计算

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-css-parser` 产生语法结构，`zero-style` 完成候选匹配、简写展开、级联、变量及继承计算；`ComputedStyle` 是布局和绘制共同输入，不意味着每个已识别属性都有完整实现。

### User Task Navigation
外部 CSS 合并及生成伪节点见 [管线](../pipeline.md)，字体输入见 [字体知识](../../graphics/render-foundation/fonts/index.md)。验证入口是 parser/style-system 各 tests 模块和 `crates/style-system/src/matcher/tests/index_buckets.rs`，本次未运行。

## Gotchas
- parser 接受、supports 判断、cascade 有效性探测与 apply 是四个不同关口；属性“已支持”声明不能证明其错误恢复、继承、布局与绘制全链正确。 (`crates/css-parser/src/parser/mod.rs`, `crates/style-system/src/matcher/supports.rs`, `crates/style-system/src/cascade.rs`, `crates/style-system/src/property/apply.rs`)

## Architecture
- compute_styles 先建规则索引再按父到子恢复 custom 继承，伪元素计算保存快照；把递归改为无序节点遍历会改变继承和 container 条件。 (`crates/style-system/src/lib.rs`)

## Child Knowledge Nodes
- [CSS token、规则、选择器与条件](syntax-and-selectors.md) — 规则吞噬、forgiving、nesting 限额与条件 AST。
- [层叠、匹配、继承与伪元素](cascade-and-matching.md) — 来源与层序差异、非法声明回退、索引差分、伪元素污染。
- [属性、简写、计算值与自定义属性](values-and-properties.md) — pending 简写、变量冲突、注册属性和相对单位转换。
