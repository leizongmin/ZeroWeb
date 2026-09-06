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
