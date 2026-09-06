# M1 — WPT 键盘交互基线（2026-09-07）

**用例来源**：上游 WPT @ `315976933870b34d6ea30e3f6643403edae678ba`（与 selection/editing
套件同 pin），`fetch-keyboard-subset.sh` 拉取。6 用例 + 1 helper。

**导入清单**：
uievents/keyboard/{keydown-input-events, keyboardevent-composed, keyboardevent-legacy,
keypress-not-fired-for-modifier-shortcuts, modifier-keys}.html +
html/semantics/forms/form-submission-0/implicit-submission.optional.html +
resources/targetted-form.js

**排除项**（有据）：`key-101en-us-manual.html` / `key-102fr-fr-manual.html`（manual——
需真物理键盘布局）；`keyboardevent-composed.html` 等 composed 断言面单测未过不入第一批
（后续切片复评）。uievents/keyboard 共 8 html，本批 5。

**执行入口**：`make testharness-keyboard`（test-guard 包裹；`FILTER=<子串>` 透传）

## 基线（首轮，2026-09-07）

```
TOTAL 1P / 4F
```

| 用例 | P | F | 失败根因 |
|---|---|---|---|
| uievents/keyboard/keyboardevent-legacy.html | 1 | 0 | ✅ |
| uievents/keyboard/keydown-input-events.html | 0 | 1 | runner testdriver Actions stub 无 key 系列链（`addKeyboard is not a function`）——R142 stub 仅 pointer；键盘链是 K2 分发层修复的驱动面 |
| html/.../implicit-submission.optional.html | 0 | 3 | ① runner send_keys 此前拒 WebDriver ENTER（uE007 落 PUA 拒绝分支）→ 本轮已修（uE007 → Submit 动作，webview 分发 CE 宿主换行/表单隐式提交）；② 剩余 F = js-dom 共享面：`insertAdjacentHTML('afterbegin')` 后 iframe.nextSibling 视图断链（populateForm 拿到 null form）——probe 实证，非键盘分发层缺陷 |

## 本轮修复（随基线落地）

1. **runner send_keys ENTER 映射**（testharness.rs）：uE007 → `HtmlUserAction::Submit`
   ——webview Submit 臂分发序「编辑宿主优先消费」：CE 宿主 → `__zw_ce_enter` 换行
   （即便宿主在表单内也不提交）；未消费才走 enclosing form 隐式提交管线。
2. **webview Submit 臂重排**（user_actions.rs）：CE probe 前置于 form 解析（原切片 3
   只在无表单时探测——CE 宿主在表单内的 Enter 会误提交）。

## 失败聚类 → 修复队列（M1 后续切片 / M2）

1. **testdriver Actions 键盘链**（addKeyboard/addKey.../send）——keydown/keyup 事件
   派发断言的基建（keydown-input-events 驱动）。
2. **js-dom 共享面**：insertAdjacentHTML 后兄弟视图断链（implicit-submission
   populateForm 依赖）——属 js-dom 域，跨 goal 碰撞管理（master.md 记录）。
3. **隐式提交规则**（K3）：uE007→Submit 已通 runner 层；text control 无 submit button
   的单控件提交规则、disabled submit button 阻断语义——待 js-dom 视图修复后可复评。

---

# M1 切片 2 — Actions 键盘链 + 事件序（2026-09-07，同日追加）

**修复**（commit aad135b52 + 后续扩展，testharness.rs + part05.js）：
1. Actions stub 键盘链：addKeyboard/keyDown/keyUp 记步骤 → send() 按序入队
   keydown/keyup（旧版 reject「unsupported」）。
2. keydown/keyup 命令：cancelable KeyboardEvent 派发（script_dispatch_dom_event，
   'ok'/'prevented' 判定）→ keydown 未取消接 InsertText 默认动作 + keypress 派发；
   preventDefault 抑制编辑事件与 value 变更。
3. KeyboardEvent.composed 缺省 true（UI Events spec）：shim ctor prop 链 + R109
   native 包装器（native 模板恒设 false，以 init dict 为事实源回填）。
4. send_keys WebDriver 键扩展：滚动/导航键（arrows/pages/home/end，uE00E-uE015）→
   keydown+keyup 事件对（keyboard-page-scrolling 共享基建）；修饰键（Shift/
   Control/Alt/Meta，uE008/9/A/D）同款成对派发。

**基线演进**：1P/4F → **6P/12F**（9 用例全部可执行，Timeout/Unhandled rejection 清零）。

| 用例 | P | F | 状态 |
|---|---|---|---|
| keydown-input-events | 2 | 0 | ✅ 事件序 + cancel 语义全过 |
| keyboardevent-composed | 3 | 0 | ✅ composed 缺省修复 |
| keyboardevent-legacy | 1 | 0 | ✅ |
| modifier-keys | 0 | 4 | runner 无持久修饰键状态（down/up 成对派发 vs getModifierState 断言）——M2 修复队列 |
| keypress-not-fired-for-modifier-shortcuts | 0 | 1 | 同上（修饰快捷键 keypress 抑制语义） |
| implicit-submission | 0 | 3 | js-dom 共享面：insertAdjacentHTML 后兄弟视图断链（populateForm null form） |
| css-scroll-snap/input 三案 | 0 | 4 | 断言依赖真滚动管线（runner 侧无——keyboard-page-scrolling M2 滚动分发到 webview 层后复评） |
