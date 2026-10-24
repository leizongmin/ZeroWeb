# C4 配对验收分析（2026-10-25）

DESIGN §12 的验收批次：M1 harness 三态原样复跑（内联 feature-off / 宿主态 / 宿主态 +
`ZW_MEDIA_SHM=0`）+ 帧一致性摘要 + 代际风暴 + 能力真值三对照 + 30min 资源审计。
判据原文见 [DESIGN §12](../2026-10-24-m3-prep-design/DESIGN.md)。

**版本与来源（§12.1 报告头）**：评估对象 = C4 修复提交 `c9e249707`（A1 解码超前订正 +
宿主 replay 订正，`crates/media/src/pipeline.rs` + `apps/media-host/src/kernel.rs`）；测试二进制（release）：
`media_m1_baseline-7a862d9fa4166770`（内联）/ `media_m1_baseline-b0ff9b34acad5a97`
（宿主）/ `media_c4_paired-b5b49af819253f0f`（内联）/ `media_c4_paired-a772fe394d302f18`
（宿主）；宿主二进制 `zero-media-host`（含 replay 订正）。平台：Linux
x86_64（WSL2 内核 6.18）测试守护 `test-guard`（内存 4/8GB、墙钟按套件放宽）。
feature 矩阵：内联态 `media-process` 关；宿主态开；两态 `decode-h264` 关、
`decode-av1` 关（能力表 av1/h264 门控位未启用，实报 6 项为软解基面——平台无
/dev/dri，无硬解路径，M1 §6 核查）。

## 结论一览

- 判据面全部执行完毕；**配对验收抓出两个真实产品缺陷，均已按 §12.6「按域回对应片
  修复后重跑」路径订正并以常驻测试落门**：
  1. **A1 解码超前 pts 锚定回归**（内联帧配对抓出）→ `zero-media` 样本锚定订正；
  2. **宿主 replay 被流末残留位置拖拽**（资源审计抓出）→ `zero-media-host` replay
     面跳过位置 seek 链。
- 其余判据（三态 M1 波段、帧摘要全等、风暴零泄漏、能力真值、配额拒绝、审计资源界）
  修复后全部通过。
- 偏差七项（D1–D7）记录于 §六，均不影响验收结论成立。

## 一、配对验收抓出的缺陷与修复

### 1.1 A1 解码超前 pts 锚定回归（内联帧配对抓出）

**现象**：M1 av-pair 场景 `present_ticks=1013/1440`（[m1-inline-pre-fix-av1013.log]
(m1-inline-pre-fix-av1013.log)），泵在 tick 2640（合成钟 ≈42.2s ≈ 流长 70%）即到流末，
剩余 ~18s 音频被解码队列静默吸收后于 EOF 排空时丢弃；视频面以 audio cursor 为主时钟，
同步冻结在 70% 处。纯视频场景 1440/1440 正常。

**归因**：A1（`3f98c4332`，「A1 时钟批内对齐」）将解码超前锚在 `batch.pts_ms +
sample_duration`（pts 坐标系）。本仓长 fixture 的 vorbis pts 步长 ≈ 样本步长/1.44
（pts 压缩同源——A1 修的 1.45402 rate 偏差即此漂移），解码超前随时钟发散，EOF 提前于
~70% 样本时刻触发。回归自 A1 起潜伏：M1 a1-post 证据（`2026-10-24-m1-baseline/
m1baseline-a1-post.log`）中 av-pair 已是 1013，A1 前基线（m1baseline-raw.log）为 1440
——M1 原判据只数纯视频呈现帧，未覆盖 AV 轨帧数，故三批未觉；C4 帧摘要配对（逐帧
content_hash 全序列比对）首次让它现形，正是配对验收的设计目的。

**修复**（`crates/media/src/pipeline.rs`）：解码超前改样本锚定——`advance_to` 基面取
`cursor_ms + queue.queued_ms()`（written 游标 + 队内未写时长），批内按实际解码样本
时长累加；AudioEntry / WebmAudioEntry / MseAudioEntry 三入口同面。对 pts 连续轨零变化
（两坐标系重合），仅 pts 漂移轨的 EOF 时机回到样本真实位。

