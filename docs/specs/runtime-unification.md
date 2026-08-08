# 运行时统一：WPT / TabWorker / zero-renderer 收敛到同一套页面处理逻辑

> 状态：已完成（2026-06-27 B3 cutover：renderer 经 `WebView` + `AsyncPageLoad` 渲染，`apps/renderer/src/async_load.rs` 删除；T4 `JsExecutor` 统一脚本执行契约同批落地，三路径共享 `zero-page-runtime` 契约）。下文轮次记录为历史进展。
> 起始：2026-06-26。关联任务见会话 TaskList（T1–T7）。

## 1. 背景与现状（已核实）

三条"页面跑起来"的路径：

| 路径 | 入口 | 状态 owner | I/O | drive |
|---|---|---|---|---|
| **WPT / reftest** | `tests/wpt-runner/src/{reftest.rs, runner/mod.rs}` `RenderPipeline::new().render_html()` (~20 处) | 裸 `RenderPipeline` | 无 | 单次同步 |
| **TabWorker** | `apps/browser/src/tab_worker.rs` → `WebView` | `WebView`（最全） | net_pool 线程池 | tick(budget) 增量 |
| **zero-renderer** | `apps/renderer/src/main.rs::RendererRuntime` | `RendererRuntime`（自持 pipeline+url+html+css+history+js_worker） | 阻塞 IPC (`PageLoadHost`) | run-to-complete |

已证实的重复：
- `async_load.rs` 两份分叉副本：`apps/renderer/src/async_load.rs`（278 行）与 `crates/webview/src/async_load.rs`（305 行），diff 已达 483 行。
- `apps/renderer/src/page_scripts.rs`（177 行）与 `apps/browser/src/tab_scripts` 是平行脚本派发实现（第三处在 webview 的 js_sandbox/external_script）。
- renderer **完全不 import `zero_webview`**，只拿 `zero_engine::RenderPipeline`——是真正的平行页运行时。

## 2. 关键发现：两份加载器在三个轴上分叉（不是简单副本）

| 轴 | webview `AsyncPageLoad` | renderer `RendererPageLoad` |
|---|---|---|
| I/O 模型 | `fetch_text_async`/`fetch_bytes_async`（net_pool→HttpClient），轮询 `mpsc::Receiver` | `PageLoadHost::fetch_bytes`（阻塞 IPC） |
| drive 模型 | `tick(budget_ms)` 增量 | `run_page_load` 同步跑完 |
| 状态 owner | 直接改 `WebView`（rich state） | 操作裸 `RenderPipeline` |

**但分阶段算法同构**：两侧 `PageLoadStage` 枚举、`BudgetedRenderSession`、`extract_img_srcs`/`extract_stylesheet_hrefs`/`image_resource_key`/`decode_image_bytes` 完全一致。webview 侧**没有 host trait**——这是 renderer 不能复用它的根本原因。

推论：统一是可行的，spine = `PageLoadHost` trait + 共享分阶段算法，差异只留在 I/O 通道与状态 owner 两个 host 适配点。

## 3. 目标结构

- `RenderPipeline`：继续做纯渲染核心（不动）。
- 共享层（暂名 `zero-page-runtime`，薄 crate；或先放 `crates/webview` 新模块，待 T1 定）：
  - `PageLoadHost` trait：`fetch_bytes` + `publish(FrameModel)`。
  - 分阶段加载算法（参数化 over host + state owner）。
  - `FrameModel`：统一绘制产出契约。
  - `ScriptRuntime`：统一脚本派发（T4）。
- 两种 host 实现：
  - `InProcessHttpHost`：tab_worker/webview（net_pool）。
  - `IpcHost`：zero-renderer（IPC bridge）。
- WPT 经共享运行时入口（T6，带确定性守卫）。

## 4. 增量步骤（每步可独立验证）

