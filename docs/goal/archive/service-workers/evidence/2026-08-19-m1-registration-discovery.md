# M1-4d Service Worker Registration Discovery

**日期**：2026-08-19
**状态**：M1-4d complete
**前置**：[M1-4c renderer bridge](2026-08-19-m1-renderer-bridge.md)

## 0. Typed contract

`ServiceWorkerOperation` 末尾追加：

- `GetRegistration { client_url }`；
- `GetRegistrations`。

`ServiceWorkerResult` 末尾追加：

- `OptionalSnapshot(Option<ServiceWorkerSnapshot>)`；
- `Snapshots(Vec<ServiceWorkerSnapshot>)`。

所有 nested enum 新变体保持 append-only，既有 Register/Snapshot/Boolean/Empty 判别值不变。
`client_url` 必填且最多 64 KiB；browser 将相对 URL 基于 committed document 解析，并拒绝非
HTTP(S) 或跨 origin URL。

## 1. Manager representative

browser manager 每个 `(origin, scope)` 只返回一个 web-visible representative：

1. active；
2. waiting；
3. installing。

replacement 安装或等待期间继续返回旧 active，避免新 renderer 丢失当前控制版本。首次安装尚无
active 时返回 installing。`getRegistration(clientURL)` 使用最长 scope 匹配；
`getRegistrations()` 按 normalized scope 稳定排序。

## 2. Renderer projection

renderer 新增：

- `__zw_sw_get_registration(clientURL)`；
- `__zw_sw_get_registrations()`。

shim 按 browser registration ID upsert JS 对象：

- 重复查询复用同一 registration 对象；
- snapshot 更新 worker state、scriptURL 与 scope；
- browser 列表为 authoritative，删除本地 stale projection；
- embedded host 未提供 discovery callback 时保持既有 in-process 投影。

## 3. 验证

- protocol 9 项 Service Worker contract 测试通过，含 discovery round-trip、snapshot list、
  empty/oversized client URL fail-closed 与 nested enum 判别值回归；
- manager 2 项 discovery 测试通过：active-first representative、首次 installing 可发现；
- owner 2 项测试通过：新 tab 无旧 ID discovery、跨 origin client URL 拒绝；
- fresh V8 与 QuickJS browser/renderer E2E：
  - tab1 register `/sw.js` with scope `/app/` 并等待 activated；
  - 销毁 tab1 renderer，browser normal manager 保留；
  - tab2 同 origin 新 renderer 调 `getRegistration('/app/page')` 与 `getRegistrations()`；
  - 恢复 absolute scope、activated worker、list length 1；
  - 单项与列表返回同一 JS object identity；
  - recovered registration unregister 返回 true。
- WebView 双引擎 replacement identity：同 scope 第二次 register 复用首个 registration 对象，
  `getRegistrations()` 保持单项且对象引用一致；
- default 与 QuickJS protocol/page-runtime/browser/renderer all-targets clippy 通过；
- `make test` 通过：fresh peers、workspace V8、94 项 adapter GPU、QuickJS WebView 565/565、
  QuickJS WPT runner 110/110、QuickJS renderer；
- `make bench-gate`：16/16 microbenches；welcome/medium/morning total p95 分别为
  18.97/487.57/116.92 ms；retained form p95 0.031 ms、jank 0；绝对预算通过。

## 4. 未完成边界

- 下一导航 `navigator.serviceWorker.controller`；
- `controllerchange` 与 `clients.claim()`；
- update/updatefound、skipWaiting 页面语义；
- WPT Tier A runner 与 M2 fetch interception。
