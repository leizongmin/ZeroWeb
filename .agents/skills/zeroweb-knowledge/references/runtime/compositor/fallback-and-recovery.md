# GPU/CPU 回退、沙箱与故障恢复

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未执行 GPU、沙箱或故障注入。 -->

## Module Structure

入口为 compositor `gpu_raster`、`recovery`、`sandbox` 和 browser `CompositorClient` watchdog。区分 GPU 算法回退、像素交付回退和进程失联后的 legacy 发布，三者恢复对象不同。

## Gotchas
- GPU 启动后台预热未完成时当前帧走 CPU；预热收编后只有 frame_gpu_rasterized 为真才允许 Get dma-buf，防止把未渲染 1×1 GPU 纹理用 CPU 帧 ID 冒充交付。 (`apps/compositor/src/lib.rs`, `git:f199c7b56`, `git:5af8cc846`)
- 部分 GPU 重绘遇到图片直接返回 false；全量路径也可能因场景能力或 read_pixels 失败返回 false，调用者才走 CPU，不是 GPU 开关打开就保证 GPU 帧。 (`apps/compositor/src/gpu_raster.rs`, `apps/compositor/src/lib.rs`)
- 真实 device lost 丢弃 GPU context，本帧 CPU 回退、下帧惰性重建；模拟丢失开关每次都要求回退，即使本来已是 CPU，不能把两者恢复节奏混同。 (`apps/compositor/src/recovery.rs`, `apps/compositor/src/gpu_raster.rs`)
- `ZW_COMPOSITOR_SANDBOX=0` 只跳过 env 剥离，seccomp 与 Landlock 仍由各自开关控制；关总名开关不意味着取消全部沙箱措施。 (`apps/compositor/src/sandbox.rs`)

## Architecture
- seccomp 在启动早期、线程创建前安装，Landlock 在字体加载后安装；Linux GPU 开关选择 GPU-aware 过滤/文件访问策略，不能先禁字体路径再尝试加载。 (`apps/compositor/src/sandbox.rs`, `apps/compositor/src/lib.rs`, `apps/compositor/src/seccomp_linux.rs`, `apps/compositor/src/landlock_linux.rs`)
- seccomp/Landlock 安装失败当前记录 warn 并继续运行，非 Linux 跳过 OS 过滤；日志中的“启用”与成功安装不是同一个条件。 (`apps/compositor/src/sandbox.rs`)
- GPU 与 CPU 路径在 swap 后统一 image_cache.gc，避免 GPU 长滚动加载绕过 2048 条目/256 MiB cache 的回收。 (`apps/compositor/src/lib.rs`)
- 像素共享交付失败回退 inline RGBA；这不重建 GPU，也不等同于 compositor 已失联，仍受通用管道帧大小上限约束。 (`apps/compositor/src/lib.rs`, `crates/protocol/src/transport.rs`)

## Patterns
- watchdog 只从已发出请求开始计时，响应 progress 延后期限，完成清除；空闲十秒后第一次发帧不应直接判超时。 (`apps/browser/src/compositor_client.rs`)
- 已有请求十秒无 progress 时 client 标 Disconnected 并 kill compositor，使阻塞 recv 能结束；只改状态而不终止卡死 child 不足以恢复。 (`apps/browser/src/compositor_client.rs`)
- worker 连接/处理失败清 completed 与 present cache 并关闭命令队列，browser ProcessBackend 随断连切 legacy；不能继续把旧 compositor 缓存冒充可用新帧。 (`apps/browser/src/compositor_client.rs`, `apps/browser/src/process_backend.rs`)
- 双向大位图管道按请求/响应完成后再发送下一项，避免 UI 上传与 present 回读相互堵住；这与 latest-wins 合并是不同层的背压保护。 (`apps/browser/src/compositor_client.rs`, `git:29f9473c8`)
