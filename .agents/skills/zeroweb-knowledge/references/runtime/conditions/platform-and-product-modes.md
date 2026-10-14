# 平台、发布路径、测试宿主与 GPU 交付分支

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查发布 browser、单元测试、嵌入 WebView、Android 角色及 GPU/CPU 交付为何行为不同。
平台与启动入口同时影响进程、网络、存储和像素路径；不能只按 Cargo feature 或截图格式判断执行链。
具体 IPC 和资源所有权见 [共享协议](../protocol/index.md)，脚本传播见 [脚本配置](script-features.md)。

### Decision Entry

`apps/browser/src/main.rs` 的 cfg/CLI、`headless/session.rs` 的测试分支、
Android `nativeRunRole`、`frame_shm.rs` 的平台门控及 compositor `GetCompositorFrame`
共同决定真实路径。产品功能实跑从 [验证入口](../../../../zeroweb-verify/SKILL.md) 按请求选择，本次只核对源码。

## Branching Table

| 维度 | 发布/主路径 | 替代分支与限制 | 来源 |
|---|---|---|---|
| browser worker | 普通发布使用 tab_worker_stub，页面由 renderer peer 执行 | test 或 test-support 编译进程内 TabWorker；stub::spawn 直接 unreachable | `apps/browser/src/main.rs`, `apps/browser/src/tab_worker_stub.rs` |
| headless session | 非 test 创建 RendererHandle 并设置 compositor 帧发布模式和视口 | cfg(test) 直接创建 WebView，无 renderer 消息流 | `apps/browser/src/headless/session.rs` |
| GUI 调试端口 | 不指定端口时不创建 CDP server | 指定端口创建独立 headless 会话，不直接接到现有 GUI 标签 | `apps/browser/src/main.rs` |
| Android role | Android JNI 用 socket FD 接入通用角色循环 | renderer 还需 android-renderer；未链接时该 role 落拒绝分支 | `apps/android-browser/rust/src/lib.rs`, `apps/android-browser/rust/Cargo.toml` |
| GPU 帧交付 | Linux 有 shm、mailbox、dma-buf 分支及默认开关 | 非 Linux 门控恒 false、dma_frame=None，回退像素交付 | `crates/protocol/src/frame_shm.rs`, `apps/compositor/src/lib.rs` |
| GPU 导出资格 | 当前帧确由 GPU 光栅且导出条件满足才交 dma-buf | GPU 预热完成但最近帧为 CPU，仍回退 RGBA；scroll transform 开启也阻止此导出 | `apps/compositor/src/lib.rs` |
| CPU 工作线程 | 产品 helper 默认启用，仅精确0关闭 | 测试 helper 默认关闭，只认1/true，线程模式仍同步等待光栅结果 | `crates/render-foundation/src/rendering_thread.rs` |
| 窗口呈现 | GPU/Auto 在适用平台初始化窗口 GPU surface | Wayland 强制 CPU present；Auto 无可用 GPU 也走 CPU surface | `apps/browser/src/main.rs` |

## Affected Scope

| 模块 | 判断用途 |
|---|---|
| `apps/browser/src/main.rs` | 发布/测试模块树、CLI 与窗口后端 |
| `apps/browser/src/headless/session.rs` | CDP 独立会话、网络及存储回包 |
| `apps/browser/src/tab_worker_stub.rs` | 发布路径禁止创建进程内 worker |
| `apps/android-browser/rust/src/lib.rs` | JNI role、FD 与 renderer 链接 |
| `apps/renderer/Cargo.toml` | 角色脚本后端默认及传播 |
| `apps/compositor/src/lib.rs` | GPU 预热、实际帧资格与像素交付 |
| `crates/protocol/src/frame_shm.rs` | Linux 共享帧能力 |
| `crates/render-foundation/src/rendering_thread.rs` | 产品与测试线程默认及同步等待 |

## Gotchas

- headless 的 cfg(test) 分支没有 RendererHandle，输入 IPC helper 可以为空实现；该分支单测通过不能证明发布 peer 已收到输入或帧。 (`apps/browser/src/headless/session.rs`)
- headless 未提供完整 IDB/SW 后端时仍必须应答请求：IDB 返回统一不可用错误，SW 读控制器/注册返回空、写操作返回 NotFound；静默丢弃会让同步桥等待超时。 (`apps/browser/src/headless/session.rs`)
- Android 的 target_os=android 不等于 target_os=linux，不能由同用 Unix socket 推断 Linux shm/dma-buf 包装器会启用。 (`apps/android-browser/rust/src/lib.rs`, `crates/protocol/src/frame_shm.rs`)
- GPU 上下文存在不是最新帧的导出资格，frame_gpu_rasterized 必须对应当前 surface 最近帧；预热收编后的旧1×1纹理不能替代先前 CPU 帧。 (`apps/compositor/src/lib.rs`)

## Architecture

- Android compositor/image-decoder 和桌面角色复用泛型 IpcChannel 循环，平台入口负责 socket/stdio bootstrap；共用角色代码不代表共用 GPU 或 OS 沙箱能力。 (`apps/android-browser/rust/src/lib.rs`, `apps/compositor/src/lib.rs`)
- RenderingThread 把 GlyphCache 所有权放到 worker，但 rasterize_sync 用 reply channel 等待结果；“渲染线程化”不能解释成调用者不阻塞。 (`crates/render-foundation/src/rendering_thread.rs`)

## Branching Behavior

- Linux browser CPU 或 parity_smoke 在子进程启动前关闭 Browser dma-buf 导入，以获得 compositor RGBA；parity 随后仍可上传该帧到 Browser GPU 场景，不能把读取 RGBA 当作全链 CPU。 (`apps/browser/src/main.rs`)
- GUI remote-debugging-port 使用辅助 headless session，CDP 导航与窗口标签不是同一个页面状态；验收 GUI 输入须选择真实 UI 驱动路径。 (`apps/browser/src/main.rs`, `apps/browser/src/headless/session.rs`)
- 非 Linux 的共享帧读取/发布 API 返回平台错误，不能靠设置环境变量把未编译的平台实现打开；调用方必须使用适用交付分支。 (`crates/protocol/src/frame_shm.rs`)
- Android nativeRendererLinked 只报告编译 feature；nativeRunRole 返回线程启动结果，不证明 renderer 初始化、IPC 握手或第一帧已完成。 (`apps/android-browser/rust/src/lib.rs`)
- 关闭 RenderingThread 必须先释放发送端，再 join 等待接收循环退出；产品与测试默认不同不改变这项所有权顺序。 (`crates/render-foundation/src/rendering_thread.rs`)
