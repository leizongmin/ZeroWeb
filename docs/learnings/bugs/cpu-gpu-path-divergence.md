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

## 修复进展（2026-08-12，R3256-R3265 已落地）

- **P0-1 回退机制**：`gpu::scene_support::scene_supported` 检测 GPU 未实现特性 →
  `render_full_scene_gpu` 返回 false → 合成器/浏览器回退 CPU 整帧（慢但对）
- **P0-2 对照测试**：`gpu/renderer/parity_tests.rs` CPU↔GPU 全帧逐像素对比
  （headless sRGB target 编码语义：fill/图片中间色被编码、渐变 sRGB 纹理↔target
  恒等链无损——场景用不动点色 + 渐变中间色）
- **P1-3 窗口冒烟**：`window_smoke_tests.rs` 真实 winit 窗口全生命周期
  （EventLoop 每进程一次 + any_thread；无显示环境跳过）
- **P1-4 合成器进程验证**：frame_flow 新增回退/半透明渲染测试（真实子进程）
- **P1-5 断言加固**：阴影/渐变/blur 从 1-2 像素采样改为区域与语义断言
- **P2-6 图片超限**：> max_texture_dimension_2d 回退 CPU（device_limits 测试）
- **P2-7 blend_modes**：CPU 16 模式（源层重渲染，render_draw_order Blend 标记 →
  独立源缓冲 → composite_blend 区域合成；typed 逃生舱保持跳过）
- **P2-8 视觉对齐**：半透明 alpha（顶点 7→8 float，image 管线独立布局）、
  conic 角度约定、dash 2w:1w / dot 圆点、凹多边形耳切、synthetic italic shear、
  HiDPI scale_factor、repeating first≠0 回退

**仍待办**：
- **#2 已修（R3267）**：headless 目标改 Rgba8Unorm 直通 byte——合成器中间色偏色消除
- **#1 已修（R3268）**：canvas 显示链路（getContext 属性桥 → painter 图元 → ImageCache 注入）
- **#5 已修（R3270）**：reftest --gpu 真 GPU 渲染（取代 CPU stub，WPT 基线拉到 GPU）
- **#11 已修（R3269）**：compositor import blit 顶点格式对齐（7-float）
- **#6/#7/#8/#9 已修（R3271）**：整链契约测试、CI GPU 套件（此前 CI 只跑 cpu::）、
  回退 blit 视觉验证、多渲染器交替
- 真 Vulkan dma-buf（wgpu 24→30 升级，dependency-upgrade-backlog P1 独立工程）
- 窗口模式滤镜/变换（headless 守卫；现回退 CPU 慢但对）、clip GPU 实现、
  blend GPU 实现
- **#14 GPU draw_order**：GPU 分桶绘制 vs CSS painting order（DC-10：父 bg-image
  被子元素 bg-color 场景缺陷）——修复需绘制阶段按 draw_order 重构（每图元独立
  顶点数组 + 单 draw call），独立任务
- **#12 device-lost**：真实恢复循环（wgpu 24 无 DeviceLostCallback；现「失败→CPU
  回退」已覆盖正确性，缺 GPU 设备重建）
- **#13 atlas 2048**：已是 WebGL2 下限兼容值（所有设备支持），rebuild 机制兜底——
  设计决策保留
- **#15 bytes_per_row**：wgpu 接受非对齐（仅个别驱动性能提示），数据紧密排布
  无需重排——非问题
- **#16 CDP 截图**：CPU 路径正确可用，GPU 化纯性能项

## 如何复用 / 后续

- 改 GPU 渲染代码时对照上文缺口表逐项核对，避免新增分叉。
- GPU 新增功能后记得收窄 `scene_supported` 回退面（如 P2-8 alpha 移除半透明检测）。
- 排查工具：`VK_ICD_FILENAMES` 单 ICD 验证真硬件；`--test-threads=1` 防 wgpu SIGSEGV。
- 排查基线已文档化：改 GPU 渲染代码时对照上表逐项核对，避免新增分叉；每修一项在表中划掉。

## wgpu 30 升级与 dma-buf 结论（2026-08-12，R3275）

- **wgpu 24→30 升级完成**（R3275）：PollType/Queue::present/CurrentSurfaceTexture/
  multiview_mask/depth_slice/as_hal 等 API 迁移，render-foundation 598 + GPU 117 全过。
- **真 dma-buf 仍受 upstream 限制**：
  - 导入：wgpu-hal 30 Vulkan `texture_from_dmabuf_fd`（unsafe）+ `Device::as_hal`/
    `Texture::as_hal` 路径存在，但 hal Texture **无公开包装 API**（无 from_hal）——
    需 hal 层命令编码或 upstream 补包装
  - 导出：wgpu-hal 30 **无 dma-buf 导出 API**（仅导入）——compositor→browser
    零拷贝共享仍只能 memfd 回读
  - 结论：完整真 dma-buf 闭环需 wgpu 31+（或 upstream 暴露 export + from_hal），
    当前保持 memfd 路径；升级本身是必要前置（as_hal 骨架已就位）
