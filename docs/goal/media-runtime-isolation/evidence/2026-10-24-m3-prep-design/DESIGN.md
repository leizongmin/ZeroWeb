# M3 前置设计——A1 时钟批内对齐 + seek 冷路径归因与 A4 窄化 + A2 三缺口修复 + WPT GOP 抽样

**日期**：2026-10-24
**状态**：设计就绪（门禁 = D-RFC-2 随批；本文为设计准备，**零代码改动**；§6-§7 为同日第二轮增补）
**goal**：[media-runtime-isolation](../../master.md)
**RFC**：[media-runtime-isolation-spec-rfc](../../../../specs/media-runtime-isolation-spec-rfc.md) v1.2（A4 批次行为首轮增补；A2 行本轮补设计稿指引）

## 1. 门禁状态

两项设计均属 RFC 方案 A（实测缺陷修复，无架构含义）。本轮未改代码、未装 SDK、未变 feature 表；探针为只读分析自建 fixture 的 Python 脚本（/tmp，不入仓），matroska-demuxer 源码为本地 cargo registry 只读查阅。

## 2. A1 时钟批内对齐——机制与修复设计

### 2.1 缺陷机制（源码级确认）

三入口同构（`crates/webview/src/video_registry.rs`）：

| 入口 | advance_to | target 计算 | 游标推进 | 越界 break |
|---|---|---|---|---|
| WebmAudioEntry（A/V 伴音） | :271-330 | :277 | :312-314（批末） | :328 |
| AudioEntry（纯音频） | :138-195 | :144 | :174-177（批末） | :190 |
| MseAudioEntry（decode-h264 门控） | :373-428 | :379 | :394-397（批末） | :408 |

共同形态：`target = cursor_ms + elapsed`；while 循环按**整批**消费（vorbis/symphonia 包粒度 ~1024 样本/ch ≈ 23.3ms @44.1kHz），批末即 `cursor ≥ target` break（「包不可分割」注释 :327）。批长 > 16ms 泵节拍 → 每 tick 恰消费一整批 → 游标以批长/tick 推进 → rate = 23.3/16 ≈ 1.45（M1 实测 **1.45402**）。

### 2.2 修复设计：样本级队列（批内精确消费）

新增私有结构（video_registry.rs 内，三入口共用）：

```rust
/// 批间样本队列：advance_to 按 tick 精确消费（毫秒→样本数定点换算），
/// 消除批粒度对时钟速率的量化（A1 批内对齐）。
struct BatchSampleQueue {
    pending: VecDeque<f32>,  // 已解码未写 sink 的交错样本
    rate: u32,               // 当前批格式（批间不变；变更时清队列）
    channels: u16,
    frac_rem: u32,           // 毫秒→样本数换算余数（定点累积）
}
```

advance_to 改造（返回值、EOF/loop/错误分支语义**不变**）：

1. `target = cursor_ms + elapsed`（不变）。
2. 本 tick 应消费样本数 `need = elapsed × rate × channels / 1000`（`frac_rem` 承接余数）。
3. `while pending.len() < need`：`next_batch()` 追加；`Ok(None)` 走原 EOF/loop 分支，`Err` 走原错误分支。
4. 写出 `pending` 前 `need` 个样本（增益变换原样）；`cursor_ms = target`（流未耗尽时精确推进；EOF 停在实际末尾）。
5. `skip_until_ms` 追赶区：**保持批粒度判定**（批末 ≤ 丢弃线整批静默丢弃，现行为），不到样本级精化——seek 语义面不变，精确追赶区归后续可选项。
6. seek 交互：registry.seek 已重建轨 + `cursor=clamped` + `last_tick_ms=None`（:960-981）——队列随轨重建作废（置空），无需新逻辑。
7. 内存：16ms@44.1kHz×2ch ≈ 1411 样本 ≈ 5.6KB/队列，可忽略。

### 2.3 验收与风险

- T3 `m1_clock_fidelity_realtime_15s` rate 收 **[0.9, 1.1]**（现 sanity 带 0.5–3.0；修复后 rate≡1.0 由构造保证，带宽留 EOF/睡眠抖动裕度）。
- `m1_av_drift_media` 漂移分布**重测后钉带**（量化主项从批粒度 ±11.6ms 变为帧粒度 41.7ms + 泵节拍组合；方向不预设，断言按「不脱轨上界 < 帧间隔 + 节拍裕度」保持）。
- A/V pair 60s 容量场景不回退（每 tick 增加样本切片写，预期 <5%，测量面已备）；空 tick p50 量级不变（无播放快路径保留）。
- 回滚：单提交。

### 2.4 实施偏差记录（2026-10-24 A1 落地轮，相对 §2.2）

实施时发现设计稿三处需修正，均不改变验收判据：

1. **填充条件从 `pending.len() < need` 改为 `decoded_end < target`（crossing 语义）**。原条件在 `pending ≥ need` 时提前退出，跳过 EOF 的 `Ok(None)` 拉——精确游标下（旧码的量化越冲 δ 不复存在）流末恰在 target 跨越后一次拉才暴露，`registry_av_pair_reaches_ended_after_audio_exhausted` 实测失败（视频已 Ended、泵停、音频残尾不消费）。crossing 条件保持「解码侧拉到流位越过 target」的原批粒度终止语义，EOF 拉时序由此保证。
2. **解码超前/时钟推进耦合不变式**（设计稿未显式写出，trim 变体失败实证）：游标只能按**消费**推进，不能按解码位或 trim 推进——解码批 23.3ms/tick vs 时钟 16ms/tick，解耦后解码器以 1.45× 速率先行，流末提前耗尽（60s 流在游标 42s 处 EOF、18s 内容丢失、时钟冻结 runaway）。队列使解码超前有界（~1 批），`cursor_ms += duration_ms_of(written)` 只随写出前进。
3. **EOF 检测晚一拍 + 停止前冲队**。EOF `Ok(None)` 拉比视频 Ended 晚一 tick（生产 renderer rAF 泵 tick_all 无条件节拍，无影响）；测试泵若以 `is_playing(src)` 为停机条件会提前一拍退出——M2 切片 F 用例与 M1 harness 泵条件改 `is_any_playing()`（注释注明缘由）。另：解码超前有界后，停止/回卷时刻队内尚有未消费尾段，EOF/loop/错误停止分支先 `write_to(usize::MAX)` 全量冲队再停（原行为末批整批入 sink 不丢样），游标按冲出时长推进到流末。

