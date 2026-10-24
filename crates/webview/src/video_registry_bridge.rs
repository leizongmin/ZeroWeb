//! `__zwVideoBridge` JS 门面脚本 + `__zw_video_*`/`__zw_ms_*` native 回调注册
//! （自 `video_registry.rs` 按职责拆出——单文件行数纪律；对外路径经该模块
//! `pub use` 再导出保持 `zero_webview::video_registry::*` 不变）。

use std::sync::atomic;

use crate::video_registry::VideoPlayerRegistry;

/// `__zwVideoBridge` JS 门面脚本。native 回调（`__zw_video_*`/`__zw_ms_*`）由 sandbox
/// 回调表随每次 context 重建自动重挂，但门面是一次性 `execute` 的普通全局——js_worker
/// ResetDocument（真导航 reset_context + shim 重装）后会丢失，须由调用方在 shim 重装后
/// 重新执行本脚本补挂（renderer js_worker 路径；learning #25 武装时机同模式）。
/// decode-h264 关闭时仅含基础播放面（无 mse* 方法 → shim isTypeSupported 恒 false，
/// 不虚报）。
pub fn video_bridge_facade_script() -> String {
    let mut s = String::new();
    s.push_str(
        "globalThis.__zwVideoBridge = {\
           play: function (src, nowMs) { return __zw_video_play(src, nowMs | 0) === '1'; },\
           pause: function (src) { __zw_video_pause(src); },\
           seek: function (src, targetMs) { return __zw_video_seek(src, targetMs | 0) === '1'; },\
           currentTime: function (src) { return Number(__zw_video_current_time(src)); },\
           duration: function (src) { return Number(__zw_video_duration(src)); },\
           isPlaying: function (src) { return __zw_video_is_playing(src) === '1'; },\
           isEnded: function (src) { return __zw_video_is_ended(src) === '1'; },\
           isErrored: function (src) { return __zw_video_is_errored(src) === '1'; },\
           setRate: function (src, rate) { __zw_video_set_rate(src, Number(rate)); },\
           setGain: function (src, volume, muted) { __zw_video_set_gain(src, Number(volume), muted ? '1' : '0'); },\
           setLoop: function (src, on) { __zw_video_set_loop(src, on ? '1' : '0'); },\
           mediaMeta: function (src) { return __zw_video_media_meta(src); }\
         };",
    );
    // MSE 面 shim 契约（feature-detect 面）：`__zwVideoBridge.mseCreate` 在位 =
    // 宿主 MSE 通路可用——shim MediaSource/SourceBuffer 据此走真值挂接。
    #[cfg(feature = "decode-h264")]
    s.push_str(
        "globalThis.__zwVideoBridge.mseCreate = function (src) { return __zw_ms_create(src) === '1'; };\
         globalThis.__zwVideoBridge.mseAppend = function (src, b64) { return __zw_ms_append(src, b64) === '1'; };\
         globalThis.__zwVideoBridge.mseAppendAudio = function (src, b64) { return __zw_ms_append_audio(src, b64) === '1'; };\
         globalThis.__zwVideoBridge.mseEnd = function (src) { return __zw_ms_end(src) === '1'; };\
         globalThis.__zwVideoBridge.mseSetDuration = function (src, secs) { __zw_ms_set_duration(src, String(secs)); };\
         globalThis.__zwVideoBridge.mseDims = function (src) { return __zw_ms_dims(src); };",
    );
    s
}

