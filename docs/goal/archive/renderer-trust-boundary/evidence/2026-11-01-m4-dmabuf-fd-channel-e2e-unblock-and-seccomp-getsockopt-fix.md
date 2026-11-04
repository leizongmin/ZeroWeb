# M4：fd 通道真实进程 dma-buf 端到端补验 + GPU seccomp 漏放行 getsockopt 修复

**日期**：2026-11-01
**基线 HEAD**：`94d74f551`（切片 6 归档提交）
**性质**：环境补验转真实缺陷修复——M3 切片 2 在册的「环境受限」项补验后发现 **dma-buf fd 交付在真实 seccomp 拓扑下自始不可用**（helper 沙箱缺陷），修复 + 常驻回归 + 端到端实证收口。

## 背景与结论

M3 切片 2 将「fd 通道真实进程 dma-buf 端到端补验」记为环境受限（Xvfb 无 DRI3，gpu-dmabuf 静默回退 RGBA）。本轮搭建 DRI3-capable headless 显示完成补验，发现回退另有根因：**GPU seccomp 黑名单拦截 `getsockopt`**，accept 侧 `peer_same_user`（SO_PEERCRED euid 校验，M3 切片 2 自己引入的防线）在过滤器下恒 false，合法消费端连接被当陌生连接丢弃，浏览器侧报 `SCM_RIGHTS 未携带 fd`、通道判死回退 legacy。即：**自 M3 切片 2 起，生产拓扑下每次 dma-buf 交付都必然失败且静默回退**。切片 2 的 5 项单测全部通过——它们不经 `install_network_exec_filter_gpu_aware`，真实执行才暴露。

## 环境补验路径（DRI3 headless 显示配方）

- `kwin_wayland --virtual --xwayland --xwayland-display :52 -s zw-wayland-52`（`XDG_RUNTIME_DIR=/tmp/zw-wayland-rt`，私有 Wayland socket）+ 手动 rootful `Xwayland :52` 绑定该 socket。`xdpyinfo -display :52` 确认 DRI3 扩展在场（Xvfb 无 DRI3 是 wgpu Vulkan 硬失败点："No DRI3 support detected"）。
- 进程句柄纪律：仅按自起 PID 管理（kwin wrapper/kwin/Xwayland PID 记于 `/tmp/zw-wayland-rt/`），不按名 pkill。
- **smoke-capture 不适用**：`--smoke-capture` 在首个可呈现帧立即截图退出（main.rs:1873-1880），首帧必然落在 compositor GPU 预热窗口（wgpu 上下文后台线程初始化）内 → CPU 光栅 → 无导出。
- **非 smoke 常驻模式也不适用**：`RedrawRequested` 在无 smoke 模式且窗口未持焦时直接跳过首绘路径（含 `ensure_startup_tab`，main.rs:1676-1688）；kwin click-to-focus 策略下 Xwayland 窗口不持焦 → 永不绘帧。
- **可用驱动器**：`--ui-driver-script/--ui-driver-out`（JSONL 真实输入序列）既绕开焦点门又多帧驱动、脚本走完正常退出。配 `file://` 延迟重绘页（2.5s/4s setTimeout 改色/位移）使后续帧落在预热完成之后。

## 根因链（三层逐层拨开）

1. **导出分支门（非缺陷，配置语义）**：dma-buf 导出分支首条件为 `!compositor_scroll_transform_enabled()`（apps/compositor/src/lib.rs:834；scroll-transform 管线在 CPU 侧重采样/视口烘焙，与 GPU 纹理导出互斥）。该门 `enabled_by_default` **默认开** → 默认环境下导出分支永不可达。端到端须 `ZW_COMPOSITOR_SCROLL_TRANSFORM=0`（compositor 由 browser spawn 继承 env，一处设置两侧生效；沙箱仅剥注入型 env）。
2. **焦点门（非缺陷，既有产品语义）**：见上，用 ui-driver 绕开。
3. **真实缺陷：GPU seccomp 黑名单漏放行 `SYS_getsockopt`**：
   - `blocked_syscalls()` 含 `libc::SYS_getsockopt`（seccomp_linux.rs:53）；`gpu_allowed_syscalls()` 放行了 bind/listen/accept/accept4/sendmsg/recvmsg，**漏了 getsockopt**。
   - 失败链：browser `consume_fd` connect → compositor `publish_fd` accept → `peer_same_user` 的 `getsockopt(SO_PEERCRED)` EPERM → 返回 false → 连接被当「非本用户」丢弃（fd_socket_linux.rs:104-113 无任何日志）→ 浏览器 recvmsg 得 EOF → `SCM_RIGHTS 未携带 fd`（fd_socket_linux.rs:234）→ compositor_client 将其作为通道错误整个断开 → 「switched isolated renderers to legacy frame publishing」静默回退。
   - 修复前实测：整轮无 compositor 侧 warn（丢弃分支无日志），唯一信号是浏览器侧一行 WARN。

