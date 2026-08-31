# R372 — per-realm in-flight event（event-global-is-still-set-when-reporting-exception-onerror 收口）

**日期**: 2026-08-30
**切片**: 已知 Fail 集合 7→6——events onerror 跨 realm 文件整文件转绿（附带
`Event-dispatch-throwing` 2F 修复回归）
**改动面**: `js_dom_shim/part03.js`（fire 点 per-realm 事件窗 + 勘误 stamp 读取
+ 上报时机前移 + reporter handler-realm 槽）+ `part05.js`（win.dispatchEvent
不覆盖外层 in-flight）

## 1. 规范模型与根因

WebIDL inner-invoke：**in-flight event 记在 callback 关联 realm 的 global**，
非 target realm。既有实现只有主 realm 单槽（`globalThis.event`，R33/R114），
iframe realm 的 `frames[n].event` 槽从不被设置。探针逐层剥：

1. `window.onerror`（frames[0] realm 函数）被调用期间应 `frames[0].event ===
   myEvent` 且主槽保持外层 load 事件——既有 dispatch 级 set 使主槽 = myEvent
   （✗）、frames[0] 槽恒 undefined（✗）。
2. `foo` 抛出的异常 report 嵌套在一级 dispatch 的 per-realm 窗口内：二级
   handler（frames[1] realm）期间 `frames[0].event` 保持 myEvent、主槽保持
   load——report 必须发生在槽复原**前**。
3. `frames[1].event.error.name === "ReferenceError"`——上报的 error event 要
   进入 **handler 关联 realm** 的槽（frames[1]，非上报目标 realm frames[0]）。

## 2. 修复五件

1. **fire 点 per-realm 事件窗**（part03 `_dispatchToListeners` fire）：印记
   listener（`_zwRealmWin` 印记 ≠ 主 realm）调用期间——realm win 槽 = 本
   event、主槽 = `event._zwOuterEvent`（dispatch 级挂载的外层 in-flight 锚）；
   调用后双槽复原。主 realm listener 零变化（外层 dispatch 级 set 继续服务）。
2. **stamp 读取勘误**：fire 点 `typeof fn === 'object'` 把函数 listener 的
   印记排除（R302/R370 同款勘误第三次）——放宽到 function。
3. **上报时机前移**：印记 listener 的异常 report 从「复原后」移到「复原前」
   （spec report 嵌套在 inner-invoke 内）；未印记 listener 走原外层路径。
4. **reporter handler-realm 槽**（`_zwReportListenerError` realmWin 分支）：
   从 on* fn 读 handler 关联 realm 印记，上报事件写入 **handler realm** 槽
   （save/restore）；目标 realm 槽与主槽不动。
5. **win.dispatchEvent 不覆盖外层 in-flight**（part05）：`if (!globalThis.
   event) globalThis.event = event`——嵌套上报时主槽保持外层（d 断言）；
   standalone dispatch（主槽空）维持旧行为。附带 `Event-dispatch-throwing`
   回归修复（变量 scoping 勘误——R143 教训第十次：跨作用域搬家逐引用核对）。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标件 | `event-global-is-still-set-when-reporting-exception-onerror` 7 断言全 Pass |
| Event-\* 族 | **394P/0F**（改前 392P/2F——净 +2P/-2F：目标件 + throwing 回归修复）；Event-dispatch 221P、EventListener 24P、EventTarget 34P 恒等；event-global 9P 全绿 |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55495P/9F/16T——真实 Fail 文件集合 7→6 零新增**（含探针自抛文件 1 个非真集；Timeout +1 为 beforeunload 既存[clean HEAD 同 Timeout]） |
| engine 单测 | v8 2498 / quickjs 1473 全绿；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**：① R302/R370 同款勘误第三次（`typeof === 'object'` 排除函数
印记）——该 gating pattern 已在库内多处出现，值得专项清扫；② try/catch/
finally 的执行序（catch 在 finally 前）决定「report 嵌套在 in-flight 窗口内」
的语义成败——把 report 从外层 catch 挪进内层 catch 是本轮第二处关键改动；
③ 变量 scoping 搬家（R143 教训）：内层 catch 改名后外层守卫的引用同步。

## 4. 后续

- 已知 Fail 集合余 **6**：MutationObserver-document 3F（parse-time MO 基建
  ——host 解析管线域）、remove-and-adopt-thcrash（window.open 无 popup 通
  道）、remove-next-sibling-during-replace-with（插入期脚本执行[R328 遗留]
  + sel 域 fused innerHTML）、click-on-absolute-pseudo（Chromium 专有
  pseudo，不追）、ranges dataChange/replaceData 2F（文件级 Timeout 尾批）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
