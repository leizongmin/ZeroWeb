# 媒体音频 — 运行时控制面板（master.md）

**入口文档**: [../media-audio.md](../media-audio.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-01（**D1 获批（D-WA-1 批准 + D-WA-2 选先 NullSink）**——
Web Audio AudioContext 最小面实施开工（切片 1+2，NullSink 设备面挂真出声切片），
RFC 见 [../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md](../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md)。
此前：D2 获批项闭环——libasound2-dev 在位，cpal 编译 + 39 测全绿 +
**CpalSink 真设备流冒烟通过**（Ok 分支：构造/start/write/pause/resume 全链——
[evidence](evidence/2026-09-01-cpalsink-device-smoke.md)）；M2 A/V 同步主体
由 media-playback 流切片 D+E 兑现；opus 解码面转正。本档余：Mixer 接线（挂真出声
切片，可选））

---

## 当前状态

**专项定位**：媒体方向三拆之三（门控最深）。音频输出（解码→混音→设备）+ A/V 同步 +
volume/muted 真控制。**双重启动门控均已解除**：① M0 音频环境验证与验证策略成文
（2026-09-01 完成）；② media-playback M0 解码选型 RFC 获批（2026-09-01，路线 C）。

**M0 已收口（2026-09-01）**：
- 环境实测：内核层 HDA 声卡在；**ALSA dev 头缺失（libasound2-dev 未装）→ cpal 默认
  ALSA host 无法编译**；PulseAudio dev 在但 server 不可用（Connection refused）。
- 验证策略成立：**`AudioSink` trait 抽象 + 双实现**——`CpalSink`（feature-gated
  `audio-cpal`，设备面）+ `NullSink`（headless/CI 默认，可观测：写入帧数 + 过零率
  频域代理断言）。M1 验收 = NullSink 可观测断言（CI 常驻）+ CpalSink 人工冒烟（可选）。
- cpal 编译实测须装 `libasound2-dev`（系统级变更 → 待用户决策 D2；不阻塞 trait/
  NullSink 层设计与实施）。
- 证据：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)

**与兄弟 goal 的边界**：
- media-playback — 视频/解码选型归其管；A/V 同步接口对齐（audio clock 主时钟——契约记录
  于两流 master.md）；**解码面依赖其 RFC 选型，输出面（AudioSink trait）与选型解耦可先行**
- media-elements — 语义面归其管；volume/muted 本目标接真增益（IDL 语义已由其 M3 扩批
  III 落地：非有限 TypeError/同值短路/queued volumechange/load 清 pending）
- js-dom — volume/muted 反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时 + 2026-09-01 M0 探测 / M1 切片 1 更新）

### 现有实现

- ✅ 反射底座：muted/volume 属性反射（R3040 + M3 扩批 III IDL 语义全对齐）
- ✅ 时钟底座：rAF 帧驱动（P1a）——音频时钟对齐可挂
- ✅ 环境/策略底座：M0 收口（AudioSink trait + NullSink 验证策略成文）
- ✅ **输出面（M1 切片 1）**：`zero-media::audio`——`AudioSink` trait（start/
  write f32 交错/pause/resume/underrun_count）+ `NullSink`（写入帧数 + 过零率
  频域代理（2×频率锚定）+ 暂停拒写计 underrun）；单测 5 件常驻（启动前拒写/
  暂停门控/非整帧拒收/440Hz 过零率/重启累计语义）
- ✅ **设备面（M1 切片 2）**：`CpalSink`（feature-gated `audio-cpal`，cpal 0.16 入
  workspace optional 依赖）——f32 原生采样直通设备流、回调队列饿死计 underrun、
  pause/resume 流控（write 拒收 + 流暂停双闸）、格式变更须重建（显式报错防
  流错配）；环境自适应冒烟常驻（无设备/格式不支持 → 构造报错回落 NullSink，
  本环境实测构造成功 + start/pause/resume 全通）
- ✅ **混音面（M1 切片 3）**：`Mixer`——attach/detach 源句柄（资源生命周期面）+
  per-source volume/muted 增益 + mix_into 软削幅（clamp [-1,1]）写下游 sink；
  短源补零不断流；单测 7 件常驻
- ✅ **解码面（M2c，跨 goal：media-playback 流落地）**：`zero-media::audio_decode::
  AudioDecoder`（symphonia 0.6：mp3 + ogg/vorbis）——f32 交错 PCM 输出直写
  AudioSink 契约；全链 e2e 双件常驻（mp3 + vorbis fixture → NullSink 过零率 ≈880
  = 2×440Hz，本档 NullSink 断言契约首次真值实证）
- ✅ **播放管线宿主侧接线（M2c 后续切片 A/B，跨 goal：media-playback 流落地）**：
  `<audio>` settle → `VideoPlayerRegistry.register_audio_source` → 宿主桥 play →
  音频泵（tab_worker `audio_advance_all`，实时节奏逐包解码）→ NullSink 写入；
  volume/muted 增益联动（media-elements IDL setter 桥推 `setGain` + play 起播同步）；
  seek 追赶区静默（skip_until_ms 丢弃线）；导航离开 `clear()` 释放（DC-4）；
  tabworker/renderer 双路径对齐（SetVideoPlayers 注入）。e2e 三面常驻
  （webm video / mp3 audio / oga-opus 不登记负例）
- ✅ opus 解码面（2026-09-01 M2c opus 接线落地，跨 goal：media-playback 流）——
  `opus-decoder 0.1.1` 纯 Rust（RFC 6716/8251，零 unsafe 零 FFI）补齐 symphonia 缺位；
  `zero-media::opus_decode::open_ogg_opus`（symphonia ogg 容器 + OpusHead 解析 +
  pre-skip 丢弃）；`sample-ogg-opus.oga` 转正可播（registry 双面回落登记 + 泵推进）