/// 宿主桥 — 在 sandbox 上注册 `__zw_video_*` 回调族并注入 `__zwVideoBridge` JS 对象
///（media-playback M2a 切片 5b）。
///
/// JS 侧 `globalThis.__zwVideoBridge = { play(src, nowMs), pause(src),
/// currentTime(src), isPlaying(src) }`——shim `play()`/`pause()` feature-detect 此对象
/// 走真值路径；未注册（testharness/reftest 沙箱）时对象不存在，shim 回落 headless
/// 路径（372 基线零回归）。键 = 资源绝对 URL（settle 登记同串——shim `src` getter
/// 的 `_zwResolveFetchUrl` 产出）。
///
/// 回调签名 `Fn(&[String]) -> String`（script-sandbox 契约）；秒值以字符串往返。
///
/// M3 切片 2（2026-09-03）：`source_provider` 可选——play 未命中（源未登记）时同步
/// 回调取字节补登记后重评一次（WPT runner 注册竞态消除：宿主侧已知 wpt-data 布局，
/// registry 不感知文件系统；None = 未设置，维持「未登记 → false」原语义零回归）。
/// t8n：门面执行改消费 [`video_bridge_facade_script`]（与 ResetDocument 补挂同源）。
pub fn register_video_bridge_callbacks(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    registry: std::sync::Arc<std::sync::Mutex<VideoPlayerRegistry>>,
    source_provider: Option<crate::MediaSourceProvider>,
    clock: Option<std::sync::Arc<atomic::AtomicU64>>,
) {
    // __zw_video_play(absSrc, nowMs) -> "1"/"0"（bool 字符串避免 JS↔host 布尔歧义）。
    // M2c 后续：audio 回退——video 面未命中（非 webm/纯音频源）时试 audio 条目。
    // M3 扩批 XXV：clock（宿主泵毫秒）在位时 nowMs<=0 翻译为泵时钟现值——shim 无钟
    // 恒传 0，registry play 锚与泵 tick 时钟必须同源（原点错位使首拍 delta=泵全程，
    // 位置瞬间跳到流末——loop 回卷/长加载页面的播放推进失真根因）。
    let reg_play = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_play",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let now_ms: u64 = match clock.as_ref() {
                Some(c) => {
                    let v = args.get(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                    if v > 0 {
                        v as u64
                    } else {
                        c.load(atomic::Ordering::Relaxed)
                    }
                }
                None => args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            };
            let first = {
                let mut reg = reg_play.lock().unwrap_or_else(|e| e.into_inner());
                reg.play(src, now_ms) || reg.audio_play(src, now_ms)
            };
            if !first {
                // M3 切片 2：供给方在位且源未登记 → 同步补登记后重评一次（decode
                // 可达性仍由 open_webm 决定——失败回落 false，字节留存可重试）。
                if let Some(provider) = source_provider.as_ref() {
                    // t8o 返修 D1：登记判定与补登记各自短锁，provider 取字节（生产
                    // 直联 GET，秒级上界）在锁外——registry 锁被 renderer 主循环泵
                    // 每 tick 争用（is_any_playing 门/tick_all），持锁做网络 I/O 使
                    // 主循环与页面 JS 线程在首次 miss 期间整体停摆。调用方（js_worker
                    // 脚本线程）串行，无并发双 fetch 面。
                    let present = {
                        let reg = reg_play.lock().unwrap_or_else(|e| e.into_inner());
                        reg.contains_source(src)
                    };
                    if !present && let Some(bytes) = provider(src) {
                        // audio 判定 strip query/fragment（WPT cache-buster URL——
                        // sound_5.oga?... 以随机数结尾，直接 ends_with 恒 false →
                        // 音频条目永不登记 → 桥 play 恒 miss，audio_loop_* 族超时）。
                        let bare = src.split(['?', '#']).next().unwrap_or(src);
                        let audio_guess = bare.ends_with(".oga") || bare.ends_with(".mp3");
                        let mut reg = reg_play.lock().unwrap_or_else(|e| e.into_inner());
                        // 非 audio 常规路径 move 字节（免全量 body clone 双份峰值——
                        // t8o 审查 D2 的锁外收窄）；audio 双登记才 clone。
                        if audio_guess {
                            reg.register_source(src, bytes.clone());
                            reg.register_audio_source(src, bytes);
                        } else {
                            reg.register_source(src, bytes);
                        }
                    }
                }
            }
            let ok = {
                let mut reg = reg_play.lock().unwrap_or_else(|e| e.into_inner());
                reg.play(src, now_ms) || reg.audio_play(src, now_ms)
            };
            if ok { "1".into() } else { "0".into() }
        }),
    );

    // t8o：登记/播放源的媒体元——桥命中后 shim 升级 settle 占位 (w0h0) 面。
    // "w|h|durMs" 管道串（数字跨 host 边界的既有串契约）；未登记/解码失败
    // "0|0|0"（shim 以 0 判定保持占位态）。
    let reg_meta = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_media_meta",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_meta.lock().unwrap_or_else(|e| e.into_inner());
            match reg.source_media_meta(src) {
                Some((w, h, dur_ms)) => format!("{w}|{h}|{dur_ms}"),
                None => "0|0|0".into(),
            }
        }),
    );

    let reg_pause = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_pause",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let mut reg = reg_pause.lock().unwrap_or_else(|e| e.into_inner());
            reg.pause(src);
            reg.audio_pause(src);
            "1".into()
        }),
    );

    // __zw_video_set_gain(absSrc, volume, muted)——media-elements IDL 联动
    //（volume/muted setter 桥推；音频面增益，video 面 reserved）。
    let reg_gain = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_set_gain",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let volume: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            let muted = args.get(2).map(|s| s == "1").unwrap_or(false);
            reg_gain
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .audio_set_gain(src, volume, muted);
            "1".into()
        }),
    );

    let reg_ct = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_current_time",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_ct.lock().unwrap_or_else(|e| e.into_inner());
            format!("{}", reg.current_time(src))
        }),
    );

    let reg_dur = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_duration",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_dur.lock().unwrap_or_else(|e| e.into_inner());
            match reg.duration(src) {
                Some(d) => format!("{d}"),
                None => "NaN".into(),
            }
        }),
    );

    let reg_rate = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_set_rate",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let rate: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            reg_rate
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_playback_rate(src, rate);
            "1".into()
        }),
    );

    let reg_seek = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_seek",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let target_ms: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut reg = reg_seek.lock().unwrap_or_else(|e| e.into_inner());
            if reg.seek(src, target_ms) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    let reg_playing = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_is_playing",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_playing.lock().unwrap_or_else(|e| e.into_inner());
            if reg.is_playing(src) { "1".into() } else { "0".into() }
        }),
    );

    // M3 扩批 XVI：流末查询（桥 isEnded——语义层 ended 事件驱动源；
    // track-cues-missed 的 onended 断言面）。
    let reg_ended = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_is_ended",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_ended.lock().unwrap_or_else(|e| e.into_inner());
            if reg.is_ended(src) { "1".into() } else { "0".into() }
        }),
    );

    // C3 崩溃面（§11.3）：源错误查询（宿主失效广播/播放级错误）。FIXME(media-process):
    // shim march Error face → 元素 error 事件接线归后续切片（桥真值口先行）。
    let reg_errored = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_is_errored",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let reg = reg_errored.lock().unwrap_or_else(|e| e.into_inner());
            if reg.is_errored(src) { "1".into() } else { "0".into() }
        }),
    );

    // M3 扩批 XXIV：loop 真面（音频 entry 流末回卷；视频面由 shim Ended→play 重头）。
    let reg_loop = std::sync::Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_video_set_loop",
        Box::new(move |args| {
            let src = args.first().map(String::as_str).unwrap_or("");
            let on = args.get(1).map(|s| s == "1").unwrap_or(false);
            reg_loop.lock().unwrap_or_else(|e| e.into_inner()).set_loop(src, on);
            "1".into()
        }),
    );

    // MSE 面（t8n，decode-h264 feature）：MediaSource 生命周期 op 族——shim
    // SourceBuffer.appendBuffer/endOfStream 与 duration 赋值经此入共享缓冲。
    // appendBuffer 二进制经 base64 字符串过桥（sandbox 回调 string-only 契约，
    // atob/btoa 为 Web 平台既有面——shim 侧 btoa 编码，宿主侧 base64_decode）。
    #[cfg(feature = "decode-h264")]
    {
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_create",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                reg_ms.lock().unwrap_or_else(|e| e.into_inner()).mse_create(src);
                "1".into()
            }),
        );
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_append",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                let b64 = args.get(1).map(String::as_str).unwrap_or("");
                match crate::webview::base64_decode(b64) {
                    // t8n 返修 N2：封顶拒绝 → "0"（shim 按既有 append 失败面派
                    // error + updateend）。
                    Ok(bytes) => {
                        let ok = reg_ms.lock().unwrap_or_else(|e| e.into_inner()).mse_append(src, &bytes);
                        if ok { "1".into() } else { "0".into() }
                    }
                    Err(_) => "0".into(),
                }
            }),
        );
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_append_audio",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                let b64 = args.get(1).map(String::as_str).unwrap_or("");
                match crate::webview::base64_decode(b64) {
                    Ok(bytes) => {
                        let ok = reg_ms
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .mse_append_audio(src, &bytes);
                        if ok { "1".into() } else { "0".into() }
                    }
                    Err(_) => "0".into(),
                }
            }),
        );
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_end",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                reg_ms.lock().unwrap_or_else(|e| e.into_inner()).mse_end(src);
                "1".into()
            }),
        );
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_set_duration",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                let secs: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(f64::NAN);
                if secs.is_finite() && secs >= 0.0 {
                    reg_ms
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .mse_set_duration(src, secs);
                }
                "1".into()
            }),
        );
        // __zw_ms_dims(absSrc) -> "WxH"/""——首 append 后的 moov 维度真值探针
        //（shim settle 真值链：loadedmetadata/videoWidth/Height 消费）。
        let reg_ms = std::sync::Arc::clone(&registry);
        sandbox.register_callback(
            "__zw_ms_dims",
            Box::new(move |args| {
                let src = args.first().map(String::as_str).unwrap_or("");
                match reg_ms.lock().unwrap_or_else(|e| e.into_inner()).mse_probe_dims(src) {
                    Some((w, h)) => format!("{w}x{h}"),
                    None => String::new(),
                }
            }),
        );
    }

    // JS 侧门面对象：shim 只认它（feature-detect 单点）。
    let _ = sandbox.execute(&video_bridge_facade_script());
}

