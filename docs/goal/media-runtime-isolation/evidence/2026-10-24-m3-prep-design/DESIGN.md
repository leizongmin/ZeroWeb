# M3 前置设计——A1 时钟批内对齐 + seek 冷路径归因与 A4 窄化

**日期**：2026-10-24
**状态**：设计就绪（门禁 = D-RFC-2 随批；本文为设计准备，**零代码改动**）
**goal**：[media-runtime-isolation](../../master.md)
**RFC**：[media-runtime-isolation-spec-rfc](../../../../specs/media-runtime-isolation-spec-rfc.md) v1.2（A4 批次行为本轮增补）

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