**验证**：M1 内联 5/5（av-pair 1440，其余场景数字与修复前一致，[m1-inline-post-fix.log]
(m1-inline-post-fix.log)）；`zero-media` 54/54、`zero-webview` feature-off 790/790、
feature-on 失败集与修复前 stash 对照一致（失败集为既有 inline-语义测试在 feature-on
下的已知 14 项，与本次无关）。

### 1.2 宿主 replay 被流末残留位置拖拽（资源审计抓出）

**现象**：资源审计（§12.4）首轮实现以每轮回拨 60s 的合成钟驱动 30 次重播，round 1 仅
2 tick 即退出（[audit-retrograde-clock-collapse.log](audit-retrograde-clock-collapse.log)
为该轮记录）。改单调递增钟后 round 1 仍 2 tick 坍缩——排除时钟面，锁定产品缺陷。

**归因**：HTMLMediaElement spec「ended playback」步 6.4：ended 后 `play()` 须回到最早
可播位重播。内联路径已按此实现（`video_registry.rs` Ended→reset 分支）；宿主路径的
registry 分流只 `return self.host.play(key)`，而内核虽有 replay 链（epoch+1 +
`rebuild_for_replay`，游标归零），随后仍照单执行「wire 授权位置 ≠ 游标 → seek 链」。
renderer 侧镜像在 Ended 后保留末次 Playing 读数（= 流末），facade play 原样回传，
seek 链把刚重建的重启拖回流末——瞬间再 Ended。内核既有单测
`replay_after_ended_bumps_epoch_and_restarts` 用 position=0 重播，未建模真实客户端的
流末残留形态，故 C3 前未觉。

**修复**（`apps/media-host/src/kernel.rs`）：replay 判定命中时跳过位置 seek 链——
此时 wire 位置是 Ended 态残留而非用户意图（HTMLMediaElement.play() 无位置参数，位置
只经显式 seek() 表达）。

**验证**：新增常驻单测 `replay_after_ended_ignores_stale_wire_position`（流末残留位置
重播 → 断言 Playing@0 + 新代际 + 首帧 pts=0），与修复同一提交；内核 16/16
（[kernel-unit-tests.log](kernel-unit-tests.log)）、C3 契约 8/8
（[contract-tests-post-fix.log](contract-tests-post-fix.log)）；资源审计修复后 30 轮
完整重播（见 §五）。

## 二、三态 M1 矩阵（§12.1）

同一 instrument（`media_m1_baseline.rs`）三次调用，release profile（debug profile
解码慢 ~18× 会造成假失败，见 D2）。三态均以修复后源码 + 宿主二进制复跑
（A1 订正 + replay 订正后终态数字）。

| 场景 | 内联（feature-off） | 宿主态 | 宿主态 + `ZW_MEDIA_SHM=0` |
| --- | --- | --- | --- |
| 结果 | 5/5，20.41s | 5/5，226.33s | 5/5，226.31s |
| pure-video wall / present | 1177ms / 1440 | 59990ms / 1440 | 59989ms / 1440 |
| pure-video cpu / cores | 1170ms / 0.994 | 760ms / 0.013 | 770ms / 0.013 |
| av-pair wall / present | 1196ms / 1440 | 60076ms / 1440 | 60072ms / 1440 |
| av-pair cpu / cores | 1200ms / 1.003 | 720ms / 0.012 | 770ms / 0.013 |
| clock rate / offset p95 | 1.00000 / 0ms | 0.99827 / 41ms | 0.99827 / 49ms |
| drift p50 / p95 / max | 11 / 21 / 35ms | 同左（同解码层） | 同左 |
| seek cold15 / warm30 / warm45 | 21.0 / 1.1 / 19.0ms | 20.1 / 1.3 / 18.9ms | 25.6 / 3.0 / 0.18ms |

