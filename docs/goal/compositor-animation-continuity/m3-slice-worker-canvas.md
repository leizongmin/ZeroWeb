# M3-S1 切片：真线程 Worker 与 Worker Canvas 呈现（DC-4）RFC

状态：Active ｜ 日期：2026-10-22 ｜ 上游：[目标契约](../compositor-animation-continuity.md) DC-4 / [控制面](master.md)
前置：M2 收官（采样/查询/暂停/取消/命中全链路，[S3 RFC](m2-slice-state-input-consistency.md)）；COMP-04 基线 FROZEN（[基线证据](evidence/2026-10-22-m3-comp04-baseline/README.md)）。flag 默认全关。

## 0. 执行摘要

DC-4：「Canvas 转 worker，主线程阻塞，worker 退出/上下文丢失 → 支持的 2D 子集继续呈现；退出后释放资源，错误路径明确」。基线已钉死双缺口：页面 `new Worker` 为同 realm shim（`new Function('self',body)`，绘制沿作用域链泄入主线程 CanvasRegistry，beat 忙窗停顿 949ms），且 canvas 像素链全绑定 renderer 主线程（`draws_in_load=0`）。本切片按四个批次推进：**S1a** 真线程 worker 执行面（renderer 进程内 `WorkerRuntime` 宿主，flag `ZW_THREADED_WORKERS` 门控，`service_worker_host` IPC 同构）；**S1b** worker canvas 桥（worker 线程本地 `CanvasRegistry` + `transferControlToOffscreen` 迁移绑定，复用 `canvas_context_op` 纯栅格核）；**S1c** 呈现管线（协议末尾追加 patch 消息，worker 数据面经 `SharedWriter` 多线程共写直发 browser → browser 中继 → compositor patch surface 贴到冻结期元素矩形）；**S1d** 生命周期与 DC-4 证据批（terminate/导航清理/context lost 错误路径 + 双向对照）。关键架构前提已核源码：renderer stdout 管道多线程可写（`compositor_publish_thread.rs` 的 `SharedWriter` 主线程/发布线程共写先例）——主线程阻塞期数据面仍通，呈现连续性成立；页面 onmessage 回调本身活在主线程，忙窗内页面侧可见性停摆为固有语义，DC-4 断言只针对呈现与 worker 执行连续性。

## 1. 背景与验收场景

### 现状事实（源码核查，2026-10-22）

| 事实 | 位置 | 影响 |
|---|---|---|
| worker 体经 `new Function('self', body).call(wctx, wctx)` 在页面 JS 线程执行 | `crates/engine/src/js_dom_shim/part05.js:15343/15463` | 无独立执行线程：主线程忙 = worker 停（beat 949ms 实测）；worker「绘制」沿作用域链泄入主线程 `__zw_canvas_op` |
| canvas 像素链 = 主线程 CanvasRegistry → painter 快照 → ImagePrimitive → 帧 → compositor | `crates/engine/src/js_dom_bridge/canvas.rs`（`CanvasRegistry`）、`paint/painter` | 呈现绑定主线程：忙窗绘制停摆（`draws_in_load=0`） |
| `transferControlToOffscreen` 已存在（主线程共享 host handle） | R3313 | S1b 的迁移入口已具备，无需新增 API 面 |
| 页面 rAF 为预算制同步 stub；worker 侧 rAF 为 setTimeout 逐回调派发 | 基线证据 §附加发现、workers-compat P7 | 连续 canvas 动画驱动器只能选定时器族（S1d fixture 设计约束） |
| `WorkerRuntime`（ThreadedRuntimeCore + 看门狗 `terminate_execution` + `WorkerEvent::{Message,Error,Closed}`）双引擎同名（v8 `worker.rs` / quickjs `quickjs_worker.rs`，feature 对齐页面引擎） | `crates/script-sandbox/src/worker.rs:116`、`quickjs_worker.rs:49` | 真线程执行面可复用，宿主代码引擎无关 |
| renderer 进程内线程化运行时 + 事件转 wire 消息先例 | `apps/renderer/src/service_worker_host.rs:76`（`HashMap<u64, ServiceWorkerRuntime>`） | S1a 宿主同构，spawn/terminate/事件桥接有既定模式 |
| renderer→browser stdout 管道多线程可写 | `apps/renderer/src/compositor_publish_thread.rs`（`SharedWriter = Arc<Mutex<Box<dyn Write + Send>>>`，主线程与发布线程共写） | **worker 数据面可在主线程阻塞期直发**——呈现管线绕开被阻塞主线程的架构前提 |
| browser 忙窗期持续轮询 compositor（16ms GetFrame）；声明中继独立排队 latest-wins；epoch 寻址 (surface, epoch) | M2 S1d/S1c/S2c | 中继与轮询面已就绪，patch 消息复用同批机制 |
| 主线程阻塞期布局矩形恒定（导出锚定帧不再更新） | M1 BASELINE_FROZEN + M2 作用域锚定 | patch surface 可钉在冻结期元素矩形上，矩形漂移问题不存在 |

