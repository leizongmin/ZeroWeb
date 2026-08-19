# M1-3b Manager 自动推进与 WebView Host Bridge

**日期**：2026-08-19
**状态**：M1-3b complete
**前置**：[M1-3a lifecycle runtime](2026-08-19-m1-lifecycle-runtime.md)

## 0. Manager 自动推进

- script evaluate success 后 manager 自动 dispatch install；
- install outcome 由 manager 原子应用到 version slot：
  - fulfilled：`Installing -> Installed`；
  - rejected：`Redundant`，清 installing 并终止 runtime；
- 首个 active 不存在时，manager 自动 `Activating` 并 dispatch activate；
- 已有 active 时新版本保持 waiting，不提前替换；
- host 仅通过 `activate_waiting(id)` 触发 replacement activation；
- activate fulfilled 仅替换同 `(origin, scope)` active；
- activate rejected 保留旧 active。

M1-2 暴露的手工 completion 入口已从 production API 删除。`poll()` 是 runtime outcome 到
registration/slot 的单一状态推进路径；内部协调失败会产生 typed `CoordinationFailed` 并
回收版本。

## 1. WebView in-process adapter

`WebView` 新增唯一 `ServiceWorkerManager` owner，并提供真实 runtime registration API：

1. 解析 document/script/scope URL；
2. document 必须是 HTTPS，或 localhost/loopback HTTP；
3. script 与 scope 必须是同源 HTTP(S)；
4. script URL fragment 被拒绝；
5. 使用配置的 `script_source_fetcher`，否则复用 `ResourceLoader` 抓取脚本文本；
6. 把规范化 URL、scope、origin 和真实脚本字节交给 manager；
7. `poll_service_worker_runtime_events()` 推进 evaluate/install/activate。

现有 `sw_registry` API 暂时保留，仅供既有 cache-first/coverage 测试，真实路径不读取其状态。
后续页面 bridge 和 controller 必须使用 `ServiceWorkerManager`，不能写第二份状态机。

## 2. 验证

manager 双后端同一组 11 项测试继续通过，现由真实 runtime outcome 自动驱动状态。

WebView 双后端新增 4 项端到端测试：

- 相对 script/scope 解析成绝对同源 URL；
- fetcher 收到真实 document 与规范化 script URL；
- install/activate listener + fulfilled `waitUntil()` 最终进入 Activated；
- 默认 scope 等于 script 所在目录；
- rejected install 最终 Redundant；
- insecure document 与 cross-origin script 在 fetch 前拒绝。

V8 + QuickJS feature union 下 page-runtime 与 WebView `--all-targets -D warnings` clippy 通过。
全套 WebView 回归：V8 610 unit + 17 integration，QuickJS 563 unit + 17 integration，零失败。

## 3. 未完成边界

- `navigator.serviceWorker.register()` 尚未调用 WebView manager；
- R3318 JS 私有数组和 setTimeout 生命周期仍存在；
- registration/worker JS 对象尚未从 manager snapshot 投影；
- production browser/renderer IPC owner 未接；
- `Service-Worker-Allowed` response header 尚未校验；
- module worker、importScripts、update byte comparison 尚未实现；
- M2 fetch event/interception 未触碰。

M1-3c 将页面 register/query API 接到 host bridge，并以 manager snapshot 推进
installing/waiting/active，删除 timer 模拟作为状态权威。
