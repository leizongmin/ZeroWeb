# 导航 epoch、加载抢占、预算与消息循环

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未运行导航。 -->

## Module Structure

入口为 `RendererRuntime::run`、`recv_next_or_timeout`、`navigate_with` 和 browser `ProcessBackend`。处理在途加载、用户输入抢占、旧文档回包或无帧时读取；脚本内部队列见 scripting，本文说明角色循环与跨进程代际。

## Gotchas
- 导航抢占不是全队列 latest-wins：提升积压中的 Navigate，丢弃它之前的 LoadHtml/导航命令，保留非加载消息相对顺序；队首已经是 Navigate 时不排干，后续导航保持 FIFO。 (`apps/renderer/src/runtime.rs`, `apps/renderer/src/navigation_preemption_tests.rs`)
- 仅提前 Navigate 而保留旧 LoadHtml 会使旧文档覆盖新 pending load，迟到生命周期又被 browser 拒收，形成导航不提交和零帧；回归钉覆盖旧加载丢弃及输入保序。 (`apps/renderer/src/navigation_preemption_tests.rs`, `apps/browser/src/process_backend.rs`)
- renderer 每帧预算常量为 16 ms，而 test-support TabWorker 当前为 8 ms；代码中的“与 tab_worker 对齐”注释不能作为数值一致证据。 (`apps/renderer/src/runtime.rs`, `apps/browser/src/tab_worker.rs`)
- 120 秒加载 deadline 在 FetchingImages 时放弃非关键资源并保留页面，其他未完成阶段转错误页；不能把所有超时都分类成主文档抓取失败。 (`apps/renderer/src/runtime.rs`, `crates/webview/src/async_load.rs`)

## Architecture
- navigate_with 先丢当前帧事务，清 pending load/script/dynamic/fetch、重置 JS 文档状态、图片 sent 与字体/焦点，再采用 navigation_epoch 并递增 document_generation；只改 URL 会留下旧文档工作。 (`apps/renderer/src/runtime.rs`)
- browser 的 NavigationCommitted 必须匹配在途 pending URL 与 epoch；ViewPainted 必须匹配当前 snapshot epoch，CompositorFrame 还校验 envelope 与 paint 的 epoch 一致。 (`apps/browser/src/process_backend.rs`)
- 主循环在等待输入前推进加载/脚本预取/动态脚本，取回 mutation、页面导航、console/error、fetch 观测与焦点；这些不是只能在初始加载结束执行的一次性动作。 (`apps/renderer/src/runtime.rs`)
- 页面脚本阶段在脚本间发现导航时中止旧文档收尾并回灌 deferred 队首；不能继续发布旧文档 DCL/load/final frame。 (`apps/renderer/src/runtime.rs`, `apps/renderer/src/page_scripts.rs`)

## Patterns
- GET 导航走 AsyncPageLoad::start；POST 当前同步 fetch_post 后 from_html，失败有回落 GET 分支；共享 host 默认 GET 限制不能被忽略。 (`apps/renderer/src/runtime.rs`, `crates/page-runtime/src/lib.rs`)
- 正常 browser 断连让角色循环返回 Ok，desktop main 随后 process::exit(0)，避免长 JS 回调的 Drop/join 拖住进程退出；Android library role 返回不等同于执行 desktop main。 (`apps/renderer/src/runtime.rs`, `apps/renderer/src/main.rs`, `git:04b4f8ed2`)