### 验收场景（S1d / COMP-04，fixture `comp04-worker-canvas-transfer.html` 扩展 + dc4 observe/analyze）

flag on（`ZW_THREADED_WORKERS=1` + 呈现 flag）真实窗口/headless 双向：

- **(a) 阻塞期持续呈现**：页面 900ms+ 忙窗内，worker 定时器驱动 canvas 位图持续更新——frame_dump 帧序列哈希连续变化、patch 更新消息流不断（browser 侧日志计数），beat 投递停顿从 ~949ms 量级降到忙片间隔量级（worker 线程不被忙窗阻塞）。
- **(b) flag 关对照**：现行同 realm 语义逐字节不变（beat 停顿 ~949ms 复现、绘制停摆、零 patch 消息、零新协议消息）——workers-compat WPT 钉住面零回归。
- **(c) 退出即冻结、资源释放**：`worker.terminate()`（或 worker 内 `close()`）后 patch 不再更新、位图定格最后一帧、宿主 map 项删除（单测断言）；导航换代 patch 全清（epoch 清理，无跨代残留）。
- **(d) 错误路径明确**：worker 脚本抛错 → 页面 `onerror` 收到（flag on 路径）；宿主运行时异常退出 → patch 冻结 + 页面 error 事件，不静默悬挂。

范围裁剪（明示）：**2D 子集**（WebGL 遵守 webgl-compat 门控不触碰）；真线程 worker 内 `importScripts`/嵌套 Worker 不在 S1（worker 体需自包含，主线程预处理抓取可后补，§3）；`postMessage` transfer 面仅覆盖 OffscreenCanvas 迁移所需（MessagePort 全量 transfer 语义仍走既有 shim 面）；页面主线程 JS 可见性（onmessage 回调、DOM 更新）忙窗内停摆为固有语义不在断言面；GPU 上传路径（零拷贝 shm）记优化待定，S1 走 Vec<u8> 位图。

## 2. 已定设计（Decision）

### 2.1 真线程 worker 执行面（S1a）

**renderer 进程内 `WorkerRuntime` 宿主（`service_worker_host.rs` 同构），页面 Worker 构造器 flag 分流**：

- renderer 新增 `worker_host.rs`：`HashMap<u64, WorkerRuntime>`（键 = 宿主侧 worker id）+ spawn/terminate/postMessage 入口 + `try_recv` 事件泵。`WorkerRuntime::new(script, config)` 的 script 源由页面侧取：flag on 时 shim `new Worker(url)` 走既有取源路径（workers-compat 的 fetch/同源校验面不动，含 blob/data URL 语义），取得源文本后经宿主桥 op `__zw_worker_host_op(op, argsJson) -> String` 交 renderer spawn——**同源校验与 URL 解析仍在主线程 shim（信任边界不移位），宿主只收已校验的源文本**。
- 事件回流：宿主泵把 `WorkerEvent::Message/Error/Closed` 转宿主桥回调进页面任务队列，`setTimeout(0)` 派发 `message`/`error` 事件（对齐既有 shim 投递语义与 task source 注释）；`Closed` 触发宿主 map 清理。
- 页面侧代理对象：flag on 的 `Worker` 实例把 `postMessage`/`terminate` 转宿主桥 op；`onmessage`/`onerror` 挂接事件派发位。structuredClone 序列化沿用既有 shim 面（跨线程边界传字符串载荷，与 `WorkerRuntime::post_message(&str)` 契约一致）。
- flag off 零行为：shim 路径一字不动（workers-compat 语料钉住）；env `ZW_THREADED_WORKERS` 解析同 `compositor_animations_enabled()` 惯例（默认关，非法值关）。
- 引擎无关：宿主只引用 `script_sandbox::WorkerRuntime` 同名类型（v8/quickjs feature 对齐页面引擎，lib.rs 既有 cfg 纪律），宿主代码不写引擎分支。
- 看门狗：页面 worker 用无脚本超时的 `SandboxConfig`（timeout 面关闭，`terminate()` 强制中断仍可用）——页面 worker 生命周期由页面控制，不预设脚本时长上限；`terminate_execution` 死循环兜底保留（核对 `service_worker_host` 的 config 取值后对齐，实施推演点）。

