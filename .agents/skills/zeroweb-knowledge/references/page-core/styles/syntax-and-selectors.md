# CSS token、规则、选择器与条件

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 tokenizer→Parser→Stylesheet/Selector AST、media/supports 条件语法；匹配与级联见 [层叠与匹配](cascade-and-matching.md)。模糊测试执行守卫归 [工程门禁](../../engineering/build-and-release/guards-and-ci.md)。

### User Task Navigation
规则吞噬从 consume_rule 和 at_rules 各分支的消费边界查起；常驻测试在 `crates/css-parser/src/tests/forgiving_selector_list.rs`、`crates/css-parser/src/tests/css_nesting.rs` 和 parser 内测试，本次未运行。

## Gotchas
- Number token 保留 integer/number 书写旗标，2 与 2.0/2e0 即使数值相同也不同；串化整值 number 时保留小数形态，防止下游整数属性错误接受。 (`crates/css-parser/src/tokenizer.rs`)
- Url token 含空白、引号或括号时需加引号并转义反斜杠，否则 token→声明字符串的往返会破坏 data URI 或函数嵌套。 (`crates/css-parser/src/tokenizer.rs`)
- `:is/:where/:has` 当前实现采用 forgiving 列表，未知成员跳过后继续，`:not` 和 nth 的 of S 遇非法成员使选择器失效；这是本实现行为，不能仅凭规范名称外推最新版支持面。 (`crates/css-parser/src/parser/mod.rs`, `crates/css-parser/src/tests/forgiving_selector_list.rs`)
- forgiving 不豁免参数中的伪元素，含伪元素会使整函数及所在选择器非法；全未知成员的空列表则可保留规则但不匹配任何元素。 (`crates/css-parser/src/parser/mod.rs`, `crates/css-parser/src/tests/forgiving_selector_list.rs`)
- nesting 展开同时限制规则深度 16、展开选择器数量 256 与单链 parts 256，分别限制深度、叉乘和链长度；只保留一个限额仍有病态输入风险。 (`crates/css-parser/src/parser/mod.rs`)
- Function token 自带隐含左括号，at-rule prelude 和声明分隔扫描必须把它计入 group_depth；内层逗号/分号不能作为外层分隔。 (`crates/css-parser/src/parser/mod.rs`, `crates/css-parser/src/parser/at_rules.rs`)
- 无效 font-face 分支已消耗自身 extent，consume_rule 必须直接收尾；再进入共享 malformed-qualified-rule 跳过逻辑会吞下一条合法规则。 (`crates/css-parser/src/parser/mod.rs`, `crates/css-parser/src/parser/at_rules.rs`, `git:4e13fffb41312d7a279516457b5fe49349925415`)

## Architecture
- specificity 的 is/not/has 取参数最大值，where 固定为零，nth of S 叠加参数最大值；不能在 matcher 只按实际命中成员重新计算。 (`crates/css-parser/src/selector.rs`)
- media 查询列表拆分保留括号深度，supports 条件另有语法树；条件解析成功只是结构信息，是否生效由 style-system 的上下文与属性支持判据决定。 (`crates/css-parser/src/media_query.rs`, `crates/css-parser/src/supports_condition.rs`, `crates/style-system/src/matcher/supports.rs`)

## Decisions
- font-face 错误恢复由具体分支承担，是为了区分“已经完整消费但无可用字体”与“中途失败仍有 body”两种 None，不能把 Option 的失败统一当作需要再次扫描。 (`crates/css-parser/src/parser/mod.rs`, `git:4e13fffb41312d7a279516457b5fe49349925415`)
