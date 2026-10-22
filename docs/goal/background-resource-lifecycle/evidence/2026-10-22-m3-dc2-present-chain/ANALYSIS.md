# M3 DC-2 呈现链与 rAF 门控定性 + 自举死锁/导出缺口修复

**运行身份**：HEAD 本轮工作区（worker 面钳制 e5217e6fa 之后）；binary `make browser-build` 产物（2026-10-23 00:02 本地 = 10-22 16:02 UTC）。
**环境**：GUI 腿 X display :20、真实窗口 1920x893、GPU compositor（lavapipe）；headless debug 腿仅日志。采样 250ms 进程树 `/proc` 真实采样。
**协议**：`scripts/background-resource-observe.sh`（场景 `bg02-{anim,onscreen,static}-compgui`，`ZW_COMPOSITOR_ANIMATIONS=1`）；settle 腿为手工编排（`/tmp` scratch fixture，finite animation 3s）。重跑命令见各腿 `run-config.txt`。

## 结论

1. **激活链两处断点定性并修复**：M1 的「动画呈现链未激活」由两个独立缺陷叠加——(a) browser 侧 keep-alive 自举死锁；(b) engine 预算渲染路径从不导出动画声明。两处修复后，动画页面 30s 帧数 2→512+（采集器封顶），settle 后回落事件驱动。
2. **资源面未回归**：静态页保持 3 帧/30s（M1 2 帧），renderer CPU 0.010 cores（M1 0.004–0.009 同量级）。
3. **代价面首次可测**：动画活跃期 browser present-blit 路径 ~2.7–3.3 cores + present 缓存 RSS 增至 ~540MB（~43fps 全窗位图往返），compositor 采样重光栅 ~0.4–0.5 cores。后续切片的量化基线。

## 激活链定性（源码核对 + debug 日志实证）

链路：engine 导出（声明 + paint 作用域 + 通道表）→ renderer `CompositorAnimationState`（全量替换）→ compositor `animation_states[surface]` → browser keep-alive 每 16ms `GetCompositorFrame` → compositor 采样重光栅（`animations_active` 随响应回传）→ browser 采纳置位 → `needs_redraw` → present Get round-trip 上屏。

**断点 A（browser 自举死锁）**：`animations_active` 唯一产生源是 Get 响应采样（`apps/compositor/src/lib.rs:683-699`），而常态 Get 唯一发起方 keep-alive 以该标志为前置（`apps/browser/src/process_backend.rs:612`）；GUI 默认 present 链响应硬编码 `animations_active: false` 不采样（`lib.rs:1254-1255`）——false→true 启动沿不存在。M2-S1d 的 keep-alive 机制本身完好，只是永远等不到启动沿。

**断点 B（预算路径导出缺口）**：生产导航帧全部走 `AsyncPageLoad` 分步渲染（`pipeline_budget.rs` `advance_budgeted_render`），该路径结果恒发空声明——源码自注「load 帧无声明，稳定后 render() 全量帧接手（M2-S1b 记档缺口）」，但「稳定后 render()」在生产无调用方（首帧后增量走 `apply_dom_mutations_and_render`/`repaint_cached_viewport`），声明永不达 compositor。M2-S1b 的导出逻辑只接在 `render_html` 全量入口；renderer 侧换算/发送/终结事件测试全绿但测的是手工构造声明——生产端到端从未激活。debug 实证：修复前 compositor 收到 `declarations=0`，修复后 `declarations=1` + `compositor_animations_active false -> true`。

## 修复

