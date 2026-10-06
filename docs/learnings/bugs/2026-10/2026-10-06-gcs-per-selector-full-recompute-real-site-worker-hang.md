---
date: 2026-10-06
modules: engine/js_dom_bridge, renderer/js_worker
---

# getComputedStyle 每新 selector 全量 parse+cascade 压死真站 js worker（E14 停摆根因）

## 问题

真站（bilibili.com 首页）加载 ~13s 后 tab js worker 停摆：CDP evaluate/screenshot/navigate 全部超时，浏览器级即时面（getVersion/getTargets）正常。synthetic 测试（reporter-pb + kv-sdk 单独加载 45 探针全绿）无法复现，需完整真站上下文。

## 根因

`__zw_get_computed_style` 回调（`js_dom_bridge/callbacks.rs`）的缓存只有 per-selector `ComputedStyle` 值层；**每个新 selector 未命中即跑一次 `compute_document_styles_with_inline_overrides` = 整文档 html5ever parse + 全量 cascade**。设计注释本意是 per-snapshot 缓存 `(doc, styles)`，但 `Document` 非 `Send`（observer/listener 闭包 + tendril `Cell`）不能进 `Send + Sync` 回调闭包，实现退化成每 selector 重算（单次 ~628ms 实测）。

真站首页动态脚本每 tick 量测几十个新元素（轮播 `div.vui_carousel`/`div.recommended-swipe` 等）→ 单臂 15-48 次全量重算 → 臂达 9.5s-30s → 30s TAB_JS_EXEC_TIMEOUT watchdog 击杀长臂 + 后续命令排队 → CDP evaluate 5s probe 超时。诊断证据（hang-diag-r9~r13）：170 次慢调用均值 628ms，单轮 ~112s 中合计 106.8s。

## 解决方案

host 回调只在 js worker 单线程执行——用 `thread_local` 持有 `(html, style_version)` 代际键下的 `(Document, styles)`（`js_dom_bridge/computed_style_cache.rs` 的 `with_cached_document_styles`），规避 `Send` 约束；同代际所有 selector 摊销为一次 parse+cascade，html 快照或 mutation 计数（R3030）前进即换代重算。修复后 30s 臂绝迹（max arm 30586ms → 3698ms），生产二进制 60 探针 57 ok 且零连宕。

## 如何避免

- host 回调内禁止 O(文档) 的重算路径按调用次数付费——任何 parse/compute 结果若因 `!Send` 进不了回调闭包，优先考虑 `thread_local`（回调单线程执行是本仓既有契约）而非放弃缓存。
- synthetic 测试过 ≠ 真站可用：量测循环（carousel/getComputedStyle 链）是真站标配模式，性能型缺陷只能在真站 journey 里暴露；新增性能敏感回调时用真站 probe 测单臂时长分布。
