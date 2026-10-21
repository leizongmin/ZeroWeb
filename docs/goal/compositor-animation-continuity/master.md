# 合成动画连续性控制面

**日期**：2026-10-21
**状态**：Active；M1 ✅、M2-S1 全链路 ✅（S1a-S1e 已落，DC-1 有条件成立：非忙窗 256 帧连续采样双通道像素归因；忙窗 ~1s 空洞归因浏览器主线程 Wayland 耦合，见证据）。flag 默认仍关。
**当前卡点**：无阻塞卡点。遗留缺口两项待排期：① 忙窗采样空洞（keep-alive tick 挂在 `wayland_client::conn::blocking_read`，呈现路径与 Wayland 解耦后消失）；② renderer 忙窗每文本更新全文档重解析（token 爆发 1:1 锁定忙片，放大主线程占空）。
**下一步**：M2-S2（查询一致性/DC-2）或 S1 遗留缺口修复，见 [切片 RFC](m2-slice-compositor-sampling.md) §5 与下方缺口表。
**入口**：[目标契约](../compositor-animation-continuity.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 动画是否独立推进及实际支持子集 | COMP-01 基线探针已跑：GPU 截图 3 帧全同 + 查询恒定（见 M1 证据） | BASELINE_FROZEN（2026-10-21） |
| 时间/状态/命中几何共享契约 | 暂停、取消、结束和后台恢复联合验证 | NOT_STARTED |
| Worker OffscreenCanvas 2D 呈现与释放 | 转移、worker 退出和 context lost fixture | NOT_STARTED |
| WebGL 依赖与扩展属性 | 遵守 webgl-compat 门控，独立记录未支持项 | 待核查 |
| headless 阻塞面备忘：同步 evaluate 占用 mux 主循环（1100ms 截图等待实测） | M2 切片设计时核对 #24/#15 学习 | 已记录（M1 证据 README §结论 4） |
| 窗口模式无 automation 通道：负载时序无法在窗口观察中触发（首跑已记） | DC-1 完整验收走 M2 切片（compositor 采样工具+headless 负载通道证据组合） | 已解决（S1e autostart fixture + frame_dump 观察到负载窗，见 DC-1 证据） |
| 忙窗采样空洞 ~1s：browser 主线程 tick 挂在 `wayland_client::conn::blocking_read`（WSLg Weston 同步往返），keep-alive Get 停 → compositor 零采样；帧相位不跳、忙后即恢复（4 次复现确定性） | 呈现/tick 与 Wayland 读取解耦（泵线程或事件驱动唤醒），属既有呈现路径不在 S1 采样链路 | OPEN（S1e 证据 §空洞归因；非本切片修） |
| renderer 忙窗每文本 DOM 更新全文档重解析（token 爆发与忙片 1:1） | 增量解析或标记行文本更新走局部失效 | OPEN（engine 既有行为，两模式一致，放大主线程占空） |
| 加载期 budget 路径不导出动画声明（发空集，加载窗无采样属预期） | runner 泵路径接入导出（S1b 已知边界记档） | OPEN（低优先：flag 关同视觉，冻结不劣化） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 真实窗口阻塞基线与支持矩阵 | ✅ 完成（headless 基线 + 真实窗口帧序列 FROZEN，内容归因像素三重证实；窗口多帧采样工具 frame_dump 已入库） | [2026-10-21-m1-comp01-baseline](evidence/2026-10-21-m1-comp01-baseline/README.md) / [2026-10-21-m1-window-frames](evidence/2026-10-21-m1-window-frames/README.md) |
| M2 | 合成动画、状态与输入一致切片 | IN_PROGRESS（S1 全链路 ✅：S1a protocol 契约、S1b renderer 导出+作用域、S1c compositor 存储+采样、S1d browser keep-alive、S1e DC-1 验收有条件成立；下一步 S2 查询一致性） | [m2-slice-compositor-sampling](m2-slice-compositor-sampling.md) / [S1e DC-1 证据](evidence/2026-10-21-m2s1e-dc1-acceptance/README.md) |
| M3 | Worker 2D 呈现与生命周期 | NOT_STARTED | 无 |
| M4 | GPU/语义/资源与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | COMP-01-transform-opacity-under-js-load | ✅ 有条件成立（ON：256 帧连续、dt 中位 13ms、哈希全异、红盒 alpha 连续扫描、蓝盒旋转包络；OFF：2 帧冻结对照。忙窗 ~1s 空洞已归因浏览器主线程 Wayland 耦合，非采样链路缺陷） | [S1e DC-1 证据](evidence/2026-10-21-m2s1e-dc1-acceptance/README.md) / [M1 基线](evidence/2026-10-21-m1-comp01-baseline/README.md) / [窗口帧序列](evidence/2026-10-21-m1-window-frames/README.md) |
| DC-2 | COMP-02-script-state-and-end-event | NOT_RUN | 无 |
| DC-3 | COMP-03-hit-test-pause-cancel | NOT_RUN | 无 |
| DC-4 | COMP-04-worker-canvas-transfer | NOT_RUN | 无 |
| DC-5 | 关闭/导航释放与支持审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：engine/protocol/compositor/canvas/workers、Android 协议消费者。2026-10-21 本轮改动仅 fixture（tests/fixtures/compositor-animation/）、探针脚本（scripts/compositor-animation-comp01-probe.{sh,mjs}）与本 goal 文档，未触 engine/protocol/renderer/compositor 源码，无共享面冲突；开工前 `git pull --rebase` 已同步 main。
2026-10-21 续轮：新增 `apps/compositor/src/frame_dump.rs`（env 门控帧落盘，默认关闭，含单测）及 lib.rs 接线；M1 真实窗口证据落账（evidence/2026-10-21-m1-window-frames/）。触碰 compositor 源码但仅新增诊断旁路，不改合成语义；M2 RFC 动笔前待核对 protocol 消费侧（Android/JNI）共享面。
2026-10-21 续轮（RFC）：三路源码核查（protocol 追加式纪律/唯一编译穷举面 runtime.rs:3358/Android 全通配安全；engine 插值核心纯函数可重放、paint 无元素身份；compositor 零自主唤醒、浏览器 16ms 轮询+平滑滚动先例）。S1 切片设计定稿于 RFC：新增消息末尾追加（Android 消费方通配不受影响）、flag 默认关保回滚；S1b 将触 painter/renderer、S1c 触 compositor、S1d 触 browser/page-runtime——各步提交前按 run-rules §9 复核并行流工作面。
2026-10-21 续轮（S1a）：protocol 契约落地——`animation.rs`（声明/作用域/计时类型 + 采样数学单一实现 + 仿射折叠 + 6 单测；transform 按函数列表传输、采样后折叠，保证与 engine 逐函数插值同径）、`CompositorAnimationState` 消息末尾追加、`PaintSnapshotParams.animation_scopes`（serde default）、`compositor_animations_enabled()`（`ZW_COMPOSITOR_ANIMATIONS`，默认关）、runtime-environment 文档行、序列化穷举清单扩条、renderer 穷举 match 补臂。全链路 flag 关零行为变化；workspace check/clippy 干净、zero-protocol 321 测试全绿。
2026-10-21 续轮（S1b）：renderer 声明导出 + painter 作用域/中性化 + 全链路接线（flag 默认关零行为变化）。engine `animation/compositor_export.rs`：分类规则六条（通道均质、2D px/deg 子集、端点同构、underlying/paint-effect 干净门、单通道单动画、t=0 已结束不导出）+ 14 单测（含 3 条 render_html flag 端到端：flag 开声明/作用域发射且静态跳写中性化、flag 关恒空烘焙、matrix 拒绝保持冻结）；painter 导出通道发射恒等 TP/中性化 opacity 作用域（与 t=0 值无关恒发）、`transform_origin_point` 单一实现、`cull_invisible_with_scopes` 视口剔除区间/槽位重映射（2 条 render-foundation 单测）；pipeline flag 门控导出 + 增量臂（paint-only/form-value/resize）复用最近全量帧通道表重发作用域（本帧 draw_order 全新，缓存区间不可跨帧复用；声明原样携带防伪空清理）；webview `set_compositor_animations` + `WebViewRenderResult` 双字段；renderer `AnimationExportState`（代际/锚点/发送判等：集合变化才发、退出域发空集清理、锚点=首次导出单调钟、换代重置）+ `animation_declaration_to_ipc` 秒→毫秒换算 + 作用域随帧入 `PaintSnapshotParams`；9 条 renderer 单测。S1b 已知边界（记档不阻塞）：budget 路径（AsyncPageLoad）与 runner 泵路径（render_html_animated）不导出（稳定后 render() 全量帧接手）；perspective 祖先 × transform 动画、individual transform 属性关键帧、Matrix 关键帧、from-only opacity 不在导出域（保持冻结=flag 关同视觉）；导出面只在全量帧刷新，动画元素后续 JS inline style 脏化 underlying 的场景待 S1c 核对。验证：make test 全量通过（含 QuickJS 检查）、zero-engine 91 条动画测试、render-foundation scope remap 2、integration 787 全绿，workspace clippy -D warnings 干净。
2026-10-21 续轮（S1c，commit 72183d732）：compositor 动画声明存储与 Get 路径采样覆盖——新增 `animation_overrides.rs`（共享单调钟采样：opacity 区间 alpha 乘、重叠作用域乘积合成单次舍入、transform 恒等基线矩阵就地改写含 origin 并入与旋转 bbox 扩）+ SurfaceState `animations_active`/`front_cpu_resampled`（结束后回落基线一次再停；CPU 重采样期间回落 Rgba/shm 交付不以过期 GPU 纹理冒充 dma-buf）+ 声明最新生效存储（同 (surface,epoch) 代际单调替换、epoch 推进清理、Release 随清）+ CompositorFrameData.animations_active（serde default）+ browser 中继转发（独立排队、同类同 surface 原地替换、fire-and-forget）。单测 10 条；make test 全量绿。
2026-10-21 续轮（S1d，commit 77d4db07f）：browser 动画活跃期帧轮询保持——animations_active 透传入 TabSnapshot.compositor_animations_active（导航复位）；新增 WorkerCommand::GetFrame 主动采样（与帧/批/声明互不替换、同类同 surface 原地替换），响应走既有 Data 分支入完成帧缓存下一 tick 采纳置 needs_redraw；poll_compositor_frames 对活跃 surface 每 tick 发 GetFrame（renderer 阻塞期由浏览器 16ms 轮询驱动 compositor 重采样），全部结束后回落事件驱动。单测 2 条；make test 全量绿。
2026-10-21 续轮（S1e，commits 48c1d7297 + 证据批）：DC-1 真实窗口验收——frame_dump 序号命名防重采样撞名 + Get 采样路径落盘；新增 observe 脚本（comp01 autostart fixture 14s 双 flag 对照）与分析脚本（sha256/红盒 opacity/蓝盒 transform/marker 行负载窗定位，零依赖）；ZW_LOG_LEVEL env（默认 INFO 零变化）+ 六文件诊断埋点（tick 心跳/keep-alive/Get 采样/声明收发）。实测：ON 256 帧连续（dt 中位 13ms、哈希全异、双通道像素归因、frame_id 冻结 5→8 期间 compositor 独立采样 251 帧）；OFF 2 帧冻结对照；忙窗 ~1s 空洞四重证据闭合（Get 静默 1.019s 互括空洞 985ms、窗内 keep-alive/心跳/Get 三零、gdb 主线程栈 wayland blocking_read、renderer 全文档重解析诱因）。DC-1 判有条件成立；证据落账 evidence/2026-10-21-m2s1e-dc1-acceptance/，空洞修复候选与 renderer 重解析放大记入缺口表。

复用资产：timing-animation（runner 泵/查询面）、workers-compat（worker 生命周期）；不重复实现。授权记录：2026-10-20 规划轮不授权提交/推送；2026-10-21 用户启动指令授权按仓库规则提交推送。生产 GPU 证据口径：headless GPU 光栅为生产渲染管线+生产 GPU 光栅路径，非窗口合成器 vsync 帧序列（局限见证据 README）。
