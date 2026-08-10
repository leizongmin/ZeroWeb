# 合成器独立进程 + GPU 隔离 RFC（#4 调研建议，D 组多进程演进）

版本：v1.6 ｜ 日期：2026-08-11 ｜ 状态：**实施中（C1 ✅ / C2 ✅ / C3 ✅ S1+S2 / 4.1–4.5 + 4.2-S2 ✅）**

> 实施状态（2026-08-11 更新）：
> - **C1（合成执行层显式化）✅**：`BackingStoreManager` 双缓冲已落地。
> - **C2（合成器独立进程）✅**：surface 级页面主显示链路已接通。
> - **C3（GPU 隔离）✅ 切片 S1+S2**：
>   - S1：compositor 进程内 headless wgpu（`ZW_COMPOSITOR_GPU=1`）+ `gpu_raster.rs` + CPU 回退。
>   - S2：GPU partial dirty 光栅（clip blit 到 back buffer）；与 CPU partial dirty 路径对齐。
>   - 跨进程 GPU 纹理传输、Viz 式 surface 所有权仍为后续切片。
> - **4.1（Renderer compositor 发布线程）✅ 切片**：`ZW_RENDERER_COMPOSITOR_THREAD=1` 时
>   `CompositorPublishThread` 异步发送 `CompositorFrame`，队列满时回退同步 IPC。
> - **4.2（异步滚动）✅ 切片 + S2**：
>   - 元数据：`CompositorSetScroll` + `CompositorFrameData.scroll_x/y`；
>     `ZW_COMPOSITOR_ASYNC_SCROLL=1` 时 Browser 推送滚动并消费 compositor 回读偏移。
>   - S2：`ZW_COMPOSITOR_SCROLL_TRANSFORM=1` 时 compositor 在 `GetCompositorFrame`
>     将 scroll 烘焙进 RGBA、回读 scroll 归零；Browser 不再对位图做 scroll 偏移。
> - **4.3（OS 共享资源）✅ S1 / ⚠️ S2 协议预留**：
>   - S1：Linux `ZW_COMPOSITOR_SHM=1` 时经 `/dev/shm/zeroweb-cmp-*` 传递 front 像素。
>   - S2：`GpuSharedImageDescriptor` + `CompositorFrameData.gpu_image` 协议字段（当前始终 `None`）。
> - **4.4（Viz UI surface）✅ 切片**：`CompositorRegisterUiSurface` + Browser 启动时登记
>   Chrome UI surface（`CHROME_UI_SURFACE_ID`）；最终 present 仍为后续。
> - **4.5（沙箱）✅ 钩子**：`ZW_COMPOSITOR_SANDBOX=1` 时 compositor 启动剥离
>   `LD_PRELOAD` 等危险 env；seccomp 最小权限沙箱仍为后续。

> 本状态不代表完整 Chromium/Chrome compositor 对齐。当前完成的是页面位图主链路 + 上述协议/线程切片。
> Browser 仍拥有窗口最终场景和呈现。

## 一、当前架构

### 1.1 页面主显示链路

```text
renderer
  └─ CompositorFrame(surface_id, navigation_epoch, frame_id, PaintSnapshot)
       ↓ renderer → Browser 管道（可选 ZW_RENDERER_COMPOSITOR_THREAD 异步发布）
Browser process_backend（broker）
  ├─ 校验帧标识
  ├─ 提取滚动、文档尺寸和命中测试元数据
  └─ 非阻塞提交到 compositor-client worker
       ↓ Browser → zero-compositor 管道
zero-compositor
  ├─ surface_id → SurfaceState（含 scroll_x/y 元数据）
  ├─ 拒绝旧 navigation epoch 和倒序 frame
  ├─ CPU 或 GPU（ZW_COMPOSITOR_GPU）光栅页面图元
  └─ per-surface back buffer → front buffer
       ↓ 完成回执 + RGBA front bitmap（或 shm_name / 未来 gpu_image）
Browser compositor-client worker
  └─ 每个 surface 只缓存最新完整位图 + scroll 元数据
       ↓ 非阻塞轮询
Browser TabSnapshot
  └─ page bitmap → ImagePrimitive → 页面视口 + Chrome UI → 窗口
```

