# M3：主线程 canvas 呈现前置阻塞修复——绘制侧 CanvasRegistry 单实例共享

日期：2026-10-22 ｜ 里程碑：M3（DC-4 前置修复批，S1b 前置）｜ 判据：同批发现批证据
（[2026-10-22-m3-s1d-scaffold](../2026-10-22-m3-s1d-scaffold/README.md)）中定位的孤儿
registry 缺陷，本批修复并全链复验。

## 判定

**修复成立**。`js_worker_main`（页面 JS 线程）不再自建 CanvasRegistry，改由 renderer
runtime 构造期创建单实例 `Arc<Mutex<CanvasRegistry>>`，同时注入 worker 线程
（`spawn_with_handlers` 参数）与 WebView（新增 `WebViewConfig.canvas_registry`，
构造期绑定管道）。复验（comp04a flag 关全链）：

| 面 | 修复前（scaffold 批证据） | 修复后（本批） |
|---|---|---|
| canvas 区域逐帧哈希（24,24)-(264,144) | `distinct_region_hashes=1`（恒定，从不出图） | **`distinct_region_hashes=49`**，113/115 帧变化 |
| 视觉 | s1 截图空白 | **蓝条出图**（comp04a-s1-canvas-bar.png） |
| 条带周期 | 无 | 哈希复现间隔 ~950–980ms = 58 次 draw × 16ms 的 mod-232 回绕周期，逐帧像素自证动画 |
| 忙窗签名 | 忙窗 beat 停顿 931ms | 保留：wt 跨忙窗 939ms（基线 949ms 同量级）；帧序列 2227→4015ms 静默覆盖忙窗（shim 模式主线程同载语义） |
| 探针判定 | SHIM_BASELINE_PASS | **SHIM_BASELINE_PASS**（错误路径/正控/canvas_idle 全绿，drawsMain 399→418 推进） |

DC-4 的「主线程阻塞时 canvas 2D 子集持续呈现」在 OFF 腿（shim 模式）的判据语义随
本批修正：修前该判据不可判定（位图从不进绘制结果）；修后 shim 模式 = 区域动画 +
忙窗期停摆（与主线程同载，DC-4 (b) 的对照腿）。

## 改动面（三文件，零语义变化）

- `crates/webview/src/webview.rs`：`WebViewConfig` 增 `canvas_registry: Option<Arc<Mutex<CanvasRegistry>>>`（默认 None = 自建，进程内路径 run_page_scripts/reftest/WPT 行为不变——构造分支 `unwrap_or_else` 等价替换）；`new_with_indexed_db_owner` 采纳注入实例。构造期绑定，无 post-construction 置换窗口。
- `apps/renderer/src/js_worker.rs`：`spawn_with_handlers` 增 registry 参数，`js_worker_main` 删除孤儿 `Arc::new(CanvasRegistry::new())`（js_worker.rs:1173 原址）；测试便捷构造 `spawn`/`spawn_with_indexed_db_handler` 自建传入（100+ 既有测试调用点零改动）。
- `apps/renderer/src/runtime.rs`：`with_io` 构造期创建单实例，同时传入 worker spawn 与 WebViewConfig（参照 fetch_observed 单实例纪律）。

## 验证

| 门 | 结果 |
|---|---|
| cargo check / fmt / clippy -D warnings | 全干净 |
| make test（guard 包裹，全量 workspace） | EXIT=0（renderer lib 299 passed 可见尾行） |
| dc4-observe off + dc4-analyze | 见上表（observe-off-analysis.txt 全量） |
| comp04a-probe flag 关 | SHIM_BASELINE_PASS（comp04a-probe.json） |
| testharness-workers FILTER=constructors/ | 仅 `use-base-url.html` 1 Fail——**既有已知失败**（workers-compat 归档 evidence/2026-10-21-m2-slice3-error-pipeline.md 记「use-base-url 1F，runner 无重定向语义；blob URL base 解析」），且 zero-wpt-runner 不依赖 zero-renderer，本批在 runner 侧仅触默认 None 等价分支——零回归 |

## 局限

- threaded 腿（`ZW_THREADED_WORKERS=1`）仍待 S1a 宿主桥落地后跑：mode 探测回落 shim
  为预期（`__zw_worker_create` 未实现），transfer/terminate 判定面未启用。
- 忙窗期帧静默（2227→4015ms，1788ms）长于 900ms 忙窗本体：含首帧启动爬坡与忙后
  渲染积压，属既有呈现节奏（comp04 基线同签名）；忙窗精确界由探针页面侧
  `loadWindow`（3446→4346ms）承载。
- 画布动画的上屏节奏依赖既有 mutation→重渲染驱动（beats 条带 50ms），canvas op 本身
  不触发多进程路径的主动重渲染——独立呈现管线（S1c patch 面）前维持该耦合。