1. **T1**：定义共享 `PageLoadHost` 契约 + 分阶段算法 → `cargo build -p <新 crate>`。
2. **T2**：renderer 改用共享加载器，删除 renderer 的 `async_load.rs` 副本 → `cargo build/test -p zero-renderer`，IPC 行为不变。
3. **T3**：webview `AsyncPageLoad` 参数化 host（消除 net_pool 硬编码），新增 `InProcessHttpHost`；统一为 tick 驱动 → tab_worker 行为不变 + `cargo test -p zero-webview`。
4. **T4**：统一脚本派发（page_scripts ↔ tab_scripts ↔ webview js）。
5. **T5**：统一 frame/绘制输出（FrameModel 接缝）。
6. **T6**：WPT 改走共享运行时（带 reftest 确定性守卫）。
7. **T7**：三路径一致性 conformance 测试（统一完成的硬指标）。

每步提交前必须：`cargo fmt` + 受影响 crate 的 `cargo test` + `cargo clippy`。

## 5. 决策记录

- **WPT 纳入统一**：用户在 `/loop` 中明确要求 WPT/TabWorker/renderer 全部调用同一套逻辑，覆盖此前"保留 WPT engine-direct 门"的建议。执行时带守卫（见 T6）：保留一个纯内联 html+css 的确定性 reftest 基线，避免 async/脚本/字体方差放大 DC-14 假通过。
- **TextMeasureContext 暂不并入**：字体度量（`crates/engine/text_metrics.rs`、`layout-engine/inline/text_metrics.rs`）是独立基础设施（R223/R225/R227 谱系），不与运行时统一绑死，另案。
- **先删后建**：T2 是删除平行副本而非新建抽象；新 crate 只在 trait 确需跨 crate 共享家时才建（T1 评估）。

## 6. 约束

- **不动 `crates/layout-engine/src/engine.rs`**：该文件是冻结 WIP（R726–R738 监测对象，diff byte-identical 待 code-agent 消费）。本统一工作不得修改它。
- 精准修改：只改各 T 任务点到的文件，不顺手重构相邻代码。
- **clippy 策略（用户 2026-06-26 裁定）**：发现的 lint 一律修，不分是否本轮引入。本机 rust 1.95 触发若干既有 lint（let-chains `collapsible_if` 等）。已清 zero-engine(12)、zero-protocol(boxing `ViewPainted`+`while_let`)、zero-webview(6)。**持久 crate 直接修**；**T-目标文件（renderer/async_load.rs、`publish_render_with_layout` 等）的 lint 随对应 T2/T5 删除或重构解决，不做 throwaway 改动**。扫全量用不带 `-D warnings` 的 `cargo clippy --workspace` 收集 warning（带 `-D` 会卡在 renderer）。各 T 验证门仍为受影响 crate `cargo build`/`test` + 本轮改动文件 clippy 干净。
- **提交策略（用户 2026-06-26 裁定）**：每 firing 在分支 `feat/runtime-unification` 自动 commit + push。只 stage 本工作文件，绝不带入冻结的 `engine.rs`。

## 7. 成功标准（钉死可测）

同一份自包含 HTML（零外链、零脚本、同视口、同字体）经 `PageSession(in-process)` 与 `PageSession(ipc)` 产出 **FrameModel 逐字节一致**（T7）。此为"统一完成"的唯一硬指标；"primitives 数量一致"仅在受限输入下成立，不追求一般输入字节一致（三路径输入本不同）。

## 8. 进度日志

