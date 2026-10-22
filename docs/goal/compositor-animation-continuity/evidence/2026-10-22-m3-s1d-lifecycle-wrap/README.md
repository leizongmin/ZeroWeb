# M3-S1d 收尾：错误路径 patch 面核对 + 关闭/导航资源审计收口

日期：2026-10-22。S1 RFC §1 场景 (d)「错误路径明确」的 patch 面观察腿 + §2.4 资源释放面全表审计。

## 判定

**(d) 错误路径明确——成立**。comp04c 观察腿（双 flag，fixture `comp04c-worker-error-paths.html`）：

1. **脚本错误不清 patch 呈现**：worker 绘制环 @t0+2500ms（worker 钟）单次 `throw`——bootstrap
   TryCatch 捕获 → `WorkerEvent::Error` → 页面 `onerror`。strip-err 条带带哈希
   `3c4fc9f93f0d`（err=none）→ `1fd8320d760a`（错误文本）恰一拍翻转（[strip-band.txt](strip-band.txt)，
   翻转帧 t=3265 为加载窗空洞后首帧）；洞后蓝条连续推进 107 帧（t=3267→5083，
   distinct_region_hashes=59），条带解卷逐拍 +4px（[unwrap.txt](unwrap.txt) 尾段 page_ms
   2682→2868 跨回绕点 228→0→4→8）——错误事件后呈现面照常更新。
2. **worker `close()` 终末冻结**：@t0+5000ms worker 内 `close()` → 单出口 `Closed` → 宿主
   `finish_worker` → sink `destroy_all`（Destroy → compositor detached 定格最后位图）。
   采集止于 t=5083（bar x=8，sha `6ab5c149f470`）——compositor 随 patch Destroy 转静默，
   「采样停止 + 末条带恒定」形态（同 comp04a terminate 腿判读）；末段前最后 5 帧仍逐拍
   变化（close 在 drawFrame 绘制之后同拍触发，末帧含最后次 Update）。全 log 零
   panic/abort（[browser-log-keylines.txt](browser-log-keylines.txt)）——不静默悬挂。
3. **加载窗空洞跨洞解卷**：空洞 2276→3265（991ms，既有 Wayland keep-alive 挂阻塞读
   缺口的加载窗形态）两侧 59 ticks = **16.8ms/tick**——数据面穿洞连续（[unwrap.txt](unwrap.txt)）。

**实现修正（本批，审计发现的缺口）**：宿主异常退出探测。panic 绕过
`worker_thread_fn` 线程入口的单出口 `Closed` 发送，而 `ThreadedRuntimeCore::try_recv`
把 Empty/Disconnected 折叠为 `None`——宿主永不感知，patch 滞留 + 页面无事件 = RFC §2.4
明令禁止的静默悬挂。修复两处：

- `script-sandbox`：`ThreadedRuntimeCore::is_thread_finished()`（`JoinHandle::is_finished`，
  send happens-before 线程返回，正常退出 Closed 先入队故判定无竞态）+ 两引擎
  `WorkerRuntime::thread_finished()` 对称暴露。
- `apps/renderer/src/worker_host.rs`：`pump_events` 队列排空后线程 finished 且无 Closed
  → `finish_worker(id, "error", "worker thread exited unexpectedly")`（全表摘除 + 终末
  Destroy + 页面 error 派发，RFC §2.4「patch 冻结 + 页面 error 事件，不静默悬挂」）；
  Closed 臂与异常臂共用 `finish_worker`。

## 关闭/导航资源审计收口

