# M3：S1c patch 呈现管线——worker 位图直写呈现链四腿观察（DC-4 (a) 数据面 + (c) 终末冻结）

日期：2026-10-22 ｜ 里程碑：M3（S1c）｜ 实现：4b899ad52 ｜ 前置：S1b canvas 桥
（[2026-10-22-m3-s1b-canvas-bridge](../2026-10-22-m3-s1b-canvas-bridge/README.md)）
｜ RFC：[m3-slice-worker-canvas.md](../../m3-slice-worker-canvas.md) §2.3

## 判定

**DC-4 (a) 数据面成立**：双 flag 腿两跑（on-patch-leg、on-patch-debug-leg），忙窗呈现
空洞两侧条带解卷均落在 worker 16ms tick 带内（**17.2 / 16.9 ms/tick**）——主线程忙窗
期间 worker 持续绘制，位图经 PatchSink→SharedWriter 直写→browser 独立排队中继→
compositor 贴片呈现，不经过被阻塞的主线程 painter。

**DC-4 (c) 终末冻结成立**：terminate @6500 后条带定格（末活跃条 x=140 @page 6677，
含 ~180ms 在途更新尾）；7500 静默核查帧 = 灰底基线 commit + 定格条重贴（detached
定格位图仍参与，呈现不回退空白，RFC §2.3 语义）；此后 `compositor_patches_active`
清零、Get 轮询停、dump 止——终末冻结以「末条带恒定 + 采样停止」形态存在，
dc4-analyze 的 `terminal_freeze` 探测不到（它要求冻结段内有持续 dump），判读见
[unwrap](on-patch-debug-leg/unwrap.txt) 末行 `same-x:frozen-or-wrap` + 静默核查重贴同 x。

**呈现面忙窗连续性部分受限（既有 OPEN 缺口，非本批回归）**：忙窗内 keep-alive Get
挂在 `wayland_client::conn::blocking_read`（S1e 已归因、4 次复现），呈现采样出现
~1-1.7s 空洞；off 腿同现。本批两 on 腿一个空洞只盖忙窗前半（on-patch-leg 忙窗内仍有
22 dump 连续条带），一个全盖（debug 腿忙窗内 0 dump）——呈现面级判定待该缺口收口。

## 四腿设置与结果

| 腿 | flag | 区域 distinct | 忙窗(3000-3900)内 dump | 跨洞解卷 | 角色 |
|---|---|---|---|---|---|
| [off-leg](off-leg/analysis.txt) | 双关（同 realm） | 47/113 帧 | 1（冻结段 2250-4032 跨忙窗，1298+484ms） | ambiguous（同 realm tick 真停，位置差跨段不可解卷） | 基线：主线程忙即页面停 |
| [on-nopatch-leg](on-nopatch-leg/analysis.txt) | THREADED=1 patch=0 | **1**/29 帧 | 0 | 无条带（bar_dumps=0） | S1b 中间态：worker 绘制不可见（像素困在 worker 本地 registry） |
| [on-patch-leg](on-patch-leg/analysis.txt) | 双开 | 59/183 帧 | **22**（3356 起连续条带） | 1030ms→60 tick=**17.2ms/tick** | S1c 判定腿 |
| [on-patch-debug-leg](on-patch-debug-leg/analysis.txt) | 双开（补采全 log 腿） | 59/205 帧 | 0（空洞 2278-4020 全盖） | 1742ms→103 tick=**16.9ms/tick** | 数据面判定腿（log 关联用） |

## 条带解卷方法（数据面连续性证据）

区域哈希只见呈现面；忙窗呈现空洞期无 dump，区域哈希判不了 worker 是否在画。
[dc4-bar-unwrap](../../../../../scripts/compositor-animation-dc4-bar-unwrap.py) 从每帧
canvas 区域解码蓝条 x=(seq*4)%232 → seq mod 58，跨 >300ms 空洞按 elapsed/16 选整圈数
解卷：解卷 tick 数 ÷ 跨度 ≈16-17ms/tick 即 worker 绘制未停（数据面存活）；off 腿同
窗真停（同 realm 共享主线程）则位置差落不进任何整圈读数（ambiguous）——两腿互为
对照。busy 窗内 dump 数（呈现面占空）一并列出。

## 观察明细（on-patch-debug-leg，页钟原点 = 文档解析 02:35:24.308Z）

- page 52 首帧；52→2239 区域灰、无 dump——加载窗 tokenizer 爆发期呈现 poll 饥饿
  （既有行为，与忙窗空洞同机制，见 [browser-log-keylines](on-patch-debug-leg/browser-log-keylines.txt)）。
- page 2240 首条（lazy Register：首 Update flush 时布局快照就绪才发 Register，RFC §2.3；
  此前不注册不发消息）。
- 忙窗 3000-3900：呈现 0 dump；keep-alive Get wall 02:35:26.586→28.319 停/复（1733ms，
  与跨洞 1742ms 对齐）；跨洞解卷 2278→4020 = 103 tick = **16.9ms/tick**——worker 数据面
  穿洞连续。
- 洞后 4020→6677 连续贴片，dump 间隔 5-30ms、逐条 x+4。
- terminate @6500：末活跃条 x=140 @6677（其后在途 Update 尾 ~180ms）；6723 灰（基线
  commit 覆盖）+6725 定格条重贴；静默核查 @7500 → 7705 灰 + 7706 x=140 重贴（不变）；
  随后轮询停、dump 止。
- 灰+条 1-6ms 双 dump 贯穿全跑：beats 条带 50ms 变更推基线帧（canvas 区域灰）覆盖贴片，
  下次 Get 重贴——正确路径固定开销，非抖动。

页钟原点说明：debug 腿原点取 renderer 首次 tokenize 时刻（log 可查）；其余腿原点 =
navigate+125ms 估计（同腿 log 无 tokenize 行，±50ms 不影响 900ms 忙窗定位与 ms/tick
判定）。

## 已知观察缺口（记档）

- 忙窗呈现空洞：既有 OPEN（呈现路径与 Wayland 耦合，master.md 遗留缺口 ①）；本批
  复现于全部四腿，条带解卷证明 patch 数据面不受其影响。
- 终末冻结不可被 dc4-analyze `terminal_freeze` 探测（采样停止形态）：以 unwrap 末行
  same-x + 静默核查重贴同 x 判读；如需 analyzer 可判，须让 patch Destroy 后保底轮询
  数帧——S1d 生命周期批再评估是否值得。

## 复现

```sh
make browser-build
scripts/compositor-animation-dc4-observe.sh <out>/off-leg off 16
ZW_WORKER_CANVAS_PATCH=1 scripts/compositor-animation-dc4-observe.sh <out>/on-nopatch-leg on 16
ZW_WORKER_CANVAS_PATCH=1 scripts/compositor-animation-dc4-observe.sh <out>/on-patch-leg on 16
scripts/compositor-animation-dc4-analyze.py <out>/on-patch-leg/frames > <out>/on-patch-leg/analysis.txt
# 数据面解卷（origin 取各腿 navigate+125ms 或 renderer 首 tokenize 时刻）：
scripts/compositor-animation-dc4-bar-unwrap.py <out>/on-patch-leg/frames \
  --origin <ISO8601> --busy 3000-3900 > <out>/on-patch-leg/unwrap.txt
```

各腿 [run-config.txt](on-patch-leg/run-config.txt) 记录 flag 与墙钟；manifest.jsonl 保留
seq→wall_ms 映射（frames/ 位图因体积不入库，见 [.gitignore](.gitignore)）。
