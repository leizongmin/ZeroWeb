# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**M4g-d canPlayType 能力表落地**（media-playback M0 选型
联动——跨 goal 依赖兑现）：webm/ogg 容器 'maybe' + vp9/vorbis/mp3 'probably'，
域外 codec 诚实 ''；**93.5%**（386P/0F/0T/27PF，90.1%→93.5%，+14P）；
Fail/Timeout 维持双清零）

---

## 当前状态

**专项定位**：媒体方向三拆之一（可立即启动）。HTMLMediaElement 非解码语义面（状态机/
事件序列/API 行为），WPT media-elements 真实用例驱动。**不被解码选型 RFC 阻塞**——
headless 近似驱动先行，兄弟目标建成后替换驱动源。

**M1 已完成（2026-08-31）**：`make testharness-media` 基础设施落地（fetch-media-subset.sh
+ runner `testharness-media` 子命令 + Makefile 目标）；首批 30 用例 / 245 subtest 基线
**46.5%**（114P/77F/13T/41PF）；失败聚类 F1~F6 成文（见 evidence/2026-08-31-media-baseline.md）。

**M1 切片 3 已落地（同日，F2/F6）**：媒体元数据 IDL 面（currentTime/duration NaN/
playbackRate/volume clamp/seeking/paused/preload/crossOrigin 枚举反射 + has-trap 白名单）
+ HTMLTrackElement 反射（kind/label/srclang/default/src + setter + R122 实例层同步）
+ `<a href="">` 空串解析修复。**73.1%**（179P/12F/13T/41PF，+26.6pp）；余账 12 Fail
全部为 TextTrack 家族（M3 域）。单测 `test_media_metadata_idl_face_r388`；make test
65 套件全绿、fmt/clippy 干净。

**M2 已落地（同日，F4 事件序列）**：`_zwSettleResourceKey` handle/sel 双身份泛化 + headless
加载序列（loadstart→progress→durationchange+loadedmetadata→loadeddata→canplay→
canplaythrough，networkState LOADING→IDLE）+ 动态 `.src=` setTimeout 模拟（精确空串仅
loadstart）+ play/pause 事件面 + seeking/volumechange/ratechange + on* 兜底派发。
**79.6%**（214P/12F/2T/41PF）。单测 `test_media_load_event_sequence_r389`；FR-009 集成
测试契约更新（error 路径断言零改动通过 = A/B 佐证）。

**M3 已落地（同日，TextTrack 家族）**：TextTrack/TextTrackCueList/TextTrackList/TextTrackCue
接口声明（Illegal constructor + 工厂原型链）+ `addTextTrack`（枚举精确校验 + WebIDL
DOMString 转换）+ `textTracks`（same-object + 增量同步）+ `track.track`（身份缓存 +
default→showing）。**84.0%**（226P/0F/2T/41PF）——**Fail 清零**。单测
`test_text_track_family_r390`。余账 2 Timeout 均深域：error 错误码语义（随资源选择/解码层）、
currentSrc 的 source-child 插入触发（mutation 面）。

**M3 扩批已落地（2026-09-01，event_* 族）**：下游计划 #1 兑现——25 个 event_* 用例
（autoplay + noautoplay + order_* 五件）接入 MEDIA_TEST_FILES / fetch-media-subset.sh。
支撑语义补齐（part03.js / part06.js）：
- `play()` 返 **pending** Promise（spec：resolve 于播放真正推进）——setTimeout(0) 后仍
  playing 则 resolve；`pause()` 先行 → reject **AbortError**（spec「pause() rejects all
  pending play Promises」，event_play_noautoplay 断言面）；already playing → resolved。
- `_zwMediaLoadSequence` 增 tag 参数：video 派 **resize**（durationchange 后、
  loadedmetadata 前，event_order_durationchange_resize_loadedmetadata 断言面；audio 无）。
- autoplay 与 play() 路径均补派 **timeupdate**（event_timeupdate* 断言面）。
单测 `test_audio_constructor_and_media_methods_r2835` 扩展 pending→pause→reject 链 +
already-playing resolved 断言。**88.3%**（324P/0F/2T/41PF，+80 subtest 全绿）。evidence：
`evidence/2026-09-01-media-event-family.json`。make test 65 套件 18555 全绿、fmt/clippy 干净。

