//! Web Audio 最小面宿主注册表（media-audio M3 切片 2，D1 批复 / D-WA-2 NullSink 先行）。
//!
//! [`WebAudioRegistry`] 持有页面级 `WebAudioContext`（zero-media `webaudio` 模块），
//! 暴露 JS 宿主桥回调（`__zwWA*`——`__zwVideoBridge` 同款 feature-detect 单点模式）
//! 与音频泵推进（tab_worker `audio_advance_all` 同节拍；headless NullSink 可观测）。
//!
//! RFC 简化面（`docs/specs/web-audio-audiocontext-minimal-face-spec-rfc.md` §3）：
//! 每页面一上下文；`createOscillator`/`createGain`/`destination`/`start`/`stop`
//! 经字符串参数桥（u64/f32 文本——bool 用 "1"/"0" 避 JS↔host 布尔歧义）。

use std::sync::{Arc, Mutex};

use zero_media::{AudioSink, OscillatorType, WebAudioContext};

/// 上下文时钟（构造起的单调毫秒——宿主注入，测试可快进）。
pub struct WebAudioRegistry {
    ctx: WebAudioContext,
    sink: zero_media::NullSink,
    sink_started: bool,
    epoch: std::time::Instant,
    /// JS 桥句柄 → 上下文振荡器句柄（节点对象 ↔ 源映射）。
    next_node: u64,
    nodes: std::collections::HashMap<u64, usize>,
}

impl Default for WebAudioRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WebAudioRegistry {
    /// 新建空注册表（NullSink 端点——CI/WPT 环境强制面；CpalSink 设备面挂
    /// M1 真出声切片）。
    pub fn new() -> Self {
        Self {
            ctx: WebAudioContext::new(),
            sink: zero_media::NullSink::new(),
            sink_started: false,
            epoch: std::time::Instant::now(),
            next_node: 0,
            nodes: std::collections::HashMap::new(),
        }
    }

    /// 创建振荡器节点（返回 JS 侧句柄）。
    pub fn create_oscillator(&mut self, osc_type: OscillatorType, frequency: f32) -> u64 {
        let h = self.ctx.create_oscillator(osc_type, frequency);
        self.next_node += 1;
        self.nodes.insert(self.next_node, h);
        self.next_node
    }

    /// 设置振荡器频率（`frequency` AudioParam 当前值面）。
    pub fn set_frequency(&mut self, node: u64, freq: f32) -> bool {
        match self.nodes.get(&node) {
            Some(&h) => {
                if let Some(o) = self.ctx.oscillator_mut(h) {
                    o.frequency = freq.max(0.0);
                }
                true
            }
            None => false,
        }
    }

    /// 调度 start（when 为相对 epoch 毫秒——JS 面换算）。
    pub fn start(&mut self, node: u64, when_ms: u64) -> bool {
        let now = self.now_ms();
        match self.nodes.get(&node) {
            Some(&h) => {
                if let Some(o) = self.ctx.oscillator_mut(h) {
                    o.start_at(now.saturating_add(when_ms));
                }
                true
            }
            None => false,
        }
    }

    /// 调度 stop。
    pub fn stop(&mut self, node: u64, when_ms: u64) -> bool {
        let now = self.now_ms();
        match self.nodes.get(&node) {
            Some(&h) => {
                if let Some(o) = self.ctx.oscillator_mut(h) {
                    o.stop_at(now.saturating_add(when_ms));
                }
                true
            }
            None => false,
        }
    }

    /// 是否存在活跃源（宿主泵门禁——无活跃源零开销）。
    pub fn is_any_active(&self) -> bool {
        self.ctx.active_count(self.now_ms()) > 0
    }

    /// 音频泵推进（tab_worker 同节拍调用；返回是否有写入）。
    pub fn advance(&mut self, now_ms: u64) -> bool {
        if self.ctx.active_count(now_ms) == 0 {
            return false;
        }
        if !self.sink_started {
            let fmt = self.ctx.format();
            let _ = self.sink.start(fmt);
            self.sink_started = true;
        }
        // tick 帧数：按 1ms 泵节拍换算（48000Hz → 48 帧/ms）。
        let frames = (zero_media::WEBAUDIO_SAMPLE_RATE / 1000) as usize;
        match self.ctx.advance(now_ms, frames, &mut self.sink) {
            Ok(active) => active > 0,
            Err(_) => false,
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// NullSink 可观测面（e2e/CI 断言：写入帧数 + 过零率）。
    pub fn frames_written(&self) -> u64 {
        self.sink.frames_written()
    }

    /// 过零率（Hz 代理）——440Hz sine ≈ 880（2×频率）。
    pub fn zero_crossings_per_second(&self) -> Option<f64> {
        self.sink.zero_crossings_per_second()
    }
}

/// 注册 JS 宿主桥回调（`__zwWA*` 面——webview 构建/测试同款 late-injection）。
///
/// 契约（字符串参数/返回，避免 JS↔host 类型歧义）：
/// - `__zw_wa_create_osc(type, freq)` → node handle（十进制串）
/// - `__zw_wa_start(node, whenMs)` / `__zw_wa_stop(node, whenMs)` → "1"
/// - `__zw_wa_set_freq(node, freq)` → "1"
/// - `__zw_wa_active()` → "1"/"0"（泵门禁查询面）
pub fn register_webaudio_bridge_callbacks(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    registry: Arc<Mutex<WebAudioRegistry>>,
) {
    let reg = Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_wa_create_osc",
        Box::new(move |args| {
            let t = args.first().map(String::as_str).unwrap_or("sine");
            let freq: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(440.0);
            let osc_type = match t {
                "square" => OscillatorType::Square,
                "sawtooth" => OscillatorType::Sawtooth,
                "triangle" => OscillatorType::Triangle,
                _ => OscillatorType::Sine,
            };
            let mut reg = reg.lock().unwrap_or_else(|e| e.into_inner());
            format!("{}", reg.create_oscillator(osc_type, freq))
        }),
    );

    let reg = Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_wa_start",
        Box::new(move |args| {
            let node: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let when: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut reg = reg.lock().unwrap_or_else(|e| e.into_inner());
            reg.start(node, when);
            "1".into()
        }),
    );

    let reg = Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_wa_stop",
        Box::new(move |args| {
            let node: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let when: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut reg = reg.lock().unwrap_or_else(|e| e.into_inner());
            reg.stop(node, when);
            "1".into()
        }),
    );

    let reg = Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_wa_set_freq",
        Box::new(move |args| {
            let node: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let freq: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(440.0);
            let mut reg = reg.lock().unwrap_or_else(|e| e.into_inner());
            reg.set_frequency(node, freq);
            "1".into()
        }),
    );

    let reg = Arc::clone(&registry);
    sandbox.register_callback(
        "__zw_wa_active",
        Box::new(move |_args| {
            let reg = reg.lock().unwrap_or_else(|e| e.into_inner());
            if reg.is_any_active() { "1".into() } else { "0".into() }
        }),
    );
}