### 2.2 worker canvas 桥（S1b）

**worker 线程本地 `CanvasRegistry`，`canvas_context_op` 纯栅格核零改动复用**：

- worker 体（真线程）内 `new OffscreenCanvas(w, h).getContext('2d')` → worker 线程本地 registry 注册 ctx handle，op 经同一 `canvas_context_op(reg, handle, op, args)` 求值（engine 侧纯 Rust 栅格，无宿主回调依赖）。本地 registry 无 paint 路径 = 修掉同 realm 幻影注册语义（像素只存 worker 本地）。
- `transferControlToOffscreen()`（主线程，flag on）：返回 OffscreenCanvas 占位，宿主登记「元素 host handle ↔ 待绑定 patch 键」；该 OffscreenCanvas 经 `postMessage` transfer 到真线程 worker 时，宿主把绑定落成 worker 本地 registry 的 surface 键注册（`self.postMessage` 入站 transfer 分支截获）——HTML spec §transferred-canvas 语义近似：迁移后主线程对该 canvas 的 2d ctx 获取失效（flag on 面内抛 InvalidStateError 对齐 spec 措辞）。
- 呈现触发：worker 宏任务边界自动 present 脏位图（任务尾查 dirty 位，脏则打包 patch 更新）；逐 op present 与显式 commit 面记 §3 待定。
- 位图口径：`Vec<u8>` RGBA 全帧（fixture 尺度 48² 量级）；dirty-rect 与 shm 零拷贝记 §4 优化预留。

### 2.3 呈现管线（S1c）：数据面绕开被阻塞主线程

**协议末尾追加三类消息（FR-11 纪律，serde default）**：

- `WorkerCanvasPatchRegister { surface_id, navigation_epoch, rect }`（控制面，主线程发：元素布局矩形取自最近发布快照，冻结期恒定）；`WorkerCanvasPatchUpdate { surface_id, navigation_epoch, width, height, rgba: Vec<u8> }`（数据面，worker 线程发）；`WorkerCanvasPatchDestroy { surface_id, navigation_epoch }`（控制面）。序列化穷举清单扩条，renderer/browser/compositor 穷举 match 补臂（S1a 先例）。
- **数据面发送**：worker 线程持 `SharedWriter` clone（compositor_publish_thread 同一 `Arc<Mutex<…>>` 写端）直写 stdout。帧原子性设计点：patch 更新必须在**单次锁持有内写完整帧**（IpcChannel 帧封装一次 write），禁止跨锁分段——否则与发布线程并发交错撕帧；patch 与帧消息无顺序依赖（按 (surface, epoch) 幂等采纳，latest-wins），无 L1 保序需求。
- **browser 中继**：入站 patch 三消息独立排队（S1c 声明中继同构：同类同 surface 原地替换 latest-wins、fire-and-forget），主循环 tick 中继给 compositor；Update 消息到达频率高但排队在 browser 进程（不被 renderer 阻塞），无背压面（fixture 尺度；真实大位图背压记 §3）。
- **compositor patch surface**：`PatchSurface` 存储键 (surface, epoch)（声明存储同款单调替换 + epoch 推进清理 + Release 随清）；合成时把最新位图贴到注册矩形（冻结期基线帧之上）；detach 后（Destroy 未到、worker 已死）保留最后位图显示（spec：canvas 保留位图语义）。合成顺序：贴层在基线帧之后、导出动画作用域同层（canvas 元素矩形与动画作用域互不重叠为 S1 断言面；z 序组合记 §3 待定）。
- 呈现连续性闭环：browser 16ms GetFrame 轮询（S1d 先例）忙窗期持续 → compositor 每 tick 合到最新 patch → 帧序列连续。**数据面全程不触 renderer 主线程**（worker 线程 → stdout → browser → compositor），这是 DC-4 (a) 的结构性保证。