**M3 扩批第二批已落地（2026-09-01，余账 2T 清零）**：
- **source-child 资源选择触发**（appendChild 钩子，part04.js）：source 子插入 audio/video
  父（handle 父、无自身 src、资源未 settle）→ `_zwMediaScheduleLoad` 以 source 的 src
  调度加载序列——父元素 loadstart + currentSrc 真值化（WPT currentSrc「adding source
  element」族 8 subtest）。
- **空 src 错误码语义**（`_zwSettleResourceKey` 增 errorCode 参数）：空 src 资源选择失败
  → error 事件 + `error.code = MEDIA_ERR_SRC_NOT_SUPPORTED(4)` + MediaError 实例 +
  currentSrc 恒空（WPT error-codes「empty string」族 4 subtest）。
- **source.src URL 绝对化**（get trap 增 SOURCE 分支）：URL 属性反射 + base 解析 +
  C0/space 剥离（同 track.src / a.href 模式；WPT currentSrc 断言 `e.currentSrc === s.src` 面）。
单测 `test_media_source_child_and_error_code_r391`。**89.1%**（334P/0F/**0T**/41PF，
+10 subtest 全绿，Timeout 清零）。evidence：`evidence/2026-09-01-media-source-child.json`。

**M3 扩批 III 已落地（2026-09-01，volume/muted + Audio 构造器）**：
- **volume IDL setter spec 语义**（part05）：非有限数值 → TypeError（spec dom-media-volume
  步 2，volume_nonfinite 断言面）；同值短路不派事件；volumechange 改 **queued** 派发
  （media element task source——同 turn 内后注册 handler 也收到，event_volumechange
  「repeatedly fires」断言面）；`_mediaState[key].pendingVc` 标记 + **load() 清除**
  （spec dom-media-load「pending events 丢弃」——「before load will not fire」断言面）。
- **volume getter 缺省守卫**（part04）：`_mediaState` entry 可被 muted setter 先建
  （volume 字段未写）→ `!= null` 守卫防 undefined 漏出（1-volume=NaN→TypeError）。
- **muted IDL setter + getter**（part04/part05）：setter 增分支（此前落 expando 吞）——
  现值读法与 getter 同源（dirty 镜像优先/attr presence 回落，attr 已设时 `e.muted=true`
  值未变不派）；值变 → attr 反射同步 + queued volumechange；has-trap 白名单补 'muted'。
- **Audio 构造器 spec 面**（part02/part01b/part03）：设 preload='auto'（spec dom-audio）+
  无 new 调用抛 TypeError（WebIDL constructor 语义）；HTMLAudioElement 接口构造器
  Illegal constructor + prototype 链补接（audio_constructor 断言面 11 subtest 全绿）。
- R2835 旧断言「Audio() 无 new 亦返 proxy」与 spec 冲突——随用例导入一并修正。
- 导入 volume_nonfinite.html / event_volumechange.html / the-audio-element/
  audio_constructor.html（fetch 脚本 + MEDIA_TEST_FILES 同步）。
- 单测 `test_media_volume_muted_semantics_r392`（6 断言组）。**90.0%**
  （369P/0F/0T/41PF，+35 新通过 0 回归）。evidence：
  `evidence/2026-09-01-media-volume-audio.json`。

**M3 扩批 IV 已落地（2026-09-01，controlsList + the-video-element 起步）**：
- **controlsList IDL**（part03/part04/part05）：`_classListProxy` 增 supportedTokens
  参数——`supports(token)` 精确匹配四值表（nodownload/nofullscreen/noplaybackrate/
  noremoteplayback，大小写敏感；未传表恒 false，spec 其它 DOMTokenList 无 supported
  tokens 定义）；get trap 增 controlsList 分支（gate：HTML ns 的 audio/video，R374
  gate-miss → undefined 同款）；has-trap 白名单补列。
- 导入 controlsList.tentative.html（2 subtest）+ the-video-element/video-tabindex.html
  （1 subtest，UA 面不凭空加 tabindex）。
- 单测 `test_media_controls_list_r393`（6 断言组）。**90.1%**（372P/0F/0T/41PF，
  +3 新通过 0 回归）。evidence：`evidence/2026-09-01-media-controlslist.json`。
- **决策记录**：audio_volume_check/video_volume_check **不导入**——越界值期望
  IndexSizeError 是 Intel 旧 spec 时代断言，与现行 spec（clamp 不抛）及已导入的
  volume_nonfinite.html 冲突，导入即制造假失败面。

