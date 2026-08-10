# 合成器独立进程 + GPU 隔离 RFC（#4 调研建议，D 组多进程演进）

版本：v1.3 ｜ 日期：2026-08-11 ｜ 状态：**实施中（C1 ✅ / C2 ✅ / C3 切片 S1 ✅）**

> 实施状态（2026-08-11 更新）：
> - **C1（合成执行层显式化）✅**：`BackingStoreManager` 双缓冲已落地。
> - **C2（合成器独立进程）✅**：surface 级页面主显示链路已接通。
> - **C3（GPU 隔离）⚠️ 切片 S1**：compositor 进程内 headless wgpu（`ZW_COMPOSITOR_GPU=1`）+
>   `gpu_raster.rs` 模块 + CPU 回退；renderer 无 GpuRenderer（隔离测试）。跨进程 GPU 纹理传输、
>   沙箱、Viz 式 surface 所有权仍为后续切片。

> 本状态不代表完整 Chromium/Chrome compositor 对齐。当前完成的是页面位图主链路。Browser 仍拥有窗口最终场景和呈现。

## 一、当前架构

### 1.1 页面主显示链路

```text
renderer
  └─ CompositorFrame(surface_id, navigation_epoch, frame_id, PaintSnapshot)
       ↓ renderer → Browser 管道
Browser process_backend（broker）
  ├─ 校验帧标识
  ├─ 提取滚动、文档尺寸和命中测试元数据
  └─ 非阻塞提交到 compositor-client worker
       ↓ Browser → zero-compositor 管道
zero-compositor
  ├─ surface_id → SurfaceState
  ├─ 拒绝旧 navigation epoch 和倒序 frame
  ├─ 光栅页面图元
  └─ per-surface back buffer → front buffer
       ↓ 完成回执 + RGBA front bitmap
Browser compositor-client worker
  └─ 每个 surface 只缓存最新完整位图
       ↓ 非阻塞轮询
Browser TabSnapshot
  └─ page bitmap → ImagePrimitive → 页面视口 + Chrome UI → 窗口
```

Browser 是 renderer 与 compositor 之间的 broker。renderer 不直接连接 `zero-compositor`。

worker 独占阻塞式管道 IPC。Browser UI 线程只提交命令和轮询缓存。待提交帧按 surface 执行 latest-wins。命令队列和完成缓存都有界。

`zero-compositor` 为每个 surface 保存独立导航世代、帧序号和双缓冲。Tab 关闭时释放对应 surface。Browser 退出时终止 compositor 子进程。

### 1.2 页面与 Chrome UI 的职责

compositor 健康时，Browser 不再光栅页面 `RenderPrimitives`。Browser 只接收 compositor 完成的页面 RGBA 位图。

Browser 仍把页面位图转换为 `ImagePrimitive`。Browser 仍应用页面滚动、缩放和视口裁剪。Browser 仍绘制标签栏、地址栏、菜单和窗口控件。Browser 最终合成页面位图与 Chrome UI，并提交窗口场景。

因此，当前是“页面位图主链路接通”。当前不是“最终显示 surface 由 compositor 拥有”。

### 1.3 传输边界

当前跨进程传输使用 `PipeTransport`。`PaintSnapshot` 和 RGBA 位图都在协议消息中传输。该路径存在序列化和像素复制。

`SharedMemoryChannel` 不是 OS 跨进程共享内存。它基于 `Arc<Mutex<VecDeque<IpcMessage>>>`。它只用于测试和同进程多线程模拟。C2/C3 不得把它描述为共享内存 transport。

## 二、故障回退

Browser 默认启用 compositor，renderer 默认发布 `CompositorFrame`，正常启动不需要设置
`ZW_COMPOSITOR_PROCESS`。仅当该变量精确设置为 `0` 时，Browser 和 renderer 使用 legacy
`ViewPainted` 页面图元路径，供故障诊断和双模式回归测试使用。

compositor 启动失败或 IPC 断开时：

