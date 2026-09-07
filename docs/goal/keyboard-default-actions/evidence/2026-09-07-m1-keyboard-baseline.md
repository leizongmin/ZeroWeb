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

## M1 切片 3（2026-09-07）— 修饰键位透传（K2 分发表扩展）

**改动面**：DomEventDetail（script_gen.rs）新增 shift_key/ctrl_key/alt_key/meta_key 四位
→ script_gen detail JSON 透传 → `__zw_dispatch_event` KeyboardEvent 分支 init dict
（shiftKey/ctrlKey/altKey/metaKey）→ KeyboardEvent constructor（part05 modifier 位全接）。
runner send_keys 修饰键映射改四元组（key 名 + 对应位 true）。

**修复聚类**：

1. modifier-keys.html 4F 全灭——event.shiftKey === (key === 'Shift') 断言族全过
   （位经 init dict → getModifierState/shiftKey 等 IDL 位可读）。
2. ~~keypress-not-fired-for-modifier-shortcuts 随位透传转 Pass~~ **勘误（残余切片 4）**：
   此记录有误——该用例实际仍 Timeout（keyup listener 因 send_keys 无键事件永不
   resolve），真实转 Pass 在 M1 残余切片 4（send_keys 事件序补全）落地。

**残余 4F 精确根因（探针定位，2026-09-07）**：

- implicit-submission.optional.html 3F：`populateForm` 前置失败——
  insertAdjacentHTML('afterbegin', iframe+form) 后 `iframe.nextSibling = null`、
  `body.childNodes.length = 0`（stale）而 `querySelectorAll('form') = 2`（双计）。
  js-dom 融合视图 pending/host 双计 + childNodes 视图断链——跨域 js-dom 共享面
  （editing goal E2 残余聚类同根因），非隐式提交实现缺陷。协调点：js-dom 流
  R3031 _zwFragmentAdded 融合视图。
- paged.html 1F：断言依赖真滚动管线（runner 侧无）——keyboard-page-scrolling
  共享面（P1/M1 记录延续）。

**验证**：keyboard 套件 6P/12F → **10P/4F**；selection 套件 2005P/779F 零回归；
engine 2631 全绿；clippy -D warnings 零警告；单测
test_modifier_key_dispatch_r3254_k2_slice3（五组：Shift/Control/Alt/Meta 位独立 +
缺省兼容 + getModifierState 联动）。

## M1 残余切片 4（2026-09-07）— send_keys 普通字符键事件序（K2 收尾）

**勘误**：切片 3 段上方「keypress-not-fired-for-modifier-shortcuts 随位透传转 Pass
（10P/4F 中已计入）」为**误记**——该用例从未 Pass（本轮基线表即 0P/1F，切片 3 后
实测 3/3 稳定 Timeout，非负载 flake、非 js_dom_shim 改动回归：事件从未派发）。
10P/4F 的实际构成不含该用例的 Pass。

**根因**：runner `send_keys` 普通字符只走裸 `InsertText`（无 keydown/keypress/keyup
事件派发），用例 3 个 promise_test 均等 `keyup` listener resolve——永不触发 →
全 pending Timeout。

**修复**（commit ece2282ec，testharness.rs）：
1. send_keys 普通字符补全 **keydown（cancelable）→ 未取消则 InsertText 默认动作 →
   keypress → keyup** 事件序（UI Events §keydown/§keyup 默认动作序；与 Actions
   keydown 路径同款 cancel 语义）。
2. **Ctrl/Meta 按住时 keypress 抑制**（UI Events：修饰快捷键不产生字符点击——
   keypress-not-fired 主断言面）。
3. send_keys 串内修饰键持久化：修饰字符设置状态位，后续普通字符的事件对继承
   （`uE009+'v'` 复合序的 v 事件对 ctrlKey=true）；跨 send 清零。非字符键
   （Backspace/Tab/ENTER）行为不变。

**验证**：keyboard 套件 10P/4F → **13P/4F**（keypress-not-fired 3 Timeout 全灭）；
runner 205 单测全绿（新增
send_keys_dispatches_key_event_sequence_and_suppresses_modifier_keypress 三组断言 +
MINI_HARNESS 补 assert_true）；testharness-html 14/14、selection
onselectionchange 6/6 零回归；clippy -D warnings 零警告。

**残余 4F**（根因均已在案）：implicit-submission 3F = js-dom 融合视图双计（跨域
协调点）；paged.html 1F + snap 三案 Timeout = runner 无真滚动管线/帧循环
（keyboard-page-scrolling 共享面 P1/P3 记录）。