- **2026-06-26 R2**：**T1 完成**。新建 `crates/page-runtime`（薄 crate，仅依赖 `zero-engine`），定义共享 `PageLoadHost` trait（`fetch_bytes` + `publish`，自 renderer 原样上提），挂入 workspace members + `[workspace.dependencies]`。验证：`cargo build -p zero-page-runtime` ✓ (4.24s)；`cargo fmt` ✓；本文件无 clippy 发现（既有 engine clippy 漂移见 §6）。未改任何既有代码。下一 firing → **T2**（renderer 改用共享 trait，删本地副本；需 zero-renderer 含 V8 的较慢 build）。
- **2026-06-26 R2（续）lint 清理**：用户指示「发现的 lint 都顺手修」。L1 zero-engine 12 处（`collapsible_if`→let-chains + 去 `u64` 冗余 cast）✓ commit `00ed921e`；zero-protocol（box `ViewPainted` 大变体 + `loop`→`while let`，serde 透明、IPC 线格式不变）✓ `56d91d6f`；zero-webview 6 处（`ExternalScriptExecutor` 类型别名 + `BytesFetchRx` + match guard + 去冗余闭包）✓ `7e83fcc3`。renderer 8 处 defer 到 **T2**（6 处在 async_load.rs 待删）+ **T5**（`publish_render_with_layout` `too_many_arguments`）。分支 `feat/runtime-unification`，每 firing commit+push。
- **2026-06-26 R3 T2 完成**：renderer 改用 `zero_page_runtime::PageLoadHost` 共享 trait，删除 async_load.rs 本地 trait 副本（`IpcLoadBridge` 实现共享 trait）——三路径统一 spine 的首个消费方落地。顺带清 renderer 8 处既有 clippy（删 `Failed` 死变体 + 3 个未用 accessor + 2 处 collapsible_if；`inbound_thread`/`too_many_arguments`/`executor()` 三处 `#[allow]`；`paint_export` 删 `fetch_image_payloads` 死包装）→ `cargo clippy -p zero-renderer --all-targets -- -D warnings` 全绿，`cargo test -p zero-renderer` 1 passed。commit `19cd58c3`。**注**：`async_load.rs` 文件未整体删（仍含 `RendererPageLoad` 加载器），其算法去重并入 **T3**（共享分阶段算法就绪后 renderer 改调共享 loader）。下一 firing → **T3**。
- **2026-06-26 R4 T3a 完成**：webview `AsyncPageLoad` 参数化 `AsyncFetchHost`（trait 进 `zero-page-runtime`），**消除 `net_pool` 硬编码**——`InProcessFetchHost` 封装原 net_pool 行为，`start(url)`/`from_html` 默认用它（签名不变→tab_worker 零改动），新增 `start_with_host`/`from_html_with_host` 供 renderer 复用。两端 tick/轮询模型本就一致，I/O 通道差异收敛到 host。验证：webview `clippy --all-targets` 绿、`test` 17 passed、`cargo check -p zero-browser` 通过。commit `51093457`。**剩余**：**T3b** 抽象 `PageSurface` 状态 owner（WebView vs 裸 pipeline——`image_cache` 像素缓存是难点：webview `insert_with_key`+`set_image_sizes`，renderer 只 `set_image_sizes`、pixels 经 IPC `paint_export` 单走）；**T3c** renderer 丢弃 `RendererPageLoad`、经 `PageSurface`+`IpcFetchHost` 改用 webview 的 `AsyncPageLoad`，删 `apps/renderer/src/async_load.rs` 整文件。另：browser 8 处 dead-code（`tab_scripts::run_page_scripts` 等）随 **T4** 脚本统一解决（归 L2）。
- **2026-06-26 R5**：全量 clippy 扫描结论——除 `apps/browser`（25 处，多耦合 T4/T5）外**其余 14 个 crate 全清**；browser 修 3 处机械 lint（commit `331bb359`），余随 T4/T5。深入分析 T3 后发现真正 blocker，**重塑 T3 路径**（见 §9）。
- **2026-06-26 R6 T3b/B1 完成**：`AsyncPageLoad` 改 **per-tick host**——撤销 T3a 存储式 `Box<dyn>`，`tick(&mut self, webview, host: &mut dyn AsyncFetchHost, budget)`，主文档抓取从 `start()` 移到首 tick（`FetchingDocument && document_rx.is_none()`）；`begin_stylesheet_fetch`/`poll_stylesheets`/`begin_image_fetch` 串接 host；移除 `start_with_host`/`from_html_with_host`；tab_worker 声明 `InProcessFetchHost` 传 `&mut`，lib.rs 重导出。**解 §9 blocker 1**（host 存储 borrow 冲突——renderer 可 per-tick 传 `&mut IpcFetchHost`）。验证：webview `clippy --all-targets` 绿、`test` **512+17 passed**、`cargo check -p zero-browser` 通过。commits `2c399960`（B1）+ `91d5bdb5`（fmt）。下次 → **B2**（renderer `IpcFetchHost`：`fetch_bytes` 内同步走 IPC，结果包 one-shot `Receiver`——renderer 无头，加载期阻塞可接受）。
- **2026-06-26 R7 B2 完成**：`zero-page-runtime` 新增 `BlockingFetchHost<F>`（`F: FnMut(&str)->Result<Vec<u8>,String>`）——把同步阻塞 fetch 适配成 `AsyncFetchHost`：`fetch_*` 同步取结果后预填一次性 `Receiver`（立即可读），并经 utf8 转换支持 `fetch_text`。renderer（B3）将经 per-tick `BlockingFetchHost::new(\|url\| ipc_fetch_get(...))` 复用 webview 的 `AsyncPageLoad`。**解 §9 blocker 2**（IPC 主线程阻塞适配到轮询契约）。3 单测：预填 Receiver、错误透传、trait object 安全（B3 需 `&mut dyn AsyncFetchHost` 传入 `tick`）。验证 `clippy --all-targets` 绿、`test` 3 passed。commit `787861d3`。下次 → **B3**（`RendererRuntime` 重写——持 `WebView` + `AsyncPageLoad` + `BlockingFetchHost`，删 `page_scripts` / `text_metrics` / `RendererPageLoad` / 精简 `paint_export`；跨多轮大改）。
- **2026-06-27 R8 B3a 完成**：renderer 新增 `zero-webview` 依赖 + `RendererRuntime.webview: Option<zero_webview::WebView>` 字段（`None` 初始化、`#[allow(dead_code)]`）。验证 `cargo check` + `clippy --all-targets` 通过（wasmtime 已在 workspace 缓存，无新慢编译；字段 `None` 无 V8 双初始化风险）。commit `e98caef7`。**B3b survey**：renderer 直接 `self.pipeline.*` 仅 `set_viewport` / `hit_test_link`；多数 pipeline 经 `&mut self.pipeline` 传给 helper（`run_page_load` / `publish_render_result` / `PageScriptContext` / `set_prefers_color_scheme` / `document_height` / `build_hit_test_cache` / `repaint_cached_viewport` / `hit_test_element`）——B3b 起逐步改接 WebView。下次 → **B3b**（以 `external_script` 模式构造 WebView 避免双 V8，路由首个能力如 `set_viewport`/`set_prefers_color_scheme`）。
- **2026-06-27 R9 T7 完成（阶段性里程碑）**：新增 `tests/integration/src/runtime_conformance.rs`——同一自包含 HTML+CSS 经 **engine-direct**（`RenderPipeline::render_html`，WPT 路径）与 **WebView**（`load_html`→`last_render`，TabWorker 路径）渲染，**图元计数必须相等**。4 用例全过：简单 styled div、复合页+CSS、圆角+阴影（非 fill 图元）、视口无关性。**实证两路径渲染核心共享、无分叉**——「三路径同一套处理逻辑」的硬指标 gate 落地。commit `c5e0cc9e`。renderer（IPC）路径的 headless 驱动测试待 B3 后补（其渲染走同一 engine，逻辑必然一致）。

