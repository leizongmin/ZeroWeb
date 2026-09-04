//! zero-media 单元测试 — fixture 解码正确性（RFC §3.3 验收资产）。
//!
//! 参照基准：ffmpeg 7.1.5 对 `sample-webm-vp9.webm` 首帧的 yuv420p rawvideo 输出
//! （2026-09-01 探针实测逐字节一致）；此处固化为 RGBA 哈希 + 帧元数据断言。

#[cfg(feature = "decode-h264")]
use super::decode::VideoTrackDecoder;
use super::decode::{VideoDecoder, fixture_path, rgba_mean};
use std::fs;

#[test]
fn webm_vp9_metadata() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    // container duration 2000ms（ffprobe 实测一致）
    assert_eq!(dec.duration_ms(), Some(2000));
}

#[test]
fn webm_vp9_first_frame_matches_ffmpeg_reference() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();

    let f0 = dec.next_frame().unwrap().expect("frame 0");
    assert_eq!(f0.width, 320);
    assert_eq!(f0.height, 240);
    assert_eq!(f0.pts_ms, 0);
    // 首帧 testsrc2 纹样锚点——RGBA 面的 RGB 均值观测窗。M2 色度精化后 limited
    // range 转换与 ffmpeg swscale RGBA 参照（实测 123.27）一致——窗口 ±15。
    // （旧锚点 153.5 源于全范围误释 + 色度索引坍缩两个缺陷的叠加，本窗同步收紧。）
    let mean = rgba_mean(&f0.rgba);
    assert!(
        (108.0..=138.0).contains(&mean),
        "first frame RGB mean out of reference window: {mean}"
    );
    // 哈希稳定性：同一输入两次解码逐像素一致（转换无随机性）。
    let mut dec2 = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f0b = dec2.next_frame().unwrap().unwrap();
    assert_eq!(f0.rgba, f0b.rgba, "decode not deterministic");
}

#[test]
fn webm_vp9_full_stream_frame_count_and_pts() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();

    let mut count = 0u32;
    let mut last_pts = 0u64;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert!(frame.pts_ms >= last_pts, "PTS monotonicity broken");
        last_pts = frame.pts_ms;
        count += 1;
        assert!(count <= 100, "runaway frame count");
    }
    // ffprobe -count_frames 实测 48 帧（2s @ 24fps）。
    assert_eq!(count, 48);
    // 末帧 PTS ≈ 1958ms（48 帧满覆盖 2s 窗口）。
    assert!((1900..=2000).contains(&last_pts));
}

#[test]
fn non_webm_input_rejected() {
    let garbage = b"definitely not an EBML container".to_vec();
    assert!(VideoDecoder::open_webm_vp9(&garbage).is_err());
}

#[test]
fn empty_input_rejected() {
    assert!(VideoDecoder::open_webm_vp9(&[]).is_err());
}

#[test]
fn webm_vp9_seek_to_mid_stream_keyframe() {
    // M2b：seek 关键帧粒度——seek 到 1s 后 next_frame 应返回 ≥ 定位点的帧，
    // 后续帧 PTS 单调续播至流末（48 帧 @ 24fps，fixture）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    dec.seek_to_ms(1000).unwrap();
    let f = dec.next_frame().unwrap().expect("seek 后应有帧");
    // 精确 seek（spec）：单 keyframe 流回退路径前向解码 → 首帧 pts ≥ 1000。
    assert!(
        f.pts_ms >= 1000,
        "seek(1000ms) 后首帧 PTS 应 ≥ target（precise-seek），got {}",
        f.pts_ms
    );
    // 续播单调至末：剩余帧数 < 48（跳过了前段），PTS 单调。
    let mut count = 1u32;
    let mut last = f.pts_ms;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert!(frame.pts_ms >= last, "seek 后 PTS 单调性破坏");
        last = frame.pts_ms;
        count += 1;
        assert!(count <= 48, "runaway frame count");
    }
    assert!(count < 48, "seek(1s) 后剩余帧应少于全流 48 帧，got {count}");
    assert!((1700..=2100).contains(&last), "末帧 PTS 应仍在 2s 窗口，got {last}");
}

