# 事件分发、默认动作、焦点与表单激活

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查“listener 执行了但默认动作仍发生”、连续输入覆盖与宿主合成事件。底层表单状态与焦点算法见 [文档表单](../../page-core/document/forms-and-focus.md)，页面输入入口见 [桌面输入](../../products/desktop/navigation-and-input.md)。

### User Task Navigation

`crates/webview/src/tests/user_actions.rs` / `event_dispatch.rs` 是宿主事件与动作判据；`dom_bindings/tests_events.rs` 是不经 shim 的 native 事件判据。真实指针验收从 [zeroweb-verify](../../../../zeroweb-verify/SKILL.md) 导航，入口未提交、本轮未实跑。

## Gotchas
- checkbox/radio 激活先 prepare checkedness 再派 cancelable click，让 listener 看见预激活态；取消时走 rollback，不能把所有 mutation 延后到 listener 返回之后。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`)
- `resolve_html_action` 取消分支保留 dispatch 的 html_changed 并仅回滚计划动作；listener 自己的 DOM 修改不是默认动作，不能随取消一起丢掉。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`)
- 文本 InsertText/DeleteBackward 从宿主快照计划改成一段 JS 读 live value/selection、派 beforeinput、写值及派 input；延迟应用快照整值会覆盖并发击键，宿主计划只保留 noop 判定。 (`crates/webview/src/webview/user_actions.rs`, `crates/engine/src/js_dom_shim/part01.js`)
- 文本原子输入中的 readonly、maxlength 判定也读 live 状态，maxlength 按 UTF-16 单元截断；仅写值原子而守卫仍读旧快照，仍会发生迟到状态覆盖。 (`crates/engine/src/js_dom_shim/part01.js`, `crates/webview/src/webview/user_actions.rs`)
- 不能把全部删除动作归为同一路径：text-control 的 ForwardDelete 仍走计划管线，contenteditable 的 ForwardDelete 有独立接线，InsertText 的修复不自动覆盖它们。 (`crates/webview/src/webview/user_actions.rs`)
- 当前 WebView dblclick 以同目标连续激活累计，没有真实点击间隔窗口；第二次及以后补派 dblclick，不能据此声称平台双击阈值已模拟。 (`crates/webview/src/webview/user_actions.rs`)
- Touch、TouchList、TouchEvent 构造面与 ontouch* 属性曝光分开接线；构造器存在不能推出 document 有 ontouchstart，也不能证明宿主已经投递真实多点触摸事件。 (`crates/engine/src/js_dom_shim/part05.js`)
- TouchInit target 通过 addEventListener 函数可达性检查，identifier 经 Number(v)|0 转成整数、坐标非有限数回退0；这是当前 shim 的近似转换，不能当作完整 WebIDL 品牌和数值校验。 (`crates/engine/src/js_dom_shim/part05.js`)
- TouchList 禁止直接 new，TouchEvent 用内部工厂复制传入序列并保留元素身份，越界 item 返回 null；工厂未逐项校验 Touch 品牌，且 getModifierState 尚未接通，不能把三列表和修饰键字段存在当成完整 TouchEvent 兼容。 (`crates/engine/src/js_dom_shim/part05.js`)

## Architecture
- native 事件分发沿 listener 表完成捕获/目标/冒泡，shim 与 native 有各自监听入口；宿主页面派发要覆盖适用监听面，单独调用 native element.click() 不等于宿主完整用户动作。 (`crates/engine/src/dom_bindings/event_target.rs`, `crates/engine/src/dom_bindings/tests_events.rs`, `crates/webview/src/webview.rs`)
- native focus/blur 既更新 JS activeElement 缓存，也镜像 live Document 焦点供 :focus/:focus-within 样式使用；只改页面对象会让 CSS 焦点状态分叉。 (`crates/engine/src/dom_bindings/tests_events.rs`, `crates/engine/src/dom_bindings/element.rs`)
- reset 动作在宿主 prepare 与结果应用之间开启/结束 host action transaction；表单激活与导航效果由宿主计划解析，不能用派 reset 事件代替表单默认值恢复。 (`crates/webview/src/webview/user_actions.rs`, `crates/engine/src/js_dom_bridge/form_activation.rs`, `crates/page-runtime/src/html_actions.rs`)