**M3 扩批 V 已落地（2026-09-01，playbackRate 非有限 TypeError）**：
- `playbackRate`/`defaultPlaybackRate` IDL setter：非有限数值 → TypeError
  （spec dom-media-playbackrate / dom-media-defaultplaybackrate 步 2；旧静默回落 1
  与 volume TypeError 修复同款缺口）。合法值照常设置 + ratechange 派发不变。
- 上游 playbackRate.html 断言面不含非有限值（7P 全绿零回归）——纯 spec 对齐修复，
  无新用例可导入（WPT 无对应断言用例）。单测
  `test_media_playback_rate_non_finite_r394`（3 断言组）。evidence：
  `evidence/2026-09-01-media-playbackrate-typeerror.json`（90.1% 维持）。

**M3 扩批 VI 已落地（2026-09-01，preload setter 补缺 + sweep 巡检收口）**：
- `preload` IDL setter：enumerated 反射（写 preload 内容属性原样值——invalid 原样写、
  getter 归一 'metadata' 分离面；DOMString 非 nullable，null→'null' 串）。旧无 setter
  分支 → 落 expando 吞、attr 不写 → set→get round-trip 断。
- **全 IDL setter sweep 巡检**（探针实证）：controls/loop/autoplay/playsInline/
  crossOrigin/defaultMuted/muted/currentTime/volume/track.kind/label/srclang/src/
  media.src 全部 round-trip 正确——语义面 setter 缺口清零，巡检收口。
- 单测 `test_media_preload_setter_roundtrip_r395`（2 断言组）。evidence：
  `evidence/2026-09-01-media-preload-setter.json`（90.1% 维持 0 回归）。

**里程碑归档（2026-09-01）**：M1~M3 与六轮扩批的过程记录、排除用例决策清单已归档至
[archive/2026-08-31_m1-m3-and-2026-09-01_batches.md](archive/2026-08-31_m1-m3-and-2026-09-01_batches.md)
（只追加不修改；本控制面保留最新态与缺口清单）。

**DC 达成审计（2026-09-01）**：DC-1~4 实质满足——① 60 用例导入 + 8 份 evidence JSON
（基线演进 46.5→90.1 全程可追溯）；② 状态机/事件序列 WPT 断言面全绿（headless 近似
驱动逐项记录）；③ API 语义面全对齐（canPlayType 空表 + M4g-d 显式记录为跨 goal 依赖项；
play-pause 异常/元数据 setter/track 面全绿）；④ make test 65 套件全绿 + clippy 零警告 +
每修复带单测与用例资产化。
**治理注记**：入口文档 DC-1 第三条「经 `make import-wpt` 记入 imported-tests.txt」字面
指 reftest 通道；media 用例的资产化通道实际为 testharness-media pin（fetch 白名单 +
MEDIA_TEST_FILES + evidence JSON 序列）——两通道并行为 CLAUDE.md 测试资产化规则的
等价实现，差异在此记录，不改入口文档。**完成度**：语义面在当前 headless 驱动形态下
已达饱和（可导入面吃尽）；进一步提升依赖兄弟目标解锁（media-playback 解码层真值化
→ videoWidth/buffered/真 seek 面重评导入）。

**与兄弟 goal 的边界**：
- media-playback — 解码/帧渲染归其管（RFC 门控）；本目标的 readyState 真实驱动源由其
  供给（接口契约记录于两流 master.md）
- media-audio — 音频输出归其管；volume/muted 本目标只做语义（真增益归其接线）
- js-dom — 媒体反射段（part01.js R3040）共享；该流已归档（2026-08-31 核对），媒体段
  近 14 天无并发编辑，可安全作业

## 实测基线（2026-08-31 首批跑通后）

### 现有实现（修正后的真实面）

- ✅ 属性反射：R3040 autoplay/controls/loop/muted/playsInline + 布局占位渲染
- ✅ FR-009 资源 settle：audio/video/source/track 的 fetch 状态提交 + load/error 事件
  （error/readyState/networkState 初值、src 反射、historical 面均绿——**修正 2026-08-17
  立项时低估**：初值面与状态常量并非全缺）
- ✅ R2835 四方法：play()/pause()/load()/canPlayType()（canPlayType 能力表 2026-09-01
  真值化——webm/ogg 容器 maybe + vp9/vorbis/mp3 probably，域外 ''）
- ⚠️ 媒体专有 IDL 属性面缺失：currentTime/duration(NaN)/playbackRate/preload/
  crossOrigin 归一/seeking/paused 等（F2）
