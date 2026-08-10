# 渲染线程化 RFC（#3 调研建议）— 主线程录制 → 独立线程光栅化

版本：v1.1 ｜ 日期：2026-08-10 ｜ 状态：**实施中（S1/S2 基础设施 ✅ / S3 接线中）**

> 依据：Ladybird 渲染架构演进（调研报告 §3.4）——2024 起独立 RenderingThread
>（主线程录 display list → 独立线程 Skia 光栅化，BackingStoreManager 双缓冲），
> 2026-04 起每 Navigable 独立栅格化线程。收益：主线程不承担光栅化开销，
> 滚动/交互/动画响应性显著提升。

> **与 compositor-process RFC 的关系（2026-08-10）**：
> 页面位图光栅化 + 双缓冲已由 [`compositor-process-rfc`](compositor-process-rfc-2026-08-07.md)
> C2 在 `zero-compositor` 进程承接（renderer 录制 → compositor 光栅 → Browser 显示）。
> 本 RFC 剩余 scope：**DisplayList 显式契约（S1）**、**Browser UI 合成线程化 +
> 持久 RenderingThread（S2）**、**dirty region 端到端接线（S3）**。
> 不在 renderer 内重复建设第二套 compositor 平行路径。

## 一、现状审计（2026-08-10 更新）

| 环节 | 当前实现 | 位置 |
|---|---|---|
| 样式/布局/绘制命令生成 | 主线程同步 | `crates/engine/src/pipeline/mod.rs` |
| 图元序列（display list 雏形） | `RenderResult.display_list`（S1 显式化） | `render-foundation/display_list.rs` |
| 页面光栅化（默认路径） | compositor 进程 + `render_full_scene_threaded` | `apps/compositor/src/main.rs` |
| Browser 最终合成光栅化 | `rasterize_full_scene`（默认 scope 线程，`ZW_RENDER_THREAD=0` 直连） | `apps/browser/src/app_platform.rs` |
| 帧缓冲管理（页面） | compositor per-surface `BackingStoreManager` | `render-foundation/backing_store.rs` |
| 区域光栅化 API | `render_full_scene_region(_into)` | `render-foundation/cpu/mod.rs` |
| mutation 增量录制（M3-S9） | 活 DOM + 增量 style/layout/paint | `pipeline::render_with_dom_mutations` |
| dirty region 消费 | S3 接线：IPC → compositor 区域重绘 | 本 RFC v1.1 实施 |

## 二、目标架构（分三片，可独立验收回退）

```
切片 S1：display list 显式化（纯重构，零行为变更）✅
  DisplayList = primitives + draw_order（在 primitives 内）+ dirty_rects
  RenderResult 持 DisplayList；stats.dirty_rects 与 display_list 同步

切片 S2：独立 RenderingThread + 双缓冲（核心）⚠️ 部分
  主线程：render_html → DisplayList（录制）——renderer/compositor 路径已达成
  渲染线程：RenderingThread 持久 worker + BackingStoreManager（compositor ✅）
  Browser UI 合成：`rasterize_full_scene`（scope 线程，默认开）✅
  回退：ZW_RENDER_THREAD=0

切片 S3：增量重绘（dirty region）⏳
  DirtyTracker / mutation 变更盒 → DisplayList.dirty_rects
  → IPC PaintSnapshot → compositor render_full_scene_region_into（保留 front 像素）
  验证：与全量重绘逐像素一致 + 重绘面积基准
```

## 三、收益与成本

| 收益 | 成本/风险 |
|---|---|
| 主线程不承担光栅化 → 滚动/交互响应性 | 线程同步复杂度（DisplayList 所有权转移） |
| 动画帧率提升（录制∥光栅 pipeline） | 双缓冲内存 +1 帧 |
| S3 局部重绘降 CPU（mutation/样式变更） | dirty rect 合并与 stale 帧丢弃逻辑 |
| 为 GPU 光栅化（wgpu C3）铺路 | reftest headless 保留单线程（确定性） |

## 四、分片实施计划

| 切片 | 动作 | 验证 | 状态 |
|---|---|---|---|
| S1 | DisplayList 包装 primitives + dirty_rects | reftest 全量无 diff | ✅ |
| S2 | RenderingThread + BackingStoreManager + Browser 接线 | A/B 逐像素 + product-smoke | ⚠️ |
| S3 | dirty region IPC + compositor 区域重绘 | 与全量一致 + 基准 | ⏳ |

- 每片遵守 `docs/goal/ai-refactor-acceptance.md`
- 回退开关：`ZW_RENDER_THREAD=0`（默认开；headless/reftest 默认单线程，显式 `=1` 做 A/B）
- headless 路径（wpt-runner/reftest）：单线程直连（测试确定性优先）

## 五、明确不做（本 RFC 范围外）

- per-Navigable 多线程栅格化（Ladybird 2026-04）——后期
- GPU 光栅化线程（wgpu）——见 compositor-process RFC C3
- 在 renderer 内再建 compositor 平行光栅路径——已由 C2 承接

## 六、验收标准（S2 合入）

1. `make test` / `make reftest` / `make reftest-oracle` 与基线无差异（逐像素）
2. `make layout-golden` 0 diff
3. `make product-smoke` diff ≤ 阈值
4. 基准无关键路径回退（render-foundation 基准）
5. 回退开关 `ZW_RENDER_THREAD=0` 可用
