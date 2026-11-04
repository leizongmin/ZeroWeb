# M3 切片 2：fd 传递通道对端信任收口 + helper 沙箱时序审计

**日期**：2026-11-01
**HEAD（基线）**：`e58201648`（切片 1 push 后）
**范围**：`crates/protocol/src/fd_socket_linux.rs`（单文件实现 + 2 新增常驻回归）；helper 时序为只读审计（无代码变更）。

## 缺口

M1 矩阵在册第 5 项（M3 范围）：fd 辅助 socket（dma-buf SCM_RIGHTS 通道，`/dev/shm/zeroweb-fd-*`）单连接无对端凭证校验。具体面：① bind 默认 0777&~umask，共享 `/dev/shm` 上其他用户可连；② accept 侧对首个连接即交付 fd，不验对端身份；③ connect 侧不验监听者身份（同路径抢位无感知）；④ socket 名 `surface-frame-fd` 全可预测，被攻破的同机进程（威胁模型内的 renderer）可在发布窗口内撞路径消耗其他 surface 的帧 fd（跨 tab 像素面）。

## 修复（crates/protocol/src/fd_socket_linux.rs）

三层防线（模块头新增信任模型说明）：

1. **socket 文件 owner-only**：bind 后 `set_permissions(0o600)`（set 失败不致命，Linux 侧仍有 euid 校验兜底）。
2. **双侧 `SO_PEERCRED` euid 校验（Linux）**：`peer_same_user`——accept 侧非同 euid 连接被丢弃且继续等合法消费端（原一次性 accept 升级为「首个同用户连接」，陌生连接不挤占交付窗口）；connect 侧对端非同 euid 报错拒绝收 fd（失败语义明确优于静默接受）。macOS 无 `SO_PEERCRED`：恒 true 并注释说明由 ①③ 保障（模块本就 Linux-only 生产路径，dma-buf 构建点均 `#[cfg(target_os = "linux")]`）。
3. **每发布随机 token**：`fd_socket_name` 改为 `surface-frame-<128-bit hex>`（`/dev/urandom`，std-only 不引依赖；urandom 不可读退化 pid+纳秒混合）。名字经既有 `CompositorFrameDelivery.fd_socket_name` 描述符送达消费端（consumer 逐字使用，两侧同源），**IPC wire 格式不变**；`mailbox_name` 全仓无格式解析消费点，token 串透传安全。

## 常驻回归（2 新增）

- `fd_socket_name_carries_per_publish_token`：同 surface/frame 两次生成名不同、token 32 位 hex。
- `fd_socket_publish_sets_owner_only_mode`：发布窗口内 stat socket 文件 mode==0600，随后完整往返成功。
- 既有 3 测试（roundtrip + R3340 成功/错误路径 fd 计数）在新检查下原样通过——真实 socket 握手含 euid 校验正路径的活体回归。

## helper 沙箱/线程时序审计（只读，M3 项）

- **renderer**：`run_desktop_role`（runtime.rs:5175）首语句即 `sandbox::apply_early_if_enabled()`，先于 `RendererRuntime::new` 与一切线程创建——「须在任何子线程之前」契约成立；重启即重新 exec→main→重放钩子。当前钩子为 env 剥离占位（RFC §六 P3，「完整 OS sandbox 仍为后续」，sandbox.rs:19 自述）——renderer 无 OS 级沙箱为**当前平台契约**，非本切片缺陷。
- **image-decoder**：全仓无任何沙箱钩子（无 sandbox 模块、无 env 剥离）。记为剩余缺口（补齐属新能力设计切片，镜像 RFC P3 模式，非核查类工作）。
- **compositor**（切片 1 已验）：seccomp 于任何子线程前（lib.rs:360），landlock 字体加载后（lib.rs:379）；重启走进程重启重放，真实进程日志已证。

## 环境发现（影响验证面，均已复核）

1. **X :20 是用户实时 xfce 桌面**（Xvfb + xfce4-panel 常驻）——GUI smoke 往其开窗/杀窗污染桌面，且桌面交互使结果不可靠：本轮 browser-compositor-smoke 在 :20 先后出现 CPU 腿、GPU dmabuf 腿「卡死」，**同命令在自起干净 Xvfb 上均一次通过**；早前在 :20 上做的 sandbox on/off 分层 bisect（「seccomp/landlock 各自独立致卡」）全部被环境噪声推翻作废。GUI smoke 已改用一次性 Xvfb（用后即停），经验已记入长期记忆。
2. **Xvfb 无 DRI3**（`vulkan: No DRI3 support detected - required for presentation`）——gpu-dmabuf smoke 在 Xvfb 上无法真实触发 dma-buf 交付（静默走 RGBA transport，`compositor_dmabuf_adopted` 事件不出现）。本容器内无可用的 DRI3-capable 显示（:0 为用户真桌面不可动），**fd 通道的真实进程 dma-buf 端到端验收受环境所限**，以 5 个真实 socket 单元级测试覆盖；DRI3 显示（硬件 Xorg/Xwayland）上的补验记 M4 剩余风险。

## 真实进程验证（干净 Xvfb :52，sandbox 全开）

- CPU compositor 腿（`--renderer=cpu`）：exit 0，`SMOKE_CAPTURE mode=compositor` PNG 落盘，`compositor_bitmap_adopted`/`frame_captured source=compositor_bitmap fallback=false`，无 `SMOKE_FAILURE`/`panicked`/fallback/安装失败。
- GPU renderer 腿（`--renderer=gpu` + `ZERO_BROWSER_GPU_DMABUF_SMOKE=1`）：exit 0，`SMOKE_CAPTURE mode=gpu-dmabuf` PNG 落盘，`frame_captured ... fallback=false`，无失败信号（dma-buf 交付因 DRI3 缺失未触发，见上）。
- 两腿日志均见 `seccomp 网络/exec 过滤已启用 (gpu=true)` 与 `landlock 文件系统沙箱已启用 (gpu=true)`，零降级。

## 验证命令与结果（test-guard 包裹）

- `cargo test -p zero-protocol --lib` → 335 passed / 0 failed（含 fd_socket 5 项）
- `cargo test -p zero-compositor --lib` → 49 passed / 0 failed（下游消费面）
- `cargo clippy -p zero-protocol --all-targets -- -D warnings` → 零警告
- `cargo fmt -p zero-protocol -- --check` → clean
- `cargo build --release -p zero-browser` → 成功（三进程同源重建）

## 清理

一次性 Xvfb(:52) 已停；smoke 产物在 `target/browser-compositor-smoke/`（gitignore 面）；无残留 zero-browser/zero-compositor/zero-renderer 进程。