- ⚠️ `<track>` 反射 + TextTrack/TextTrackList/addTextTrack/textTracks 全缺（F1/F3）
- ⚠️ load 算法事件序列（loadstart/canplay/loadedmetadata/...）未派发——FR-009 只有
  load/error 两事件（F4）
- ⚠️ track.src 未按 URL 属性解析（同 R2838 a.href 模式未接）（F6）

## 缺口清单（含失败聚类映射）

| # | 缺口 | 状态 | 失败聚类 |
|---|------|------|----------|
| M1g | WPT media-elements 用例覆盖 | ✅ 60 用例已导入（含 event_* 族 25 + volume/Audio 构造器 + controlsList 面），**90.1%** | — |
| M2g | load 算法 + 状态机（事件序列派发） | ✅ M2 落地（13T→**0T**） | F4 闭合 |
| M3g | 事件序列 headless 近似驱动 | ✅（同 M2g；source-child 触发已落地） | F4 闭合 |
| M4g-a | 媒体元数据 IDL 反射（初值面） | ✅ 切片 3 落地 | F2 闭合（-9 Fail） |
| M4g-b | `<track>` 反射 + TextTrack 最小面 | ✅ M3 落地（TextTrack 接口/addTextTrack/textTracks/track.track 全绿） | F1/F3 闭合 |
| M4g-c | track.src URL 解析 + \0 剥离 | ✅ 切片 3 落地（含 `<a href="">` 修复） | F6 闭合（-6 Fail） |
| M4g-d | canPlayType 能力表（空表→选型面更新） | ✅ 2026-09-01 落地（media-playback 流代行——跨 goal 联动兑现）：能力表由解码面真值驱动（webm/ogg 容器 maybe + vp9/vorbis/mp3 probably，VP8/Opus/Theora/H.264/AAC 域外诚实 ''）；**93.5%**（386P/0F/0T/27PF，+14P）；单测 `test_media_can_play_type_capability_table_m4gd`（18 断言面） | F5（41→27 PF，in-face 全转 Pass） |

## 下一步计划

1. **扩大导入面（下一轮首选）**：the-video-element 反射面（video-tabindex / video_crash_empty_src
   等）+ media-elements 剩余可跑面（loading-the-media-resource 的 resource-selection
   pointer 族——依赖真网络 fetch 判定，视 mutation 面支撑情况）→ 扩大基线盘子。
2. ~~**M4g-d**：canPlayType 能力表联动更新~~ ✅ 2026-09-01 兑现（能力表真值化，
   93.5%——后续新增解码面（AV1/H.264，media-playback M3）时同步扩表）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 摸底 | ✅ 完成（2026-08-31，46.5% 基线 + 切片 3 → 73.1%） |
| M2 — 状态机与事件序列 | ✅ 完成（2026-08-31，13T→2T） |
| M3 — API 语义 + track 面 + 播放层衔接 | ✅ 完成（2026-08-31，12F→0F，84.0%；2026-09-01 扩批 event_* 族 → 88.3%） |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | canPlayType 能力表（依赖 media-playback M0 解码选型——选型决定支持面） | ⬜ 跨 goal 联动，media-playback master.md D1 同源 |

## 验证基线

- 基线时点：2026-08-31，WPT rev `3159769338`；基线 **114/245 = 46.5%** → 切片 3
  **179/245 = 73.1%** → M2 **214/245 = 79.6%** → M3 **226/245 = 84.0%**（Fail 0 /
  Timeout 2 / PF 41）→ M3 扩批（2026-09-01，event_* 族）**324/367 = 88.3%**
  → 扩批第二批（同日，source-child + error 码）**334/375 = 89.1%**
  → 扩批 III（同日，volume/muted + Audio 构造器）**369/410 = 90.0%**
  → 扩批 IV（同日，controlsList + the-video-element）**372/413 = 90.1%**
  → 扩批 V/VI（同日，playbackRate TypeError + preload setter 补缺）**90.1% 维持**
  （Fail 0 / **Timeout 0** / PF 41）
- 入口：`make testharness-media`（FILTER 透传，`--json` 捕获 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：`evidence/2026-08-31-media-baseline.md`（+ 同名 .json 机读版）、
  `evidence/2026-09-01-media-event-family.json`、`evidence/2026-09-01-media-source-child.json`、
  `evidence/2026-09-01-media-volume-audio.json`、`evidence/2026-09-01-media-controlslist.json`、
  `evidence/2026-09-01-media-playbackrate-typeerror.json`、`evidence/2026-09-01-media-preload-setter.json`
