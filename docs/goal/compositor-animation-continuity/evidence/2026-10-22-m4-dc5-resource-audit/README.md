# M4 / DC-5：关闭/导航资源审计 + 支持范围与门禁记录

日期：2026-10-22。DC-5「关闭/导航后验收资源与回归：无旧动画/Canvas 句柄或队列残留；
支持范围、未执行 WebGL 与适用门禁如实记录」+ M4「GPU/语义/资源证据、默认路径性能与
DC 审计」的审计批。本批零源码改动（审计核对既有实现与测试 + 两项生产门禁实测）。

## 1. 资源残留审计（关闭/导航后无残留）

本 goal 触及的全部句柄/队列/存储面，导航（navigate/load_html 世代推进）、worker 终末
（terminate/close/异常退出）与 tab 关闭（Release）三个时点的释放路径：

| # | 残留面 | 释放路径 | 测试/证据 |
|---|---|---|---|
| 1 | renderer `AnimationExportState`（声明/锚点/事件去重表） | 世代推进 `Self::new` 整表重建；导出域退出 `deactivate` 清表 | S2/S3a 单测（换代旧事件不重放、域退出清积压）；comp02 nav 腿换代旧事件不重放实测 |
| 2 | engine compositor 声明存储（`animation_overrides`） | 同 (surface,epoch) 单调替换；epoch 推进清理（帧推进点）；`Release` 随清 | compositor `animation_state_latest_wins_by_epoch_and_generation` / `animation_state_cleared_when_surface_epoch_advances` |
| 3 | renderer worker 宿主三表（runtimes/canvas_bindings/patch_sinks） | Terminate 臂、Closed 臂、异常退出臂、导航 `teardown_for_navigation` 全表清理 | worker_host 四臂单测（含 `teardown_for_navigation_clears_tables_and_fences_late_flush`、异常退出探测批）；comp04b 导航腿像素判定零跨代残留 |
| 4 | compositor `patch_surfaces` | 帧推进 `cleanup_stale_patches`（旧世代）；Destroy → detached 定格；`ReleaseCompositorSurface` 臂 `patch_surfaces.remove(&surface_id)`（lib.rs:1029） | `patch_register_latest_wins_and_stale_epoch_dropped` / `patch_destroy_detaches_and_stale_destroy_is_noop` / `patch_cleanup_removes_stale_epoch_and_empty_entries` |
| 5 | browser 中继（声明 + patch 三消息） | **无队列残留面**：`forward_*` 直发 compositor 命令通道 FIFO（fire-and-forget，compositor_client.rs:1551-1604），browser 侧不持声明/位图状态；Release 后迟到 Update 落 compositor 未知 surface → store 函数 no-op | browser `patch_relay_is_fire_and_forget`（含同批 ReleaseSurface 排序）；compositor `patch_update_requires_registration_same_epoch_and_not_detached` |
| 6 | browser `TabSnapshot` 轮询标志（`compositor_animations_active`/`compositor_patches_active`） | animations_active 导航复位；patches_active 随 Register/Destroy 置位复位；Destroy 缺失时（renderer 进程随 tab 关闭退出、无 Destroy 发出）由 Release 臂摘 surface 兜底，轮询因 surface 移除自然停止 | process_backend.rs:546-606 中继臂代码审计；comp01/comp04 腿轮询起停实测 |
| 7 | compositor 其他随 surface 生命周期存储（`pending_image_batches`/`animation_states`/font namespace） | `ReleaseCompositorSurface` 臂同批清理（lib.rs:1027-1033） | `pending_batch_injects_when_matching_frame_arrives` / `stale_pending_batch_is_dropped_on_epoch_mismatch` + 代码审计 |
| 8 | 进程边界 | browser/renderer/compositor 任一退出：宿主线程为 detached 轮询线程、worker 线程随 `WorkerRuntime` drop terminate、共享内存管道随 fd 关闭——OS 统一回收 | 代码审计（进程模型语义） |

**判定：无残留**。导航时点像素级证据为 comp04b 腿（清理时刻起 9381ms 零跨代残留，
[S1d 导航清理](../2026-10-22-m3-s1d-nav-teardown/README.md)）；tab 关闭时点由 Release
臂 + no-op 兜底路径覆盖；全部面均有单测或审计记录。

## 2. 支持范围如实记录（支持矩阵）