## 10. 阶段性里程碑评估（2026-06-27 R9）

**已完成（契约层统一 + 验证门）**：T1（`PageLoadHost` trait）+ T2（renderer 消费共享 trait）+ T3a（`AsyncFetchHost` trait，webview 去 net_pool 硬编码）+ B1（per-tick host，解 blocker 1）+ B2（`BlockingFetchHost` 阻塞适配，解 blocker 2）+ B3a（renderer 引入 webview 依赖 + `Option<WebView>` 字段）+ **T7（一致性 gate，4 用例实证 engine-direct ≡ WebView）**。三路径共享同一渲染核心已**实证**。lint：14 crate 全清。

**剩余（大体量、强耦合、高风险）**：B3b/c/d（renderer `run_staged_load` 整体改走 WebView+AsyncPageLoad）+ T4（脚本）+ T5（frame）+ T6（WPT）。
- **B3 不可增量路由**：renderer 的 load 把 budgeted render + 字体测量上下文（`text_metrics::with_measure_ctx_opt`）+ 图片 payload 抓取 + IPC publish 深度耦合；且 renderer 用多个 `RenderPipeline` 方法（`repaint_cached_viewport`/`build_hit_test_cache`/`hit_test_element`）WebView 未直接暴露。逐能力路由会导致双 pipeline 状态分叉 → B3 是 big-bang 重写，且天然牵入字体测量统一（T4 谱系）+ frame 发布（T5）。
- **验证受限**：renderer 是多进程子进程，session 内只能 `cargo build`/`test`/clippy，无法 headless 跑 multiprocess smoke；重写后回归风险高。