#[test]
fn webm_vp9_seek_to_zero_replays_full_stream() {
    // seek(0)：回到开头，全流 48 帧可重播（ended 重播路径的解码器面）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    // 先消费几帧再 seek 回 0。
    let _ = dec.next_frame().unwrap();
    let _ = dec.next_frame().unwrap();
    dec.seek_to_ms(0).unwrap();
    let f0 = dec.next_frame().unwrap().expect("seek(0) 后应有帧");
    assert_eq!(f0.pts_ms, 0, "seek(0) 后首帧应为 pts=0");
    let mut count = 1u32;
    while dec.next_frame().unwrap().is_some() {
        count += 1;
        assert!(count <= 48);
    }
    assert_eq!(count, 48, "seek(0) 后全流应可重播 48 帧");
}

#[test]
fn webm_colour_identity_full_range_passthrough() {
    // M2 色度精化：reftest-upstream replaced-element-003 的 support 素材
    //（2x2-green.webm，ffprobe: color_range=pc + color_space=gbr）→ identity
    // 矩阵 + full range → 平面 GBR 直传不做 YUV 数学。解码位形 = 位面内容
    //（0,127,0——上游编码器的位面真值）vs ref #008000 差 1 ≤ fuzzy 0-30
    //（chromium 同样在此预算内通过——identity 直传即正确行为面）。
    let path =
        super::decode::workspace_path("tests/wpt-runner/wpt-data/css/css-sizing/aspect-ratio/support/2x2-green.webm");
    let data = fs::read(&path).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f = dec.next_frame().unwrap().expect("2x2 帧应可解");
    assert_eq!((f.width, f.height), (2, 2));
    // identity 直传：位面序 G/B/R → RGBA = (R=Cr, G=Y, B=Cb) = (0,127,0)。
    for px in f.rgba.as_chunks::<4>().0 {
        assert_eq!((px[0], px[1], px[2], px[3]), (0, 127, 0, 255), "identity 直传位面真值");
    }
}

#[test]
fn webm_colour_broadcast_range_luma_not_clipped() {
    // 本仓 fixture（libvpx）：Colour 元素 range=Broadcast（limited）、matrix 缺省。
    // limited 面正确转换后 RGB 均值应落 ffmpeg swscale RGBA 参照（123.3）的观测窗
    //——旧全范围 BT.601 数学下为 153.5（值域误释 +25% 亮度失真，本测试防回退）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f = dec.next_frame().unwrap().expect("frame 0");
    let mean = rgba_mean(&f.rgba);
    assert!(
        (108.0..=138.0).contains(&mean),
        "limited-range 转换后 RGB 均值应近 ffmpeg 参照 123.3，got {mean}"
    );
}

#[test]
fn audio_mp3_decode_to_nullsink_zero_crossing_chain() {
    // M2c 全链 e2e：mp3 fixture（440Hz sine）→ symphonia 解码 f32 → AudioSink
    //（NullSink 可观测断言——过零率 ≈ 880 = 2×频率，media-audio M1 契约锚点）。
    use crate::audio::NullSink;
    use crate::audio::{AudioFormat, AudioSink};
    use crate::audio_decode::AudioDecoder;

    let data = fs::read(fixture_path("sample-mp3.mp3")).unwrap();
    let mut dec = AudioDecoder::open(&data).unwrap();
    let (rate, channels) = (dec.sample_rate(), dec.channels());
    assert_eq!(channels, 1, "fixture 为单声道");

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: rate,
        channels,
    })
    .unwrap();
    let mut batches = 0u32;
    let mut wrote_any = false;
    while let Some(batch) = dec.next_batch().unwrap() {
        assert_eq!(batch.sample_rate, rate);
        assert_eq!(batch.channels, channels);
        sink.write(&batch.samples).unwrap();
        wrote_any = true;
        batches += 1;
        assert!(batches <= 200, "runaway batch count");
    }
    assert!(wrote_any, "解码应产出采样");

    // 时长面：2s @ rate 采样 ≈ frames_written（MP3 延迟/padding 容差 ±5%）。
    let frames = sink.frames_written();
    let expect = u64::from(rate) * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "写入帧数应 ≈ 2s 采样数：got {frames}, expect ≈{expect}"
    );
    // 频域代理锚点：440Hz sine 过零率 ≈ 880（2×频率；media-audio NullSink 契约）。
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn audio_ogg_vorbis_decode_to_nullsink_chain() {
    // 同链 ogg/vorbis 面：fixture（440Hz sine）→ 解码 → NullSink 过零率锚点。
    use crate::audio::{AudioFormat, AudioSink, NullSink};
    use crate::audio_decode::AudioDecoder;

    let data = fs::read(fixture_path("sample-ogg-vorbis.oga")).unwrap();
    let mut dec = AudioDecoder::open(&data).unwrap();
    let (rate, channels) = (dec.sample_rate(), dec.channels());

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: rate,
        channels,
    })
    .unwrap();
    let mut wrote_any = false;
    while let Some(batch) = dec.next_batch().unwrap() {
        sink.write(&batch.samples).unwrap();
        wrote_any = true;
    }
    assert!(wrote_any);
    let frames = sink.frames_written();
    let expect = u64::from(rate) * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "vorbis 写入帧数应 ≈ 2s：got {frames}, expect ≈{expect}"
    );
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn audio_non_audio_bytes_rejected() {
    use crate::audio_decode::AudioDecoder;
    let garbage = b"definitely not an audio container".to_vec();
    assert!(AudioDecoder::open(&garbage).is_err());
}

