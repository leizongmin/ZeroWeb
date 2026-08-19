# M1-3a Service Worker Lifecycle Runtime

**日期**：2026-08-19
**状态**：M1-3a complete
**前置**：[M1-2 manager](2026-08-19-m1-manager-lifecycle.md)

## 0. 交付

- `ServiceWorkerRuntime` 初始化独立 Service Worker global bootstrap：
  - `self`；
  - `addEventListener` / `removeEventListener`；
  - `ExtendableEvent` / `InstallEvent`；
  - `skipWaiting()` Promise；
  - install/activate listener registry。
- 新增 typed install/activate command 与 `LifecycleSettled` event。
- `waitUntil()` 在 dispatch 活跃窗口收集 Promise；同步异常、Promise rejection 和 deadline
  均返回 failed outcome。
- fulfilled lifetime Promise 完成后才返回 success；Promise callback 对 persistent global 的
  写入在 outcome 前可见。
- lifecycle deadline 使用 runtime 归一化后的 timeout，最长 5 秒。
- manager 新增受状态约束的 `dispatch_install` / `dispatch_activate`，只由 manager 持有
  runtime；typed outcome 经 `ServiceWorkerManagerEvent::LifecycleSettled` 穿透。

## 1. 双引擎差异修复

QuickJS 对内部 host script 的 `undefined` 完成值会在结果字符串转换阶段报错。bootstrap、
dispatch 和 microtask checkpoint 现在都返回显式字符串完成值。生命周期结果使用
`execute("JSON.stringify(...)")` 读取，不叠加 `execute_json()` wrapper。

V8 与 QuickJS 走同一 bootstrap、command 和结果解析逻辑。

## 2. 验证

runtime 双后端同一组 10 项测试，新增覆盖：

- install listener 收到 `InstallEvent`；
- fulfilled `waitUntil()` 延长事件并写入 global；
- rejected `waitUntil()` 产生失败及 rejection message；
- `onactivate` property handler 被派发；
- activate fulfilled 产生 typed success。

manager 双后端同一组 11 项测试，新增覆盖：

- 未 evaluate 时禁止 dispatch install；
- manager dispatch install/activate；
- runtime outcome 保持 phase、registration ID、success 与 message。

V8 + QuickJS feature-union 下 `zero-script-sandbox` 与 `zero-page-runtime`
`--all-targets -D warnings` clippy 均通过。

## 3. 未完成边界

- manager 尚未自动根据 lifecycle outcome 移动 installing/waiting/active slot；
- 当前测试显式调用 completion，M1-3b 将收回该外部入口；
- 无 host timer/network Promise，事件 loop 仅能推进引擎内 microtask；
- 尚未抓取 script URL，未接 WebView manager adapter；
- R3318 setTimeout 模拟仍未萎缩；
- 未实现 FetchEvent/message/importScripts。

M1-3b 必须先让 manager 自动消费 install/activate outcome，再接 WebView script fetch；不能让
WebView 根据 runtime event 自行维护第二份 slot。
