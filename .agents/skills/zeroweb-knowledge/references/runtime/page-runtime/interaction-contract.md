# 交互状态、HTML 动作与宿主副作用契约

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未执行输入。 -->

## Module Structure

本文管理 `PageNodeRef`、`PageInteractionState`、`HtmlActionPlan`、`PageEffect` 与 JsExecutor 的宿主边界。具体 HTML 默认动作算法见 scripting；browser/renderer 执行导航、焦点和重绘副作用，WebView adapter 负责 live DOM 状态与事件。

## Gotchas
- PageNodeHandle 是 opaque u64，PageNodeRef 还携带 navigation_epoch 和 document_generation；节点值相同不能证明跨导航或 document replacement 的目标仍有效，规划器先拒绝 stale scope。 (`crates/page-runtime/src/form_control.rs`, `crates/page-runtime/src/html_actions.rs`)
- 更新 pointer_target 不隐式更新 focus_owner；键盘和 IME 必须路由到单一焦点所有者，不能用最近鼠标命中元素代替。 (`crates/page-runtime/src/form_control.rs`, `apps/renderer/src/runtime.rs`)
- 文本 selection 使用 UTF-16 code unit，backward delete 按前一个 Unicode scalar 删除；不能把字节长度或 grapheme 数直接当 selection offset。 (`crates/page-runtime/src/form_control.rs`, `crates/page-runtime/src/html_actions.rs`)
- JsExecutor 的有界执行超时表示结果未知，renderer 中脚本仍留在优先队列执行；超时不能解释成动作被取消，更不能立即无条件重复动作。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/js_worker.rs`)

## Architecture
- HtmlActionPlan 区分 prepare、pre_events、cancelable_event、rollback/commit、followup_events 与 effects；宿主先应用 prepare 并派发事件，再依据 EventDispatchResult 解析提交或回滚。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`)
- PageEffect::SubmitForm 在 submit listener 完成后才让 adapter 构造表单导航意图；不能用规划前的 HTML 快照提前固定 entry list。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`)
- PageEffect::SetFragment 属于同文档动作；Navigation invalidation 表示替换文档并丢弃当前帧事务，Paint 表示 retained 状态改变，二者不能统一为全量重载。 (`crates/page-runtime/src/html_actions.rs`, `apps/browser/src/tab_worker.rs`, `apps/renderer/src/runtime.rs`)
- JsExecutor 抽象快照、同步脚本、module 与 mutation 读回；renderer 覆写优先快照和有界执行，TabJsWorkerHandle 保留默认语义，不能假设每个宿主均有优先通道。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`)

## Patterns
- FormControlState 的 dirty_value 相对本次获焦值计算，编辑版本只在可观察值/选区变化时递增；失焦根据该基线决定 change，不是每个按键都发 change。 (`crates/page-runtime/src/form_control.rs`)
- blur 后移除 retained 控件状态，重新获焦建立新基线；composition 变化可递增 revision，而普通编辑变更会清掉旧 preedit，避免控件状态长期累积。 (`crates/page-runtime/src/form_control.rs`)
- WebView 的外部动作脚本先调用 set_dom_snapshot_priority，再按两秒上限 execute_script_priority_bounded；成对通道保序是条件，单独提高执行优先级会越过快照。 (`crates/webview/src/webview/user_actions.rs`, `crates/page-runtime/src/lib.rs`, `apps/renderer/src/js_worker.rs`)
