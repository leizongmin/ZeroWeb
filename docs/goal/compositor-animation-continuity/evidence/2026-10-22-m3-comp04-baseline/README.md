# M3：DC-4（COMP-04）基线——worker canvas 支持子集实测

日期：2026-10-22 ｜ 里程碑：M3（基线探针）｜ 判据：DC-4 前置基线

## 判定

**BASELINE_FROZEN**。生产 headless（生产 GPU 光栅截图）实测 worker canvas 真实支持子集：
worker 内 OffscreenCanvas 构造/`getContext('2d')`/绘制**表面全部成功**，但三重证据钉死其
为**同 realm 泄漏**、与主线程同载——无独立执行线程、无独立呈现路径。DC-4 要求的
「主线程阻塞时 worker 2D 持续呈现」当前不成立，缺口定版为两项：①真线程 worker 执行面，
②脱离 renderer 主线程的呈现管线。

## 环境

| 项 | 值 |
|---|---|
| 平台 | Linux (WSL2/WSLg)，headless `--viewport-width=480 --viewport-height=360`，`ZW_HEADLESS_GPU_SCREENSHOT=1` |
| fixture | tests/fixtures/compositor-animation/comp04-worker-canvas-transfer.html（主线程 canvas 移动条 + blob worker 探针 + 50ms beat + 900ms 单片忙窗） |
| 驱动 | scripts/compositor-animation-comp04-probe.sh + .mjs（采样 s0–s3：空闲→空闲→忙窗后→mutation 正控） |
| 工件 | comp04-probe.json、comp04-s{0..3}.png、headless.log（本目录） |

## 实测（comp04-probe.json verdict）

| 面 | 指标 | 值 | 解读 |
|---|---|---|---|
| worker 能力面 | worker 内 `new OffscreenCanvas(48,48).getContext('2d')` | `ctx=object draw=ok` | 表面可用——**同 realm 泄漏**（见下） |
| API 面 | `canvas.transferControlToOffscreen` | `function` | 已存在（R3313，主线程共享 host handle；part05.js:7595「无该 API」注释过时） |
| 支持子集 | 主线程 OffscreenCanvas 2D 绘制+getImageData | `moc=pass` | 主线程 2D 子集可用（对照通道） |
| 并发面 | 相邻 beat 最大间隔（跨 900ms 忙窗） | **949ms**（忙窗前 51ms） | worker beat 投递停顿整窗——worker 与主线程同载 |
| 并发面 | 忙窗内 canvas 绘制时间戳 | `draws_in_load=0`，`draw_log_spans_load=true` | 主线程绘制停摆（非空真：稀疏日志确覆盖忙窗） |
| 帧活性 | 空闲期 s0≠s1 | `ANIMATED` | 定时器驱动 canvas 连续绘制 |
| 正控 | mutation → 新帧 | `OK` | 截图/发布链路活跃 |

## 同 realm 泄漏归因（源码核对）

worker「绘制成功」非独立 worker canvas 能力：shim worker 脚本体经 `new Function('self', body).call(wctx, wctx)`
执行（part05.js Worker 构造器），函数体内 `__zw_canvas_op` 非局部量，沿**页面全局作用域链**
解析到主线程 host 回调——绘制写入 renderer 主线程 `CanvasRegistry`（js_dom_bridge/canvas.rs）。
后果：① 绘制代码在页面 JS 线程执行，忙窗同停（beat 949ms 与 draws_in_load=0 双证）；
② worker 自建的 OffscreenCanvas ctx id 未挂任何元素 `data-zw-canvas-ctx`，无 paint 路径，
写入即幻影注册表项。part05.js:7594「wctx 无 `__zw_canvas_op`」注释与实测不符（过时）。

## 附加发现：页面 rAF 是预算制同步 stub

初版 fixture 用 rAF 自续环驱动 canvas，两次试跑均在 ~513 次绘制后确定性停摆——页面
`requestAnimationFrame` 为 reftest 边界近似：单次脚本执行内预算（`_rafBudget`）内立即执行、
耗尽后静默不再注册（part01.js），非连续帧驱动。改 `setInterval` 后连续。**对 DC-4 设计的
约束**：页面/worker 连续 canvas 动画的驱动器只能选定时器族（worker 侧 rAF 为
workers-compat P7 的 setTimeout 逐回调派发，同样非 vsync 帧）。

## DC-4 缺口定版与 M3 切片方向（详细 RFC 下轮定稿）

| 缺口 | 现状证据 |
|---|---|
| 真线程 worker 执行面：页面 `new Worker` = 同 realm shim，无独立线程/隔离 | beat 停顿 949ms；workers-compat M4 记账「独立 isolate/线程隔离不预先承诺」 |
| 脱离主线程的呈现管线：canvas 像素链 = 主线程 CanvasRegistry → painter 快照 → ImagePrimitive → 帧 → compositor，全绑定 renderer 主线程 | draws_in_load=0；M1 已证忙窗帧冻结（CSS 动画合成采样除外） |

切片方向（复用既有件）：真线程 worker 复用 script-sandbox `WorkerRuntime`（ThreadedRuntimeCore
+ 看门狗 + terminate_execution，WPT runner 已消费；service_worker_host 为 renderer 进程内
线程化运行时的 IPC 先例）；worker canvas 桥复用 `canvas_context_op` 字符串派发契约 +
worker 本地 registry；呈现复用 M2 的 browser 中继 + compositor surface 面（patch surface
于导出的元素布局矩形，主线程冻结期矩形恒定）。生命周期（worker 退出释放/context lost
错误路径/导航清理）随切片落。

## 局限

- headless 口径：生产渲染管线 + GPU 光栅截图，非窗口合成器 vsync 帧序列（同 M1 记录）。
- worker beat 经同 realm `postMessage`+queueMicrotask 投递，停顿是「worker 不跑」与
  「页面收不到」的合取证据，两者本就同线程，不区分。
- `draws_in_load` 由 16 帧稀疏采样承载（~256ms 粒度），非逐帧证明；`draw_log_spans_load`
  守卫非空真。
