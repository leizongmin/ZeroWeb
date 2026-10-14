# compositor、呈现与恢复

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次仅静态核验。 -->

## Module Structure

`zero-compositor::run_role<C: IpcChannel>` 把页面图元变成 surface 帧并处理 UI/present 回读。browser 的 compositor_client 把阻塞 transport 放在独立 worker，UI 提交与窗口重绘不直接等待大帧 IPC。字体/GPU/CPU 算法见 graphics，mailbox/FD 见 protocol。

## Gotchas
- 页面、UI 和 present 回读都可使用 CompositorFrameData，但页面带真实 epoch/frame，UI/present 常为零；应按请求类型解释回执而非对全部消息套同一页面帧规则。 (`apps/compositor/src/lib.rs`, `apps/browser/src/compositor_client.rs`)
- GPU 成功、共享帧交付成功与 native present 是分开的阶段，任一前阶段成功都不能单独证明用户窗口像素正确。 (`apps/compositor/src/gpu_raster.rs`, `apps/compositor/src/lib.rs`)

## Architecture
- 页面 surface 独立 backing/资源与序号，共享字体 namespace/glyph/GPU 状态在激活时切换；修改资源作用域必须核对交替多 surface，而非只看单页。 (`apps/compositor/src/lib.rs`, `apps/compositor/src/font_resources.rs`)
- client watchdog 与 worker 断连清理共同提供进程终止及 legacy 恢复，GPU device lost 则只丢上下文并保留 compositor 角色；两类故障不共用一种重启。 (`apps/browser/src/compositor_client.rs`, `apps/browser/src/process_backend.rs`, `apps/compositor/src/recovery.rs`)

## Child Knowledge Nodes
- [surface、frame、滚动与最终呈现](surfaces-and-present.md) — 旧帧拒收、DPI、滚动烘焙、UI 与页面合成。
- [GPU/CPU 回退、沙箱与故障恢复](fallback-and-recovery.md) — 预热/device lost、沙箱安装、watchdog 和背压。
