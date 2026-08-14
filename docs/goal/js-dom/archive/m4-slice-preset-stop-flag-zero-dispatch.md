# M4 切片 R39 — pre-set stop propagation flag → dispatch 零触发

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-3
**证据**: [../evidence/2026-08-14-r39-preset-stop-flag-zero-dispatch.json](../evidence/2026-08-14-r39-preset-stop-flag-zero-dispatch.json)

## 切片动机

R38 后按 master.md 下轮候选 (a)「Event-dispatch 系列最小可独立 land 子集」推进。先诊断完整 document/window 入 dispatch chain 的可行性，确认其为深结构：

- **host 侧**：`parent_selector_for('html')` 返空串（`<html>` 无元素父）→ shim `_dispatchWithBubble` 祖先链止于 html，document/window 不入 chain。完整方案需 host 返 `'doc'`/`'win'` 哨兵选择器。
- **shim 侧**：document/window/html 三个目标的 listener 共存于 `_elKey('html', null)` 三合一 key（part06:1265/1285 document.dispatchEvent / window.dispatchEvent 都派发到该 key；`_globalAddEventListener` 也注册到该 key）。拆分独立 key 波及 part06 postMessage/onerror/inline-handler + part02 5 处 `_dispatchToListeners(_elKey('html'),…)` 派发点，共 12 个派发点。

但从失败聚类中提取出**可独立 land 的最小子集**：dispatch 前已设 stop flag 的零触发语义。

## 根因

`_dispatchWithBubble`（part03.js）各阶段循环**先派发后才查** `bubbleStopped()`。用例在 `dispatchEvent()` 之前调 `stopPropagation()`（flag 已设），但 capture 阶段第一站 html 先触发 2 个注册的 listener 才检查 flag。

WPT 实测：`Event-dispatch-propagation-stopped` expected `[]` got `[html, html, html]` length 3（window/document 的 listener 都存 html key，capture 第一站触发 2 个）。

spec `concept-event-dispatch` 步骤 2：dispatch 开始时若 stop propagation flag 已设 → 跳过全部 listener 触发（capture/target/bubble 三阶段全不进）。

## 修复

part03.js `_dispatchWithBubble` try 块开头加：

```js
if (bubbleStopped()) return !event._defaultPrevented;
```

- 三阶段全不进；target/composedPath/Window.event 等 dispatch 期赋值保留（finally 正常清理）
- `bubbleStopped()` 认双 flag（`_propagationStopped` polyfill + `__zw_stop` native 叠加，R34 既建）
- finally 重置 flag（R29 spec 步骤14）→ 同一 event 再派发恢复正常三阶段（单测第 4 场景验证）

## 结果

| 用例 | 路径 | 前 | 后 |
|------|------|-----|-----|
| Event-dispatch-propagation-stopped | polyfill / native | 0P/1F | **1P/0F（100%）** |
| Event-dispatch-bubble-canceled | polyfill / native | 0P/1F | **1P/0F（100%）** |
| dom/events 全量 polyfill | | 177P/151F（54.13%）| 179P/148F（**54.74%**）|
| dom/events 全量 native | | 157P/171F（47.87%）| 159P/168F（**48.62%**）|

双路径对等差 6.12pp（各 +2 pass 同步提升）。dom/nodes polyfill 2502P（零回归）。

## 验证门禁

- 单测 `test_preset_stop_flag_zero_dispatch_r39`（4 场景：三形态零触发 + 无 flag 正常 dispatch `a-cap,c,a-bub,win` 不受影响）
- engine v8 2120 / quickjs 1415 / wpt-runner 171 / webview 595 全绿
- clippy 双矩阵（v8 workspace + quickjs canonical）零警告，fmt 无 diff
- quickjs 测试矩阵（script-sandbox 547 / webview 17 / wpt-runner 106）全绿

## 遗留（深结构，未做）

document/window 入 dispatch chain 完整方案（~29 个 0-pass Event-dispatch 主力）——host 哨兵 + 12 派发点 + 三合一 key 拆分，波及 postMessage/onerror/inline-handler。独立深结构切片或随 M1 L2。