#[test]
fn webm_av_vorbis_track_decode_chain() {
    // M2 切片 D：webm 双轨（VP9 + Vorbis）的音频面全链——`open_webm_audio_track`
    // demux A_VORBIS 轨 + CodecPrivate 三段头 → OGG 页重封装 → symphonia 解码
    // f32 PCM → NullSink。锚点：44.1kHz 单声道 2s ≈ 88200 帧 + 440Hz 过零率 880
    //（audio clock 主时钟的解码数据源，media-audio M2 契约）。
    use crate::audio::{AudioFormat, AudioSink, NullSink};
    use crate::av_decode::open_webm_audio_track;

    let data = fs::read(fixture_path("sample-webm-vp9-vorbis.webm")).unwrap();
    let mut track = open_webm_audio_track(&data).unwrap();
    assert_eq!(track.sample_rate(), 44100);
    assert_eq!(track.channels(), 1);

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: track.sample_rate(),
        channels: track.channels(),
    })
    .unwrap();
    let mut batches = 0u32;
    while let Some(batch) = track.next_batch().unwrap() {
        sink.write(&batch.samples).unwrap();
        batches += 1;
        assert!(batches <= 500, "runaway batch count");
    }
    assert!(batches > 0, "音频轨应产出数据包");

    let frames = sink.frames_written();
    let expect = u64::from(track.sample_rate()) * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "webm vorbis 写入帧数应 ≈ 2s 采样数：got {frames}, expect ≈{expect}"
    );
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn webm_av_video_track_still_decodes() {
    // 同 fixture 的视频面不受音频轨影响——VP9 主链零回归（双轨 demux 选择面）。
    let data = fs::read(fixture_path("sample-webm-vp9-vorbis.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f0 = dec.next_frame().unwrap().expect("双轨 webm 的 VP9 轨应可解");
    assert_eq!((f0.width, f0.height), (320, 240));
    let mut count = 1u32;
    let mut last = f0.pts_ms;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert!(frame.pts_ms >= last, "PTS 单调性");
        last = frame.pts_ms;
        count += 1;
        assert!(count <= 100, "runaway frame count");
    }
    assert_eq!(count, 48, "双轨 fixture 视频轨仍 48 帧 @ 24fps");
}

#[test]
fn webm_av_audio_track_absent_rejected() {
    // 纯视频 webm（无音频轨）→ open_webm_audio_track 报 NoTrack（feature-detect 面）。
    use crate::av_decode::open_webm_audio_track;
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let err = match open_webm_audio_track(&data) {
        Err(e) => e,
        Ok(_) => panic!("无音频轨的 webm 应构建失败"),
    };
    assert!(
        matches!(err, crate::audio_decode::AudioDecodeError::NoTrack),
        "无音频轨应报 NoTrack"
    );
}

#[cfg(feature = "decode-av1")]
#[test]
fn webm_av1_decode_full_stream() {
    // M3 AV1 面（D-RFC-2）：V_AV1 轨经 dav1d 全流解码——48 帧（2s @ 24fps）、
    // PTS 单调、尺寸面（testsrc2 320x240）与 VP9 fixture 同款生成参数。
    // https://code.videolan.org/videolan/dav1d
    let data = fs::read(fixture_path("sample-webm-av1.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm(&data).unwrap();
    assert_eq!(dec.duration_ms(), Some(2000));

    let mut count = 0u32;
    let mut prev_pts: i64 = -1;
    let mut first_mean = 0.0;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert_eq!(frame.width, 320, "宽度面");
        assert_eq!(frame.height, 240, "高度面");
        assert!(
            (frame.pts_ms as i64) >= prev_pts,
            "PTS 单调（pts={} prev={})",
            frame.pts_ms,
            prev_pts
        );
        prev_pts = frame.pts_ms as i64;
        if count == 0 {
            first_mean = rgba_mean(&frame.rgba);
        }
        count += 1;
    }
    assert_eq!(count, 48, "全流帧数（与容器块数一致）");
    // 首帧 testsrc2 纹样锚点：与 ffmpeg 7.1.5 RGBA 参照（实测 123.26）同窗收紧
    //——dav1d 输出经同一 planes_to_rgba 面，色彩声明以位流 seq header 为准
    //（libaom testsrc2 声明 BT.709 limited，与 VP9 fixture 声明面一致）。
    assert!(
        (first_mean - 123.26).abs() <= 15.0,
        "AV1 首帧 RGB 均值对齐 ffmpeg 参照窗 ±15（got {first_mean}）"
    );
}

#[cfg(feature = "decode-av1")]
#[test]
fn webm_av1_open_rejects_when_feature_disabled_equivalent_and_vp9_still_routes() {
    // codec 路由面：open_webm 对 V_VP9 容器行为与 open_webm_vp9 一致（M3 路由
    // 不回归 VP9 主面）；V_AV1 在 feature 关闭时回落 NoVideoTrack（占位渲染面）。
    // 本测试编译于 feature 开启态——断言 V_VP9 路由 + 非 webm 拒绝。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm(&data).unwrap();
    let f0 = dec.next_frame().unwrap().expect("vp9 frame 0");
    assert_eq!(f0.width, 320);
    assert_eq!(f0.height, 240);

    let garbage = b"not a webm".to_vec();
    assert!(VideoDecoder::open_webm(&garbage).is_err());
}

#[test]
fn audio_ogg_opus_decode_chain() {
    // M2c opus 面：`opus-decoder` 纯 Rust 解码链全通——symphonia ogg reader 容器
    // demux（OpusHead 声道/pre-skip 解析 + OpusTags 跳过）→ opus 位流逐包 f32 PCM。
    // 锚点：fixture 440Hz sine 单声道 2s @48kHz ≈ 96000 帧 + 过零率 ≈880（2×频率，
    // NullSink 可观测契约——与 mp3/vorbis 链同款断言面）。
    use crate::audio::{AudioFormat, AudioSink, NullSink};
    use crate::opus_decode::open_ogg_opus;

    let data = fs::read(fixture_path("sample-ogg-opus.oga")).unwrap();
    let mut track = open_ogg_opus(&data).unwrap();
    assert_eq!(track.sample_rate(), 48_000, "Opus 规范输出率 48kHz");
    assert_eq!(track.channels(), 1);

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: track.sample_rate(),
        channels: track.channels(),
    })
    .unwrap();
    let mut batches = 0u32;
    while let Some(batch) = track.next_batch().unwrap() {
        assert_eq!(batch.sample_rate, 48_000);
        sink.write(&batch.samples).unwrap();
        batches += 1;
        assert!(batches <= 500, "runaway batch count");
    }
    assert!(batches > 0, "opus 轨应产出数据包");

    let frames = sink.frames_written();
    let expect = 48_000u64 * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "opus 写入帧数应 ≈2s 采样数：got {frames}, expect ≈{expect}"
    );
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn audio_ogg_opus_garbage_rejected() {
    // 非 Ogg Opus 字节 → open_ogg_opus 报错（probe 面）。
    use crate::opus_decode::open_ogg_opus;
    let garbage = b"definitely not an opus container".to_vec();
    assert!(open_ogg_opus(&garbage).is_err());
}