## 修复与回归

**修复**（apps/compositor/src/seccomp_linux.rs `gpu_allowed_syscalls`）：放行 `libc::SYS_getsockopt`。安全面评估：只读元数据查询、作用对象只能是本进程已持有的 socket；GPU 过滤器域门控下仅能创建 AF_UNIX socket 且 `connect` 仍阻断——无新增网络外联面。非 GPU 过滤器不动（fd 通道仅 GPU 链路）。

**常驻回归**：`gpu_filter_execution_permits_fd_channel_publish_and_consume`（seccomp_linux.rs tests）——隔离子进程真实安装 GPU 过滤器后 `publish_fd` 发布写入探针字节的 memfd，父进程 `consume_fd` 接收并校验字节。既有结构断言测试同步把 getsockopt 加入「不得残留黑名单」清单。

**反例证据（修复前）**：

```
panicked at .../seccomp_linux.rs:345:
GPU 过滤器下 fd 通道发布/消费必须可达（getsockopt 放行）: Channel("SCM_RIGHTS 未携带 fd")
test result: FAILED. 0 passed; 1 failed
```

与生产故障串完全一致（同根因、同信号）。修复后同测试 PASS。

## 端到端实证（修复后，seccomp/landlock 默认开）

```
compositor: seccomp 网络/exec 过滤已启用 (gpu=true)
compositor: landlock 文件系统沙箱已启用 (gpu=true)
compositor: surface 1 帧 #2 已光栅化并合成（epoch=1，800x600，fills=1）
SMOKE_EVENT component=browser event=compositor_dmabuf_adopted tab=2 surface=1 epoch=1 frame=2
compositor: surface 1 帧 #3 已光栅化并合成（epoch=1，800x600，fills=1）
SMOKE_EVENT component=browser event=compositor_dmabuf_adopted tab=2 surface=1 epoch=1 frame=3
```

两帧均走 GPU 光栅 → memfd 导出 → SCM_RIGHTS 交付 → 浏览器导入采纳；全程无 disconnect、无 legacy 回退。驱动序列与日志：`/tmp/zw-wayland-rt/dma-e2e{.jsonl,4.log}`（临时产物，不入仓）；对照组：seccomp 关闭时同序列即出现 2 次 adopted（证明缺陷唯一同源），开启时修复前 0 次且报 `SCM_RIGHTS 未携带 fd`。

## 验证汇总

- `cargo test -p zero-compositor --lib`（test-guard --compile-first，4G/8G/900s）→ **50 passed; 0 failed**（含 3 项 gpu_filter 执行型，其中 1 项为本轮新增 round-trip）
- 修复回退反例：新测试 FAILED（`SCM_RIGHTS 未携带 fd`）→ 重放修复 PASS（见上）
- `cargo clippy -p zero-compositor --all-targets -- -D warnings` → 零警告（初版 1 处 unused_mut 已改）
- `cargo fmt -p zero-compositor -- --check` → clean
- 端到端：`--ui-driver` 真实进程 3 轮（关闭/开启 seccomp × 修复前/后），exit 0，事件见上

## 边界与残留

- `ZW_COMPOSITOR_SCROLL_TRANSFORM=0` 是**当前导出能力的配置前提**：scroll-transform 管线与 GPU 纹理导出按设计互斥（front CPU 重采样不能冒充 GPU 导出）。端到端验证须带此 env；产品默认（scroll-transform 开）仍走 RGBA/shm 交付，行为不变。
- browser-compositor-smoke 脚本 legacy/compositor 两腿 pre-existing 漂移（M3 切片 1 已记档）本轮未触碰；gpu-dmabuf 腿如需自动化，须补 `ZW_COMPOSITOR_SCROLL_TRANSFORM=0` 与 ui-driver（或等价多帧驱动）两条前提。
- macOS 真机验收仍为环境受限在册项（fd 通道/seccomp 均 Linux-only 编译门控，macOS 代码级对照已在前置审计完成）。
- 无残留进程；显示设施（kwin --virtual + Xwayland :52）保留供后续 M4 验证，PID 见 `/tmp/zw-wayland-rt/`。