### 2.4 生命周期与错误路径（S1d）

- `terminate()` / worker `close()`：宿主 drop `WorkerRuntime`（线程退出 + 看门狗回收）→ 发 Destroy；compositor patch 转 detached（定格最后位图，不再接受 Update）；页面 `Worker` 实例置 terminated（shim 既有语义）。
- worker 脚本错误：`WorkerEvent::Error` → 页面 `error` 事件（flag on 路径与 shim 语义对齐，WPT workers-compat error 面回归核对）。
- 宿主运行时异常（线程 panic / V8 崩溃）：宿主捕获 join error → 页面 error 事件 + patch detached，不静默悬挂。
- 导航换代：epoch 推进清理全部 patch（S1c 声明清理同款）；宿主 map 按页面归属清空 worker。
- 证据批：fixture comp04 扩展（transfer + 忙窗 + terminate + 导航）+ dc4 observe/analyze（ON/OFF 双向 + terminate 前后帧段分析），判据 §1 场景 (a)–(d)。

### 2.5 触面清单与回滚切点

| 步 | 触点 | 新增 IPC |
|---|---|---|
| S1a | renderer `worker_host.rs`（新）+ `runtime.rs`（宿主桥 op 注册 + 事件泵挂点）；shim `part05.js`（flag 分流 Worker 构造器 + 宿主桥 op）；protocol 穷举清单（宿主桥走既有 host 回调通道则零新消息——实施推演核对 `__zw_*` op 面通道） | 视宿主桥通道而定，倾向零新消息 |
| S1b | engine `canvas_context_op` 零改动；shim OffscreenCanvas/transferControlToOffscreen flag 分支；renderer 宿主绑定表 | 零（绑定走宿主桥 op） |
| S1c | protocol `message.rs` 末尾追加 patch 三消息 + 穷举清单；renderer `worker_host.rs`（SharedWriter 数据面 + 控制面主线程双通道）；browser demux/中继 + compositor 存储/合成/清理 | patch 三消息（末尾追加，Android 通配安全） |
| S1d | 生命周期清理（宿主/compositor/epoch）+ fixture/observe/analyze + 证据目录 | 零 |

**回滚切点**：S1a/S1b/S1c 各自成批、双 flag 门控（`ZW_THREADED_WORKERS` 执行面 + patch 呈现面，默认全关=现行语义逐字节）；S1c 协议消息 serde default 向后兼容（FR-11），compositor 未注册 patch 时消息 no-op。翻默认属后续独立决策（perf-gate 前置不变）。

## 3. 假设与待定

| 项 | 性质 | 说明 |
|---|---|---|
| `WorkerRuntime` 宿主桥走既有 host 回调通道（`__zw_*` op 面）可行，零新 IPC | Assumption | spawn/postMessage/terminate 为页面→renderer 主线程同步/异步入站（主线程活着时才发起）；事件回流走宿主回调注册位。实施推演核对 `service_worker_host` 的 evaluate 命令面后定通道，若需新消息按 FR-11 末尾追加 |
| 页面 worker `SandboxConfig` 无脚本超时 + `terminate_execution` 兜底 | Assumption（实施推演点） | 对齐 `service_worker_host` config 取值；看门狗线程每 worker 一条的成本（fixture 尺度个位数 worker）记档 |
| 真 worker 内 `importScripts`/嵌套 Worker 不支持（worker 体自包含） | Open（S1 范围裁剪） | 主线程预处理抓取（取源时展开 import 链）可后补；workers-compat 语义面在该路径不承诺 |
| 位图 Vec<u8> 全帧口径的背压与带宽 | Open | fixture 尺度（48²×4B×更新率）无压力；真实大画布需 dirty-rect/shm（§4 预留）+ browser 中继背压策略 |
| patch 合成 z 序（canvas 元素与兄弟内容交叠） | Open | S1 断言面限定互不重叠（fixture 设计约束）；z 序组合需合成器层序信息，独立小切片 |
| 逐 op present vs 宏任务边界 present vs 显式 commit | Open | S1 定宏任务边界（spec OffscreenCanvas 无隐式每 op 上屏承诺；commit 面如需对齐 spec `commit()` 另议） |
| `transferControlToOffscreen` 迁移后主线程 2d ctx 失效面 | Assumption | flag on 面内对齐 HTML spec §transferred-canvas（InvalidStateError）；flag off 现行语义不动（R3313 共享 handle 面不变） |
| WPT 钉住面 | 待核对（S1a 批执行） | workers-compat 语料（worker 构造/postMessage/error/importScripts 面）flag 关零回归必须同批核对；flag on 面内新增行为不回流 shim |

