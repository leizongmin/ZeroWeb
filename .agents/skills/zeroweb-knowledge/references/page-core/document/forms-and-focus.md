# 焦点、表单状态、校验与事件模型

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 FocusManager 的键盘顺序、Document 状态伪类、静态校验和原生 Event。点击默认动作与脚本事件见 [事件与默认动作](../../scripting/event-loop/events-and-default-actions.md)。

### User Task Navigation
Tab 顺序查 `crates/dom/src/focus.rs`；CSS/DOM 状态分歧查 document 的 focus/form_state/validation。验证入口是 focus.rs、event.rs 内常驻测试，本次未运行。

## Gotchas
- FocusManager 的正 tabindex 先按升序，再排 0/自然焦点的文档序；负 tabindex 不进入 Tab 列表，但 set_focus 允许程序化聚焦列表外节点。 (`crates/dom/src/focus.rs`)
- 空或非法 tabindex 被解析为 0，而缺属性返回 None；属性移除与赋空串会产生不同的键盘顺序。 (`crates/dom/src/focus.rs`)
- FocusManager 与 Document.focus_element 是两个存储面；`:focus`/`:focus-within` 只读 Document 的运行时注入值，解析时 autofocus 不会自行使其匹配。 (`crates/dom/src/focus.rs`, `crates/dom/src/document/focus.rs`)
- fieldset disabled 的首个直接 legend 子树获豁免，但外层 disabled fieldset 仍约束；Tab 扫描与状态伪类需分别核对各自禁用 helper。 (`crates/dom/src/focus.rs`, `crates/dom/src/document/form_state.rs`)
- 表单状态方法主要读取内容属性；checkbox 的动态 indeterminate 不在静态判定中，radio 的 indeterminate 则按同 name、同最近 form 祖先且无 checked 成员求值。 (`crates/dom/src/document/form_state.rs`)
- 静态 form_owner 只认最近 form 祖先，未实现 form 属性的跨树关联；radio 分组及默认 submit 判定继承这个边界。 (`crates/dom/src/document/form_state.rs`)
- `:placeholder-shown` 要求 placeholder 属性存在且无值，`:blank` 不要求 placeholder；textarea 的纯空白内容在当前实现被视为空。 (`crates/dom/src/document/form_state.rs`)
- 原生约束校验只覆盖 required 空值和 min/max 数值日期范围；pattern、email/URL typeMismatch 不参与 invalid，不能当成完整 ValidityState。 (`crates/dom/src/document/validation.rs`)
- invalid 对 form/fieldset 可由后代无效候选触发，valid 当前仅覆盖候选控件而没有对称祖先语义；不要把 valid 当作所有元素上的 invalid 取反。 (`crates/dom/src/document/validation.rs`)
- 数值范围比较保留三位小数，日期时间值走可比整数近似；该静态子集不能替代浏览器完整日期解析与 stepMismatch。 (`crates/dom/src/document/validation.rs`)
- stopPropagation 保留当前节点后续监听器，stopImmediatePropagation 同时停止它们；preventDefault 仅在 cancelable 时置位。 (`crates/dom/src/event.rs`, `crates/dom/src/document/mod.rs`)

## Architecture
- DOM 查询与 style-system 共享 Document 的焦点、禁用、范围和目标状态判定，新增伪类应沿共享路径接入而不是只修一个 matcher。 (`crates/dom/src/document/focus.rs`, `crates/dom/src/document/form_state.rs`, `crates/style-system/src/matcher/mod.rs`)
- Event 初始化派发会重置传播与取消状态，原生事件可复用；默认动作执行仍属于调用者，不由 Event 数据结构完成。 (`crates/dom/src/event.rs`)