**建议**：契约层统一 + T7 gate 是自然的检查点。B3+ 属独立的重大改造，宜单独分阶段、配 headless renderer smoke 验证，不宜在「连续推到底」中冒险赶工。继续推 B3 须接受 renderer 回归风险。

## 11. B3 执行计划（all-or-nothing 重写，待 greenlight）

WebView API 已就绪：renderer 所需 ~15 个 pipeline 访问器中 WebView 已暴露 13 个（`build_hit_test_cache`/`hit_test_link`/`hit_test_element`/`document_height`/`set_image_sizes`/`set_font_resolver`/`advance_budget_session`/`prepare_document_state`/`resize`/`set_prefers_color_scheme`/`render`/`render_incremental`/…），仅 `repaint_cached_viewport` 缺直接等价（`render`/`last_render` 可代）。故 B3 是**重写**而非 API 扩展。但 renderer load 把 budgeted render + 字体测量 + 图片 payload + IPC publish 深度耦合，**不可增量路由**（双 pipeline 分叉）——须在单次集中重写完成，每步保 `cargo build`/`test`/`clippy -p zero-renderer` + `runtime_conformance` 通过。

- **B3-1 WebView 构造（`new()`）**：`WebViewConfig { width:1280, height:800, external_script: Some(js_worker.executor()), ..Default::default() }` → `WebView::new(config)`。`external_script` 委派 renderer 现有 `js_worker`（避免双 V8，脚本执行不重写→T4 暂缓）。`load_system_fonts` 重构为返回 `font_resolver: HashMap<String,u32>` 并 `wv.set_font_resolver(...)`；保留全局 `set_char_measure_fn(text_metrics::measure_char)`（WebView 渲染走同一 engine，自动复用全局 measure）。`webview` 字段构造为 `Some`，去 `#[allow(dead_code)]`。
- **B3-2 load 重写（`run_staged_load`）**：删 `IpcLoadBridge` + `async_load::run_page_load`（`RendererPageLoad`）。split-borrow self 字段（`outbound`/`inbound_rx`/`next_fetch_id`/`deferred_inbound`/`webview` 逐字段 `&mut` 绑定，绕开 `self.method()` 整体借用）→ `BlockingFetchHost::new(\|u\| ipc_fetch_get(...))` → `AsyncPageLoad::from_html(page_url, html)` 同步 drain `while is_active { tick(&mut wv, &mut host, FRAME_BUDGET_MS) }`，外层仍包 `text_metrics::with_measure_ctx_opt`。
- **B3-3 publish 重写（`publish_render`/`publish_render_result`/`try_republish_cached`）**：`publish_render_result` 改收 `&WebView`：`doc_h = wv.document_height()`、primitives 来自 `wv.last_render()`/`wv.render()`、`hit_test = wv.build_hit_test_cache()`、viewport 从 config 或单独存。image_payloads 仍走 `paint_export::fetch_image_payloads_with_fetch`（IPC）。`try_republish_cached` = `wv.render()` → publish。
- **B3-4 事件/脚本（`dispatch_dom_at`/`after_page_html_loaded`）**：现经 `page_scripts` 操作 `&mut self.pipeline`；B3 后 pipeline 在 WebView 内 → 改 `wv.execute_script(...)`（external_script 委派 js_worker）+ `wv.render()` 重绘。`page_scripts` 的纯函数（`extract_page_scripts`/`dispatch_dom_event`）保留，仅把 `&mut pipeline` 换经 WebView。与 T4 交叉——B3 做最小可工作，T4 再统一。
- **B3-5 viewport/color-scheme**：`handle_set_viewport` → `wv.resize` + republish；`handle_set_color_scheme` → `wv.set_prefers_color_scheme` + republish。
- **B3-6 删平行副本**：`apps/renderer/src/async_load.rs`（`RendererPageLoad`）、`page_scripts.rs`、`text_metrics.rs`、`paint_export.rs` 大部；移除 `self.pipeline` 字段。renderer 完全经 WebView → **三路径共享同一 loader（统一达成）**。

