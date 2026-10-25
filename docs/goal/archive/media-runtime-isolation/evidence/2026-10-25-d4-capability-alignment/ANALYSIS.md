# D4 能力清单对齐（2026-10-25）

## 背景

C4 配对验收能力真值三对照（§四）记 D4 偏差：宿主 `codec_capabilities` 欠申报
`audio/ogg; codecs=vorbis` 与 `audio/wav`（内核路由面早已可解，申报滞后）；`audio/webm;
codecs=opus` 与 `audio/mp4; codecs=aac` 双侧在列但仓内无 fixture 未验。本切片收口两面。

## 变化

- **宿主补申报**（`apps/media-host/src/kernel.rs` `codec_capabilities`）：追加
  ogg-vorbis / wav 两条目——与内核实际路由面（`AudioDecoder::open` symphonia probe：
  mp3 / ogg-vorbis / wav / mp4-aac）同源，「能播却报不支持」的可播性收窄风险消除。
- **无 fixture 主张实解落位**（`crates/webview/tests/media_c4_paired.rs` 能力真值段）：
  - `audio/mp4; codecs=aac` → `sample-mp4-h264.mp4` 的 AAC LC 音轨（fixture README
    生成记录在案；symphonia isomp4+aac 已在 workspace feature 面，feature 关时
    `open_media` 报 `NoVideoTrack` 回落 `AudioDecoder::open` 命中音轨）。不新增 fixture。
  - `audio/webm; codecs=opus` → wpt-data 既有资产 `movie_5.webm`（VP9+Opus 双轨，
    字节级核对 A_OPUS/V_VP9）经 `open_webm_opus_audio_track` 直解——与 zero-media
    常驻单测 `webm_av_opus_track_decode_chain` 同源。wpt-data 按 fetch 白名单按需
    拉取不入库（既有约定），不新增 fixture、不破坏 fixtures 目录来源清白声明。
- **常驻回归门**：`apps/media-host/src/lib.rs` handshake 单测追加 ogg-vorbis/wav
  在列断言——欠申报回归默认构建即红，不依赖 instrument 显式运行。
- instrument 内原「未验记录段」「欠申报探针段」移除：两段语义分别被双断言表与
  宿主常驻断言取代。

## 验证

[capability-truth-post-d4.log](capability-truth-post-d4.log)（test-guard 包裹，显式运行）：

- 宿主实报 8 项基础主张 + 2 项 feature 门控主张（av1/h264 默认关如实不在列）。
- 8/8 `capability-verified`：每项主张「宿主在列 ⇔ fixture 实解成功」双断言全过。
- 配额面不变（QuotaExceeded 端到端）。
- 宿主 handshake 单测（含新在列断言）过。

## 边界

- engine 静态表 `_cptTable`（页面可见 `canPlayType` 面）本轮无改动——D4 偏差本就
  只在宿主侧欠申报，engine 侧各主张原已齐备。
- feature 门控主张（av1/h264）维持「声明与 feature 门控同源」的既有形态，未验解码
  主张如实按 `probe_ok=cfg!(feature)` 表达。
