# DC-4 验收记录：COMP-04 worker canvas（有条件成立）

日期：2026-10-22。M3（S1a–S1d 四批 + 收尾）全链路落账后的 DC-4 判定汇合。
本目录为验收汇合记录，不产生新运行——四个判定面的证据均为当日各批实测，
链接如下。

## 判定：有条件成立

DC-4「Canvas 转 worker，主线程阻塞，worker 退出/上下文丢失 → 支持的 2D 子集继续呈现；
退出后释放资源，错误路径明确」：

| 面 | 判定 | 证据 | 条款 |
|---|---|---|---|
| (a) 主线程阻塞期持续呈现 | **数据面成立**；呈现面忙窗连续性为条件项 | 忙窗呈现空洞两侧条带解卷 17.2/16.9 ms/tick（[S1c patch 管线](../2026-10-22-m3-s1c-patch-pipeline/README.md)）+ 加载窗跨洞 16.8 ms/tick（[S1d 收尾](../2026-10-22-m3-s1d-lifecycle-wrap/README.md)）+ wt 跨忙窗 50ms vs 主线程 943ms（[threaded 腿](../2026-10-22-m3-s1d-threaded-leg/README.md)）——worker 绘制与 patch 数据面穿忙窗连续（worker 线程 → SharedWriter 直写 → browser 中继 → compositor 贴片，全程不触被阻塞的 renderer 主线程） | 条件项：忙窗期 frame_dump 呈现采样受既有 Wayland 空洞缺口（keep-alive tick 挂 `wayland_client::conn::blocking_read`）限制——观测通路问题，非管线缺陷（off/nopatch 对照腿同洞同型；缺口 ① 收口或另行采样通路后升无条件） |
| (b) flag 关对照 | **成立** | `ZW_THREADED_WORKERS=0` 逐字节现行语义：SHIM_BASELINE_PASS 基线复现（wt 跨忙窗 931/939/941/959ms 带内）、忙窗期帧静默、零 patch 消息；workers-compat WPT 钉住面 693P/12F 与基线同数零新增（[S1d 脚手架](../2026-10-22-m3-s1d-scaffold/README.md) / [registry 修复](../2026-10-22-m3-canvas-registry-fix/README.md) / [S1b canvas 桥](../2026-10-22-m3-s1b-canvas-bridge/README.md)） | 双 flag 默认关，回滚切点保持 |
| (c) 退出即冻结、资源释放 | **成立** | `terminate()` 后条带定格 + 静默核查重贴同 x + 轮询停止（S1c 终末冻结腿）；导航换代全表清理（世代守卫 + 终末 Destroy + sink 栅栏，t=4367 起零跨代残留 9381ms，[S1d 导航清理](../2026-10-22-m3-s1d-nav-teardown/README.md)）；六释放面审计（terminate/close/异常退出/导航/tab Release/进程退出，[S1d 收尾](../2026-10-22-m3-s1d-lifecycle-wrap/README.md)） | 宿主 map 项删除有单测断言 |
| (d) 错误路径明确 | **成立** | 脚本错误 Error 事件恰一拍送达页面、错误后呈现面照常 107 帧；`close()` 终末冻结不静默悬挂（comp04c 腿，S1d 收尾）；异常退出探测（panic 绕过单出口 Closed 盲区修复 → `thread_finished()` + pump 异常臂补 error 派发与终末 Destroy） | 同上 |

「支持的 2D 子集」按基线定版口径（[COMP-04 基线](../2026-10-22-m3-comp04-baseline/README.md)）：
主线程 2D 绘制子集经 `canvas_context_op` 纯栅格核零改动复用（S1b），WebGL 遵守
webgl-compat 门控不触碰（M4 审计面）。

## 条件项与后续

1. **呈现面忙窗连续性**（DC-4 (a) 条件项）：遗留缺口 ① Wayland 空洞（缺口表 OPEN 行）
   收口或另行采样通路后补呈现面直接判定，届时可升无条件成立。数据面连续性已由
   解卷三腿独立证明，不受该条件影响。
2. **flag 默认翻面**：`ZW_THREADED_WORKERS` / `ZW_WORKER_CANVAS_PATCH` 默认关保持；
   翻默认属后续独立决策（perf-gate 前置，S1 RFC §2.5 回滚切点）。
3. **M4/DC-5 衔接**：关闭/导航资源面已在 S1d 收尾审计落账（六释放面）；DC-5 全量
   验收（无旧动画/Canvas 句柄或队列残留 + 支持范围如实记录）在 M4 批执行。

## 判定面证据目录索引

- 基线：[2026-10-22-m3-comp04-baseline](../2026-10-22-m3-comp04-baseline/README.md)
- 执行面：[threaded 腿](../2026-10-22-m3-s1d-threaded-leg/README.md) / [registry+S1a 验证](../2026-10-22-m3-registry-s1a-verify/README.md)
- canvas 桥：[2026-10-22-m3-s1b-canvas-bridge](../2026-10-22-m3-s1b-canvas-bridge/README.md)
- patch 呈现管线：[2026-10-22-m3-s1c-patch-pipeline](../2026-10-22-m3-s1c-patch-pipeline/README.md)
- 生命周期：[S1d 导航清理](../2026-10-22-m3-s1d-nav-teardown/README.md) / [S1d 收尾](../2026-10-22-m3-s1d-lifecycle-wrap/README.md)
- 切片设计：[S1 RFC](../../m3-slice-worker-canvas.md)