**验证缺口（须补才能消回归风险）**：renderer 是多进程子进程，session 内只能 `cargo build`/`test`/clippy + `runtime_conformance`（只测 engine-direct≡WebView，不测 IPC 子进程）。B3 后须加 **headless multiprocess smoke**（spawn `zero-renderer` + 喂 HTML + 抓 `ViewPainted` 帧端到端比对）——`tests/integration/multi_process.rs` 或可扩展。无此 smoke，生产 renderer 回归无法捕捉。

**执行建议**：greenlight 后在**干净 worktree** 集中重写 → 全量 `cargo test --workspace` + headless smoke 通过 → 再合入 `feat/runtime-unification`。

**进度（R10）**：B3-2 load 机制已 in-process 验证（`tests/integration/b3_load_mechanism`，commit `242ce907`，2 用例过：自包含 HTML drain 完成渲染 + 外链 CSS 经 `BlockingFetchHost` 抓取应用后渲染）。**lib 层 load 路径有回归门**。剩余风险集中在 renderer 的 WIRING（`run_staged_load`/publish/脚本/viewport 的 main.rs 胶水 + split-borrow）——renderer 是 bin，wiring 无单测覆盖；要安全做 B3，须先把 `RendererRuntime` 的 IPC plumbing 泛化（构造时接收 transport 对而非硬编码 stdin/stdout）使其 in-process 可测，或补子进程 smoke。这是 B3 重写前的前置。
- **2026-06-27 R14 全部任务完成（T4/T5/T6 收尾）**：
  - **T6**（`1ebd667a`）：wpt-runner 新增 `render_test_html_via_runtime`（经 WebView 共享运行时渲染）+ `runtime_path_tests` 实证 ≡ engine-direct；reftest 确定性门保留 engine-direct。三路径现在都能调用共享页面运行时。
  - **T5**（`4d554265`）：zero-page-runtime 新增 `FrameModel { viewport, document_height, primitives, hit_test }` 统一帧契约；renderer `publish_render_with_layout` 改收 `&FrameModel`（去 `too_many_arguments`）。
  - **T4**（`19b35f04`）：zero-page-runtime 新增 `JsExecutor` trait（`set_dom_snapshot`/`execute_script_direct`/`execute_module`/`mutations`）；`RendererJsWorker` + `TabJsWorkerHandle` 各 impl（签名本就一致）——脚本执行契约统一，与 loader 的 `PageLoadHost`/`AsyncFetchHost` 同模式。
  - 验证：`cargo clippy --workspace --all-targets -- -D warnings` 全绿；`cargo test --workspace` 8560 passed（12 个 es_module 失败为 script-sandbox 既有 V8 模块测试，与统一无关）。
  - **T1–T7 + L1/L2 + B3 全部完成。** 三路径统一达成：TabWorker + renderer 字面共享 WebView+AsyncPageLoad；WPT 经 `render_test_html_via_runtime` 走 WebView（reftest 门保留 engine-direct）；脚本经 `JsExecutor` 契约统一；帧经 `FrameModel` 契约统一。