- **browser 自举探测**（`process_backend.rs` + `tab_snapshot.rs`）：flag 开启时每个新提交 frame 代探测一次 Get——响应 active=true 由既有 keep-alive 接管，false 本代止步、下一提交代重开；导航 reset 清标记。静态页每发布代至多一 Get（无动画时 compositor 不重光栅仅回元数据），flag 关（默认）零探测。
- **预算路径同源导出**（`pipeline_budget.rs` + `pipeline/mod.rs` 可见性）：Style 步镜像 render_html 序列（register_from_stylesheets → export_composable_animations → t=0 叠加），Paint 步注入通道表 + `take_animation_scopes` + `cull_invisible_with_scopes` + 写 `compositor_export_cache`（增量臂既有携带逻辑随之生效）。
- **回归钉**：`pipeline_inline.rs` 两测——render_html 全量路径 + 预算会话路径各钉 transform 声明 + 作用域非空（预算路径测试即本缺口的生产路径钉）。

## 腿结果（30s GUI ×3 + settle 22s）

| 腿 | 帧数（dump 文件） | browser CPU | compositor CPU | renderer CPU | renderer RSS |
|---|---|---|---|---|---|
| bg02-anim-compgui（offscreen infinite） | **512+manifest（封顶）** | 3.301 cores | 0.498 cores | 0.011 | 108→115MB |
| bg02-onscreen-compgui（视口内 infinite） | **512+manifest（封顶）** | 2.708 cores | 0.427 cores | 0.013 | 77→115MB |
| bg02-static-gui（静态对照） | 3 | 0.024 cores | 0.006 cores | 0.010 | 108→111MB |
| settle scratch（finite 3s + 19s 静置） | 187 后冻结 | – | – | – | – |

- M1 对照：anim-compgui 3 帧→512+；onscreen 2 帧→512+；static 2 帧→3 帧（+1 为发布代探测 Get 采纳，无持续轮询）。
- settle 腿（ZW_LOG_LEVEL=debug）：3s 动画期 187 dumps（~62fps 采样），动画结束 `compositor_animations_active true -> false` 翻转一次，其后 19s 零新增 dump——轮询停、资源面回落。
- dump 封顶说明：`ZW_COMPOSITOR_FRAME_DUMP_MAX` 钳到 1..=512（`frame_dump.rs:33`），30s 腿 ~8s 即封顶；持续流由 keep-alive GetFrame 节奏（debug 腿 12s 1930 次 ≈ 2/tick）与 settle 腿冻结共同佐证。
- browser RSS 538–544MB 为 present 帧缓存逐帧采纳的峰值驻留（每帧全窗 RGBA ~6.8MB 采纳路径），动画停止后不自动回落——归入后续窗口回收面（与 M1 BG-04 renderer 复用不回落同族观测）。

## 已知缺口（后续切片）

1. **offscreen 声明级空转**：视口外元素声明照发（导出为声明级、cull 只剔作用域）——offscreen 腿 512 dumps 中元素根本不在 paint 内，compositor 每次采样重光栅对不可见动画空转（0.5 cores）。DC-2「可跳过工作不持续重复呈现」在 offscreen 维度仍 PARTIAL；修复方向 = 全作用域被剔时声明置 inactive 或导出期视口门控。
2. **browser present-blit 代价**：~43fps 全窗软件 blit + present 往返 2.7–3.3 cores——动画呈现节奏上限（30fps 帧率门控）与 GPU present 采纳属 M3 后续。
3. **rAF 帧驱动饥饿**：rAF tick 挂在「帧发布」上（`publish_webview_now → tick_observers`），纯 rAF 循环不改 DOM 无失效无发布即饥饿（M1 `__zw_raf`=2）；本轮 compositor 采样不产生 renderer 发布，rAF 面未变。修复方向 = renderer 侧动画时钟泵/独立 rAF 通道（`pump_animation_clock` 仅有 runner 调用方），与可见性消息一起构成下一个切片。
4. **SW 面 timer**：无延迟 FIFO 简化模型，钳制语义不适用（上轮已记）。

## 附注

- 首轮重跑观测到 `frame dump 写失败，停用本会话采集: File exists`——observe 脚本不清 frames/ 目录，同目录重跑触发文件名碰撞后 dumper 自禁用；本批证据为清理后重跑（`dumpfail=0`）。脚本目录清理行为记入观测资产待办（未改脚本）。