Browser 是 renderer 与 compositor 之间的 broker。renderer 不直接连接 `zero-compositor`。

worker 独占阻塞式管道 IPC。Browser UI 线程只提交命令和轮询缓存。待提交帧按 surface 执行 latest-wins。命令队列和完成缓存都有界。

`zero-compositor` 为每个 surface 保存独立导航世代、帧序号、滚动元数据和双缓冲。Tab 关闭时释放对应 surface。Browser 退出时终止 compositor 子进程。

### 1.2 页面与 Chrome UI 的职责

compositor 健康时，Browser 不再光栅页面 `RenderPrimitives`。Browser 只接收 compositor 完成的页面 RGBA 位图。

Browser 仍把页面位图转换为 `ImagePrimitive`。Browser 仍应用页面滚动、缩放和视口裁剪（滚动可来自 compositor 元数据，见 4.2）。Browser 仍绘制标签栏、地址栏、菜单和窗口控件。Browser 最终合成页面位图与 Chrome UI，并提交窗口场景。

Chrome UI surface 已在 compositor 侧登记（4.4），但像素仍由 Browser 绘制。因此，当前是“页面位图主链路接通 + 协议切片”。当前不是“最终显示 surface 由 compositor 拥有”。

### 1.3 传输边界

当前跨进程传输使用 `PipeTransport`。`PaintSnapshot` 和 RGBA 位图都在协议消息中传输。该路径存在序列化和像素复制。

**4.3 S1（Linux）**：`ZW_COMPOSITOR_SHM=1` 时 compositor 将 front 像素写入 `/dev/shm/zeroweb-cmp-{name}`，
`CompositorFrameData` 仅传输 `shm_name`；Browser worker 读取后删除文件。

**4.3 S2（协议预留）**：`GpuSharedImageDescriptor` 已定义；mailbox/fence 接线与零拷贝 present 为后续。

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
| C3 GPU S1+S2 | `gpu_raster.rs` + partial dirty GPU 路径；`ZW_COMPOSITOR_GPU=1` |
| 4.3 shm S1 | `frame_shm.rs` + `ZW_COMPOSITOR_SHM=1`（Linux） |
| 4.3 gpu_image S2 | 协议类型 + `CompositorFrameData.gpu_image` 字段 |
| 4.1 发布线程 | `CompositorPublishThread` + `ZW_RENDERER_COMPOSITOR_THREAD=1` |
| 4.2 滚动元数据 | `CompositorSetScroll` + async scroll env gate |
| 4.4 UI surface | `CompositorRegisterUiSurface` + Browser 启动登记 |
| 4.5 沙箱钩子 | `sandbox.rs` + `ZW_COMPOSITOR_SANDBOX=1` env 剥离 |

## 四、环境变量

| 变量 | 作用 |
|---|---|
| `ZW_COMPOSITOR_PROCESS=0` | legacy ViewPainted |
| `ZW_COMPOSITOR_GPU=1` | compositor headless GPU 光栅 |
| `ZW_COMPOSITOR_SHM=1` | Linux POSIX shm 帧像素 |
| `ZW_RENDERER_COMPOSITOR_THREAD=1` | renderer 异步 compositor IPC 发布 |
| `ZW_COMPOSITOR_ASYNC_SCROLL=1` | compositor 滚动元数据 + Browser 消费 |
| `ZW_COMPOSITOR_SCROLL_TRANSFORM=1` | compositor 侧 scroll 烘焙（回读 scroll=0） |
| `ZW_COMPOSITOR_SANDBOX=1` | compositor 启动 env 剥离 |

## 五、下一阶段（完整对齐）