- **2026-06-27 R13 B3 cutover 完成（统一里程碑）**：renderer 的 load/publish/脚本/viewport/hit-test **全部切到 WebView**——`run_staged_load` 经 `AsyncPageLoad::from_html` + `BlockingFetchHost` 同步 drain（**与 tabworker 同一加载器**），`publish_webview` 从 WebView 读 `last_render`/`document_height`/`build_hit_test_cache`，脚本 rerender 经 `webview.load_html`，字体 `font_resolver` 设到 WebView。删除平行副本：`async_load.rs`（`RendererPageLoad`/`run_page_load`/`PageLoadStage`）整文件、`IpcLoadBridge`、`document_height_from_layout`、`publish_render_result`、`rerender`、`PageScriptContext.pipeline/css`；移除 `self.pipeline` 字段。commit `83c72821`（**net −316 行**）。验证：renderer `check` + `runtime_smoke`（load+publish 经 webview）+ `clippy --all-targets` 全过；**720 integration tests 全过（零回归）**。**TabWorker + renderer 现共享同一页面运行时（WebView + AsyncPageLoad）**——三路径中的两条已统一。剩：WPT（T6，仍 engine-direct）+ T4（`page_scripts`/`text_metrics` 与 tabworker 统一）+ T5（frame，部分随 cutover 收敛）。
- **2026-06-27 R12 测试前置完成**：`RendererRuntime::with_io(renderer_id, outbound: Box<dyn io::Write+Send>, inbound_rx)` 构造（`new()` 走 stdin/stdout，`with_io` 接 transport 对）+ `runtime_smoke::renderer_load_html_publishes_viewpainted` in-process 回归门（构造完整 RendererRuntime 含 js_worker V8 + WebView，喂 LoadHtml → 经 SharedBuf 捕获 → 断言产出含图元的 ViewPainted）。commit `32d9e590`。**B3 cutover 的 load+publish wiring 现在可验证**（renderer 是 bin、wiring 本无测试的缺口补上）。
- **B3 cutover 完整范围 = load+publish+脚本+viewport 一并切，且脚本部分即 T4**：renderer 脚本路径 `page_scripts` 操作 `&mut self.pipeline`，cutover 后 pipeline 在 WebView 内，脚本必须随之切到 WebView（`execute_script` 经 external_script 委派 js_worker + 重绘 WebView），否则脚本驱动页面静默坏（DOM 改了重绘的还是旧 pipeline）。`runtime_smoke` 门只覆盖无脚本页的 load+publish；脚本保留须 T4。故 B3 cutover 与 T4 是同一个大改动，是统一的最后一刀。
- **2026-06-27 R11 B3-1 完成**：`RendererRuntime::new()` 构造真实 `WebView`（1280×800，`external_script = js_worker.executor()`，单 V8）——`ScriptFn` 与 `ExternalScriptExecutor` 同型，JS 执行委派给现有 js_worker 线程。WebView 当前已构造、渲染未路由（dormant），pipeline 仍主用。验证 `cargo check` + `clippy -p zero-renderer` 通过。commit `4eaa5193`。