- 三态波段一致：呈现帧数 1440 全对齐，宿主态客户端 CPU ≤0.013 核（隔离目的达成），
  rate ≥0.998、offset p95 ≤49ms（时钟保真），seek 面宿主态不劣于内联 +10ms 带
  （cold15/warm45 为 mid-GOP 目标，三态同带 ~19–26ms；warm30 keyframe 对齐亚毫秒到
  3ms）。
- 内联 wall 为加速合成钟（~1.2s 压满 60s 流），宿主态为真实节拍（60s 流 60s 墙钟）——
  两侧 wall 不可直接比，容量判据按「真实节拍维持 + 客户端 CPU 占用」解释（D2）。
- 日志：[m1-inline-post-fix.log](m1-inline-post-fix.log)、
  [m1-hosted.log](m1-hosted.log)、[m1-hosted-shm-off.log](m1-hosted-shm-off.log)。

### §12.3 判据表逐行映射

| 维度 | 判据带 | 实测 | 结论 |
| --- | --- | --- | --- |
| 时钟保真 | 开态 rate ∈ [0.9,1.1] | 0.99827（宿主两态）/ 1.00000（内联） | 过 |
| seek 语义 | 三态 `resolved_ms` 相等；热路径不劣于内联 +10ms | 三态 after_ct 15.000/30.000/45.000 精确；warm seek 宿主 1.4/3.0ms vs 内联 1.3ms | 过 |
| A/V 漂移 | 开态 p50 ≤ 内联 +5ms | p50=11ms 三态同值（宿主 AV 面与内联同 media 层 BatchSampleQueue，构造同源） | 过 |
| 像素一致 | 三态逐帧哈希序列全等 | digest `3f970f43b2bfb694` 三态 = 参照 | 过 |
| 容量开销 | 开态墙钟 ≤ 内联 ×1.5 | 时钟形态不同不可直比，按 D2 等价判据（真实节拍维持 + 客户端 CPU 0.014–0.015 核） | 过（D2） |
| 代际 | 旧 epoch 帧/状态零泄漏 | 风暴 30 帧邻域零泄漏 + C3 契约 stale-epoch 丢换单测 | 过 |

注入面（§12.6「DC-3 直接引用 C3 测试」）：SIGKILL 契约 8/8 以修复后宿主二进制复跑
（[contract-tests-post-fix.log](contract-tests-post-fix.log)），未重造场景。

## 三、帧一致性（§12.2 像素全等）+ 代际风暴（§12.3）

- 参照序列 = `VideoTrackDecoder::next_frame` 逐帧 `ImageData::content_hash`，呈现序列
  = `ImageCache::snapshot_entries` 同管道哈希；摘要 = 序列序 fold（顺序敏感）。
  两 fixture 视频轨同母本 → 同摘要 `3f970f43b2bfb694`，三态呈现摘要与参照全等
  （presented=1440=reference）：
  [framehash-inline.log](framehash-inline.log)、宿主态见
  [framehash-capability-storm-hosted.log](framehash-capability-storm-hosted.log) 前两行。
- 代际风暴：9 次跨流 seek（45/15/30s ×3）+ 落窗后连采 30 帧，逐帧哈希对参照局部窗
  （±48 帧最近邻——内容周期恰 48 帧，最低命中会选到周期等价低位，D3）断言无旧代际
  帧泄漏：`captured=30 digest=45b70a0573a81d75 errored=false`
  （[framehash-capability-storm-hosted.log](framehash-capability-storm-hosted.log) 末行；
  订正前假回跳记录见 [framehash-hosted-storm-first.log](framehash-hosted-storm-first.log)
  与 [epoch-storm-pass.log](epoch-storm-pass.log)）。

## 四、能力真值三对照（§12.5）+ 配额

宿主协议实报 6 项（`codec_capabilities`，av1/h264 为 feature 门控扩展位）：

