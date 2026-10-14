# 共享页面运行时契约

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次仅静态核验。 -->

## Module Structure

`zero-page-runtime` 提供加载、脚本执行、帧失效和用户动作的共享类型。嵌入 WebView、renderer、测试支持 TabWorker 和 WPT testdriver 实际消费不同部分；不要把 crate 注释中的“三路径统一”理解成每个类型都被三路径使用。SW/IDB/Cache/OPFS 业务语义归 web-data。

## API Surface

| 真实 API | 契约 | 来源 |
| --- | --- | --- |
| `AsyncFetchHost` | per-tick 宿主、Receiver 返回、资源 meta、请求与最终 URL 能力 | `crates/page-runtime/src/lib.rs` |
| `ResourceFetchMeta` | 资源 destination 与调度优先级 | `crates/page-runtime/src/fetch_meta.rs` |
| `JsExecutor` | 快照、脚本/module、mutation、优先有界执行扩展 | `crates/page-runtime/src/lib.rs` |
| `FrameModel` | CSS viewport、文档高度、图元、脏区和 hit-test | `crates/page-runtime/src/lib.rs` |
| `FrameInvalidation`、`FrameTransaction` | 失效依赖展开与嵌套事件合并 | `crates/page-runtime/src/frame_invalidation.rs` |
| `PageNodeRef`、`HtmlActionRequest`、`plan_html_action`、`resolve_html_action` | 文档身份、纯计划及提交/回滚结果 | `crates/page-runtime/src/form_control.rs`, `crates/page-runtime/src/html_actions.rs` |
| `PageLoadHost`、`BlockingFetchHost` | 库中仍定义的同步契约/适配器，不等于当前生产加载入口 | `crates/page-runtime/src/lib.rs` |

## Usage Examples

renderer 每 tick 借用实际 IPC host，然后判定是否有可发布 render。来源：`apps/renderer/src/runtime.rs`，`tick_pending_load`。

```rust
let mut host = IpcAsyncFetchHost::new(outbound, next_fetch_id, inflight);
let changed = pending.load.tick(webview, &mut host, budget_ms);
```

WPT testdriver 使用同一用户动作请求，不直接在 runner 中复制默认动作。来源：`tests/wpt-runner/src/testharness.rs`，`dispatch_action` 调用片段。

```rust
webview.dispatch_loaded_page_user_action(HtmlActionRequest {
    target,
    action,
    shift: false,
})
```

## Gotchas
- AsyncFetchHost 的 Receiver 返回形态不证明 fetch 本身非阻塞；BlockingFetchHost 在返回前执行同步回调，IpcAsyncFetchHost 才通过 pending 回复表延后完成。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/ipc_fetch.rs`)
- 库定义与消费范围要分别核验：当前 FrameModel 在 renderer 发布中构造，TabWorker 直接维护 TabSnapshot，WPT runner 还有直接 RenderPipeline 路径。 (`apps/renderer/src/runtime.rs`, `apps/browser/src/tab_worker.rs`, `tests/wpt-runner/src/runner/mod.rs`)

## Architecture
- 共享动作层输出计划和 PageEffect，不持有平台窗口或网络能力；WebView adapter 解析 live DOM，宿主执行实际导航/焦点副作用。 (`crates/page-runtime/src/html_actions.rs`, `crates/webview/src/webview/user_actions.rs`, `apps/browser/src/tab_worker.rs`)

## Consumer Analysis
- 嵌入 WebView 用 AsyncPageLoad/InProcessFetchHost 和用户动作 adapter，default trait 的 POST/最终 URL 能力不能替代 adapter 实现。 (`crates/webview/src/async_load.rs`, `crates/webview/src/webview/user_actions.rs`)
- renderer 用 IpcAsyncFetchHost、FrameModel、FrameTransaction 和 RendererJsWorker 的 JsExecutor，导航要同时清 fetch、动作身份和待发布帧。 (`apps/renderer/src/runtime.rs`, `apps/renderer/src/ipc_fetch.rs`, `apps/renderer/src/js_worker.rs`)
- browser 的 test-support TabWorker 用 InProcessFetchHost 与 HtmlActionRequest，TabJsWorkerHandle 实现 JsExecutor；这是测试支持路径，不是发布版进程内渲染授权。 (`apps/browser/src/tab_worker.rs`, `apps/browser/src/tab_js_worker.rs`, `apps/browser/src/main.rs`)
- WPT testharness 的 dispatch_action 消费 HtmlActionRequest，DisabledTarget 对自动化 resolve 视为成功，是否派发事件仍由页面断言判断；不能用 automation 成功回执证明实际激活。 (`tests/wpt-runner/src/testharness.rs`)

## Child Knowledge Nodes
- [宿主 trait、异步加载、资源阶段与帧事务](hosts-and-loading.md) — 调整加载阶段、host 能力、资源调度或帧合并。
- [交互状态、HTML 动作与宿主副作用契约](interaction-contract.md) — 排查失焦、过期节点、超时动作与默认副作用。
