# R331 Evidence — dispatch listener-swap 快照语义 + query identity 桥双源去重（Vue reconciliation 复活）

**日期**: 2026-08-28
**切片**: M4——R331 两件独立修复（① spec 派发快照语义；② R322/R323 回归修复）
**改动面**: `part03.js`（dispatch bubble 段快照刷新）、`part04.js`（R322/R323 归并域 identity 反查 + 去重前置）、`part05.js`（`_zwQueryWrapIdentity` 全局发布）、`part24.rs`（+1 回归测试）、`e2e_vue_library.rs`（探针轮已还原，零 diff）

## 一、修复 ①：Event-dispatch-handlers-changed 1F→Pass（M4 events 备档 4F 之一收口）

**R320 归因精确化**（「add-then-later-station 快照时序」）落地的根因：`_dispatchToListeners` 的
`snap = list.slice()` 在函数入口做**一次**快照，capture 段与 bubble 段共用。而 spec
`concept-event-dispatch` 步骤 4「let listeners be a **clone** of event's currentTarget's event
listener list」在**每个结构遍历步**（每站）重新克隆——AT_TARGET 站（`phase='all'`）的
copy-of-listeners 把当时表拷两遍，capture 段消费 swap 前表、bubble 段消费 swap 后表。

**修复**：bubble 段（`phase !== 'capture'` 分支）开头重取快照，同步应用 slot 过滤
（html/doc/win 三站共存 `_elKey('html')` key——新 add 的 listener 若是本站身份才进快照；
未过滤时 doc/win 虚站会把 html 站新 add 的 listener 误触发，R40 门语义保持）。

**验证**（zz-r331 临时探针已删）：`N=17 |L=0,0,0,0,0,0,0,0,1,3,1,1,1,1,1,1,1` 与 WPT
`expected_listeners` 逐位一致；正式用例双路径 Pass。

## 二、修复 ②：R322/R323 归并域 identity 双源双计（vue_reconciliation 复活）

**发现**：`vue_reconciliation_lands` 在干净 main（R322 起）持续 4/4 复现失败
`lis:A,B,A,B`——R322（`5dc2065d5`）落地的 pending 归并使**已 apply 入 host 快照的 handle
子**被双计：host 快照结果（sel wrapper）+ pending 桶归并（handle proxy）同一 li 两个
identity。R322/R323 轮的 A/B 列表未含 vue e2e，回归潜伏两轮。

**修复两层**（part04 querySelectorAll sel 分支）：
1. **去重前置**：归并前扫描 pending 桶 added——经 `__zw_selector_for_handle` 正置反查命中
   host 结果列表（= 快照已含该节点）即跳过归并，直接返回 host 结果（经反查包装）。跨
   execute 旧 handle（反查未命中 = 同 turn append 快照滞后形态）保持归并——pending 语义不变。
2. **identity 反查包装**：host 结果包装 `_wrapSelector` → `_zwQueryWrapIdentity`（R100
   反查——命中 createElement 建立的 handle 时返回原 handle proxy），两处（归并路径 +
   无归并路径）同改。归并产物（handle proxy 本体）与快照包装（反查后同 handle proxy）
   identity 合流。

**配套**：`_zwQueryWrapIdentity` 经 `globalThis` 发布（part05，R79 poke 模式）——execute
路径 page script 无法读 shim IIFE 闭包内函数（探针 `nofn` 实证），发布幂等。

**回归验证**：engine v8 2469 全绿（R39/R40 首版 slot 泄漏被既有测试当场抓回后修复——
教训：**改共享 dispatch 函数时其回归哨兵必须同轮跑**）；engine quickjs 1466 绿；Vue e2e
3/3 + integration 781 + lit/WC e2e 全绿；ParentNode-querySelector 全族 polyfill 2054P
（All-content Timeout 单跑 Pass = 已知调度 flake）/native **2055P/0F**；dom/events
polyfill 586P/17F+T（净 +1P=handlers-changed，余为已备档 Timeout 族）/native 587P/15F+T；
traversal 1604P/1T（cross-realm 备档）；collections 49P/0F；dom/nodes 12763P/21F+T（fail
集 = 备档集恒等，MO Timeout 族 clean-HEAD 同值）。

## 三、流程教训

1. **A/B 列表覆盖面**：R322/R323 改查询归并时 A/B 只列 WPT 套件，未含 vue/lit e2e——
   回归潜伏两轮。查询路径的回归面必须同时含「消费查询结果的框架 e2e」。
2. **first-run 探针漂移**：zz-r331 首轮探针断言 `N=17|L=…1,3,1…` 与最终一致，但初版探针
   在旧二进制上跑（`Finished` 不代表重编，R327 教训第三次实证）——每次改 shim 后
   `touch js_dom_bridge.rs` + 确认 `Compiling zero-engine` 是硬性流程。
3. **共享函数回归哨兵同轮跑**：R331 ①首版未同步 slot 过滤，R39/R40 两个既有测试当场
   抓回（html 站新 add 的 listener 泄漏进 doc/win 虚站）——改 `_dispatchToListeners` 这类
   全局函数，其历史测试是最快的正确性 oracle。
4. **product-smoke 23.37% 双跑逐字节一致**（clean-HEAD 同值；渲染流 8/23 巡检记录的
   23.61% 时代数字已因 oracle 渲染环境微差漂移到 23.37%——差值非本切片引入，oracle
   re-capture 仍属渲染流放行项）。

## 四、A/B 汇总

| 套件 | R330 基线 | R331 | Δ |
|---|---|---|---|
| Event-dispatch-handlers-changed | 1F | **1P** | -1F（本切片目标）|
| dom/events（polyfill） | 582P/4F+T | 586P/17F+T | +1P 净（Timeout 族既有）|
| dom/events（native） | — | 587P/15F+T | 双路径对等 |
| dom/nodes | 54151P 全量内 | 12763P/21F+T | fail 集 = 备档恒等 |
| traversal / collections | 1604P/1F | 1604P/1T / 49P/0F | 恒等 |
| ParentNode-querySelector（native） | 2054P | **2055P/0F** | 双路径对等 |
| engine v8 / quickjs | 2468/1466 | **2469**/1466 | +1（r331 回归测试）|
| integration / webview / Vue e2e | 781/657/3F | 781/657*/**3P** | Vue 复活；*webview 1F=SW 流既存 flake（clean-HEAD 同败）|
| fmt / clippy（v8+quickjs） | — | 干净 | — |
