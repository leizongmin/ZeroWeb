# M1 Worker Runtime 抽取前基线

**日期**：2026-08-19
**状态**：M0 readiness evidence（零 production 源码改动）
**关联 RFC**：[M0 执行环境 RFC](../m0-execution-environment-rfc.md)

## 0. 结论

- 方案 C 的抽取输入已定位：V8 `worker.rs` 与 QuickJS `quickjs_worker.rs` 各自实现
  `WorkerRuntime`，WebView 是唯一 production consumer。
- Dedicated Worker 当前公开行为已用双引擎 WebView 集成测试固定；批准后 M1-1 可以在不改
  WebView API 的前提下抽取线程核。
- 两后端当前并非完整对称：
  - V8 worker 有 `timeout_ms` 看门狗和超时后恢复测试；
  - QuickJS worker 只在 `terminate()` 时通过 interrupt flag 中断，未消费
    `SandboxConfig::timeout_ms`。
- `WorkerRuntime::new()` 在线程启动后直接返回 `Running`，没有初始化完成/脚本求值成功事件；
  Service Worker 注册不能复用这一近似，typed runtime 必须显式报告 evaluate 结果。

## 1. 当前边界

### 共享形状

两后端具有相同的公开形状：

- `WorkerRuntime::{new, post_message, execute_script, try_recv, recv, recv_timeout, terminate}`
- `WorkerState::{Initializing, Running, Terminated}`
- `WorkerEvent::{Message, Error, Closed}`
- `Execute` / `PostMessage` / `Terminate` 三种内部命令
- command/event channel、worker `JoinHandle`、terminate flag、bounded join、`Drop`

这些是 M1-1 可抽取的线程生命周期和通道外壳。现有 public `WorkerRuntime` 仍应作为
Dedicated Worker adapter 保留，WebView 无需感知内部抽取。

### 引擎专属部分

| 后端 | 必须留在 engine adapter 的能力 |
|------|-------------------------------|
| V8 | isolate 创建与 thread-safe handle、heap limits、watchdog Arm/Disarm、Context/bootstrap、V8 scope |
| QuickJS | Runtime/Context 创建、memory limit、interrupt handler、Context eval/bootstrap |

V8 isolate handle 和 QuickJS Runtime/Context 都不能进入跨线程纯值协议。抽取核只拥有通道、
线程生命周期和引擎无关状态；执行对象留在 worker thread。

### 消费方

- production：`zero-webview::WebView` 的 `HashMap<u64, WorkerRuntime>`
- WebView API：create/configured create、post message、execute script、poll、terminate、
  count/running query、terminate all
- 非 production：`script_sandbox_bench`

未发现其他 production owner。M1-1 不应改 WebView 上述 API 或 worker ID/map 语义。

## 2. 实测基线

### Crate 内部

| 矩阵 | 命令 | 结果 |
|------|------|------|
| V8 worker | `cargo test -p zero-script-sandbox worker::tests` | 20/20 |
| QuickJS worker | `cargo test -p zero-script-sandbox --no-default-features --features quickjs quickjs_worker::tests` | 3/3 |

V8 20 项覆盖消息、状态保持、多 worker 隔离、终止后拒绝、double terminate、死循环强制中断、
watchdog timeout 与恢复。QuickJS 3 项覆盖基本消息、死循环 terminate 和 Drop bounded return。

### WebView consumer

| 矩阵 | 命令 | 结果 |
|------|------|------|
| V8 WebView | `cargo test -p zero-webview tests::worker_integration` | 17/17 |
| QuickJS WebView | `cargo test -p zero-webview --no-default-features --features quickjs tests::worker_integration` | 17/17 |

17 项共同覆盖创建/终止、消息、状态保持、额外脚本、多 worker 隔离、JSON、批量终止、
render 并行、自定义配置、终止后操作和 ID 单调性。这组双后端结果是 M1-1 的行为不变锚点。

### 编译门禁

以下 `--all-targets -D warnings` 均通过：

1. 默认 V8；
2. QuickJS-only；
3. V8 + QuickJS feature union。

feature union 当前按 `lib.rs` 约定导出 V8 `WorkerRuntime`，QuickJS worker 只在
`all(quickjs, not(v8))` 下导出。抽取不得重新引入同名 re-export 冲突。

## 3. 批准后 M1-1 顺序

1. 先把上述 17 项 WebView 测试保持为双后端 conformance gate。
2. 抽取 command/event channel、state、bounded join、Drop/Debug 等线程外壳。
3. V8 和 QuickJS 分别实现 engine-thread adapter，不跨线程传引擎对象。
4. Dedicated Worker public adapter 保持方法、事件和错误行为不变。
5. 再新增 `ServiceWorkerRuntime` typed command/event 骨架；不得用
   `postMessage(String)` 模拟 install/activate。
6. 为 SW evaluate 增加明确 success/error handshake，并分别验证两后端。
7. 单独补齐 QuickJS `timeout_ms` 语义后，才可宣称双引擎超时对称。

## 4. 禁止偷换

- 不把当前 `Running` 即“脚本已成功执行”用于 SW registration。
- 不把 QuickJS terminate interrupt 误记为 event timeout。
- 不把 V8 watchdog/isolate handle塞进公共 wire 类型。
- 不触碰 R3318 页面 shim、fetch bridge、WebView worker API 或 worker ID 管理。
- 不因抽取而复制第三套 bootstrap、watchdog 或 bounded join。

## 5. M1-1 验收

- [ ] 本文四条测试命令继续全绿。
- [ ] 三种 clippy feature matrix 继续全绿。
- [ ] Dedicated Worker public API 和 WebView worker map 无调用方改动。
- [ ] 死循环 terminate/Drop 仍在既有上限内返回。
- [ ] SW evaluate 有 typed success/error，不依赖 Dedicated Worker 字符串消息。
- [ ] QuickJS timeout 差异被实现并测试，或作为未完成项明确阻止 M1-1 完成声明。
