# CPU/GPU 双链路分叉：测试全绿 ≠ 用户所见正确（排查基线）

**日期**：2026-08-12
**相关模块**：`crates/render-foundation/src/gpu/renderer/mod.rs`、`src/cpu/*`、`tests/wpt-runner/src/reftest.rs`、`apps/browser/src/app_platform.rs`、`apps/compositor/src/*`
**触发**：本机 GPU 测试环境验证（安装 `mesa-vulkan-drivers` 前后对比）+ 全链路覆盖审计

## 问题描述

本仓渲染有 CPU 与 GPU 两条独立实现路径，共享同一 `RenderPrimitives` 中间表示。全量测试
（WPT reftest + 105 个 GPU 单测）全部通过，但审计发现：**测试体系验证的是「CPU 软件渲染
正确性 + GPU 无头单图元正确性」，用户实际走的是「GPU 窗口渲染 + 合成器多进程链路」**，
两条线交集很小，存在多个已验证的「测试过了但用户看错」点——不是风险预警，是现状。

## 根因分析

### 三大结构性分叉

1. **WPT reftest 只跑 CPU，`--gpu` 是文档自认的 stub**（`reftest.rs:1184-1199`，另见
   [[wpt-reftest-gpu-cpu-stub]]）。reftest 像素对比是「CPU vs CPU」——GPU 渲染器与 WPT 基线零交集。
2. **GPU 测试全在无头 llvmpipe/lavapipe 软件渲染**。`new_for_window`（surface format 选择、
   swapchain `AutoVsync` 配置、present、resize 重配、frame pacing）全工作区**零测试执行**。
3. **真实多进程链路**（渲染进程导出绘制命令 → 浏览器转发 → 合成器进程光栅 → 像素/dma-buf
   回传 → present）只靠 smoke 事件，无自动化测试。

### GPU 生产路径功能缺口（静默丢弃或画错，无降级、无日志）

`render_full_scene_gpu` 只处理 9 类图元；以下特性在 GPU 路径失效：

| 特性 | GPU 现状 | 用户可见后果 |
|---|---|---|
| 半透明填充 | fill shader 固定 `alpha=1.0`（`pipeline.rs:57`） | `rgba()` 背景变实色 |
| box-shadow | 硬边矩形，无模糊/alpha/inset（`mod.rs:1111-1126`） | 阴影糊成黑块 |
| clip | 生产路径不消费 `primitives.clips` | 裁剪失效，内容画出界 |
| 滤镜+变换 | `headless_texture.is_some()` 守卫（`mod.rs:930-941`），窗口模式跳过 | opacity/transform 失效 |
| blend_modes | GPU 零引用；CPU 也是空实现（`cpu/effects.rs:331-345`） | mix-blend-mode 无效果 |
| conic 渐变 | `atan2(dy,dx)` vs CPU CSS 约定 `atan2(dx,-dy)` | 方向旋转 90°+镜像 |
| dash/dot 边框 | 3w:2w vs CPU 2w:1w；dot 方块 vs 圆点 | 虚线/点线视觉不同 |
| 凹多边形路径 | fan 三角化只凸正确，CPU even-odd 正确 | canvas 凹形填充错误 |
| draw_order | GPU 永远按类型分桶，忽略 CSS painting order | 父背景图盖在子内容上 |
| HiDPI | 浏览器 GPU 路径固定 scale=1.0（`app_platform.rs:250`） | 高分屏下 1x 光栅 |

### 测试断言弱（「过了」但没测到东西）

- 效果类断言稀疏：阴影 1 像素「非纯白」、渐变左右 2 像素、模糊 2 像素相对阈值——「阴影缺失
  一半」都能通过；无 CPU↔GPU 全帧对照测试。
- llvmpipe 掩盖真硬件问题：图片纹理无 `max_texture_dimension_2d` clamp（测试全用 1×1 图）、
  真 Vulkan dma-buf 是 stub（`texture_export.rs:41-46` 恒 Err，永远 memfd 自循环）、
  atlas 2048 恰好卡 WebGL2 下限、并发被 `#[serial]` 锁死（多线程建 Instance 曾 SIGSEGV）。

## 解决方案 / 验证方法（本轮已落地）

1. **Linux 依赖补齐**：README.md / CONTRIBUTING.md 依赖清单加 `mesa-vulkan-drivers`
   （wgpu Vulkan 后端必需；CI 一直装、本地清单漏了）。缺它时 GPU 链路**静默回退**软件渲染——
   无任何报错，这是最容易踩的环境坑。
2. **真硬件验证技巧**：测试代码 `force_fallback_adapter: true` 默认优先软件适配器（确定性设计），
   验证真实硬件路径用：
   ```bash
   VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/intel_icd.json \
     cargo test -p zero-render-foundation --lib gpu:: -- --test-threads=1
   ```
   （只枚举 Intel ICD；本机 Intel Arc A720 上 107 个测试全过，2.5s。）
   注意 `--test-threads=1` 必须：wgpu 多线程并发创建 Instance 会 SIGSEGV。
3. 验证设备枚举：`vulkaninfo --summary`（vulkan-tools 包）。

## 如何复用 / 后续

- **止血（P0）**：GPU 不支持的图元回退 CPU 重画（合成器已有先例：`gpu_raster.rs` 有图片即回退），
  把「画错」降级为「慢但对」；加 CPU↔GPU 全帧像素对照测试（复用 reftest `compare_pixels_labeled`），
  把缺口表变成可量化失败清单。
- **补盲区（P1）**：Xvfb 上真实 `new_for_window` 冒烟（本机即可跑）；渲染进程→合成器→present
  集成链路自动化；效果类断言改为区域统计或对照。
- **真硬件（P2）**：一台有 Vulkan ICD 的机器跑 GPU 套件；图片上传补尺寸 clamp；真 dma-buf
  （wgpu 30+ API）；blend_modes 先 CPU 实现（16 模式按 CSS 合成规范）。
- 排查基线已文档化：改 GPU 渲染代码时对照上表逐项核对，避免新增分叉；每修一项在表中划掉。
