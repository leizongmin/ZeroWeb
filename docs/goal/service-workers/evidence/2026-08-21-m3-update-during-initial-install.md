# M3-25a Service Worker Client Update During Initial Installation

**日期**：2026-08-21
**状态**：complete

## 实现

- `ServiceWorkerManager::coalesced_update_candidate()` 现在区分首次 installing
  candidate 与已有 active/waiting 的 replacement candidate。
- client 在首次安装完成前调用 `registration.update()` 时复用当前 candidate，
  返回 `changed=false`，不重复 fetch、创建 runtime 或派发第二次 `updatefound`。
- Browser owner 与 embedded WebView adapter 共用该 manager 决策。

## 验证

- manager 回归固定首次 installing candidate 的 `(id, false)` 结果与单 runtime。
- browser owner 回归固定 update 直接响应且不产生 fetch。
- embedded WebView 的 V8、QuickJS 回归均确认 Promise 返回同一 registration 和
  installing worker。
- `update-not-allowed.https.html` 的第 1/3 项语义已覆盖；完整 case 仍保持 gated，
  因后两项还依赖 worker global `registration.update()` 与跨上下文 MessagePort transfer。

## 下一步

- 增加 typed worker update request/response，并按 caller worker state 拒绝 installing
  worker、允许 active worker 复用当前 replacement。
- 将 MessagePort endpoint 作为受限 typed wire 在 page、browser owner 和 worker runtime
  之间转移，再运行完整 `update-not-allowed.https.html`。
