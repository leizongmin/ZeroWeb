# 运行时统一：WPT / TabWorker / zero-renderer 收敛到同一套页面处理逻辑

> 状态：进行中（由 `/loop` 驱动，cron 每 10 分钟推进一次）。
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
- **clippy 工具链漂移（2026-06-26 发现）**：本机 rust 1.95 下 `cargo clippy` 对 `zero-engine` 报 12 个 `collapsible_if`（let-chains 在 1.95 已稳定，MSRV 1.85 下不触发）——**与本统一工作无关的既有 lint**。因此各 T 的验证门 = `cargo build` / `cargo test` 受影响 crate 通过即可；clippy 仅 best-effort，**不得顺手修这些既有 engine lint**（scope creep + 触碰 R-series 状态）。仅需保证**本次新增/改动文件本身**无 clippy 发现。

## 7. 成功标准（钉死可测）

同一份自包含 HTML（零外链、零脚本、同视口、同字体）经 `PageSession(in-process)` 与 `PageSession(ipc)` 产出 **FrameModel 逐字节一致**（T7）。此为"统一完成"的唯一硬指标；"primitives 数量一致"仅在受限输入下成立，不追求一般输入字节一致（三路径输入本不同）。

## 8. 进度日志

- **2026-06-26 R2**：**T1 完成**。新建 `crates/page-runtime`（薄 crate，仅依赖 `zero-engine`），定义共享 `PageLoadHost` trait（`fetch_bytes` + `publish`，自 renderer 原样上提），挂入 workspace members + `[workspace.dependencies]`。验证：`cargo build -p zero-page-runtime` ✓ (4.24s)；`cargo fmt` ✓；本文件无 clippy 发现（既有 engine clippy 漂移见 §6）。未改任何既有代码。下一 firing → **T2**（renderer 改用共享 trait，删本地副本；需 zero-renderer 含 V8 的较慢 build）。