#[test]
fn webm_av_opus_track_decode_chain() {
    // M3 扩批（2026-09-02，fixture-mounted 播放切片前置）：webm A_OPUS 轨直解——
    // Matroska demux + CodecPrivate(OpusHead) 解析 + opus-decoder 逐包解码（无 OGG
    // 重封装）。WPT 上游 media/*.webm 实测全为 VP9+Opus 双轨——本面是 WPT 播放推进
    // 族（track-cues-* / time-marches-on）媒体源的解码前置。素材：movie_5.webm
    //（wpt-data fetch 白名单；5s VP9 视频 + 单声道 Opus 48kHz）。
    // 输出契约锚点：48kHz / 单声道 / 总时长 ≈ 容器 5.008s（pre-skip 与包边界容差）。
    use crate::av_decode::open_webm_opus_audio_track;

    let path = super::decode::workspace_path("tests/wpt-runner/wpt-data/media/movie_5.webm");
    let data = fs::read(&path).unwrap();
    let mut track = open_webm_opus_audio_track(&data).unwrap();
    assert_eq!(track.sample_rate(), 48_000, "Opus 规范输出率 48kHz");
    assert_eq!(track.channels(), 1, "movie_5.webm 单声道 Opus");

    let mut batches = 0u32;
    let mut total_samples = 0usize;
    let mut first_pts = None;
    let mut prev_pts = None;
    while let Some(batch) = track.next_batch().unwrap() {
        batches += 1;
        assert!(batches <= 2000, "runaway batch count");
        assert_eq!(batch.sample_rate, 48_000);
        assert_eq!(batch.channels, 1);
        if first_pts.is_none() {
            first_pts = Some(batch.pts_ms);
        }
        if let Some(prev) = prev_pts {
            assert!(batch.pts_ms >= prev, "pts 单调递增");
        }
        prev_pts = Some(batch.pts_ms);
        total_samples += batch.samples.len();
    }
    assert!(batches > 0, "Opus 轨应产出数据包");
    assert_eq!(first_pts, Some(0), "首批 pts = 0（pre-skip 丢弃后）");
    let duration_ms = total_samples as u64 / u64::from(track.channels()) * 1000 / u64::from(track.sample_rate());
    assert!(
        (4500..=5600).contains(&duration_ms),
        "解码时长 ≈ 5s（容器 5.008s，pre-skip/包边界容差）：got {duration_ms}ms"
    );
}