验收实测（m1baseline-a1-post.log）：T3 rate=1.00000（修复前 1.45402）；漂移 p50=11.0ms 与基线持平；容量/seek 场景无回退；786 lib 测试全绿。

## 3. seek 冷路径归因（Lint Warning 解除）与 A4 窄化设计

### 3.1 证据链（三方交叉，全部一手）

**M1 实测**（m1baseline-raw.log）：15s→301717µs、30s→1129µs、45s→999822µs；`after_ct` 全部精确；PROBE next frame pts = 15000/30000/45000 精确。

**fixture 块级探针**（Python，本日）：long-webm-vp9-60s.webm 共 1440 SimpleBlock（24fps×60s）、**30 个 keyframe 全部在偶秒**（GOP=2s，母本簇重复结构使然）；块级 keyframe 旗标——tc=15000 `False`、tc=30000 `True`、tc=45000 `False`。cue 点 30 个 = 偶秒簇首（生成器只写 CueTime+CueTrack+CueClusterPosition，无 CueRelativePosition）。

**demuxer 源码**（matroska-demuxer 0.8.1 lib.rs）：

- broad phase（:1707-1721）：cue 二分 `binary_search_by` 未命中取 `saturating_sub(1)` → target 15s 落 **cue 14000 的簇首**（簇级偏移，无 relative position）。
- narrow phase（:1793+）：簇内逐块探时，跳过 `t < seek_timestamp`，停在**首个 ≥ target 的块**——15s 目标跳过簇首 keyframe（14000<15000），落 tc=15000 非 keyframe 块。
- `CuePoint` 为私有结构（:1393），crate **不暴露 cue 访问 API**。

**decode.rs 判定**（`crates/media/src/decode.rs:258-304`）：phase-① 要求落点块 `is_keyframe == Some(true)`（:273-275）——仅在 target 恰为 intra 帧位置（偶秒）时成立；失败 → phase-② `demuxer.seek(0)` + **全量前向解码**至 ≥ target（:285-303）。

### 3.2 因果链（闭合）

mid-GOP 目标（15s/45s，奇秒）→ narrow phase 落非 keyframe 块 → phase-① 判定失败 → phase-② 从 0 前向解码全流 → 耗时 ∝ target：302ms@15s（20.1ms/s 连续解码档，与容量档 22.3ms/s 同量级）、1000ms@45s（22.2ms/s）。keyframe 对齐目标（30s）→ phase-① 直落 → 1.1ms。**冷路径主项不是 I/O 或 cue 表解析，是 phase-② 全量前向解码**；M1 ANALYSIS §3.4 原「冷 I/O + 解码器重建」归因就此修正（解码器重建为次项，量级 ms）。

次要项：registry.seek 每次**重建伴音轨**（:945-959，OGG 重封装 + symphonia/opus open，源字节 1.4MB）——ms 级，A4 不动，记录备查。

### 3.3 A4 修复设计：cue 锚定窄化前向解码

phase-① 失败后不再 `seek(0)` 全量前向，改为：

1. **cue 时间列表**：open_webm 时对源字节自解析 Cues 元素（EBML 平扫顶层 + CuePoint/CueTime，零新依赖；解析先例 = fixture 生成器 `scripts/make-long-video-fixture.py` 同构逻辑，约 60 行）。仅缓存 `Vec<(cue_time_ms, cluster_pos)>` 视频轨条目。
2. **窄化 fallback**：取 ≤ target 的最近 cue（二分）→ `demuxer.seek(cue_time)`（broad phase 落同一簇首；narrow phase 首块 = 簇首 keyframe，tc=cue_time 不再被跳过）→ codec reset → 前向解码至首个 `pts ≥ target`（≤ GOP 帧数，循环含 EOF/错误出口）。
3. **无 Cues 流**：现 phase-② 全量前向保留为最终回退（spec precise-seek 形态不变）。

复杂度：O(target) → **O(GOP)**。本 fixture 15s 预期 302ms → ~25ms（1s GOP = 24 帧 @45×）；45s 同量级。生产流收益随 GOP 增大（2–5s 常态）。

### 3.4 验收与待验证（实施期）