| 释放面 | 触发 | 路径（核对于本批） | 测试/证据 |
|---|---|---|---|
| `worker.terminate()` | 页面主动 | `HostCommand::Terminate` 臂：`runtime.terminate()`（terminate_execution + 限时 join/detach）+ 摘绑定表 + `sink.destroy_all()` | worker_host `patch_sink_destroy_sends_only_registered_then_clears`；comp04a terminate 腿（[S1c 证据](../2026-10-22-m3-s1c-patch-pipeline/README.md)） |
| worker `close()`/正常退出 | isolate 退出 | 事件泵 `Closed` → `finish_worker("closed")`（摘全表 + Destroy） | worker_host `host_op_round_trip_spawns_threaded_worker_and_dispatches_events`；本证据腿判定 2 |
| **异常退出（panic 绕过单出口）** | 线程死亡无 Closed | 事件泵异常臂（本批新增）→ `finish_worker("error", …)` | script-sandbox `test_worker_thread_finished_detects_panic_bypassing_closed`（drain panic 探针端到端）+ worker_host `abnormal_thread_exit_reports_error_and_destroys_patches`（真线程 + 注册态 sink：全表摘除 + Destroy 帧） |
| 导航换代 | `navigation_epoch` 共享槽推进（navigate/load_html store 点） | 宿主 `teardown_for_navigation`（terminate 全表 + Destroy 直发，S1d 首批）+ compositor `cleanup_stale_patches` 旧世代清理（帧推进点）+ sink `torn_down` 栅栏 | worker_host `teardown_for_navigation_clears_tables_and_fences_late_flush`；comp04b 腿（[S1d 首批证据](../2026-10-22-m3-s1d-nav-teardown/README.md)） |
| tab 关闭 | `ReleaseCompositorSurface` | compositor Release 臂 `patch_surfaces.remove(&surface_id)`（挂起批/声明态同批清）——apps/compositor/src/lib.rs:1029 | 代码审计（存储随 surface 生命周期，无跨 surface 滞留面） |
| patch 生命周期内 | Register 前死 | 惰性 Register（首 Update flush 才注册）——未注册 patch 终末零消息 | worker_host `patch_sink_lazy_register_then_update_and_defer_on_missing_rect` |
| 进程退出 | browser/renderer 退出 | 宿主线程为 detached 轮询线程、worker 线程随 `WorkerRuntime` drop terminate；进程边界 OS 统一回收，无跨进程泄漏面 | 代码审计（进程模型语义） |

## 测试

- script-sandbox：`test_worker_thread_finished_detects_panic_bypassing_closed`（v8；
  quickjs 走共享 `ThreadedRuntimeCore` 同语义）。
- renderer：`abnormal_thread_exit_reports_error_and_destroys_patches`（真 `WorkerRuntime`
  + panic drain + 注册态 patch sink）。
- `make test` 全量 EXIT=0；`cargo clippy --workspace --all-targets -- -D warnings`、
  `cargo fmt --check` 干净。

## 复现

```sh
make browser-build
DC4_FIXTURE=comp04c-worker-error-paths.html ZW_WORKER_CANVAS_PATCH=1 \
  scripts/compositor-animation-dc4-observe.sh <out-dir> on 14
python3 scripts/compositor-animation-dc4-analyze.py <out-dir>/frames
python3 scripts/compositor-animation-dc4-bar-unwrap.py <out-dir>/frames \
  --origin <manifest 首帧 wall_ms+2215> --busy 2276-3265 --table
```

本次运行 run-config（[run-config.txt](run-config.txt)）：`mode=on flag=ZW_THREADED_WORKERS=1
ZW_WORKER_CANVAS_PATCH=1 wall=14s fixture=comp04c-worker-error-paths.html`；113 帧
（manifest.jsonl 入库，帧位图体积不入库）。origin 取值：首帧 wall_ms+2215（首条带
t=2231 处 seq=1 → t0 ≈ 2215；解卷为相对判定，origin 偏差不影响 ms/tick）。

## 已知边界

- 呈现面忙窗连续性判定仍受既有 Wayland 空洞缺口限制（本腿 989ms 加载窗空洞同型）；
  数据面由解卷覆盖。
- 异常退出探测上界为一个事件泵周期（8ms）；`is_finished()` 对已退出线程即时真，
  panic 展开毫秒级，实测单泵拍内收尾（单测无重试依赖）。
- V8 CHECK 失败走进程 abort（script-sandbox lib.rs 顶部注记），进程边界回收覆盖，
  不在 worker 级探测面内。
