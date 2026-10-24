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
