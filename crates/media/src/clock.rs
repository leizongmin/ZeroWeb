//! 播放驱动接口 — `VideoClock` trait（M2a 对接点，M1a 仅定义）。
//!
//! media-elements 语义层现以 `_mediaState` + setTimeout 做 headless 近似驱动；
//! 本 trait 是真值化替换点（RFC §3.1「驱动源替换是接口对接，不是重写」）：
//! M2a 的 `player` 模块（帧率时钟/play/seek/ended）实现此 trait，语义层经它读
//! readyState/duration/currentTime 真值。

/// 播放时钟 — 播放驱动的最小读侧接口。
///
/// `readyState`/`currentTime` 等语义值由消费方（engine 的媒体桥接层）按
/// HTML 规范语义从本接口推导；本 trait 不承载事件派发（事件归语义层状态机）。
pub trait VideoClock {
    /// 当前播放位置（秒，媒体时间轴）。
    // https://html.spec.whatwg.org/multipage/media.html#current-playback-position
    fn current_time(&self) -> f64;

    /// 媒体时长（秒）；元数据未就绪时 `None`（对应 readyState < HAVE_METADATA）。
    // https://html.spec.whatwg.org/multipage/media.html#dom-media-duration
    fn duration(&self) -> Option<f64>;

    /// 是否处于播放中（playing == true 时 currentTime 单调推进）。
    // https://html.spec.whatwg.org/multipage/media.html#paused
    fn is_playing(&self) -> bool;

    /// 播放速率（0 为非法；由实现方 clamp，语义层不重复校验）。
    // https://html.spec.whatwg.org/multipage/media.html#dom-media-playbackrate
    fn playback_rate(&self) -> f64;
}