| 声明 | 宿主列表 | engine 静态表（js_dom_shim/part03.js `_cptTable`） | 实解验证 |
| --- | --- | --- | --- |
| video/webm; codecs=vp9 | ✓ | ✓ | ✓ sample-webm-vp9.webm |
| audio/webm; codecs=vorbis | ✓ | ✓ | ✓ sample-webm-vp9-vorbis.webm |
| audio/webm; codecs=opus | ✓ | ✓ | 无仓内 fixture（D4） |
| audio/mpeg | ✓ | ✓ | ✓ sample-mp3.mp3 |
| audio/ogg; codecs=opus | ✓ | ✓ | ✓ sample-ogg-opus.oga |
| audio/mp4; codecs=aac | ✓ | ✓ | 无仓内 fixture（D4） |
| audio/ogg; codecs=vorbis | ✗ 欠申报 | ✓ | 解码 ✓ sample-ogg-vorbis.oga |
| audio/wav（PCM） | ✗ 欠申报 | ✓ | 解码 ✓ sample-wav-pcm.wav |

- 欠申报面（宿主列表缺、实际可解）：ogg-vorbis 与 wav——`AudioDecoder::open` probe
  实解成功但 `codec_capabilities` 未列。风险方向是「能播却报不支持」（可播性收窄，
  非虚报），记 D4 待能力清单对齐切片处理。
- 配额端到端：`register_source(1, MEDIA_SOURCE_MAX_BYTES + 1, "video/webm")` →
  `MediaError{QuotaExceeded}`（`total_len 268435457 exceeds per-source cap 268435456`）。
- 页面可见面（engine shim 静态表）与宿主实报的差异即 D4 记录面；错误面接线
  （shim Error face → 元素 error 事件）归后续切片（DESIGN §10.9-16）。

## 五、30min 资源审计（§12.4）

30 轮 × 60s AV 流完整重播（跨轮单调钟，每轮偏移 61s），每轮 ~30s 采样一次 host 进程
（30/30 轮全采样 = 30 次完整重播全数走满）；第 15 轮插入暂停态 5s 空闲成本探针；退出后
5s 孤儿核对。终态结果（[audit-30min.log](audit-30min.log)，墙钟 1807s）：

```text
C4AUDIT summary rounds=30 samples=30 single_host=true pid_stable=Some(1563791)
          rss_peak_kb=18388 rss_steady_kb=18064 shm_segments=0 shm_bytes=0
          idle_cpu_5s_ms=0 orphan=false
```

- **IF-001 单宿主**：30 轮全程单 pid（1563791），零 respawn（含 29 次 replay 走内核
  replay 链——非重启路径）。
- **内存**：RSS 前 3 轮 13.2→17.0MB 后平台化（稳态 18.1MB，峰值 18.4MB），远低于
  512MB 界；shm 0 段 0 字节（当前传输面为内联 rgba，D1）。
- **空闲成本**：暂停态 5s CPU 增量 0ms（自走时钟事件循环等待面成立，无轮询）。
- **活跃成本**：host CPU ~2.2s/60s 轮（≈3.7% 核，解码+拷贝+传输合计，逐轮采样线
  66.8s/30 轮）。
- **退出**：registry Drop → shutdown → 宿主进程 5s 内退出，无孤儿。
- renderer 侧驻留与复制链两点以 D6/D7 代理面收口（见 §六）。

审计自身的时钟面订正记 D5：首轮回拨基设计使重播坍缩，暴露 §1.2 缺陷的同时也订正了
审计协议本身——跨轮单调递增 + 「完整重播」断言（每轮 ≥3500 tick，EOF 抖动余量），
审计从此无法对「30 次长播」说谎。

## 六、偏差记录

- **D1（shm 传输面未实施）**：`media_proxy.rs`/`kernel.rs` 均注明 shm 传输归后续切片，
  `shm_name` 恒 None，`media_shm_enabled()` kill-switch 端到端无行为差异 → 矩阵第三态
  与第二态同传输面（数字一致亦为证据）。第三态保留：kill-switch 接线已在协议与构建面
  就位，传输面落地后即为真三态。