#[cfg(all(test, feature = "v8"))]
mod bridge_tests {
    use super::*;
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    fn fixture_bytes() -> Vec<u8> {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("tests/fixtures/media/sample-webm-vp9.webm");
        std::fs::read(p).expect("webm fixture present")
    }

    const SRC: &str = "https://example.com/media/sample-webm-vp9.webm";

    /// 宿主桥端到端：register → JS __zwVideoBridge.play → currentTime 推进 → pause 冻结
    /// → duration NaN/真值两面。真实 fixture 驱动。
    #[test]
    fn video_bridge_js_face_roundtrip() {
        let registry = std::sync::Arc::new(std::sync::Mutex::new(VideoPlayerRegistry::new()));
        registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .register_source(SRC, fixture_bytes());
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).expect("v8 sandbox");
        register_video_bridge_callbacks(&mut sandbox, registry, None, None);

        // 未播放：currentTime 0、duration 2（真值）。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.currentTime('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value
                .replace('\0', ""),
            "0"
        );
        // play → isPlaying true → tick 前位置 0。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.play('https://example.com/media/sample-webm-vp9.webm', 1000))"
                )
                .unwrap()
                .value,
            "true"
        );
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.isPlaying('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value,
            "true"
        );
        // pause → isPlaying false。
        sandbox
            .execute("globalThis.__zwVideoBridge.pause('https://example.com/media/sample-webm-vp9.webm');")
            .unwrap();
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.isPlaying('https://example.com/media/sample-webm-vp9.webm'))"
                )
                .unwrap()
                .value,
            "false"
        );
        // duration 真值面（fixture 2.0s）。
        let dur = sandbox
            .execute("String(globalThis.__zwVideoBridge.duration('https://example.com/media/sample-webm-vp9.webm'))")
            .unwrap()
            .value;
        assert!(dur.starts_with('2'), "duration 应为 2 秒真值，got {dur}");
        // seek 面：seek(1000ms) → currentTime ≥ 1s（精确 seek 真值）。
        assert_eq!(
            sandbox
                .execute(
                    "String(globalThis.__zwVideoBridge.seek('https://example.com/media/sample-webm-vp9.webm', 1000))"
                )
                .unwrap()
                .value,
            "true"
        );
        let ct = sandbox
            .execute("String(globalThis.__zwVideoBridge.currentTime('https://example.com/media/sample-webm-vp9.webm'))")
            .unwrap()
            .value;
        let ct: f64 = ct.parse().unwrap_or(0.0);
        assert!(ct >= 1.0, "seek(1000) 后 currentTime ≥ 1s，got {ct}");
        // 未登记源：play false。
        assert_eq!(
            sandbox
                .execute("String(globalThis.__zwVideoBridge.play('https://x/nope.webm', 0))")
                .unwrap()
                .value,
            "false"
        );
    }

    /// MSE 桥端到端（t8n，decode-h264+v8）：mseCreate → mseAppend（base64 过桥）
    /// → play 真值 → duration 显式面经 __zwVideoBridge.duration 读回。
    #[cfg(feature = "decode-h264")]
    #[test]
    fn video_bridge_mse_face_roundtrip() {
        const MSE_SRC: &str = "blob:https://example.com/zw-mse-test-1";
        let registry = std::sync::Arc::new(std::sync::Mutex::new(VideoPlayerRegistry::new()));
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).expect("v8 sandbox");
        register_video_bridge_callbacks(&mut sandbox, registry, None, None);
        // mseCreate + 显式 duration 读回（NaN 面 → 赋值 → 真值）。
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.mseCreate('{MSE_SRC}'))"))
                .unwrap()
                .value,
            "true"
        );
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.duration('{MSE_SRC}'))"))
                .unwrap()
                .value,
            "NaN"
        );
        sandbox
            .execute(&format!("__zwVideoBridge.mseSetDuration('{MSE_SRC}', 12.5);"))
            .unwrap();
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.duration('{MSE_SRC}'))"))
                .unwrap()
                .value,
            "12.5"
        );
        // appendBuffer 过桥（base64）→ play 真值（等待面也建 player——tick 推进在
        // registry 单测已钉，此处钉桥面往返）。
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop();
        p.push("tests/fixtures/media/sample-mp4-h264.mp4");
        let data = std::fs::read(p).expect("mp4 fixture present");
        let b64 = crate::webview::base64_encode(&data);
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.mseAppend('{MSE_SRC}', '{b64}'))"))
                .unwrap()
                .value,
            "true"
        );
        sandbox
            .execute(&format!("__zwVideoBridge.mseEnd('{MSE_SRC}');"))
            .unwrap();
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.play('{MSE_SRC}', 0))"))
                .unwrap()
                .value,
            "true"
        );
        assert_eq!(
            sandbox
                .execute(&format!("String(__zwVideoBridge.isPlaying('{MSE_SRC}'))"))
                .unwrap()
                .value,
            "true"
        );
    }
}
