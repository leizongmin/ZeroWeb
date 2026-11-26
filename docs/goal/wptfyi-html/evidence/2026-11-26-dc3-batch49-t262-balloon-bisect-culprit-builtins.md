# 批四十九 · test262 气球目录二分定谳＝culprit `test/built-ins`

**日期** 2026-11-25 22:56Z – 11-26 01:29Z（UTC 标签按本机时钟）
**流** wptfyi-html（ZeroWeb-3 clone）　**性质** waiting-window 诊断，非门腿；二分二进制复用五臂门链产物（face adf767f5b），零构建

## 背景与问题

批四十七/四十八 CI `test262-builtins` 分片两次贴帽击杀（6,309,664 KB，同数字逐字重演）→ 毒 JSON → aggregate 拒写快照；本地五臂门探针 adf767f5b 面 ~70min 同形击杀（guard 读数 6,297,236 KB）。blocker 归对端 TT/CSP 链，但当时只有「全套件会炸」没有「炸在哪个目录」——本轮二分回答后者。

## 方法

`/tmp/batch48/t262-bisect.sh`（nohup）：按 test262 顶层目录逐目录过滤跑（路径子串过滤，小目录先行），每臂 test-guard 包裹（`--per-proc-mem 6 --total-mem 8`）＋ `/usr/bin/time -v` 仪表＋30s RSS 侧车采样器（采样 runner 进程 VmRSS/VmHWM）；首见 CAPKILL 即写 `CULPRIT_TOP` 并早停，末尾 `BISECT_DONE`。每臂跑前对齐对端 soak 判据（无 cargo/rustc/runner/test-guard 进程＋load1<3）。

## 判定（bisect.log 逐行）

| 臂 | 过滤 | 墙钟 | rc | 峰值 (KB) | 判读 |
|---|---|---|---|---|---|
| annexB | annexB | 3.0m | 1 | 259,068 | 完跑无气球 |
| staging | staging | 5.4m | 1 | 1,772,536 | 完跑无气球（峰值子进程持有） |
| intl402 | intl402 | 9.6m | 1 | 382,592 | 完跑无气球 |
| language | language | 68.8m | 1 | 607,972 | 完跑无气球 |
| **builtins** | built-ins | **66m（击杀）** | **124** | **6,434,960（guard 消息）** | **CAPKILL → CULPRIT_TOP** |

rc=1＝runner「有非 Pass 即 exit 1」设计行为（完跑）；击杀臂 json 0 字节。

## 定谳

1. **气球唯一顶层目录＝`test/built-ins`**：同二进制同面下其余四臂全部完跑且峰值 ≤1.77GB，唯 built-ins 在 ~66min 贴帽击杀。
2. **runner 自身无失控**：击杀臂 runner VmHWM 全程平台 ≤0.61GB（131 个采样，形状与 language 臂同）；6GB 单进程在**子进程侧**（渲染子进程），与 staging 臂 1.77GB 子进程持有、批四十八「time -v 儿进程 rusage」机制画像互证。
3. **跨面确定性**＝8485db7a76 面 CI 二演 6,309,664 KB＋本地探针 A（built-ins 显式路径）6,338,488 KB @67min；adf767f5b 面（含 s10k27 刀㉗＋刀㉘/-B）本轮 6,434,960 KB @66min——**s10k25返修＋刀㉗＋刀㉘ 均未解**，三次击杀都落在 66-67min 带。
4. **归属不变**：built-ins 气球＝对端 TT/CSP 链 blocker（嫌疑域 part12-trusted-types／part14-trusted-sinks 已两刀未中），本流不硬修；本证据包为对端定位输入。

## 工件

- `/tmp/batch48/bisect/bisect.log`（逐臂判定行）、`CULPRIT_TOP`、`BISECT_DONE`
- `/tmp/batch48/bisect/<tag>.json｜-time.txt｜-rss.log`（builtins 臂 rss.log 含击杀前 131 采样曲线）
- 交叉参照：批四十八证据（CI 击杀数字）、`/tmp/batch48/t262-probe-chain.sh`（探针 A 判据）、`/tmp/batch48/b49-gate-chain-v3.log`（五臂击杀读数）

## 后续

v3 门链 soak 在 BISECT_DONE 后自动开启，HEAD 重取至 a842bf953（刀㉙返修三g），build rc=0（01:32:58Z），全量 test262 探针在飞（tl=9000s）——判定窗口：气球未解则 ~02:40Z 击杀；已解则完跑窗在 150min 帽边缘（per-dir 墙钟和 ≈157min，帽内完跑不保证）。判定与 dispatch 由本流后续轮次承接。

**收官（02:45-02:50Z）**：HEAD 探针 ~72min 贴帽击杀 6,370,364 KB＝第四源同带（<2% 离散）＝返修三g 未解气球，与本二分 culprit=built-ins 定位一致（击杀点 ≈ annexB ~3min 后 built-ins ~69min）；workers 腿 SET-IDENTICAL、B46 平行点未触发；B49_GATE_V3_SUMMARY＝`HEAD_SHA=a842bf953 VERDICT=CAPKILL_BALLOON RC=124 PEAK_KB=6370364 JSON=INCOMPLETE / WORKERS_RC=1 COMPARE_TAIL=VERDICT: SET-IDENTICAL`。dispatch 不发，blocker 留对端 TT/CSP 链。