- 扩展 `m1_seek_cue_path`：新增 mid-GOP 目标（15s/45s 已是），断言 seek 耗时 ≤ GOP 前向上界（带 = GOP 帧数 × 单帧 p95 × 安全系数，初值实施期钉）且**不再随 target 线性增长**（同流 15s vs 45s seek 差 < GOP 解码耗时）；现有 15/30/45 三点结果不回退。
- WPT 上游 media/*.webm 抽样核对 GOP 形态（真实生产输入受控确认）。
- 待验证：带 CueRelativePosition/CueDuration 的 ffmpeg 产物流落点复核（broad phase relative 分支 :1721-1727——设计同样成立：cue_time 重 seek 落簇首；实施期以抽样 fixture 验证）。

## 4. RFC v1.2 变更清单

- §7 批次表新增 **A4**（seek 定位窄化：cue 锚定 fallback，`decode.rs` 面）；依赖行 A1–A3 → A1–A4。
- §8 测试策略新增 seek 冷路径回归行。
- §11 Warning×1（seek 未归因）标注 v1.2 已归因；已接受风险行同步收窄。
- §13 修订历史 v1.2-draft。

## 5. 后续验证计划（D-RFC-2 批复后）

A1 与 A4 分批实施、各自单提交（独立回滚点）：A1 先行（C2 依赖时钟语义），A4 随后；每批跑 `make test` 定向 + M1 harness 显式入口复测（release + test-guard），A4 完成后 seek 场景数字回写本目录（追加，不改写本文）。

## 6. A2 三缺口修复设计（同日第二轮增补）

### 6.1 三缺口定性修正（本轮源码核查）

M1 ANALYSIS §7 的缺口 2 表述「全仓零调用方」**仅对 crates/ 域成立**：`_zwMediaTimeMarchesOn` 在
`tests/wpt-runner/src/testharness.rs:9290` 有泵 tick 调用（timer 泵每拍同形提交，:9285-9292 注释
引用 spec#time-marches-on）——M3 扩批 XVIII/XXIV/XXVII 建的 cue/played/timeupdate 机器在 **WPT
runner 面工作**，生产 renderer 主循环（`apps/renderer/src/runtime.rs:4094-4138` 媒体泵）不调。
即：**死的是生产面，不是钩子本体**；M1 ANALYSIS §7 追加订正注（本轮）。

缺口 1+3 同面确认：`_zwMediaAutoplay` 由 load 序列末调用（part06.js:10273，canplaythrough/suspend
之后）——正是上游移交定位的「shim load 时武装环节」（background-resource-lifecycle master.md:45
「终末桥探测证实原生音频管线健康、定位收敛在 shim load 时武装环节」）。缺口 3 的 audio 形态与缺口 1
共用此面（bridge 武装无 video/audio 之分，registry 按容器路由 AudioEntry/VideoPlayer）。

### 6.2 Fix 1：autoplay 走 play 算法（缺口 1+3，单一修复面）

**spec 锚点**：ready-state update steps 的 potential autoplay 分支——eligible for autoplay 时
「run the internal play steps」。
https://html.spec.whatwg.org/multipage/media.html#ready-states
https://html.spec.whatwg.org/multipage/media.html#internal-play-steps

**现状**（part06.js:10131-10145）：`ms.playing = true` 直置 + 同步派 play/playing/timeupdate 三事件
——无 `__zwVideoBridge.play()`、无 `bridgeOn`、无重试、无 ended/loop/pending-seek 面。对比 play IDL
完整面（part03.js:23386+）：resource select → playing 翻转 → 无候选 pending → bridge feature-detect
→ latest-wins src 解析（handle 身份分派）→ `__zwVideoBridge.play` 恰一次 + 未命中 setTimeout 退避重试
（≤5000 次，`_pMs.playing && !bridgeOn` 门控）→ 命中后 bridgeOn + dims 升级 + pending seek 补推 +
rate/gain/loop 同步；ended 态 play = seek 最早位置（:23649+）。

**改造**：play 内核提取 `_zwMediaPlaySteps(sel, handle, key)`（返回 Promise；IDL `play` 方法与
`_zwMediaAutoplay` 共同调用）；autoplay 保持 load 序列末调用点不动，删除手写置态+三事件段。
autoplay 不向页面暴露 promise（与现一致）。

**行为差异与风险**：事件时序从「同步三事件」变为 play 算法派发面（play/playing 的 queued task 面、
promise resolve 面）——`event_play*`/`event_timeupdate*` 断言族为门禁；headless（无桥沙箱）面下
play 算法的 playing 翻转与 queued resolve 语义已由既有 372 基线覆盖（:23522-23526 注释），autoplay
切换后同面继承。无候选 autoplay 获得 pending promise 面（自动，spec 正确方向）；ended→loop 自动获得。

**验收**：WPT autoplay 族 + media event 面不回退（make test）；拟新增生产面语义测试——autoplay 元素
load 后 `bridgeOn === true` 且 registry 起播（`__zwVideoBridge.play` 可达、registry `is_any_playing`
真）；bg02-media 基线 v2 回补（audio/autoplay INCONCLUSIVE 子项复评条件，上游移交的重开条件）。

### 6.3 Fix 2：生产 renderer march 接线（缺口 2）

**现状**：媒体泵（runtime.rs:4094-4138，`any_playing` 门控 16ms 节拍）只推进 Rust registry 面
（tick_all/audio_advance_all/webaudio）；shim 侧 march（桥时钟拉取/headless 时钟/played ranges/
timeupdate 节流/cue active 面/ended 面）仅 runner 调用。生产播放中 IDL 读经 getter 镜像保持真值
（part06.js:16376），但 timeupdate/cue/ended **事件面**无驱动。

**改造**：any_playing 块内（audio_advance_all 之后）经 `submit_script_priority` 提交裸 march 脚本，
与 rAF 泵同通道同形态（runtime.rs:4152-4156 先例；runner 调用同形 testharness.rs:9288-9292）：

```rust
if self.javascript_enabled && any_playing {
    let _ = self.js_worker.submit_script_priority(
        "if(globalThis._zwMediaTimeMarchesOn)globalThis._zwMediaTimeMarchesOn();",
    );
}
```

**门控论证**：门控 = `any_playing`（沿用 :4100 已算布尔）+ `javascript_enabled`；**不加 page_hidden**
——spec time-marches-on 属播放推进，不随可见性暂停（后台音频继续播是现行为，rAF 的可见性门控
语义不适用）。无播放零提交；钩子内首行早退（`!ms.playing continue`，part06.js:16412）+ timeupdate
250ms 节流（:16460+）成本有界；bridgeOn 元素每 tick 一次 registry 锁 + cursor 读（M1 空 tick p50
431ns 同面）。

**否决备选**（记录防重提）：① shim 内 setTimeout 自泵——违「事件循环节拍，否决独立泵线程」既定
裁决（runtime.rs:4094 D4 注），后台 timer 节流面不可控；② 依赖 getter 镜像——仅页面主动读 IDL
时生效，事件面永不触发（bg02 泵推进测量已证不足）。

**风险**：march 派发的事件 handler mutations 经既有 drain_pending_script_mutations 通道下一拍
apply+渲染（rAF 回调同流，无新通道）；cue/ended 机器生产首启——track-cues 族断言在生产形态的
回归面 = bg02-media v2 + 拟新增生产面 cue/ended 语义测试（A2 验收行）。

### 6.4 A2 回滚

Fix 1、Fix 2 各自单提交（独立回滚点）；Fix 2 纯增量（一处 submit_script_priority 调用），feature
无关、默认生效面 = 生产 renderer（runner 行为不变）。

### 6.5 实施偏差记录（2026-10-24 A2 落地轮，相对 §6.2/§6.3）

1. `_zwMediaPlaySteps` 落 **part06 顶层**（§6.2 未指定落点——媒体加载序列/march 机器同域，高内聚；
   拼接单作用域下 part03 IDL play 直接转发）。元素身份留档（`_zwSel`/`_zwHandle`）随内核**内移**——
   autoplay 路径的 march/ended 钩子同需 key 反查派发面，不能只留在 IDL 包装。
2. 原 autoplay 的 `if (ms.playing) return` 手写守卫由内核 playing 检查承接（同语义，单守卫面）；
   内核 `ended = false` 置位为 autoplay 新获得面（spec 方向：自动播放清 ended）。
3. **`testharness-media` 门禁本环境未跑**：fetch-media-subset.sh 依赖外网（curl 30s 超时 ×N）；
   且该子集首批白名单本就不含 ready-states/autoplay/event_* 真解码族（脚本头注「后置到 headless
   语义层落地」）——媒体事件族断言面由 engine 桥测试族承载（既有基线 + 新增
   `test_media_autoplay_walks_play_steps_a2fix1`）。有网环境补跑 full 子集。

## 7. WPT 上游 GOP 抽样（A4 待验证项闭合，同日第二轮增补）

`tests/wpt-runner/wpt-data` 全量 4 个 webm（Python EBML 只读探针，/tmp 不入仓）：

| fixture | 时长 | 块数 | keyframe | kf 间隔 | cue 点 |
|---|---|---|---|---|---|
| css/css-contain/support/white.webm | 10s | 300 | 5 | 规则 2000ms | 5 |
| css/css-ui/support/test.webm | 6.5s | 475 | 291 | ~23ms（近全 intra） | 9 |
| media/movie_5.webm | 5s | 370 | 251 | ~20ms（近全 intra） | 1 |
| css/css-sizing/aspect-ratio/support/2x2-green.webm | 33ms | 1 | 1 | —（单帧） | 1 |

结论：A4 的「真实生产输入受控确认」项闭合——① 最坏常例（2s 规则 GOP，white.webm）在位，
O(GOP) 收益对该形态成立；② 主力媒体 fixture（movie_5）近全 keyframe，phase-① 直落为主路径，
phase-② 触发面天然小；③ **稀疏 cue 形态在位**（movie_5 仅 1 个 cue 点）——A4 设计的「无 ≤target
cue 点 → 保留全量前向回退」分支为必要路径（非防御性冗余），实施期须以 movie_5 形态覆盖该分支
测试。

## 8. A3 设计——C4 快照 Arc 化 + C2 扫描跳过 + 源驻留封顶（同日第三轮增补）

### 8.1 现状链路（源码级）

**C4 快照深拷贝**：`ImageCache::duplicate_for_snapshot`（image_cache.rs:1340-1354）逐条
`entry.data.clone()`——`ImageData.pixels: Vec<u8>` 深拷贝全缓存。唯一调用方 `snapshot_image_cache`
（webview.rs:1713）→ renderer 发布路径（runtime.rs:1443）每帧发布执行（视频播放期每帧变更触发
`try_republish_cached`，24fps ≈ 24 次/s）。快照仅在本进程内被 `fetch_image_payloads_with_cache`
（runtime.rs:1497）读取出 `IpcImagePayload`，跨进程只传 payload 字节——快照本体从不出进程。
tab_snapshot.rs:149 同面消费。

**关键不变量**（image_cache.rs:31-32 既有注释）：「像素在插入 ImageCache 后不可变」（R3254-M2，
content_hash 复用前提）——Arc 共享语义安全。同路径先例：hit-test 缓存发布已是 `Arc` +「无数据
clone，仅引用计数」（runtime.rs:1453-1455）；`svg_source: Arc<[u8]>`（image_cache.rs:58）。

**C2 纯色扫描**：视频帧插入 `ImageData::from_rgba`（video_registry.rs:1188/1204 → image_cache.rs:76）
——全帧逐像素纯色检测（视频帧恒 false）+ `hash_pixels` 全帧读（content_hash）。320×240 帧每次插入
2 次全帧读。

**源驻留**：`sources`/`av_sources: HashMap<u64, Vec<u8>>`（video_registry.rs:444-455）无总量封顶
（M1 §5 权限表缺口）。**功能性依赖**：av_sources 为 seek 重建 `WebmAudioTrack` 的字节源
（seek :945-959）——封顶不得驱逐在播源与存活 player 的伴音源。

### 8.2 Fix C4：快照 Arc 化

- `CacheEntry.data: ImageData` → `Arc<ImageData>`；`insert`/`insert_with_key` 包 `Arc::new`；
  `get` 返回 `Option<&ImageData>`（Arc deref，**签名不变**）；`duplicate_for_snapshot` 变 Arc
  引用计数循环（改名不做——API 面稳定，文档注释改写为「共享句柄快照（像素不可变，R3254-M2）」）。
- 快照内 LRU/字节数预算按句柄自计（快照为瞬态只读对象，发布后即弃；不跨对象核算——与现行为
  等价，无核算漂移）。
- 效果：每帧发布的全缓存深拷贝 → O(entries) 引用计数（µs 级）；跨进程 payload 序列化保持不变
  （帧传输字节是必要面，归 C 批次帧传输优化，不在 A3 范围）。
- 风险：无互斥需求（ImageData 构造后只读）；`Clone` derive 保留（外部 decode 路径仍按值移动）。

### 8.3 Fix C2：视频帧扫描跳过

- 新增 `ImageData::from_rgba_variable(pixels, w, h)`（跳过纯色检测，`solid_color: None`，
  content_hash 照算——GPU 纹理键/指纹依赖不破）；video_registry :1188/:1204 两处切换。
- 图片解码路径（PNG/JPEG/WebP/SVG）保持扫描（UI 图片纯色常见，fast-path 有价值）。
- 效果：视频每帧省 1 次全帧读（C2 项消除）；content_hash 全帧读保留——若 A3 后 profiling 显示
  仍显著，后续切片可评估 pts 键控哈希（本设计不扩scope）。

### 8.4 Fix 源驻留封顶

- 预算：registry 内 `sources + av_sources` 总字节常量上限（**256MB**，对齐 MSE_FACE
  `MSE_FEED_MAX_BYTES` 既有量级）。
- 策略（register/settle 路径准入时）：超限 → 先驱逐**非在播** LRU 源（`sources` 优先于
  `av_sources`，后者有 seek 重建依赖、随 player 存活保留）；仍超 → 拒绝登记，走既有 play 失败/
  不可解码错误面（不新增语义，与 MSE QuotaExceeded 形态对齐）。
- 永不驱逐：任一在播 player/audio entry 的键；存活 player 的 av_sources 键。
- release/clear 导航清理面不变；驱逐记 tracing。

### 8.5 验收与回滚

- `make bench-gate` 定向（`ZERO_WEB_BENCH_CRATES=zero-render-foundation` 或 media/render 域）+
  M1 harness 复测：纯视频 60s 墙钟预期下降（C2 省 1 读 + C4 省全缓存 clone），数字回写本目录。
- 既有 image_cache 契约测试（ref_count/gc/世代）全绿；快照 Arc 化后 `duplicate_for_snapshot`
  单测补 Arc 共享断言（同像素指针、源缓存驱逐不影响快照读取）。
- 回滚：三个子项各自单提交（C4 / C2 / 封顶互相独立）。

### 8.6 实施偏差记录与验收结果（A3 落地后）

三单提交：C4 `2003c55dc`（快照 Arc 化）/ C2 `aa540dc4d`（扫描跳过）/ 封顶 `340fcde8b`。

偏差（C2/C4 无语义偏差，按设计落地；封顶四处）：

1. **驱逐序 LRU → HashMap 迭代序任一**：设计写"非在播 LRU"，实现无时间戳账本，按 HashMap
   迭代序驱逐任一非在播键——svg_rasterized FIFO 同先例的缓存语义（预算目的只要求逐出足够
   字节，任一非在播键等价）；trace 逐键记录。
2. **play 内 sources→av_sources 搬移不加检**：设计"仍超拒绝登记，走既有 play 失败/不可解码
   错误面"仅适用 register/settle 准入路径；play 是 `sources.remove` → `av_sources.insert`
   字节搬移（总量守恒），加检反而可误拒已准入源的伴音 seek 重建。
3. **register_audio_source 准入失败 = plain return**（解码器已建即弃）：设计草案曾提"remove
   audio_entries key"，实际不采用——同键重登记被拒时保旧 entry/源对一致，remove 会留下
   sources 有、entry 无的分裂态。
4. **驱逐逻辑 `_with_budget` 预算参数化**（256MB 常量单测不可达）：语义不变，仅测试面增补。

验收（§8.5 逐项）：

- M1 harness 复测（release + test-guard，[m1baseline-a3-post.log](m1baseline-a3-post.log)）：
  5/5 PASS；T3 rate 1.00000 持平；T5 漂移 p50/p95/max 11.0/21.0/35.0 逐位持平；T4 seek
  279716/1042/850756us 与 A1 同量级；容量场景 A3 带 av-pair 1211–1279ms、pure-video
  1226–1316ms（含两次复跑），对 A1 单值 1206/1191 在单次运行噪声带（±7%）内，tick/present
  计数逐位一致、pure-video tick p50 411→401us（C2 省读方向一致）。**§8.5"纯视频墙钟预期
  下降"未在墙钟显现**——C2 单帧省读（约 300KB 一次线性扫）低于测量噪声；C4 收益在页面隐藏
  快照路径，harness 无快照触发，不进此数。
- `make bench-gate` 定向（zero-render-foundation + zero-webview）：2/2 microbench PASS，
  绝对预算过（硬件基线 WARN 为既有常态）。
- image_cache 契约测试 116 全绿（含新增 Arc 共享断言）；webview lib 790 全绿（含封顶 4 单测）。

## 9. C1 实施级设计——媒体消息族 wire format + media-process 骨架（同日第四轮增补）

### 9.1 协议面现状核对（拟新增命名冲突排查）

- `ProcessRole`（protocol/channel.rs:20-31）现有 Browser/Renderer/Network/ImageDecoder/Compositor
  五变体，无 Media——拟新增 `ProcessRole::Media`；`child_process_args`（process.rs:133-142）role→type
  名映射拟增 `Media => "media"`（产出 `--type=media --instance-id=N`；先例 :138 `"image-decoder"`）。
- `IpcMessageKind`（message.rs:119-133 一带）grep 证实无任何 `Media*` 变体（最近邻 ImageDecodeRequest/
  Result :119/:121、CompositorFrame :125）——IF-002~006 全族 14 个拟新增变体命名零冲突。
- 载荷先例：`ImageDecodeParams`（message.rs:860+）`request_id` 回带匹配 = `MediaSeek{request_id}` →
  `MediaSeekResult{request_id}` 同构；请求/响应配对走既有 `IpcMessage{id, kind}` 信封（IF-002 已定）。
- 帧传输先例：frame_shm.rs:11 `SHM_PREFIX = "zeroweb-cmp-"`、:118 段名
  `{surface_id}-{frame_id}-{nonce}`、:14 `env_linux_default_on` 环境关断——媒体侧镜像为
  `zeroweb-media-` 前缀 + `ZW_MEDIA_SHM=0` 关断。

### 9.2 消息族 wire format 草案（14 拟新增变体）

renderer → host（控制面，IF-002）：

| 拟新增变体 | 载荷 | 响应 |
|---|---|---|
| `MediaRegisterSource` | source_key: u64、total_len: u64、mime: String | `MediaSourceReady` / `MediaError{QuotaExceeded}` |
| `MediaDataChunk` | source_key、seq: u64、data: Vec\<u8\> | `MediaDataAck{last_seq}`（累积回执） |
| `MediaPlay` | source_key、position_ms: u64 | `MediaStateUpdate` |
| `MediaPause` | source_key | `MediaStateUpdate` |
| `MediaSeek` | source_key、target_ms: u64、request_id: u64 | `MediaSeekResult{request_id, resolved_ms}` |
| `MediaReleaseSource` | source_key | —（fire-and-forget） |
| `MediaShutdown` | — | —（host 对 stdin EOF 亦自行退出） |

host → renderer（握手/状态/帧/错误面，IF-001/003/004/006）：

| 拟新增变体 | 载荷 |
|---|---|
| `MediaReady` | protocol_version: u32、capabilities: Vec\<String\>（codec 面真值，与 canPlayType 同源） |
| `MediaSourceReady` | source_key |
| `MediaDataAck` | source_key、last_seq |
| `MediaStateUpdate` | source_key、epoch: u64、state: MediaPlaybackState |
| `MediaSeekResult` | request_id、resolved_ms: u64 |
| `MediaFrame` | source_key、epoch、pts_ms、key: bool、像素（shm 段名或内联 rgba） |
| `MediaError` | source_key、epoch、code: MediaErrorCode、message |

伴生拟新增类型：`MediaPlaybackState`（`Playing{media_time_ms}` / `Paused{media_time_ms}` /
`Ended` / `Error{code}`）、`MediaErrorCode`（DecodeFailure / SourceCorrupt / QuotaExceeded /
FrameTooLarge / HostInternal）。全部走既有 `IpcMessage` 信封 + bincode over PipeTransport；单消息受
`MAX_FRAME_SIZE` 64MiB 约束——`MediaDataChunk` 上限内分片，`MediaFrame` 超限 → `FrameTooLarge`
（4K RGBA ≈33MB 在界内，RFC §5.1 已核）。

### 9.3 media-process 骨架

- 新 app `apps/media-host`（bin `zero-media-host`）：`run_role<C: IpcChannel>` 循环照搬 image-decoder
  先例（读信封→分发→回写；对端断开/stdin EOF 退出）；连接后首发 `MediaReady` 握手，renderer 侧
  版本/能力不符按 spawn 失败处理（IF-001）。
- `ProcessRole::Media` 接线：child_process_args 映射 + webview 侧 media_proxy 懒 spawn（首个媒体源
  play 时；image_decoder.rs:211 先例）+ spawn 失败 → SPAWN_FAILED 永久进程内（:178 注记先例）；
  握手等待超时（实施定值，5s 量级）同落此面。
- host 内核：C1 只立骨架 + 握手 + 空源生命周期（register/release/错误面，保契约可测）；zero-media
  player/audio entry 移植（携 A1 修复后实现）归 C2 接入。
- feature `media-process` 默认关（= D-RFC-1 未批不进）：关时 registry 直调 zero-media（现状路径零
  变化），proxy 不参与。
- Makefile：media-host 挂入与 image-decoder 相同的构建接线位（Makefile :120/:201/:220/:1641 先例位）。

### 9.4 契约测试计划（C1 验收面）

- protocol 单测：14 变体 bincode 往返 + 64MiB 超限拒绝 + 两个伴生枚举覆盖（image-decoder 消息测试
  同形）；process 单测：`--type=media` 映射断言。
- 集成测试（`#[ignore]` 显式运行 + test-guard 包裹，M1 先例）：host 两端契约（回声/错误注入）、
  spawn 失败永久内联回落、`ZW_MEDIA_SHM=0` 内联帧路径、feature 关 = 现状直调（回归守护）。
- 回滚：C1 全部处于默认关 feature 之后，单提交回退无行为影响。

### 9.5 边界（C1 不做）

- MSE 字节流 IPC 化不进 C1（decode-h264 门控，依赖 D-RFC-4）；重启/退避策略归 C3；host 自走时钟
  节拍与 100ms 推送校准归 C2；不新增第三种传输（复用 PipeTransport + frame_shm）。

## 10. C2 设计——状态时钟面接线：host 自走时钟 + push 供数 renderer 镜像（同日第五轮增补）

### 10.1 现状解剖（源码级）

- **拉取链**：泵 tick 提交 march 脚本（生产面缺口 = A2 Fix 2；runner 面 testharness.rs:9290）→
  march（part06.js:16407）→ bridgeOn 元素**主动拉** `__zwVideoBridge.currentTime`（:16415-16421，
  泵 tick 无人读 IDL 须主动同步）；headless 元素墙钟推进 × playbackRate（:16424-16437）。
- **页面可见节拍**：周期 timeupdate 250ms 节流（:16463 `_zwLastTuMs`，spec 15-250ms 对齐上限
  :16459-16462 注）；played ranges 250ms 采样容差合并（:16445-16451）。
- **ended/loop 面**：桥 `isEnded()` → active cue exit + paused 翻转 + timeupdate + ended
  （:16620+，`_zwEndedDispatched` 幂等）；loop 分叉 = 音频 entry registry 内部流末回卷 + 视频
  march seek(0)+play（:16640+，M3 扩批 XXIV）。
- **registry tick 面**：`tick_all(now_ms, &mut ImageCache)`（video_registry.rs:1148，视频帧解码
  直入缓存）+ `audio_advance_all(now_ms)`（:1091）；生产泵 runtime.rs:4094-4138 主动轮询驱动。
- **桥面全貌**（video_registry.rs:1700-1716）：play / pause / seek / currentTime / duration /
  isPlaying / isEnded / **setRate**(:1711) / **setGain**(:1712) / **setLoop**(:1713) / mediaMeta
  共 11 方法 + MSE 六方法（decode-h264 门控）。

### 10.2 wire format 缺口闭合（C1 §9.2 修订）

- **缺口**：setRate / setGain / setLoop 三桥方法在 C1 草案控制面（§9.2）无对应消息——而桥同步面
  在起播时同步既有设置（part03.js:23633 setRate / :23639 setGain / :23647 setLoop），跨进程即断。
- 拟新增 3 变体（控制面 7→10，全族 14→17）：
  - `MediaSetRate{source_key, rate: f64}`——宿主音频消费与时钟缩放是物理面，不能留在 renderer
    镜像（headless 墙钟 ×rate :16433 与 BatchSampleQueue rate 同语义）；
  - `MediaSetLoop{source_key, on: bool}`——音频 entry 流末回卷随内核入宿主（registry `set_loop`
    :1132 面迁移）；
  - `MediaSetGain{source_key, volume: f64, muted: bool}`——宿主持音频输出面时必须；NullSink
    现状下为记账字段。
- `MediaSourceReady` 载荷扩为 `{source_key, duration_ms: Option<u64>, width: Option<u32>,
  height: Option<u32>}`——mediaMeta 面一次带回（demux settle 后已知）。

### 10.3 host 自走时钟（IF-003 校准闭合）

- **真值**：宿主持媒体时钟（audio cursor 主时钟，携 A1 `BatchSampleQueue` 修复语义——同构代码
  随内核入宿主）；事件循环 = 非阻塞排空控制消息 + 按 (音频采样截止 | 视频帧演示截止) 等待，
  无 renderer 往返（省 60Hz 轮询）。
- **推送时机**（RFC「≥100ms 推一次」的实施定值）：
  1. 状态跃迁即时推（Playing/Paused/Ended/Error）；
  2. `media_time_ms` 随帧推——`MediaFrame.pts_ms` 自带时钟信息，视频源零额外消息；
  3. 纯音频或帧间隔 >250ms 时 250ms 心跳推——与 shim timeupdate 节流同值，页面可见节拍零变化
    （renderer 侧 250ms 节流保持，见 10.4）。
- **epoch**：源级 u64 单调；seek / replay（含 loop 回卷）/ 换源递增；renderer 收旧 epoch 的状态
  或帧一律丢弃（DC-2「不返回旧代际数据」）。

### 10.4 renderer 侧变换（march 不重写，供数方式变）

- **march 脚本（part06.js:16407）页面可见语义原样保留**：250ms timeupdate 节流、played ranges、
  ended 派发、loop 分叉全是 DOM 面，宿主无 DOM 不接管——回归风险集中到供数时序而非事件语义。
- 变化仅在供数：bridge getter（currentTime/isPlaying/isEnded，:1708-1710）的 backing 从「泵
  `tick_all` 推进的 registry cursor」变为「proxy 按 push 更新的镜像 cursor/状态」——getter 本体
  零改动。
- **生产泵（runtime.rs:4094-4138）媒体分支转被动**：排空 proxy 待决帧队列 → ImageCache 注入
  （复用 `from_rgba` 路径，A3 C2 扫描跳过同面适用）→ 标脏；march 提交挂 interest 机制
  （rAF 泵 :4152-4156 同通道先例）——StateUpdate/Frame 到达即标 interest，无消息无提交。
- WebAudio 泵分支不动（script 驱动、renderer 域，与解码底座边界既有）。

### 10.5 验收

- DC-2 契约测试：旧 epoch 丢弃断言、seek 后镜像一致性、pause 冻结（宿主停推）、loop 回卷
  epoch 递增；A/V 漂移真值复测（M1 harness，宿主时钟 vs 页面镜像）。
- 页面语义回归：M1 harness + bg02-media——timeupdate 节拍 / played ranges / ended / loop 断言面
  全绿（march 未重写，面不变）。
- currentTime 新鲜度：pull-per-tick（16ms）→ push 节拍（~40ms@24fps / 250ms 心跳）——页面无
  新鲜度契约，WPT 断言面（currentTime>0 类）不收窄。

### 10.6 回滚

- C2 单提交（`media-process` gate 内）；回退即回 `tick_all` 主动轮询面。wire 向前兼容由 C1 握手
  版本守门（renderer/host 版本不符按 spawn 失败处理，IF-001）——无灰度态。

## 11. C3 设计——崩溃检测/重启退避 + 降级纪律先例订正（同日第六轮增补）

### 11.1 先例核实：image-decoder spawn 失败语义订正（RFC v1.7 触发点）

- `crates/webview/src/image_decoder.rs:177-179` 注释称「spawn 失败后不再重试，后续请求直接进程内
  解码」；**代码为准**：SPAWN_FAILED 置位后 `decode_image` 直接返回 Err（:205-206），三调用方
  （async_load.rs:1125-1160、:1213-1245 落 `ResourceElementOutcome::Error` / `FailedResource`；
  webview.rs:1509 落 warn+continue 跳过）——**无任何层内联回落**。:266 的 store(false) 仅测试
  helper，生产生命周期内一次性。
- 影响：RFC v1.6 IF-001/IF-006 与本文 §9 C1 骨架「spawn 失败 → 永久进程内直调（现状行为，等价
  feature 关）」引用失实——image-decoder 现行为 = 永久**错误面**（从严：失败显性、不静默）。
- C 批次纪律随之简化：隔离开启后 spawn 失败与运行期失败**同归错误面**（纪律 1/2 合一）——spawn
  失败 → `MediaError{HostInternal}` + SPAWN_FAILED 短路（本 webview 生命周期不重试、不内联）；
  feature 默认关 = 内联直调（正常路径不受影响）；「隔离静默失效」状态不可能发生。失实注释订正
  归 C1 批次顺带（仅注释，无行为变化）。

### 11.2 崩溃检测（双路）

- 先例（protocol/process.rs:441-459）：`exit_reason` = `try_wait` + bounded stderr tail 诊断；
  `is_alive` 注释明示「崩溃管理器应使用 exit_reason」；check_crashes 崩溃管理器模式（:623-643，
  置 Crashed + 移除 + kill 收尸，返回崩溃清单）。
- media proxy 镜像双路：(a) **send 失败即时检测**——控制消息写入遇 broken pipe 即报；
  (b) 泵周期 `try_wait` 兜底——覆盖在途消息/无写入期，syscall 级成本随 16ms tick 可忽略。
- proxy 所有权：**per-webview**（registry 持有，随 webview 释放）——与 IF-001「每个 renderer
  至多一个 host」一致；image-decoder 的全局 static PROXY（image_decoder.rs:176）模式不采（媒体源
  是长生命周期状态，须随 webview 导航/释放清理）。

### 11.3 错误传播与状态机

- 检测 → 对全部活跃源发 `MediaStateUpdate{Error{HostInternal}}`（IF-006 既定）→ registry 镜像
  置 error → bridge getter 面 → march Error face → 页面 error 事件（DC-3 第一段：页面可观察）。
- 状态机：`Idle`（未 spawn）→ `Healthy` → 崩溃 → `Restarting`（退避重试 n≤3：1s/2s/4s，握手
  成功重置计数）→ `Healthy` | `Failed`（耗尽 → 本 webview 生命周期永久错误面，**不内联回落**，
  后续 play 立即 `MediaError`）；正常退出 = `MediaShutdown` → wait（renderer 退出路径）。
- C2 阶段（未接 C3）：崩溃仅报错不重启（IF-006 既定），状态机实现随 C3 批次。

### 11.4 重启语义（DC-3 第二段）

- 重启 = 全新 host 进程，全部源丢失；**不自动重播、不自动重注册**——错误事件已通知页面，页面
  `play()` 时按需重走 MediaRegisterSource/DataChunk（epoch 从 0 重新单调，旧代际数据天然失效）。
- 导航恢复：reload 走现有导航释放面（MediaReleaseSource 全部 + proxy teardown + 状态机回 Idle）
  → 全新 host——DC-3「重载恢复」验证路径。
- 崩溃诊断：stderr tail 截断入 tracing；exit status 入 `MediaError.message` 面。

### 11.5 契约测试（DC-3 验收）

真实 SIGKILL 注入（`#[ignore]` + test-guard，M1 先例）：

1. kill host → 页面 error 事件可见（march Error face 断言）；
2. renderer 进程存活（隔离目的成立，DC-3 核心断言）；
3. kill 后 `play()` → 重启路径（新 pid，重喂字节可播）；
4. 连续 3 次 kill → Failed 永久错误面 + 断言无内联回落（与 feature 关配对对照：关闭时同场景
   直调可播，开启时永久错误——两态差异即隔离边界证明）；
5. reload → 全新 host，重喂恢复。

### 11.6 回滚

- C3 单提交（gate 内）；状态机与退避常量单点，回退即回 C2「仅报错」形态。

## 12. C4 设计——M4 配对验收 + 资源审计实施级方案（同日第七轮增补）

### 12.1 配对矩阵（三态 × 固定输入 × 同 commit）

| 态 | feature `media-process` | 帧传输 | 语义 |
|---|---|---|---|
| 内联基线 | 关（现状） | 进程内直调 | M1 复测基线（A1–A3 落地后） |
| shm 开 | 开 | `zeroweb-media-*` 共享内存 | 生产目标形态 |
| shm 关 | 开 + `ZW_MEDIA_SHM=0` | IPC 内联 Vec<rgba>（64MiB 上限内） | 非 Linux 形态代理 + FrameTooLarge 路径 |

固定输入：M1 两枚 60s fixture（母本簇 ×30）；同一 commit、同一机器、同一 feature 组合声明
（DC-1「版本与来源」：报告头必含 commit / binary / 平台 / feature 开关矩阵，缺一不收口）。
非 Linux 平台实测缺口维持 TBD-4，本机以 shm 关态代理其帧传输面。

### 12.2 测量仪器复用（场景体不动，入口按态切换）

- M1 harness（`crates/webview/tests/media_m1_baseline.rs` 五场景：pure-video-60s / av-pair-60s /
  clock-fidelity-rt-15s / seek-cue / av-drift-media）场景体与 `M1BASELINE key=value` 输出行
  **原样复用**；唯一变化 = 构造入口——基线态直调 registry，开态经 proxy 面（C2 被动消费 +
  push 镜像已就位后，泵行为对场景透明）。新增配对 runner 同为 `#[ignore]` + test-guard（M1 先例）。
- 帧一致性：三态各跑同 fixture，逐帧 RGBA 哈希序列比对（§8「帧传输双路径」行的集成化）；
  以 C1 转换输出为参照点（解码内核同源，差异只允许来自传输面）。

### 12.3 判据表（拟新增值随 C4 定稿可调，语义带绑定既有契约）

| 维度 | 判据 | 来源契约 |
|---|---|---|
| 时钟保真 | 开态 rate 收 [0.9,1.1] | A1 验收带同值（host 侧 BatchSampleQueue 同构语义，§10.2） |
| seek 语义 | 三态 `resolved_ms` 相等；热路径不劣于内联 +10ms | IF-002 cue 契约（M1 热 1.1ms 基线） |
| A/V 漂移 | 开态 p50 不劣于内联基线 +5ms | M1 p50=11ms 基线（media 层） |
| 像素一致 | 三态逐帧哈希序列全等 | §8 双路径行 |
| 容量开销 | 开态墙钟 ≤ 内联 ×1.5（IPC+跨进程调度税上界） | M1 45× 产能富余下不改变可用性结论 |
| 代际 | 旧 epoch 帧/状态零泄漏（seek/重播风暴下断言） | DC-2 / IF-003 |

### 12.4 资源审计清单（开态生产形态跑 30min 长播）

- 进程树：每 renderer 恰好 1 个 media host（IF-001）；无孤儿（kill 收尸后 `ps` 核对）。
- 内存：host RSS 峰值/稳态；shm 段数量与总大小（槽位有界，IF-004）；renderer 侧源驻留
  ≤ 256MB（A3 预算复测，M1 §5 无界缺口闭环证据）。
- 空闲成本：暂停态 host CPU ≈ 0（自走时钟事件循环等待面，无 60Hz 轮询——§10 设计目标验证）。
- 复制链复测：A3 后内联基线重测 C1–C5 残留 + 开态叠加传输面成本（shm 零拷 / IPC 内联 +1 次
  全帧拷），对照 M1 §4 静态图出增量表。

### 12.5 能力真值三对照（DC-4 收口面）

`MediaReady.capabilities`（host 实报）vs `canPlayType`（页面可见）vs 实解（fixture 逐 codec
喂入）三者一致；quota 面：超 per-source 上限 → `MediaError{QuotaExceeded}`（IF-002）对齐
MSE 封顶语义。平台无硬解如实报软解（本机 /dev/dri 缺失即天然用例，M1 §6）。

### 12.6 DC 映射与产物

- C4 产出 M4 evidence（`ANALYSIS.md` + raw log）：12.3 判据表逐行 + 12.4 审计清单逐项 +
  §11.5 注入五场景结果（DC-3 直接引用 C3 测试，不重造）——DC-1/DC-4 收口证据、DC-2/DC-3
  终验面。全部 DC 有证后按 goal 收口条件走 DC-5 审计。
- 通过判据不满 → 按域回 C1–C3 对应片修复后重跑配对；不改判据带凑数（perf-gate 纪律同源）。

### 12.7 回滚

- C4 为验收批次，无产品代码改动（仅测试/工具/文档）；判据脚本单提交，回退不影响 C1–C3 形态。