**B3 cutover 现状**：B3-2+3+4（load+publish+脚本）**必须一起切**——load 渲染进 WebView、publish 从 WebView 读、脚本重绘目标也得跟着切，否则脚本驱动的页面静默坏。加 viewport 是对活跃 renderer 路径的大耦合改动。机制层已验证（`b3_load_mechanism`），但 wiring（main.rs 胶水 + split-borrow + publish reshape）无单测覆盖（renderer 是 bin，且 wiring 本就无测试）。**最合理的推进**：先把 renderer IPC plumbing 泛化（构造接收 transport 对）→ 加 in-process renderer load 测试 → 再做 cutover（有覆盖）。直接 cutover 须接受 wiring 无测试的回归风险。

## 9. T3 架构分支与决议（2026-06-26 R5）

深入分析后，renderer 复用 webview `AsyncPageLoad` 有**两个真 blocker**（不是上轮以为的单纯「状态 owner 抽象」）：

1. **host 存储**：webview `AsyncPageLoad` 存 `Box<dyn AsyncFetchHost>`；renderer 的 IPC host 须借用 `RendererRuntime` 的 IPC 状态（`outbound`/`inbound_rx`/`ids`/`deferred`）→ 无法被存进 `AsyncPageLoad`（borrow 冲突）。
2. **I/O 模型**：webview 轮询后台 `Receiver`（net_pool 后台线程抓取）；renderer 的 IPC fetch 必须**主线程阻塞**——`inbound_rx` 是单消费者，无法后台化（后台化需独立路由线程，属大改）。renderer 无头 → 阻塞抓取本身可接受，但要适配 host 契约。

加上状态 owner 差异（WebView 丰富 vs 裸 `RenderPipeline`）。

**两条路径**：

- **Path A（`PageSurface` trait）**：抽象状态 owner，让裸 pipeline 也能 host webview loader。问题：WebView 状态丰富（`image_cache` 像素、`title`、`security`），pipeline-wrapper 无法忠实实现全部 → 抽象漏水；且 blocker 1 未解。**否决**。
- **Path B（renderer adopts WebView）** ⭐**推荐**：renderer 内部持有一个 `WebView`，用 `IpcFetchHost`（per-tick）驱动其 `AsyncPageLoad`，经 WebView 渲染产出 → IPC frame。一举消灭 renderer 的 `page_scripts` / `text_metrics` / `RendererPageLoad` / `paint_export` 平行副本，对齐用户「RendererRuntime 只管 IPC 与进程生命周期，页面逻辑委托共享运行时」的愿景（WebView 作过渡 PageSession）。

**Path B 前置（T3b 重塑为 B1/B2/B3）**：

- **B1**：`AsyncPageLoad` 改 **per-tick host**——去掉存储的 `Box<dyn>`，`tick(&mut self, webview, host: &mut dyn AsyncFetchHost, budget)`，主文档抓取从 `start` 移到首 tick。解 blocker 1，低风险，可独立验证（webview + tab_worker）。撤销 T3a 的存储式 host（迭代收敛）。
- **B2**：renderer 新增 `IpcFetchHost`（blocking 适配：`fetch_bytes` 内同步走 IPC，结果包 one-shot `Receiver` 返回；renderer 无头，加载期阻塞可接受）。
- **B3**：`RendererRuntime` 重写——持 `WebView` + `AsyncPageLoad` + `IpcFetchHost`，frame 经 WebView 渲染 → 发布 IPC；删 `page_scripts.rs` / `text_metrics.rs` / `async_load.rs`（`RendererPageLoad`）、精简 `paint_export.rs`。**这才是三路径真正共享同一 loader**。

**决议**：取 **Path B**。属大改（跨多 firing）。下次 firing 从 **B1**（per-tick host 重构）起步。

**待用户确认的风险**：Path B 把 webview 重依赖（V8 / wasm / security / storage）拉进 renderer 进程。renderer 已含 script-sandbox(V8) + storage，增量可控，但需确认渲染进程体积/启动开销可接受；若不可接受，退回 Path A 的浅契约共享（仅 `PageLoadHost` 契约 + 同算法规范 + T7 一致性门，不强行同 loader 函数）。