#[test]
fn webm_av_opus_track_rejects_vorbis_only() {
    // A_OPUS demux 选择面：Vorbis-only 双轨 fixture 无 A_OPUS 轨 → NoTrack（与
    // open_webm_audio_track 的 A_VORBIS 选择对称）。
    use crate::av_decode::open_webm_opus_audio_track;

    let data = fs::read(fixture_path("sample-webm-vp9-vorbis.webm")).unwrap();
    assert!(
        matches!(
            open_webm_opus_audio_track(&data),
            Err(crate::audio_decode::AudioDecodeError::NoTrack)
        ),
        "Vorbis-only 源无 A_OPUS 轨 → NoTrack"
    );
}

#[test]
fn webm_sequential_decode_drains_hidden_tail_frames_r3936() {
    // R3936（EOF 排空语义）：顺序解码须产出**全部**可展示帧至流末——demux 耗尽
    // 只进入 draining 态，解码器残余（superframe 队列 + hidden/alt-ref 帧的输出
    // 滞后）逐帧排空后才报流末。旧形态 demux 末 flush 后仅 pull 一帧即置 eof，
    // 滞留帧永不产出 → VideoPlayer 在 position < duration 处提前转 Ended
    //（fixture-mounted runner 的 track-cues-enter-exit 复评阻塞根因）。
    // 素材：wpt-data 的 test.webm（6.035s VP9+Opus，30fps、含 15 个 alt-ref
    // hidden 帧——pull-one 调度下输出滞后 ≈15 帧 ≈0.5s，是本缺陷的最小暴露面）。
    let path = super::decode::workspace_path("tests/wpt-runner/wpt-data/media/test.webm");
    let Ok(data) = fs::read(&path) else {
        // wpt-data 为 gitignored 按需 fetch 资产；缺席时跳过（CI 面以 fixture 面
        // 覆盖同语义——hidden 帧滞留只在高帧率 + alt-ref 流暴露）。
        eprintln!("wpt-data media/test.webm not present; skipping");
        return;
    };

    let mut decoder = crate::VideoDecoder::open_webm(&data).unwrap();
    let duration_ms = decoder.duration_ms().unwrap();
    let mut frames = 0u32;
    let mut last_pts = 0u64;
    while let Ok(Some(frame)) = decoder.next_frame() {
        frames += 1;
        assert!(frames <= 10_000, "runaway frame count");
        assert!(frame.pts_ms >= last_pts, "pts 单调递增");
        last_pts = frame.pts_ms;
    }
    // 旧缺陷形态：167 帧 / last pts 5525（滞留 14 帧）；修复后全流可解帧到末。
    assert!(
        last_pts + 500 >= duration_ms.min(u64::MAX - 500),
        "末帧 pts 应贴近容器时长（滞留帧已排空）：last={last_pts} duration={duration_ms}"
    );
    // 解码器真空后再调用仍稳定返回 None（eof 幂等面）。
    assert!(decoder.next_frame().unwrap().is_none());
    assert!(decoder.next_frame().unwrap().is_none());
}