- ⚠️ 重采样/混音接线未实施——播放管线把解码帧喂 Mixer 的宿主侧接线待做
- ✅ 选型已对齐（media-playback RFC 获批：路线 C，symphonia 音频解码面归 M2c）
- ✅ 音频 e2e 资产：`tests/fixtures/media/`（sample-mp3.mp3 / sample-ogg-opus.oga，
  ffmpeg 生成、来源清白、生成命令记录于该目录 README）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| A1 | 音频环境验证 + headless 验证策略 | ✅ M0 收口（2026-09-01） |
| A2 | 解码选型未对齐（外部门控：media-playback M0） | ✅ 已对齐（RFC 获批，2026-09-01） |
| A3 | 零音频管线（解码/重采样/混音/输出） | 🔄 M1 切片 1-3 + M2c 解码面 + M2c 后续宿主接线落地（NullSink/CpalSink/Mixer/symphonia + 播放管线增益联动）；余重采样与 Mixer 多源接线（当前注册表内 NullSink 直连） |
| A4 | A/V 同步机制缺失 | 🔄 M2 主体落地（2026-09-01，media-playback 流切片 D+E）——audio clock 主时钟（webm 双轨伴生音频解码 + 视频帧调度对齐音频游标 + drift 构造校正 + currentTime 组合时钟 + seek 双轨对齐）；余设备面真输出（CpalSink 冒烟，可选） |
| A5 | 音频 e2e 资产 | ✅ 真解码链 e2e 落地（mp3 + vorbis fixture → NullSink 过零率锚点常驻）+ 合成源面 |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | AudioContext（Web Audio）最小面可行性 RFC → 是否实施 | ✅ 获批（2026-09-01，GB-20260901 批复）——D-WA-1 批准切片 1+2；D-WA-2 选**先 NullSink**（设备面挂 media-audio M1 CpalSink 真出声切片）。RFC：[../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md](../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md) |
| D2 | 安装 `libasound2-dev`（系统级 apt 变更）以解锁 cpal 编译验证 | ✅ 获批（2026-09-01）——装包后补 cpal 编译实测 |

## 下一步计划

1. **Web Audio 最小面实施（D1 已批准）**：切片 1（AudioContext/`BaseAudioContext`
   shim 面 + NullSink 可观测链）→ 切片 2（oscillator/destination 连接语义 +
   WPT webaudio 可执行子集评估导入）；设备面挂 M1 CpalSink 真出声切片（D-WA-2）。
2. **M1 收口评估（余项收窄）**：Mixer 多源混音接线**决策注记（2026-09-01）**——
   现播放管线 per-entry NullSink 直连已覆盖多源并发语义面（per-source 增益/独立
   解码流/并发泵）；Mixer（M1 切片 3 组件，7 单测常驻）的价值在**单设备输出流的
   N→1 合流**——即 CpalSink 真设备输出时的前置组件。NullSink 阶段接 Mixer 只添
   无行为变化的中间层（且破坏 per-entry sink 可观测断言面）。**结论：Mixer 接线
   挂到 CpalSink 真出声切片**（可选/桌面环境），M1 的 CI 可验面已收口。
   CpalSink 真出声冒烟仍留桌面环境（编译/枚举面 D2 已验证）。
2. **M2**：~~A/V 同步接口对齐~~ ✅ 2026-09-01 主体兑现（media-playback 流切片
   D+E：audio clock 主时钟 + 组合时钟 + seek 双轨对齐）；A/V pair ended 面回归
   守卫落地（webview `registry_av_pair_reaches_ended_after_audio_exhausted`——
   伴音流末 video player 走到 Ended、泵停）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 环境验证 + 验证策略（门控） | ✅ 完成（2026-09-01，含 D2 后 cpal 编译实测补录） |
| M1 — 首个声音输出 | 🔄 切片 1-3 + 解码链 e2e + 播放管线宿主接线落地（2026-09-01）；**D2 获批项闭环（2026-09-01）**——cpal 编译 + 39 测全绿 + CpalSink 真设备流冒烟通过（Ok 分支：构造/start/write/pause/resume 全链，[evidence](evidence/2026-09-01-cpalsink-device-smoke.md)）；余 Mixer 接线（挂真出声切片，可选） |
| M2 — A/V 同步 + 控制 | 🔄 主体落地（2026-09-01，media-playback 流切片 D+E：audio clock 主时钟 + 组合时钟 + seek 对齐）；余 CpalSink 真出声冒烟（可选） |
| M3 — `<audio>` 全路径 + Web Audio 评估 | 🔄 `<audio>` 纯音频播放全路径 e2e 已常驻（M2c 后续切片 A/B：settle → 桥 play → 泵推进）；Web Audio 最小面 **D1 已批准**（2026-09-01，D-WA-2 选先 NullSink）——切片 1+2 待实施 |

## 验证基线

- 测试基线：`make test` 全绿（zero-media default feature：17 单测 + 1 doctest =
  decode 5 + NullSink 5 + mixer 7；`audio-cpal` feature 另增 CpalSink 环境自适应
  冒烟 1 件）；clippy 零警告（default 与 `--features audio-cpal` 双配置）
- NullSink 可观测锚点：440Hz 正弦 @48kHz 过零率 ≈880（2×频率；修正 M0 evidence
  的 ≈440 笔误——evidence 只追加不修改，以代码与本档为事实源）；暂停拒写计
  underrun；非整帧写入拒收
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)

## 归档

- [archive/2026-09-01_m0-m2-and-d2-closure.md](archive/2026-09-01_m0-m2-and-d2-closure.md) —
  M0/M1/M2 过程与 D2 获批项闭环记录（只追加不修改；本控制面保留最新态）。
