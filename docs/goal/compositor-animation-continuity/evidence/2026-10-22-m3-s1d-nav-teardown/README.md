# M3：S1d 生命周期——导航换代 worker 全表清理 + patch 摘除（DC-4 (c) 换代腿）

日期：2026-10-22 ｜ 里程碑：M3（S1d 首批）｜ 前置：S1c patch 呈现
（[2026-10-22-m3-s1c-patch-pipeline](../2026-10-22-m3-s1c-patch-pipeline/README.md)）
｜ RFC：[m3-slice-worker-canvas.md](../../m3-slice-worker-canvas.md) §2.1/§2.3

## 判定

**导航换代清理成立（端到端像素 + 宿主日志双证）**：comp04b @4000ms `location.href`
导航换页——宿主 `导航换代清理 worker 宿主全表 workers=1 patch_sinks=1`（terminate 全
部 worker + Destroy 直发）；帧序列 t=2227-4117 为旧页蓝条（BAR），**t=4367（=清理
时刻）起区域恒定为目标页绿盒至采集结束（terminal_freeze onset 4367 dur 9381ms），
零蓝条跨代残留**。无 teardown 时滞留 worker 会以新世代 epoch 把蓝条 patch 注册到新
文档呈现面（cleanup_stale_patches 只清旧世代，拦不住）——此即本批关闭的泄漏签名。

## 实现（renderer `worker_host.rs`）

- **世代守卫**：宿主线程每轮（8ms）比对导航世代共享槽（navigate/load_html store 点，
  与 PatchFace 同源）——推进即 [`teardown_for_navigation`]：terminate 全部 runtime、
  摘绑定表、per-sink `destroy_all`（已注册 patch 发 Destroy 携 Register 世代快照）。
  不派发 `closed` 事件（所属文档正在卸载，页面侧无接收面；HTML spec worker lifetime
  tied to document）。
- **终末栅栏**：`PatchSinkState.torn_down`——`destroy_all` 置位后 `flush_updates` 静默。
  宿主清理与 worker 线程 drain 并发时，滞后 Update 不得复活已 Destroy 贴片（compositor
  同世代帧推进不清同世代项，栅栏是唯一确定收口；Destroy 与 pending 收集同锁序，同 sink
  内 Register/Update/Destroy 有序）。
- compositor 侧帧推进 `cleanup_stale_patches`（S1c 已落）与本次宿主清理互为双保险：
  旧世代项由帧推进清，活 worker 由宿主清。

## 测试

- `teardown_for_navigation_clears_tables_and_fences_late_flush`（renderer）：同代不触
  发、世代推进触发一次（二次检查不重复）；全表清空；Destroy 携 Register 世代快照；
  栅栏后滞后 flush 零发送（真 `WorkerRuntime` + 真 patch sink 全链）。
- S1c 既有 patch 三消息/compositor cleanup_stale_patches 测试保持绿（make test 全量
  0 FAIL）。

## 观察腿（comp04b-worker-nav-teardown.html + comp04b-target.html）

双 flag（`ZW_THREADED_WORKERS=1 ZW_WORKER_CANVAS_PATCH=1`）headless 14s：旧页 canvas
转 worker 画蓝条（16ms tick），@4000ms 导航到目标页；目标页自有 patch 画常绿盒
（50ms tick）——导航后其 Register/Update 使呈现轮询恢复，frame_dump 可继续出图，
使「新页无蓝条」可像素判定（无 patch 的静页 dump 会停止，判不了残留）。

- [analysis.txt](analysis.txt)：240 帧，BAR 相（2227-4117）+ 1000ms 加载窗呈现空洞
  （2285-3285，既有缺口；[unwrap.txt](unwrap.txt) 跨洞 60 tick/1000ms = 16.7ms/tick
  ——数据面照旧连续）+ **terminal_freeze onset 4367 dur 9381**（绿盒恒定）。
- 帧分类时序：`other`(灰)→BAR@2227→(灰/条双拍)→**GREEN@4367 恒定**；清理日志时刻
  57.584 = t 4367 ✓。
- [browser-log-keylines.txt](browser-log-keylines.txt)：两次换代清理日志（首次为
  空表——宿主启动先于首帧导航，last_seen=0，无害）。
- frames/ 位图不入库（同 S1c 证据目录纪律），manifest.jsonl 留 seq→wall_ms 映射。

## 复现

```sh
make browser-build
DC4_FIXTURE=comp04b-worker-nav-teardown.html ZW_WORKER_CANVAS_PATCH=1 \
  scripts/compositor-animation-dc4-observe.sh <out> on 14
scripts/compositor-animation-dc4-analyze.py <out>/frames > <out>/analysis.txt
scripts/compositor-animation-dc4-bar-unwrap.py <out>/frames --origin <navigate+125ms ISO>
```
