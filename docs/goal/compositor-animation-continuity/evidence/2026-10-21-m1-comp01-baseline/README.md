# M1 COMP-01 基线探针——生产帧连续性实测（2026-10-21）

**Run ID**：2026-10-21-m1-comp01-baseline
**HEAD**：`88c884f2f36a18f808940810a0f519aaad017197`（本目录探针脚本/fixture 为本轮新增，行为与记录时一致）
**判定**：生产路径 CSS opacity/transform 动画 **FROZEN（不推进）**；发布链路本身活跃（正控 OK）。

## 环境与产物身份

| 项 | 值 |
|---|---|
| 平台 | Linux (WSL2内核 6.18)，16 核 |
| 二进制 | zero-browser sha256 前 16 位 `d9fd02b065193293`、zero-renderer `5244dbc64d5c477a`（release，HEAD 构建） |
| GPU 路径 | `ZW_HEADLESS_GPU_SCREENSHOT=1` → `GpuRenderer::render_full_scene_gpu` + `read_pixels`（wgpu Vulkan 后端；本机 ICD 含 lavapipe/intel，headless 路径未记录 adapter 名，物理适配器型号未采集——按「生产 GPU 光栅代码路径执行」口径记录） |
| 视口 | 480×360 @ dsf 1.0 |
| 入口 | `zero-browser --headless --remote-debugging-port=<空闲端口>` 多进程模式（Tabs use zero-renderer child processes，见 headless.log） |

## 隔离与动作

- 命令：`sh scripts/compositor-animation-comp01-probe.sh`（本轮实跑 3 次：两次修复驱动契约〔WS resolve 缺失、evaluate 需 body+return 形态〕，最终一次归档于本目录；前一次 `target/compositor-animation-comp01/20261021-101708` 哈希逐字节同此前运行，可复现）。
- Fixture：`tests/fixtures/compositor-animation/comp01-transform-opacity-under-js-load.html`（opacity 1s / transform 1.3s 无限 alternate；`__zwBusy` 忙循环 + `__zwStartLoad` 4 片×220ms 有界负载）。
- 墙钟上界：脚本内 `timeout 120` 包裹驱动；驱动每次 RPC 30s 超时。
- 清理：脚本 trap 只 kill 本次启动的浏览器进程树（`kill $BROWSER_PID` + `pkill -P`），无残留；日志中仅本次连接记录。

## 采样结果（comp01-probe.json + comp01-s{0..6}.png）

| 采样 | t(ms) | PNG sha256 前 12 | opacity | transform | marker |
|---|---|---|---|---|---|
| s0 | 0 | `f4d45d9cf87b` | 1 | none | load=idle |
| s1 | +1000 | `f4d45d9cf87b` | 1 | none | load=idle |
| s2 | +2200 | `f4d45d9cf87b` | 1 | none | load=idle |
| s3 | 负载片1后 | `f4d45d9cf87b` | 1 | none | load=slice 1/4 |
| s4 | 负载片3中 | `cf85d929e6f5` | 1 | none | load=slice 3/4 |
| s5 | 阻塞诊断后 | `6912a151273b` | 1 | none | load=done slices=4 |
| s6 | 正控 mutation | `7a94d71c3aa8` | 1 | none | control=mutated |

**verdict**：`anim_idle=FROZEN`、`anim_after_load=MUTATION_ATTRIBUTED`、`mutation_control=OK`、`load_completed=true`、`busy_reply.wall_ms=1252`、`blocked_capture.waited_ms=1100`。

## 结论（按通道归因）

1. **动画通道 FROZEN**：静止页面 3 秒内三帧逐字节相同；`getComputedStyle` 的 opacity 恒 "1"、transform 恒 "none"；`document.getAnimations()` 返回 0。生产路径 CSS 动画不推进，与静态代码结论一致（`render_html` 仅 R3882 静态 t=0 求值；时钟泵 `tick_animation_clock` 无生产调用方）。
2. **负载通道**：4 片忙循环按期推进（marker 文本变化），每片 mutation 产出新帧——「有界 JS 负载下页面仍能经 mutation 通道产帧」，但动画属性查询恒定，帧差异完全归因 marker 文本。
3. **正控 OK**：强制 mutation 后帧变化（`s5→s6`），排除「发布链路死亡」的替代解释——FROZEN 是动画未被推进，不是截图/发布路径失活。
4. **阻塞形态**：同步 `script.evaluate` 忙 1200ms 期间，第二条连接的截图请求等 1100ms 才被放行——headless 服务端 automation 往返同步占用 mux 主循环（与 run-rules 学习 #24/#15 同族阻塞面，记录供 M2 切片设计参考）。

## 对 M1 支持矩阵的输入（代码核实 + 本实测）

| 能力面 | 进程内/测试 runner | 生产 renderer（本次实测） | compositor |
|---|---|---|---|
| keyframes 注册 + opacity/transform 插值引擎 | ✅ `AnimationClock`（engine/animation.rs） | 注册+静态 t=0 求值（R3882）；**无时间轴推进** | 无动画状态 |
| 时钟推进泵 | ✅ `pump_animation_clock`（runner/探测用） | ❌ 无调用方 | ❌ |
| 动画事件 start/iteration/end/cancel | ✅ pending buffer | ❌ 时钟不推进则事件永不产生 | — |
| getComputedStyle 反映动画值 | 静态 clock clone | ❌ 实测 opacity/transform 恒初值 | — |
| rAF | 帧驱动 opt-in（`__ZW_RAF_FRAME_DRIVEN`）/同步 stub | publish 后 tick（事件驱动，无连续帧循环） | — |
| mutation → 新帧 | ✅ | ✅（实测正控 OK） | CompositorFrame 双缓冲 ✅ |

## 局限（如实记录）

- headless 截图读 `session.snapshot.last_render` 后 GPU 光栅——是生产渲染管线 + 生产 GPU 光栅，但**不是窗口合成器 vsync 帧序列**；DC-1 的完整验收（真实窗口、帧时间序列）仍 NOT_RUN，留待 M2 切片。
- 物理适配器型号未采集（headless 路径不输出 adapter 名）；后续轮次补 GPU adapter 名采集再定 DC-1 证据口径。
