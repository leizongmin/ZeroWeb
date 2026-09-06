# M1-1 Threaded Runtime 与 Typed SW Runtime

**日期**：2026-08-19
**状态**：M1-1 complete
**决策依据**：[已批准的方案 C RFC](../m0-execution-environment-rfc.md)

## 0. 交付

- 从 V8/QuickJS Dedicated Worker 重复实现中抽取 crate-private
  `ThreadedRuntimeCore<C, E>`：
  - typed command/event channel；
  - 独立命名线程；
  - shared terminate flag；
  - bounded join / detach fallback；
  - 幂等 terminated 状态。
- V8 与 QuickJS `WorkerRuntime` 均改为消费共享核，public API、WebView worker map 和
  engine-specific 执行逻辑不变。
- 新增 public `ServiceWorkerRuntime`：
  - 独立引擎线程和持久 global context；
  - engine initialization handshake；
  - typed `Evaluated` / `ScriptError` / `Closed` 事件；
  - compile/runtime/timeout 等稳定错误分类；
  - 幂等 bounded shutdown。
- SW 专用配置强制 64 MiB heap 上限、最多 5 秒单次 evaluate deadline；错误事件不携带脚本
  正文。
- 修复 QuickJS `execute()` 异常分类：读取异常对象的 `name`，不再因 message 缺少
  `SyntaxError` 字样而把语法错误误报为 runtime error。

## 1. 架构边界

共享核不持有 V8 isolate、QuickJS Runtime/Context 或 JS 对象。引擎对象只在 engine thread
和既有 adapter 内存活：

```text
WorkerRuntime adapter ─┐
                       ├─ ThreadedRuntimeCore<Command, Event>
ServiceWorkerRuntime ──┘
        |
        +─ engine thread owns V8Sandbox or QuickJSSandbox
```

Dedicated Worker 继续使用字符串消息 adapter；Service Worker evaluate 使用 typed event，
未以 `postMessage(String)` 模拟生命周期事件。

## 2. 双引擎验证

| 矩阵 | 结果 |
|------|------|
| `zero-script-sandbox` V8 | 162 unit + 10 integration，全部通过 |
| `zero-script-sandbox` QuickJS-only | 83 unit + 1 scope + 10 integration，全部通过 |
| V8 + QuickJS feature union | 202 unit + 1 scope + 10 integration，全部通过 |
| SW typed runtime V8 | 7/7 |
| SW typed runtime QuickJS | 7/7 |
| WebView Dedicated Worker V8 | 17/17 |
| WebView Dedicated Worker QuickJS | 17/17 |

三种 `zero-script-sandbox --all-targets -D warnings` clippy feature matrix 均通过：

1. 默认 V8；
2. QuickJS-only；
3. V8 + QuickJS union。

## 3. Typed Runtime 覆盖

双后端同一组测试验证：

- evaluate success 与 persistent global；
- compile error 与 runtime error 分类；
- 死循环 timeout 后下一脚本可继续执行；
- 空脚本可执行、空 script URL 在 host 边界拒绝；
- shutdown 幂等且终止后拒绝 evaluate；
- heap / timeout / persistent-context 策略封顶；
- engine error event 不泄漏脚本正文。

## 4. 未完成边界

本切片只完成 M1-1 runtime 骨架，不代表真实 Service Worker 生命周期完成：

- 尚无 `install` / `activate` / `waitUntil()` typed command；
- 尚无 `ServiceWorkerGlobalScope` bootstrap 和事件 listener registry；
- 尚无 manager、script URL fetch、registration bridge 或 controller；
- 未修改 R3318 timer shim；
- 未触碰 M2 fetch bridge。

下一切片 M1-2 必须由 manager 作为状态单一 owner，并把 evaluate 结果接到 registration version
slot；不能让 runtime 自行推进 storage registry 状态。
