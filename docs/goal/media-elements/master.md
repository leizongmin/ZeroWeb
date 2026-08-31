# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（M3 扩批 event_* 族 25 用例接入——84.0% → **88.3%**，play pending-promise 语义 + resize/timeupdate 时序补齐，余账 2T）

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
余账 2 Timeout 不变（error 错误码 / currentSrc source-child 触发）。

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
- ✅ R2835 四方法：play()/pause()/load()/canPlayType()（canPlayType 恒 ''——合法保守值，
  能力表为空）
- ⚠️ 媒体专有 IDL 属性面缺失：currentTime/duration(NaN)/playbackRate/preload/
  crossOrigin 归一/seeking/paused 等（F2）
- ⚠️ `<track>` 反射 + TextTrack/TextTrackList/addTextTrack/textTracks 全缺（F1/F3）
- ⚠️ load 算法事件序列（loadstart/canplay/loadedmetadata/...）未派发——FR-009 只有
  load/error 两事件（F4）
- ⚠️ track.src 未按 URL 属性解析（同 R2838 a.href 模式未接）（F6）

## 缺口清单（含失败聚类映射）

| # | 缺口 | 状态 | 失败聚类 |
|---|------|------|----------|
| M1g | WPT media-elements 用例覆盖 | ✅ 55 用例已导入（含 event_* 族 25），**88.3%** | — |
| M2g | load 算法 + 状态机（事件序列派发） | ✅ M2 落地（13T→2T，余 2T 归 mutation 面/解码层） | F4 基本闭合 |
| M3g | 事件序列 headless 近似驱动 | ✅（同 M2g；source-child 触发面 defer） | F4 余账 |
| M4g-a | 媒体元数据 IDL 反射（初值面） | ✅ 切片 3 落地 | F2 闭合（-9 Fail） |
| M4g-b | `<track>` 反射 + TextTrack 最小面 | ✅ M3 落地（TextTrack 接口/addTextTrack/textTracks/track.track 全绿） | F1/F3 闭合 |
| M4g-c | track.src URL 解析 + \0 剥离 | ✅ 切片 3 落地（含 `<a href="">` 修复） | F6 闭合（-6 Fail） |
| M4g-d | canPlayType 能力表（空表→选型面更新） | ⬜ 等 media-playback M0 选型 | F5（41 PF，非 bug） |

## 下一步计划

1. **扩大导入面（下一轮首选）**：the-video-element/the-audio-element 反射面用例 +
   media-elements 剩余可跑子目录（playing-the-media-resource 已有 playbackRate；
   ready network/seeking 面视语义支撑情况）→ 扩大基线盘子并暴露下一层缺口。
2. **source-child 资源选择触发**（source 元素插入 → 资源重选派 loadstart）：mutation 面，
   currentSrc 余账 Timeout 的闭合路径。
3. **error 错误码语义**：资源选择失败注入 MEDIA_ERR_SRC_NOT_SUPPORTED——依赖真资源
   加载判定，随解码层（media-playback）或独立资源选择切片。
4. **M4g-d**：canPlayType 能力表等 media-playback M0 选型落地后联动更新（跨 goal 依赖）。

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
  （Fail 0 / Timeout 2 / PF 41）
- 入口：`make testharness-media`（FILTER 透传，`--json` 捕获 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：`evidence/2026-08-31-media-baseline.md`（+ 同名 .json 机读版）、
  `evidence/2026-09-01-media-event-family.json`
