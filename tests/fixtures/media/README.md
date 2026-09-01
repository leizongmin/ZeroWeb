# 媒体 Fixture（media-playback M0 切片 3）

**来源清白声明**：全部文件由 ffmpeg 7.1.5（Debian 13 官方包）现场生成，无外部素材、
无第三方版权内容。视频为 `testsrc2` 合成纹样，音频为 `sine` 合成正弦波（440 Hz）。
生成命令记录于本文件，可在任何装有 ffmpeg 的环境零差异重现。

| 文件 | 容器 | 视频编码 | 音频编码 | 尺寸/采样率 | 时长 | 大小 |
|---|---|---|---|---|---|---|
| `sample-mp4-h264.mp4` | mp4 | H.264 (baseline, yuv420p) | AAC LC | 320x240 / 44.1kHz 单声道 | 2s | 92KB |
| `sample-webm-vp9.webm` | webm | VP9 (yuv420p) | 无 | 320x240 | 2s | 41KB |
| `sample-webm-vp9-vorbis.webm` | webm | VP9 (yuv420p) | Vorbis (q4) | 320x240 / 44.1kHz 单声道 | 2s | 49KB |
| `sample-webm-av1.webm` | webm | AV1 (yuv420p, libaom) | 无 | 320x240 | 2s | 57KB |
| `sample-mp3.mp3` | mp3 | 无 | MP3 (libmp3lame 64k) | 44.1kHz 单声道 | 2s | 16KB |
| `sample-ogg-opus.oga` | ogg | 无 | Opus (48k) | 48kHz 单声道 | 2s | 14KB |
| `sample-ogg-vorbis.oga` | ogg | 无 | Vorbis (q4) | 44.1kHz 单声道 | 2s | 7KB |

## 生成命令

```bash
# H.264 + AAC（mp4）——`<video>` 主路径（解码器选型 RFC 的核心评估对象）
ffmpeg -f lavfi -i "testsrc2=size=320x240:rate=24:duration=2" \
       -f lavfi -i "sine=frequency=440:duration=2" \
       -c:v libx264 -profile:v baseline -pix_fmt yuv420p \
       -c:a aac -shortest -y sample-mp4-h264.mp4

# VP9（webm）——开源编解码路线（RFC 三路线之「开源先行」的评估对象）
ffmpeg -f lavfi -i "testsrc2=size=320x240:rate=24:duration=2" \
       -c:v libvpx-vp9 -b:v 200k -pix_fmt yuv420p -y sample-webm-vp9.webm

# VP9 + Vorbis（webm 双轨——media-playback M2 切片 D：A/V 同源 demux +
# 音频时钟主时钟的 e2e 输入；`open_webm_audio_track` 的验证资产）
ffmpeg -f lavfi -i "testsrc2=size=320x240:rate=24:duration=2" \
       -f lavfi -i "sine=frequency=440:duration=2" \
       -c:v libvpx-vp9 -b:v 200k -pix_fmt yuv420p \
       -c:a libvorbis -q:a 4 -shortest -y sample-webm-vp9-vorbis.webm

# Vorbis（音频，media-playback M2c——symphonia 纯 Rust 解码面；opus 不在其 0.6
# 编解码面内，oga-opus fixture 保留作后续选型对照）
ffmpeg -f lavfi -i "sine=frequency=440:duration=2" \
       -c:a libvorbis -q:a 4 -y sample-ogg-vorbis.oga

# MP3（音频，media-audio 目标）
ffmpeg -f lavfi -i "sine=frequency=440:duration=2" \
       -c:a libmp3lame -b:a 64k -y sample-mp3.mp3

# Opus（oga，音频）
ffmpeg -f lavfi -i "sine=frequency=440:duration=2" \
       -c:a libopus -b:a 48k -y sample-ogg-opus.oga
```

## 用途

- **media-playback M0**：解码器选型 RFC 的验证资产（H.264 专利池面 vs VP9 开源面，
  两格式齐备使三路线对比都可落地实测）；RFC 批准后 M1「首个视频帧上屏」的 e2e 输入。
- **media-audio M1**：音频输出链路的 e2e 输入（AAC/MP3/Opus 解码面 + 正弦波可断言
  混音总线输出频率）。

## 设计约束

- 体积 ≤ 100KB/文件（入仓成本可控；git 无 LFS 依赖）。
- 时长 2s（播放驱动/seek/ended 的 e2e 断言窗口够用，转码迭代快）。
- 分辨率 320x240（非 16 对齐的 256/512 之间取值，可暴露 stride/行对齐处理 bug）。
- H.264 用 baseline profile + yuv420p（硬件/软解最大兼容面；B 帧缺失简化帧序语义）。
- 无字幕轨/多轨道/章节（首期解码面最简；后续需要时另生成多轨 fixture 并记录于此）。

# AV1（media-playback M3 预备资产——`sample-webm-av1.webm`：libaom-av1 生成、
# 来源清白。解码引入（dav1d 绑定）待 master.md D2 用户批准后实施；fixture 先行
# 落库使 D2 批准后解码切片可直接以本资产验证。matroska-demuxer 已实测可枚举
# V_AV1 轨（CodecPrivate 在）——demux 面就绪，仅解码器面缺位）
ffmpeg -f lavfi -i "testsrc2=size=320x240:rate=24:duration=2" \
       -c:v libaom-av1 -b:v 200k -pix_fmt yuv420p -y sample-webm-av1.webm