1. worker 将状态切换为 `Disconnected`。
2. worker 清空完成位图缓存并关闭命令队列。
3. Browser 清空每个 Tab 的 compositor 提交和位图状态。
4. Browser 向 renderer 发送 `SetFramePublishMode(Legacy)`。
5. Browser 发送 `RequestFrame`。
6. renderer 从当前页面状态重新发布 `ViewPainted`。

回退不重启 renderer。Browser Chrome UI 保持响应。页面恢复后由 legacy 路径显示。

## 三、已完成范围

| 范围 | 当前状态 |
|---|---|
| surface 协议 | 提交、完成和读取都携带 `surface_id`、`navigation_epoch`、`frame_id` |
| compositor 角色 | 使用 `ProcessRole::Compositor` 和 `--type=compositor` |
| backing store | per-surface 双缓冲；支持 resize、释放和帧新旧判定 |
| Browser client | 专用异步 worker；有界队列；per-surface latest-wins |
| renderer 发布 | compositor 模式发布 `CompositorFrame`；legacy 模式发布 `ViewPainted` |
| Browser 显示 | compositor 位图是页面像素来源；Chrome UI 仍由 Browser 绘制 |
| 生命周期 | 启动失败和断线回退；Tab 关闭释放 surface；退出终止子进程 |
| C3 GPU 隔离 S1 | compositor `gpu_raster.rs` + `ZW_COMPOSITOR_GPU=1`；renderer 无 wgpu |

## 四、下一阶段

### 4.1 Renderer compositor thread

在 renderer 内增加专用 compositor thread。主线程提交 display list、属性树和资源更新。compositor thread 管理帧调度、提交节流和可见区域。它不能阻塞 DOM、JS 和布局。

### 4.2 异步滚动

把滚动偏移、滚动树和输入驱动的变换迁到 compositor thread。滚动不得等待 renderer 主线程重绘。Browser 当前对整张页面位图做变换，只是过渡实现。

### 4.3 真正的跨进程共享资源

新增 OS shared memory transport。实现必须使用可跨进程映射的句柄或文件描述符。协议必须定义大小校验、只读/读写权限、句柄转移、生命周期和崩溃清理。

随后引入 GPU shared image。需要 mailbox 或等价资源标识、同步 fence、格式和色彩空间元数据，以及设备丢失恢复。不得用 `SharedMemoryChannel` 代替这些能力。

### 4.4 Viz 式最终 surface 所有权

最终由 compositor/display 侧拥有窗口呈现 surface。renderer 提交页面 surface。Browser 提交 Chrome UI surface。compositor 聚合 surface、执行最终合成并 present。

达到该状态后，Browser 才不再拥有最终页面与 Chrome UI 场景。当前 C2 尚未达到该边界。

### 4.5 GPU 隔离与沙箱

完成 wgpu 设备、队列和 GPU 资源所有权迁移。移除 renderer 的直接 GPU 访问。为 compositor 配置最小权限 OS sandbox。用真实 GPU 后端验证设备丢失、进程崩溃和恢复路径。

## 五、Non-Goals

- 当前不实现 OOPIF、Site Isolation 或一 frame 一 renderer。
- 当前不拆 Network Service。
- 当前不实现 renderer OS sandbox。
- 当前不实现 renderer compositor thread 或异步滚动。
- 当前不实现 OS shared memory、GPU shared image、mailbox、fence 或零拷贝纹理传输。
- 当前不把 Browser Chrome UI 移入 compositor。
- 当前不把最终窗口 surface 所有权移入 compositor。
- 当前不声称完整 Chromium/Chrome compositor 或 Viz 架构对齐。

## 六、验证与关系

- C2 定向测试覆盖协议往返、双 surface、stale frame、resize、释放、worker 非阻塞、缓存有界、发布模式、Browser 位图显示和故障回退。
- 全量质量、reftest 和产品 smoke 由后续验收任务执行。
- 渲染线程前置见 `render-threading-rfc-2026-08-07.md`。
- ImageDecoder 的请求/响应协议是 compositor 消息路由的先例。
