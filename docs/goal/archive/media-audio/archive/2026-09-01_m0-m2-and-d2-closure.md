# media-audio 里程碑归档 — M0/M1/M2 与 D2 闭环（2026-09-01）

> 归档自 master.md 过程节选（控制面只保留最新态；过程细节在此只追加不修改）。

## M0 — 环境验证 + 验证策略（双门控之一）

- 环境实测：内核层 HDA 声卡在；ALSA dev 头缺失（libasound2-dev 未装）→ cpal
  默认 ALSA host 无法编译；PulseAudio dev 在但 server 不可用。
- 验证策略成立：`AudioSink` trait 抽象 + 双实现——`CpalSink`（feature-gated
  `audio-cpal`，设备面）+ `NullSink`（headless/CI 默认，可观测：写入帧数 +
  过零率频域代理断言）。
- 证据：evidence/2026-09-01-m0-environment-probe.md。

## M1 — 首个声音输出

- **切片 1 输出面**：`AudioSink` trait（start/write f32 交错/pause/resume/
  underrun_count）+ `NullSink`（帧数/过零率/暂停拒写计 underrun）；单测 5 件。
- **切片 2 设备面**：`CpalSink`（cpal 0.16 入 workspace optional）——f32 直通
  设备流、回调队列饿死计 underrun、pause/resume 流控双闸、格式变更须重建。
- **切片 3 混音面**：`Mixer`——attach/detach 源句柄 + per-source volume/muted
  增益 + mix_into 软削幅；短源补零不断流；单测 7 件。
- **解码链 e2e**（跨 goal，media-playback 流 M2c 落地）：mp3 + vorbis fixture →
  NullSink 过零率 ≈880 = 2×440Hz——本档 NullSink 断言契约首次真值实证。
- **D2 获批项闭环（2026-09-01）**：libasound2-dev 1.2.14-1 在位 → cpal 编译 +
  39 测全绿 + **CpalSink 真设备流冒烟通过**（Ok 分支：构造 48kHz/2ch → start →
  write → pause → 暂停拒收 → resume 全链）。
  证据：evidence/2026-09-01-cpalsink-device-smoke.md。
- **Mixer 接线决策注记**：per-entry NullSink 直连已覆盖多源并发语义面；Mixer
  （N→1 合流）挂 CpalSink 真出声切片——NullSink 阶段接 Mixer 只添无行为变化的
  中间层（且破坏 per-entry 可观测断言面）。

## M2 — A/V 同步 + 控制（跨 goal：media-playback 流切片 D+E 兑现）

- webm 双轨（VP9+Vorbis）伴生音频解码（OGG 重封装 → symphonia）+ audio clock
  主时钟（视频帧调度 `sync_to_media_time` 对齐音频游标，drift 构造校正）+
  currentTime 组合时钟（A/V pair 优先报音频游标）+ seek 双轨对齐。
- A/V pair ended 面回归守卫（伴音流末 video player 走到 Ended、泵停）。
- opus 解码面转正（`opus-decoder 0.1.1` 纯 Rust——registry 双面回落登记 +
  canPlayType 扩表）。

## M3 — `<audio>` 全路径 + Web Audio 评估

- `<audio>` 纯音频播放全路径 e2e 常驻（settle → 桥 play → 泵推进写 sink）。
- Web Audio 最小面可行性 RFC 成文（2026-09-01，
  [docs/specs/web-audio-audiocontext-minimal-face-spec-rfc.md](../../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md)）：
  最小面可行（底座零新增依赖，1~2 切片）；实施与否待用户批准（D-WA-1/2）——
  不批准不影响本 goal DONE。