| 面 | 已支持（导出域内） | 明示不支持/条件项 |
|---|---|---|
| 合成动画导出 | opacity/transform 单通道、2D px/deg 子集、端点同构、underlying/paint-effect 干净门、单通道单动画（S1b 六条分类规则，engine `compositor_export.rs` 14 单测） | perspective 祖先×transform 动画、individual transform 属性关键帧、Matrix 关键帧、from-only opacity——保持冻结（flag 关同视觉）；background-color/filter 未切片（goal 边界记档） |
| 查询面 | gCS provider 覆盖（结构路径 + unique 形态），与合成采样同函数同钟（S2a） | CSSAnimation 对象面未建（currentTime/playState 经 WAAPI shim 有限面；DC-2 条件项，S2 RFC §1 范围裁剪） |
| 事件面 | goal 导出域 end/cancel exactly-once（S2b/S3a，comp02/comp03 实测）；engine 全事件族 start/iteration/end（R3249-R3251）+ cancel（display:none R347→S3a 代际差分）既有派发不变 | —— |
| 命中面 | 导出域动画通道（单层 transform，含 origin 折叠全仿射逆映射，S3c） | 静态 transform 命中、getBoundingClientRect 几何查询面（缺口表记档）；嵌套动画 transform 不组合逆映射（S3c 记档） |
| worker canvas | OffscreenCanvas 2D 子集（`canvas_context_op` 纯栅格核零改动复用）+ transferControlToOffscreen 迁移 + patch 呈现管线（双 flag 门控） | `importScripts`/嵌套 Worker（S1 裁剪，worker 体需自包含）；MessagePort 全量 transfer 语义走既有 shim 面；GPU 上传零拷贝 shm（§4 优化预留）；dirty-rect（同前） |
| rAF 驱动 | 页面 rAF 为预算制同步 stub（基线记档）——连续 canvas 动画驱动器选定时器族（worker bootstrap setTimeout/setInterval） | 页面侧 rAF 自续环不适用（基线 §附加发现） |
| **WebGL/WebGL2** | **未执行**——依赖 [webgl-compat](../../../webgl-compat.md) 既有门控，本 goal 不能替代其授权（goal 边界条款）；worker 内 WebGL 同属未触碰 | 适用门禁：webgl-compat goal 的授权与验收流程，本 goal 的「支持的 2D 子集」表述不含 WebGL |

## 3. 适用门禁记录

| 门禁 | 状态 | 最近证据 |
|---|---|---|
| `make test` | 每实现批全量 EXIT=0（M3 各批记录；S1d 收尾批 EXIT=0） | 各批 master.md 执行记录 |
| `make reftest` | registry 批 508+204 全过（canvas 语料零回归）；S1b 批 testharness-workers 693P/12F 与基线同数零新增 | [registry 修复](../2026-10-22-m3-canvas-registry-fix/README.md) / [S1b canvas 桥](../2026-10-22-m3-s1b-canvas-bridge/README.md) |
| fmt / clippy -D warnings | 每批干净 | 各批执行记录 |
| `make bench-gate`（默认路径性能） | 本批实测（结果见下节） | 本目录 bench-gate.txt |
| `make product-smoke`（DC-13 生产 parity） | 本批实测（M4「现有生产 parity」面；本 goal 未触渲染/布局语义，按 M4 审计面补跑） | 本目录 product-smoke.txt |
| fuzz-repro-guard | 不适用——本 goal 无解析器/病态输入面变更 | —— |

## 4. 默认路径性能（bench-gate 实测）

双 flag 默认关 = 生产默认路径不含本 goal 的行为变化（S1a–S1d 全部 flag 门控，flag 关
零注册/零消息/零参与，各批 flag 关回归腿 SHIM_BASELINE_PASS 钉住）。bench-gate 全量
实测对照 linux-x86_64 基线（113 指标），结果与门禁裁决见 bench-gate.txt（本目录）。

## 5. DC 审计汇总

| DC | 状态 | 条件项 | 验收记录 |
|---|---|---|---|
| DC-1 | ✅ 有条件成立 | 忙窗 ~1s 空洞归因浏览器主线程 Wayland 耦合（非采样链路缺陷） | [S1e](../2026-10-21-m2s1e-dc1-acceptance/README.md) |
| DC-2 | ✅ 有条件成立 | CSSAnimation 对象面未建（范围裁剪记档）；相位逐帧容差由单一采样器结构性保证 | [S2c](../2026-10-21-m2s2c-dc2-acceptance/README.md) |
| DC-3 | ✅ 成立 | 静态 transform 命中、空白区点击兜底（预存在缺口记档） | [S3d](../2026-10-22-m2s3d-dc3-acceptance/README.md) |
| DC-4 | ✅ 有条件成立 | 呈现面忙窗连续性判定待遗留缺口 ① Wayland 空洞收口（数据面已三腿独立证明） | [DC-4 验收](../2026-10-22-m3-dc4-acceptance/README.md) |
| DC-5 | ✅ 成立 | 无——资源残留八面审计（§1）+ 支持范围/门禁如实记录（§2/§3）；导航像素证据 comp04b、tab 关闭 Release 臂 + no-op 兜底 | 本目录 |

## 复现

```sh
make bench-gate          # 默认路径性能门禁（对照 docs/perf/baselines/linux-x86_64.json）
make product-smoke       # DC-13 生产 parity 门禁
```
