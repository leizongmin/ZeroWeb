# M2 s328：testdriver 可见对象采纳（beforeinput.tentative 16F→16P）

日期：2026-11-07 ｜ 承接：s327（`2026-11-07-m2s327-survey-and-td-selector-nth-arm.md`）

## 目标与定谳

beforeinput.tentative.html 16 子测试全红（1F+15 文件 Timeout）。s327 定位到身份墙：页面经 pending 视图（R145 延迟 apply，同 turn `container.children` 枚举）持有的解析对象（`_zwMEl` 形，无 sel/handle）与 apply 后 sel 域 wrap 产物不是同一对象。按键注入链只携带 sel 串，shim `_resolveInputTarget` 据此回写 `event.target` 拿到隔离 wrap proxy——`events.has(element)` Map 对象身份键控断言 miss（偶数 inline 族 1F）；IDL `on*` own 属性赋值对 dispatch 不可见（奇数 IDL 族 handler 永不触发，promise 悬挂 → 文件 Timeout）。

## 修复（三文件）

1. `tests/wpt-runner/src/testharness.rs`（stub `__zw_td_selector` 出队）：解析成功且 `out === sel`（祖先链回退不登记——近祖 sel 错绑已移除目标）时登记 `__zwTdVisibleTargets[out] = element`；**并按 canonical 形补登记双键**（`__zw_query_match(out)`，与 out 不同形时）。
2. `crates/engine/src/js_dom_shim/part01.js`（`_resolveInputTarget`）：registry 命中（`[sel] || [resolved]`）即采纳——graft `__zwSelector`/`__zwHandle=null`/`__zwTdAdopted=true` + 覆写 `dispatchEvent` 走 `_dispatchWithBubble`（receiverProxy=页面对象保 identity）；`__zw_reset_form_state` 清 registry（跨页滞留防引用泄漏）。
3. `crates/engine/src/js_dom_shim/part03.js`（dispatch target 站）：采纳对象 own `on*` own 属性臂——`typeof === 'function'` 时跳过 `_ensureInlineHandler`（IDL 设值覆盖内容属性编译，防双 fire），站内 listener 序调用。

非采纳对象路径零差异：part01 未命中 registry 走原 R100/wrap 路径；part03 仅 `__zwTdAdopted===true` 对象读 own on*，其余对象原样 `_ensureInlineHandler`。

## 调试过程（形状墙二次现身）

首轮施工后 beforeinput 仍 1F+Timeout。探针判别：

- probe1（same-turn 同步版）：qSA nth-of-type null、`__zwTdAdopted` false——same-turn 视角延迟干扰，不可判别真实路径。
- probe2（promise_test 跨 turn 走真实 td 出队链）+ `_resolveInputTarget` 内临时调试捕获：`sel/resolved` 均为 `#container > input:nth-child(1)`（canonical nth-child 形），`regKeys` 仅 `nth-of-type(1)` 形，`hitSel/hitResolved` 双 false——**宿主 InsertText 链把 sel 经 `__zw_query_match` 归一成 canonical 形再进 `_resolveInputTarget`**，而 stub nth 兜底臂产 `:nth-of-type` 形。两形字符串不同 → 仅登记 out 时 shim 侧查找恒 miss。s327 身份墙（对象隔离）之后是第二次形状错位（字符串形状）。

修复：stub 登记时 canonical 双键补登记（出队时 apply 已完成，selectorFor nth 臂 qSA 验证同 turn 通过，`__zw_query_match` 此刻可用）。

调试捕获代码已删；探针文件（zz-s328-probe/probe2.html）用毕删除。

## 验证

| 域 | 命令 | s327 基线 | s328 with-WIP | 结论 |
|---|---|---|---|---|
| beforeinput 单测 | `make testharness-html-semantics FILTER=beforeinput` | 1F+15 文件 Timeout（16F） | **16/16 Pass** | 修复目标达成 |
| forms 全域 | `make testharness-html-semantics FILTER=forms` | 4738P/203F/11T（s327 r2） | 4755P/201F/10T（r2） | (key,state) 归一 diff：位移仅 = beforeinput 16F→16P + s327 探针文件（已删）差，零意外位移 |
| uievents 全域 | `make testharness-uievents` | 2008P/186F/85T（s327 uie-wip） | 2008P/186F/85T（r1） | 键级零位移 |
| input-events 全域 | `make testharness-input-events` | 188/1003（gap-batch1 分面干跑，trends csv L236） | 188P/812F/3T | P 数与基线精确一致，零回归 |

日志：`logs/s328-forms-wip-r1.log`（全域快照 8046P/367F/53T，无全域基线不入账）、`logs/s328-forms-wip-r2.log`、`logs/s328-uie-wip-r1.log`、`logs/s328-inputevents-wip-r1.log`（gitignored）。

## 挂账与限制

- 采纳对象 IDL `value` 写路径仍走 graft sel 的 `_controlValue`（local 值域），与静态元素同源——已知限制延续 s327。
- `_proxyCache` 不登记采纳对象（deliberate：cache key 是 sel 串，登记会让后续同 sel 查询误中页面对象）；若后续 `document.querySelector` 返回对象需与 send_keys 目标同 identity，另立切片。
- registry 持页面对象引用，`__zw_reset_form_state` 清理挂在导航清面——同页长活 registry 累积（每 send_keys 一次登记，键覆盖）无泄漏风险（同键覆盖写）。
- uievents runner 裸跑需 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（make 目标已带）；uie 全域 >590s，前台 timeout 须 ≥1400s（本轮 `timeout 590` 误杀一次，重跑收口）。
