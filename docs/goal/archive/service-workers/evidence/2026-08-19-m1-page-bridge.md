# M1-3c navigator.serviceWorker 页面 Bridge

**日期**：2026-08-19
**状态**：M1-3c complete（in-process WebView）
**前置**：[M1-3b WebView host bridge](2026-08-19-m1-webview-host-bridge.md)

## 0. Host callbacks

WebView 本地 sandbox 初始化时注册三个纯值 callback：

- `__zw_sw_register(script, scope, ignoredPageUrl)`：
  - 安全上下文只读取宿主 `page_url_wire`，不信任 JS 传入页面 URL；
  - 同源校验、真实 script fetch、manager start；
  - 等待 evaluate success/failure 后返回 JSON；
- `__zw_sw_snapshot(id)`：
  - poll manager；
  - 返回规范化 script URL、scope 与真实 worker state；
- `__zw_sw_unregister(id)`：
  - manager 原子清 slot、registration 与 runtime。

`ServiceWorkerManager` 改由 WebView 的 `Arc<Mutex<_>>` 单一持有，callbacks 与 Rust API 消费同一
owner。页面无法通过伪造 `location` 绕过 secure-context/same-origin 校验。

## 1. R3318 状态投影

删除原先两段 `setTimeout(0)` install/waiting/active 模拟。新 shim：

1. `register()` 调宿主 callback；
2. evaluate 失败时 reject `Promise`；
3. evaluate 成功后创建稳定 registration/worker JS 对象；
4. `__zw_sw_snapshot` 映射 installing/installed/activating/activated/redundant；
5. state 改变时触发 `onstatechange`；
6. active 后 resolve `navigator.serviceWorker.ready`；
7. 首次注册不追溯控制当前页面，`controller` 保持 `null`；
8. timer 仅用于轮询宿主 snapshot，不再产生状态。

`getRegistration`/`getRegistrations` 继续投影当前页面已知对象；`unregister()` 已接 manager。

## 2. 双引擎验证

WebView 页面 API 新增两项双后端测试，连同 host bridge 共 6/6：

- `navigator.serviceWorker.register()` 真实抓取脚本；
- Promise resolve 后 registration.scope 为规范化绝对 URL；
- active worker state 为 `activated`；
- `ready` 由真实 active 状态 resolve；
- 首次页面 controller 为 `null`；
- unregister 返回 true；
- script compile failure reject，错误类型为 `TypeError`。

全套 WebView 回归：V8 612 unit + 17 integration，QuickJS 565 unit + 17 integration，零失败；
page-runtime 三种 feature matrix 各 57/57，六组 page-runtime/WebView clippy 全通过。

## 3. 未完成边界

- browser/renderer `external_script` 路径没有 callbacks，当前明确 reject “host bridge
  unavailable”，不回退模拟；
- 下一次 navigation 的 controller 投影尚未接；
- update/updatefound、waiting replacement、skipWaiting 页面 API 尚未接；
- ServiceWorker/Registration prototype 与完整 EventTarget shape 尚未 WPT 校准；
- WPT runner 尚未执行 Tier A；
- M2 fetch event/interception 未触碰。

M1-4 必须把同一 callback/manager contract 搬到 browser owner + renderer IPC；不能在 renderer
恢复 JS 私有生命周期数组。