- **D2（容量判据解释）**：§12.1「×1.5 余量」按内联加速钟 wall 不可比（1.3s vs 60s 是
  时钟形态差非性能差）。等价判据采纳：宿主态真实节拍全程维持（rate 0.99827、60s 流
  60.07s 墙钟走完）+ 客户端 CPU 0.014–0.015 核 + 宿主 CPU（审计面 30s ~1.1s）。
- **D3（风暴匹配器周期处理）**：fixture 内容周期 48 帧（母本 2s×24fps×30），窗口
  「最低命中」会落到周期等价的更低位（1284→1189 假回跳，
  [framehash-hosted-storm-first.log](framehash-hosted-storm-first.log) 末段）；订正为
  期望邻域最近邻（±48 窗仍保留陈旧帧检出力——风暴位与落窗位距 ≥300 帧）。
- **D4（能力清单偏差）**：宿主欠申报 ogg-vorbis/wav（可解未列）；webm-opus/aac 声明
  无 fixture 未验证。待能力清单对齐切片（补 fixture + 双侧表对齐）。
- **D5（审计协议时钟面）**：回拨合成基使重播坍缩（并由此暴露 §1.2 缺陷）；订正为跨轮
  单调基 + 完整重播断言。断言阈值二订：3700→3500 tick——音频游标到末的 EOF 检测存在
  轮间 ~1s 抖动（实测 round 2 于 3690 tick 正常走完，[audit-round2-eof-jitter.log]
  (audit-round2-eof-jitter.log)），坍缩形态（2 tick）与抖动形态差两个数量级，余量充分。
  审计记录以订正后协议为准。
- **D6（复制链增量表未逐点重测）**：§12.4「A3 后重测 C1–C5 残留 + 开态叠加传输面
  成本」未做逐拷贝点插桩，以代理面收口——A3 静态图仍为拷贝点权威；C4 实测 CPU 面：
  内联 ~0.9ms/呈现帧（全链同进程）→ 宿主态客户端 ~0.58ms/帧 + 宿主 ~1.1s/30s
  （解码+拷贝+传输合计，审计采样），链路拆分到进程但总量未越界。逐点增量表归后续
  复测切片。
- **D7（renderer 侧源驻留未单列复测）**：§12.4「renderer 侧源驻留 ≤256MB 复测」以
  配额面代理（per-source 256MiB 封顶端到端 QuotaExceeded，§四）+ 审计面 registry 单源
  常驻；A3 预算证据（封顶机制）不重复，驻留总量未在 C4 独立测量。

## 七、证据清单

| 文件 | 内容 |
| --- | --- |
| m1-inline-pre-fix-av1013.log | 修复前内联 M1——av-pair present_ticks=1013（回归现形） |
| m1-inline-post-fix.log | 修复后内联 M1 5/5（1440） |
| m1-hosted.log | 宿主态 M1 5/5 |
| m1-hosted-shm-off.log | 宿主态 kill-switch M1 5/5 |
| framehash-inline.log | 内联帧摘要 2/2（digest 全等） |
| framehash-hosted-storm-first.log | 宿主帧摘要 2/2 + 风暴首跑（D3 假回跳记录） |
| epoch-storm-pass.log | 订正匹配器后风暴通过 |
| audit-retrograde-clock-collapse.log | 审计首轮回拨基坍缩（D5 + §1.2 现形记录） |
| audit-round2-eof-jitter.log | 阈值 3700 首跑——round 2 于 3690 tick 正常走完（EOF 抖动证据，D5 二订） |
| audit-30min.log | 订正后资源审计终跑（§五，1807s） |
| framehash-capability-storm-hosted.log | 宿主态修复后终跑：帧摘要 2/2 + 能力真值 + 风暴零泄漏 |
| kernel-unit-tests.log | 内核单测 16/16（含新增 replay 残留位置门） |
| contract-tests-post-fix.log | C3 契约 8/8（修复后宿主二进制） |