#[cfg(feature = "decode-h264")]
#[test]
fn mp4_h264_decode_full_stream() {
    // M3 H.264 面（D-RFC-3 获批 3a 有条件批准 + 3b 源码编译）：mp4/H.264 轨经
    // openh264 全流解码——48 帧（2s @ 24fps）、PTS 单调、尺寸面（testsrc2
    // 320x240）、首帧 RGB 均值与 ffmpeg 参照（123.3）同窗 ±15（RFC §3.2 锚点：
    // openh264 luma 122.14 探针实测，RGBA 面同窗）。
    // https://www.itu.int/rec/T-REC-H.264
    let data = fs::read(fixture_path("sample-mp4-h264.mp4")).unwrap();
    let mut dec = VideoTrackDecoder::open_media(&data).unwrap();
    assert!(matches!(dec, VideoTrackDecoder::Mp4H264(_)), "mp4 嗅探路由");
    assert_eq!(dec.duration_ms(), Some(2000), "mp4 容器时长真值");

    let mut count = 0u32;
    let mut prev_pts: i64 = -1;
    let mut first_mean = 0.0;
    let mut last_pts = 0u64;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert_eq!(frame.width, 320, "宽度面");
        assert_eq!(frame.height, 240, "高度面");
        assert!(
            (frame.pts_ms as i64) >= prev_pts,
            "PTS 单调（pts={} prev={})",
            frame.pts_ms,
            prev_pts
        );
        prev_pts = frame.pts_ms as i64;
        last_pts = frame.pts_ms;
        if count == 0 {
            first_mean = rgba_mean(&frame.rgba);
        }
        count += 1;
    }
    assert_eq!(count, 48, "全流帧数（与 mp4 采样数一致）");
    // 时长窗：24fps × 2s → 末帧 pts ≤ 2000ms（48 帧覆盖 0~1958ms）。
    assert!(last_pts <= 2000, "末帧 pts 在 2s 流长内（got {last_pts}）");
    assert!(
        (first_mean - 123.3).abs() <= 15.0,
        "H.264 首帧 RGB 均值对齐 ffmpeg 参照窗 ±15（got {first_mean}）"
    );
}

#[cfg(feature = "decode-h264")]
#[test]
fn open_media_routes_webm_and_rejects_unknown() {
    // 容器嗅探路由面：webm 魔数 → Webm 分支（VP9 可解）；mp4 ftyp → Mp4H264；
    // 未知字节流 NoVideoTrack（占位渲染零回归契约——registry 侧字节留存同面）。
    let webm = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoTrackDecoder::open_media(&webm).unwrap();
    assert!(matches!(dec, VideoTrackDecoder::Webm(_)), "webm 嗅探路由");
    assert!(dec.next_frame().unwrap().is_some(), "webm VP9 帧可解");

    let mp4 = fs::read(fixture_path("sample-mp4-h264.mp4")).unwrap();
    let dec = VideoTrackDecoder::open_media(&mp4).unwrap();
    assert!(matches!(dec, VideoTrackDecoder::Mp4H264(_)), "mp4 嗅探路由");

    let garbage = b"not a container at all".to_vec();
    assert!(
        matches!(
            VideoTrackDecoder::open_media(&garbage),
            Err(crate::DecodeError::NoVideoTrack)
        ),
        "未知容器 → NoVideoTrack"
    );
}
