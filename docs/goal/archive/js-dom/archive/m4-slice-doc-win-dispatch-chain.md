# M4 切片 R40 — document/window 入派发链（槽位标记 + 虚派发站）

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-3
**证据**: [../evidence/2026-08-14-r40-doc-win-dispatch-chain.json](../evidence/2026-08-14-r40-doc-win-dispatch-chain.json)

## 切片动机

R39 诊断的「Event-dispatch 系列 document/window 入 chain 深结构」（~27 个 0-pass 主力）。R40 发现可以**不拆三合一 key** 实现核心语义——槽位标记 + 虚派发站。

## 设计

### 槽位标记（不拆 key）

listener entry 加 `tgt` 字段：
- `'doc'`：`document.addEventListener` 注册（part06 直写 `_listenerStore[htmlKey]`）
- `'win'`：`window.addEventListener` / `_globalAddEventListener` 注册
- `undefined`：html 元素注册（`_makeProxy('html').addEventListener` 等，不打标）

存储仍共存 `_elKey('html')` 三合一 key（postMessage/onerror/inline-handler 的 12 个直接派发点依赖不变，它们无 slot 参数 = 全槽位触发 = 旧行为）。

### 虚派发站（_dispatchWithBubble）

元素祖先链（止于 html）之后追加 document、window 两个虚站：
- capture 反序：window → document → 元素链反序
- bubble 正序：元素链正序 → document → window
- 每站经 `_dispatchToListeners(htmlKey, …, slot)` 槽位过滤，listener 以注册身份触发（currentTarget = document/window 本体）

### targetSlot（document/window.dispatchEvent）

`document.dispatchEvent` → `_dispatchWithBubble(htmlKey, 'html', null, event, 'doc')`：
- path = [document, window]（元素链空）
- doc AT_TARGET 一次（slot='doc'）+ win 冒泡一次
- composedPath[0] = document 本体

`window.dispatchEvent` → targetSlot='win'，path = [window]，仅 AT_TARGET。

### 关键边界

- **detached 元素**（handle-only / 不在 html 子树）不经 doc/win 虚站（path 止于 root）
- **html 元素 target**（host lifecycle `__zw_dispatch_event('html', type)`）的 target 站过滤 null slot——doc/win 注册只在其虚站触发（否则 target 站全触发 + 虚站再触发 = **双 fire**。此回归被 renderer R2941/R2943 quickjs 测试捕获，同轮修复）
- **once 移除**从原始 store 数组过滤（slot 过滤后 list 可能是子集副本）

## 结果

| 用例 | 前（双路径） | 后（双路径） |
|------|--------------|--------------|
| Event-dispatch-multiple-stopPropagation | 0P/1F | **1P/0F（100%）** |
| Event-dispatch-omitted-capture | 0P/1F | **1P/0F（100%）** |
| Event-dispatch-bubbles-true/false 主路径 subtest | Fail | **Pass**（剩余各 3F = cloneNode/new Document/createHTMLDocument 深结构） |
| dom/events polyfill 全量 | 179P/148F（54.74%）| 189P/139F（**57.62%**，+10 pass） |
| dom/events native 全量 | 159P/168F（48.62%）| 169P/159F（**51.52%**，+10 pass，对等差 6.10pp） |

dom/nodes 2502P / dom/collections 17P 零回归。

## 验证门禁

- 单测 `test_doc_win_dispatch_chain_slots_r40`（5 场景）
- engine v8 2121 / quickjs 1415 / webview 595 / wpt-runner 171 / renderer quickjs 124 全绿
- quickjs 矩阵（script-sandbox 749 / integration 75 / wpt-runner 547 等）全绿
- clippy 双矩阵零警告，fmt 无 diff

## 过程记录

1. 初版漏了 `document.dispatchEvent` 的第 5 参 `'doc'`——probe（toString 注入 assert message）定位
2. doc-target 双触发（AT_TARGET + bubble 虚站）——`passDoc` 排除 isDocTarget
3. composedPath[0] 在 doc/win target 时应为本体——cpTarget 覆盖；detached 误加 win 站——passWin 加 inDoc 条件（单测 `test_event_composed_path_r3244` 场景 ③④ 捕获）
4. **html target 站双 fire**——renderer quickjs R2941/R2943 捕获（`dcl,dcl,img,window,window`），单测沙箱最小复现后修 target 站 slot 过滤

## 遗留（深结构）

document.cloneNode(true) / new Document() / implementation.createHTMLDocument() 独立 Document 实例（bubbles-true/false 各剩 3F）——独立 Document 基础设施，html-compat 邻域。