## 4. 与后续切片的契约预留

- patch 消息（surface_id, navigation_epoch, rect, rgba）是 M3 内稳定契约；DC-5 资源审计（关闭/导航无残留）直接消费 Destroy + epoch 清理语义，不新增消息。
- 性能优化预留位：Update 载荷 `rgba: Vec<u8>` 后续可切 `frame_shm` key（`crates/protocol/src/frame_shm.rs` 零拷贝面）或 dirty-rect 字段——仅在真实消费者出现时追加（S2 RFC §4 同款纪律），消息结构留 `#[serde(default)]` 尾部扩展位。
- 静态 transform 命中一致性缺口（S3 RFC §3）与本切片无交集；M4 GPU/语义/资源审计消费本切片的 flag 双向证据与 patch 消息计数（资源门禁口径：宿主 map 长度、patch 存活数随导航归零）。

## 5. 碰撞合并补记（2026-10-22）

本 RFC 与并行流同日独立起草的 `m3-slice-worker-canvas-presentation.md`（commit 01dc5f92b，未推）碰撞。按 run-rules §9 让位：本文为 S1 唯一定稿，另稿弃合入（存本地 tag keep/m3-slice-rfc-draft），其经两路源码核查证实的事实并入本节，不改变本文设计决策：

- **`WorkerRuntime` isolate 无定时器、无 host 回调**（worker.rs bootstrap 仅 postMessage 队列）——本文验收 (a)「worker 定时器驱动 canvas 位图持续更新」与 §2.2 宏任务边界 present 依赖定时器族：S1a 实施需给 worker bootstrap 补最小 setTimeout/setInterval/clear*，到期定时器在 worker 线程命令循环间隙服务（ServiceWorkerRuntime 事件等待同型）；isolate 无事件循环，纯 JS 不可自举定时器。
- **isolate 内 host 回调注册先例已核实**：`ServiceWorkerRuntime` 在 worker isolate 内注册 `__zw*` 回调（事件外发 + 共享 `Mutex<Receiver>` 阻塞等待，service_worker.rs:3624+）；页面侧 `__zw_*` 回调族注册于 renderer-js 沙盒线程；webview.rs:6521 另有无调用方的 worker 宿主 API（HashMap<u64, WorkerRuntime> + create/post/poll/terminate），与 §2.1 宿主形态同形——§3 首行 Assumption（宿主桥走既有 `__zw_*` op 通道）成立面扩大。
- **`CanvasContext` 已核实 Send**（无 Rc/Cell/RefCell；`Arc<Mutex<CanvasRegistry>>` 已被 painter 跨线程持有）——§2.2 worker 线程本地 registry 的线程面成立；`snapshot_rgba(&self)` 全量克隆可行（fixture 尺度）。
- **备选呈现路径**：painter `paint_canvas_element` 每次 paint 从 registry 现读快照、image_key=ctx_id，compositor `insert_image_payloads` 支持无帧注入（pending_image_batches 现仅帧到达 drain）——若 §2.3 patch 面实施遇矩形/z 序阻力，共享 registry + 既有 image-payload 通道 + Get 臂重采样（anim_dirty 同型，协议增量一种像素消息 + 一个活跃布尔）为已核实备选，切换决策另记 master.md。
- 两稿差异面（呈现案 patch surface vs image-payload/Get 重采样；registry worker 本地 vs 主线程共享；flag 名 ZW_THREADED_WORKERS vs ZW_REAL_WORKERS）以 git 历史为准，不重复展开。
