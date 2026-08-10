# 渲染线程化 RFC（#3 调研建议）— 主线程录制 → 独立线程光栅化

版本：v1.2 ｜ 日期：2026-08-10 ｜ 状态：**已实施（S1/S2/S3 ✅）**

> 依据：Ladybird 渲染架构演进（调研报告 §3.4）——2024 起独立 RenderingThread
>（主线程录 display list → 独立线程 Skia 光栅化，BackingStoreManager 双缓冲），
> 2026-04 起每 Navigable 独立栅格化线程。收益：主线程不承担光栅化开销，
> 滚动/交互/动画响应性显著提升。

> **与 compositor-process RFC 的关系（2026-08-10）**：
> 页面位图光栅化 + 双缓冲已由 [`compositor-process-rfc`](compositor-process-rfc-2026-08-07.md)
> C2 在 `zero-compositor` 进程承接（renderer 录制 → compositor 光栅 → Browser 显示）。
> 本 RFC scope：**DisplayList 显式契约（S1）**、**Browser UI 合成线程化 +
> 持久 RenderingThread（S2）**、**dirty region 端到端接线（S3）**。
> 不在 renderer 内重复建设第二套 compositor 平行路径。

## 一、现状审计（2026-08-10 落地）

| 环节 | 当前实现 | 位置 |
|---|---|---|
| 样式/布局/绘制命令生成 | 主线程同步 | `crates/engine/src/pipeline/mod.rs` |
| 图元序列（display list） | `RenderResult.display_list`（S1） | `render-foundation/display_list.rs` |
| 页面光栅化（默认路径） | compositor 进程 + `RenderingThread` | `apps/compositor/src/main.rs` |
| Browser 最终合成光栅化 | `rasterize_full_scene`（scope 线程，默认开） | `apps/browser/src/app_platform.rs` |
| 帧缓冲管理（页面） | compositor per-surface `BackingStoreManager` | `render-foundation/backing_store.rs` |
| 区域光栅化 API | `render_full_scene_region(_into)` + fill 裁剪 | `render-foundation/cpu/mod.rs` |
| mutation 增量录制 | 活 DOM + 增量 style/layout/paint | `pipeline::render_with_dom_mutations` |
| dirty region 消费 | IPC `PaintSnapshot.dirty_rects` → compositor 区域重绘 | `apps/compositor/src/rasterize.rs` |

## 二、目标架构（三片均已落地）

```
切片 S1：display list 显式化 ✅
  DisplayList = primitives + draw_order（在 primitives 内）+ dirty_rects

切片 S2：独立 RenderingThread + 双缓冲 ✅
  compositor：持久 RenderingThread + BackingStoreManager
  Browser UI：`rasterize_full_scene`（scope 线程）
  回退：ZW_RENDER_THREAD=0

切片 S3：增量重绘（dirty region）✅
  mutation → DisplayList.dirty_rects → IPC → copy_front + 区域重绘（fill 裁剪）
```

## 三、验收记录（2026-08-10）

| 门禁 | 结果 |
|---|---|
| `make reftest-smoke` | 42/42（含 css-variables/css-ruby，经 `fetch-wpt-smoke-subdirs`） |
| `make product-smoke` | welcome vs chromium **17.03%** ≤ 20%；struct-check 全 PASS |
| S2 scope 线程 A/B 单测 | `cpu/tests.rs` `render_full_scene_threaded_matches_direct` |
| S3 compositor 单测 | `rasterize_tests.rs` partial dirty 保留区外像素 |
| S3 compositor 集成 | `frame_flow.rs` partial dirty 端到端（进程内 copy_front + 区域重绘） |
| S3 renderer IPC | `compositor_publish_tests.rs` dirty_rects 写入 CompositorFrame |
| 回退 `ZW_RENDER_THREAD=0` | Browser/compositor 直连路径可用 |

未在本 RFC 范围跑全量 `make reftest`（16k+ case，CI/weekly 承担）。

## 四、分片状态

| 切片 | 状态 |
|---|---|
| S1 DisplayList | ✅ |
| S2 RenderingThread + Browser 接线 | ✅ |
| S3 dirty region IPC + 区域重绘 | ✅ |

- 回退开关：`ZW_RENDER_THREAD=0`（默认开；headless/reftest 默认单线程，显式 `=1` 做 A/B）
- wpt-data：`fetch-wpt-smoke-subdirs.sh` 补齐 v1.10 未打包的 smoke 子域

## 五、明确不做（后续 / 其他 RFC）

- per-Navigable 多线程栅格化（Ladybird 2026-04）
- GPU 光栅化线程（wgpu）——见 compositor-process RFC C3
- 异步「录制 N+1 ∥ 光栅 N」pipeline（Ladybird 2024+ 下一阶段）
- Browser 持久 `RenderingThread`（`FontLoader` 需 `Arc` + 动态字体加载策略，当前 scope 线程已够）
