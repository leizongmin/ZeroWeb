# 渲染线程化 RFC（#3 调研建议）— 主线程录制 → 独立线程光栅化

版本：v1.0（设计稿，待实施规划）｜ 日期：2026-08-07 ｜ 状态：设计

> 依据：Ladybird 渲染架构演进（调研报告 §3.4）——2024 起独立 RenderingThread
>（主线程录 display list → 独立线程 Skia 光栅化，BackingStoreManager 双缓冲），
> 2026-04 起每 Navigable 独立栅格化线程。收益：主线程不承担光栅化开销，
> 滚动/交互/动画响应性显著提升。

## 一、现状审计（2026-08-07）

| 环节 | 当前实现 | 位置 |
|---|---|---|
| 样式/布局/绘制命令生成 | 主线程同步 | `crates/engine/src/pipeline/mod.rs`（RenderPipeline::render_html → RenderResult） |
| 图元序列（display list 雏形） | 已存在：`RenderResult.primitives: RenderPrimitives` | pipeline/mod.rs:90-99 |
| 光栅化（图元 → 帧缓冲） | 主线程同步 | `crates/render-foundation/src/cpu/mod.rs`（render_full_scene / render_scene_to_framebuffer） |
| 帧缓冲管理 | 单缓冲（无双缓冲/BackingStore） | render-foundation surface |

**关键有利条件**：`RenderResult.primitives` 已是可传递的数据结构（图元序列），
「生成（主线程）→ 光栅化（渲染线程）」的数据边界天然存在——这是线程化的
最低风险切入点。

## 二、目标架构（分三片，可独立验收回退）

```
切片 S1：display list 显式化（纯重构，零行为变更）
  现状：primitives 隐含在 RenderResult 中，消费方直接光栅化
  目标：DisplayList = primitives + 绘制顺序 + dirty region；RenderResult 持 DisplayList
  验证：reftest/oracle 全量无 diff（同一数据，仅包装）

切片 S2：独立 RenderingThread + 双缓冲（核心）
  主线程：render_html → DisplayList（录制）
  渲染线程：消费 DisplayList → 光栅化到 back buffer → swap（front buffer）
  BackingStoreManager：双缓冲 + 视图尺寸变更重建
  验证：渲染结果与单线程逐像素一致（A/B）；交互响应性基准（滚动帧率）

切片 S3：增量重绘（dirty region）
  DisplayList 变化区域追踪（engine dirty tracking 已有雏形）→ 只重绘 dirty 区域
  验证：与全量重绘结果一致 + 重绘面积基准
```

## 三、收益与成本

| 收益 | 成本/风险 |
|---|---|
| 主线程不承担光栅化 → 滚动/交互响应性（Ladybird 同路径收益） | 线程同步复杂度（DisplayList 所有权转移） |
| 动画帧率提升（渲染线程可并行下一帧录制） | 双缓冲内存 +1 帧 |
| 为 GPU 光栅化（wgpu）铺路（渲染线程可对接 GPU 队列） | reftest 需要无渲染线程路径（headless 直连） |

## 四、分片实施计划（每片独立提交 + 全量回归）

| 切片 | 动作 | 验证 |
|---|---|---|
| S1 | DisplayList 包装 primitives（纯重构） | `make test` + reftest 全量无 diff |
| S2 | RenderingThread + BackingStoreManager | reftest/oracle 逐像素一致 + product-smoke |
| S3 | dirty region 增量重绘 | 与全量一致 + 基准 |

- 每片遵守 `docs/goal/ai-refactor-acceptance.md`（双管线对照 + 全套件零回归 + 回退开关）
- 回退开关：`ZW_RENDER_THREAD=0`（S2 起默认开，可切回单线程）
- headless 路径（wpt-runner/reftest）：保留单线程直连（测试确定性优先）

## 五、明确不做（本 RFC 范围外）

- per-Navigable 多线程栅格化（Ladybird 2026-04）——多页面并行渲染，后期
- GPU 光栅化线程（wgpu）——本 RFC 的渲染线程是 CPU 光栅化；GPU 属另一演进（见合成器 RFC）
- 合成器独立进程——见 `docs/goal/compositor-process-rfc-2026-08-07.md`

## 六、验收标准（S2 合入）

1. `make test` / `make reftest` / `make reftest-oracle` 与基线无差异（逐像素）
2. `make layout-golden` 0 diff
3. `make product-smoke` diff ≤ 阈值
4. 基准无关键路径回退（render-foundation 基准）
5. 回退开关可用
