# 宿主 trait、异步加载、资源阶段与帧事务

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未实跑加载。 -->

## Module Structure

当前加载共享点是 `AsyncPageLoad::tick(&mut WebView, &mut dyn AsyncFetchHost, budget_ms)`，不是仅凭 `PageLoadHost` 注释宣称的旧 spine。查询主文档、子资源阶段、宿主请求能力或帧合并时读取；网络策略细节归 web-data。

## Gotchas
- `AsyncFetchHost::fetch_document` 默认只允许无 body GET，其他请求立即返回错误；支持 POST 的宿主要覆写该方法，不能把 fetch_text 的 GET 适配当成表单提交支持。 (`crates/page-runtime/src/lib.rs`, `crates/webview/src/async_load.rs`)
- `BlockingFetchHost` 同步执行回调后才返回已填充 Receiver，文本按 UTF-8 解码且忽略资源 meta；它不是线程池异步抓取，当前 renderer 实际使用 IpcAsyncFetchHost。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/ipc_fetch.rs`, `apps/renderer/src/runtime.rs`)
- InProcessFetchHost 实现一次性 take_document_final_url，加载器在 preload/子资源开始前采用最终 URL；IpcAsyncFetchHost 当前没有覆写该 trait 方法，不能从 inflight 存了 document_url 推断这条接线也已完成。 (`crates/webview/src/async_load.rs`, `apps/renderer/src/ipc_fetch.rs`)
- `tick` 返回 true 只代表状态更新，不代表 Complete、最终绘制或可交付帧；图片批次到达后还会在同 tick 推进预算渲染，宿主要按加载与帧状态决定发布。 (`crates/webview/src/async_load.rs`, `apps/renderer/src/runtime.rs`)

## Architecture
- InProcessFetchHost 用 net_pool，IpcAsyncFetchHost 发 FetchRequest 后登记 request_id 与一次性回复通道；两者共用加载算法但保留不同的权限、请求能力和元数据传输方式。 (`crates/webview/src/async_load.rs`, `crates/webview/src/net_pool.rs`, `apps/renderer/src/ipc_fetch.rs`)
- IPC 图片响应分块累积，只有终块才完成 Receiver；导航 clear 丢弃 pending 和文档 URL，迟到回包不再匹配新加载。 (`apps/renderer/src/ipc_fetch.rs`)
- ResourceFetchMeta 的 document/style 优先级为 4，script/font 为 3，image/media 为 2，prefetch 为 1；net_pool 把它转换成调度器输入，renderer 通过 X-Zero 头交付宿主。 (`crates/page-runtime/src/fetch_meta.rs`, `crates/webview/src/net_pool.rs`, `apps/renderer/src/ipc_fetch.rs`)
- FrameModel 的 viewport 是 CSS 像素，dirty_rects 为空表示全量光栅化；当前 renderer 用它打包帧，不能假设 WPT 和 TabWorker 都构造这个类型。 (`crates/page-runtime/src/lib.rs`, `apps/renderer/src/runtime.rs`, `apps/browser/src/tab_worker.rs`)

## Patterns
- FrameInvalidation 插入 STYLE 自动包含 LAYOUT，LAYOUT 自动包含 PAINT/HIT_TEST，PAINT/COMPOSITE/HIT_TEST 自动包含 PUBLISH；paint-only 不反向升级为布局。 (`crates/page-runtime/src/frame_invalidation.rs`)
- FrameTransaction 只在最外层 finish 返回合并工作；导航 discard_pending 保留嵌套深度，所以文档替换不应通过额外 finish 破坏事件事务。 (`crates/page-runtime/src/frame_invalidation.rs`, `apps/renderer/src/runtime.rs`)
- 样式已呈现后的非关键资源超时可 abandon_pending_subresources，把阶段设 Complete 并请求最终重绘；这不同于主文档失败，不应把可用页面一律换成错误页。 (`crates/webview/src/async_load.rs`, `apps/renderer/src/runtime.rs`)
