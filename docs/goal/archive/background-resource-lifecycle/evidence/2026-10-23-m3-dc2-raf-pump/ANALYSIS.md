# M3 DC-2 续：帧驱动 rAF 饥饿修复（兴趣桥 + 主循环节拍泵）

**运行身份**：HEAD ce2b83b50（DC-2 呈现链修复）+ 本轮工作区；binary `make browser-build` release 产物（2026-10-23）。
**环境**：headless 腿 CDP 采样（`bg02-anim`/`bg02-static`，`ZW_RAF_FRAME_DRIVEN=1`）；GUI 腿 X :20（`bg02-onscreen-compgui`，加 `ZW_COMPOSITOR_ANIMATIONS=1`）。采样 250ms 进程树 `/proc`。重跑命令见各腿 `run-config.txt`。

## 结论

1. **饥饿修复实证**：`bg02-anim`（纯 rAF 自旋循环，不改 DOM）`__zw_raf` 30s 从 333 爬至 1575（~52 次/s 持续）——M1 基线同腿 2 次/30s（挂发布末尾、无发布即饥饿）。修复形态 = shim 兴趣回报 + renderer 主循环节拍泵派发，无需 DOM 失效。
2. **代价面**：泵驱动 52 次/s 回调派发的 renderer 代价 0.049 cores（M1 饥饿态 0.009）；三腿一致（GUI 全链腿 0.048）。
3. **资源面未回归**：`bg02-static`（无 rAF，flag 开）renderer 0.015 cores（M1 0.008 同带）、browser 0.020（M1 0.013）——兴趣恒 0 泵零活动，静态/无 rAF 页零开销。
4. **全链共存**：`bg02-onscreen-compgui` 512 dumps（封顶）——CSS 动画 compositor 采样链与 rAF 泵并存。

## 修复形态

- **shim**（`part01.js`）：帧驱动分支三处回报挂起数——`requestAnimationFrame` 注册后、`cancelAnimationFrame` 取消后、`__zw_raf_tick` 末（rAF 内重注册计入）调 `__zw_raf_interest(n)`；无宿主注册（browser 单进程路径）`typeof` 守卫 no-op。
- **js_worker**：`__zw_raf_interest` 回调无条件注册（同 TimerBridge 先例；开关留在 shim `__ZW_RAF_FRAME_DRIVEN` 分支，OFF 零开销；无条件注册使测试可 JS 侧注 flag 直驱，规避 env set_var 并行竞态），写共享 `Arc<AtomicU64>`；`ResetDocumentState` 臂清零（跨导航残留防泵空转）；`raf_interest()` 访问器。
- **主循环泵**（`runtime.rs` `RendererRuntime::run`，媒体泵同位）：挂起数 >0 时每拍提交裸 `__zw_raf_tick(performance.now())` 优先脚本（同 `tick_observers_with` 派发脚本）；挂起数归 0 泵自停。回调 mutation 由既有 `drain_pending_script_mutations` 通道下一拍 apply+渲染+发布。
- **重形态否决记录**：首版泵走完整 `tick_observers_inner`（每拍全量 SetDomSnapshot + observer tick + apply），实测 renderer 6.68 CPU-s/30s——纯 rAF 页不可承受，改为裸脚本提交后 1.48 CPU-s/30s。
- **单测**：`renderer_js_worker_raf_interest_reports_and_converges`——注册置位 / tick 无重注册归零 / tick 内重注册维持 / 取消归零。

## 腿结果（30s，cores = max(cpu_jiffies)/100/wall）

| 腿 | `__zw_raf` | renderer | browser | compositor |
|---|---|---|---|---|
| bg02-anim（headless，rAF 自旋） | 333→1575（~52/s） | 0.049 | 0.015 | – |
| bg02-static（headless，无 rAF） | – | 0.015 | 0.020 | – |
| bg02-onscreen-compgui（GUI 全链） | 不采样（无 CDP） | 0.048 | 4.355 | 0.576 |
| M1 bg02-anim（饥饿基线） | 2 次/30s | 0.009 | 0.014 | – |
| M1 bg02-static | – | 0.008 | 0.013 | – |

## 已知缺口（后续切片）

1. **每拍脚本重编译与派发常数**：52 次/s 裸脚本派发 ≈ 0.035 cores 增量，可接受但非零——后续可做脚本预编译/合并派发。
2. **同帧双源派发**：泵 tick 与发布末 tick 独立，同一帧窗内回调可能派发两次（既有 publish 多源已具同性质，非本切片引入）；spec 级 coalescing（每渲染帧至多一次 tick）留待 rAF 语义精化。
3. **GUI 腿 browser 4.355 cores**（上轮同腿 2.708）：present-blit 面本不含泵路径（泵在 renderer），差值方向与上轮双腿自身 20% 离散一致放大，夜间共享机器负载噪声不能排除——上轮 ANALYSIS 的 present-blit 代价门控缺口不变。
4. **可见性消息**：`document.hidden`/`visibilityState` 仍硬编码 visible（part06.js:5290），browser→renderer 前后台通知机制不存在——后台 rAF 节流的前置，独立切片。