- compositor 侧滚动变换（非仅元数据回读）→ **4.2-S2 ✅**（env-gated 像素烘焙）
- GPU shared image / mailbox / fence 零拷贝纹理传输与 present
- Viz 式最终 surface 所有权与 Chrome UI 像素提交
- seccomp 最小权限 OS sandbox
- 设备丢失、进程崩溃和恢复路径的全平台验证

## 六、Non-Goals（仍为后续）

- OOPIF、Site Isolation 或一 frame 一 renderer
- Network Service 拆分
- renderer OS sandbox
- 完整 Chromium/Chrome compositor 或 Viz 架构对齐

## 七、验证与关系

- C2/C3/4.x 定向测试：`frame_flow`（含 scroll/shm）、`compositor_protocol`、`compositor_client`、
  `compositor_publish_thread`、GPU partial dirty 单测。
- 全量质量、reftest 和产品 smoke 由后续验收任务执行。
- 渲染线程前置见 [`archive/render-threading-rfc-2026-08-07.md`](archive/render-threading-rfc-2026-08-07.md)（已实施归档）。
- ImageDecoder 的请求/响应协议是 compositor 消息路由的先例。

## 八、验收记录（2026-08-11，AI 重构验收规范落地）

按 [`archive/ai-refactor-acceptance.md`](archive/ai-refactor-acceptance.md)（调研 P5）对 C1/C2/C3 S1/4.3 S1 切片执行验收：

| 门禁 | 结果 |
|---|---|
| `make test`（cargo test --workspace + quickjs clippy/测试，test-guard 包裹） | **16,356 passed / 0 failed**（0 warning / 0 error） |
| `make reftest`（self 套件 686 案） | **686/686 ✓**（Layout 485 + Text 201，0 failed） |
| `make reftest-upstream`（16,601 案） | **13,326 passed（80.3%）**；同 corpus 口径 13,251 = 08-08 基线 → **0 回归**（corpus +332 来自 08-10 smoke 子域补齐，新增 75 通过 / 257 失败全归因） |
| `make reftest-oracle` | 未执行（本地无 oracle-shots 资产，需 capture-oracle/CI；DC-14 历史基线 36.2%） |
| `make reftest-smoke` | **42/42** |
| `make product-smoke` | welcome vs chromium **17.03%** ≤ 20%；8/8 struct-check PASS（桌面/窄屏双 viewport） |
| `make layout-golden` | **43/43 已提交 golden 0 diff**（附带修复 layout-dump 块尾空行格式缺陷——marker `\n#####` 前导换行注入空行，full-corpus 下 golden 全 diff，非布局回归；删前导 `\n` 后 43/43 复原） |
| `make browser-compositor-smoke`（双模式 lockstep） | **PASS**：legacy↔compositor 页面签名 close_samples=64/64、mean_luma_delta=4.484、dark_ratio=0.902；compositor 模式无 ViewPainted 泄漏、无 fallback、无 panic |
| `make bench-gate`（vs 08-08 迁移前基线） | 关键路径 page/*（parse/style/layout/paint/total/首屏，welcome/medium/morning）**两轮全 PASS 且 ≤ 基线**；微基准两轮超限 21→11，失败集不相交、集中于重构未触及 crate（dom/wasm/webview/storage/script-sandbox 等），08-09 启用后报告同指标均处基线水平 → **归因共享机器测量噪声，非重构回归**；恢复计划：CI bench-trend（github-ubuntu-latest 基线）为权威趋势 |
| 回退环境变量 | `ZW_COMPOSITOR_PROCESS=0` legacy 路径代码验证（browser compositor_client.rs / renderer main.rs）+ 双模式实测 ✓；`ZW_COMPOSITOR_SHM=1` / `ZW_COMPOSITOR_GPU=1` 开关在位 |

覆盖范围与例外：本机（linux-x86_64，16 核）执行；oracle 与 CI 专用门禁未本地跑，由 CI/weekly 承担。对照差异全部归因（corpus 增量、layout-dump 格式缺陷、基准噪声），无未解释差异。
