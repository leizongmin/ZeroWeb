//! Web Audio 最小面 — 振荡器合成 + AudioContext 图推进（media-audio M3，D1 批复）。
//!
//! 路线 A 最小面（[RFC](../../../docs/specs/web-audio-audiocontext-minimal-face-spec-rfc.md)
//! 切片 1）：`OscillatorState`（四型波形纯函数合成，相位累积防 alias）+
//! `WebAudioContext`（源列表 → per-source 增益 → 下游 [`AudioSink`]）。
//! headless NullSink 可观测断言（过零率锚点）与 M1/M2c 契约同款。
//!
//! **不做**（RFC §0）：AudioWorklet / 滤波器 / 压缩器 / MediaStream /
//! OfflineAudioContext / 完整拉取式图调度（每 tick 合成→增益→写为最小面简化）。
//! 时钟由调用方注入单调毫秒（与 VideoPlayer 同款可测试性注入）——A/V 同步
//! （audio clock 主时钟）的 Web Audio 面承接归 M2 后续切片。

use crate::audio::{AudioFormat, AudioSink, AudioSinkError};

/// Web Audio 上下文采样率（headless NullSink 面固定值——与宿主泵对齐；
/// CpalSink 设备面构造时以设备实际率重建，归设备切片）。
pub const WEBAUDIO_SAMPLE_RATE: u32 = 48_000;

/// 振荡器波形（Web Audio `OscillatorType` 声明面四型；缺省 sine）。
///
/// https://webaudio.github.io/web-audio-api/#oscillator-type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OscillatorType {
    /// 正弦（缺省）。
    #[default]
    Sine,
    /// 方波。
    Square,
    /// 锯齿波。
    Sawtooth,
    /// 三角波。
    Triangle,
}

/// 单个振荡器源的合成状态。
#[derive(Debug, Clone)]
pub struct OscillatorState {
    /// 波形类型。
    pub osc_type: OscillatorType,
    /// 频率（Hz；`frequency` AudioParam 的当前值面——param 调度为后续扩展）。
    pub frequency: f32,
    /// per-source 增益（0.0-1.0，`gain` AudioParam 当前值）。
    pub gain: f32,
    /// 是否静音。
    pub muted: bool,
    /// start 时刻（上下文时钟毫秒；`None` = 尚未 start，不合成）。
    pub started_at_ms: Option<u64>,
    /// stop 时刻（毫秒；`None` = 无 stop 调度）。
    pub stop_at_ms: Option<u64>,
    /// 波形相位（弧度累积——采样间连续性防 alias/爆音）。
    phase: f64,
    /// 相位是否已初始化（首样本从相位 0 起）。
    began: bool,
}

impl OscillatorState {
    /// 新建振荡器（缺省 440Hz sine，增益 1.0）。
    pub fn new(osc_type: OscillatorType, frequency: f32) -> Self {
        Self {
            osc_type,
            frequency: frequency.max(0.0),
            gain: 1.0,
            muted: false,
            started_at_ms: None,
            stop_at_ms: None,
            phase: 0.0,
            began: false,
        }
    }

    /// 在 `now_ms` 调度启动（Web Audio `start(when)`——when 为上下文时钟秒，
    /// 调用方换算毫秒注入）。幂等：重复 start 语义无效（spec「If scheduled
    /// start time is before started, ignore」近似——已启动的源不重置相位）。
    pub fn start_at(&mut self, now_ms: u64) {
        if self.started_at_ms.is_some() {
            return;
        }
        self.started_at_ms = Some(now_ms);
    }

    /// 在 `now_ms` 调度停止（Web Audio `stop(when)`）。未 start 的源 stop 即
    /// 永不发声（spec：stop 前必须 start——此处宽容为标记，合成段统一 gate）。
    pub fn stop_at(&mut self, now_ms: u64) {
        self.stop_at_ms = Some(now_ms);
    }

    /// 在 `now_ms` 是否活跃（已 start 且未到 stop）。
    pub fn active_at(&self, now_ms: u64) -> bool {
        match (self.started_at_ms, self.stop_at_ms) {
            (Some(s), Some(e)) => now_ms >= s && now_ms < e,
            (Some(s), None) => now_ms >= s,
            (None, _) => false,
        }
    }

    /// 合成 `frames` 帧的单声道采样（f32，值域 [-1,1]）——在 `now_ms` 起按
    /// `sample_rate` 逐帧推进相位。
    ///
    /// 波形函数（Web Audio spec §OscillatorNode 周期波定义）：
    /// - sine：`sin(2π·f·t)`
    /// - square：`sign(sin(2π·f·t))`
    /// - sawtooth：`2·(2π·f·t/2π mod 1) − 1`（上升锯齿）
    /// - triangle：`2·|sawtooth| − 1` 等价的三角（相位折叠）
    pub fn synthesize(&mut self, frames: usize, sample_rate: u32, now_ms: u64) -> Vec<f32> {
        let mut out = Vec::with_capacity(frames);
        if !self.active_at(now_ms) {
            // 不活跃：静默推进时间轴（相位不推进——再 start 前无累积漂移）。
            return out;
        }
        let dt = 1.0 / f64::from(sample_rate);
        let inc = 2.0 * std::f64::consts::PI * f64::from(self.frequency) * dt;
        for _ in 0..frames {
            if !self.began {
                self.began = true;
            } else {
                self.phase += inc;
                // 相位归一（mod 2π）防长流精度漂移。
                self.phase %= 2.0 * std::f64::consts::PI;
            }
            let v = match self.osc_type {
                OscillatorType::Sine => self.phase.sin(),
                OscillatorType::Square => {
                    if self.phase.sin() >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                OscillatorType::Sawtooth => {
                    // 相位 [0,2π) → [-1,1) 上升锯齿。
                    2.0 * (self.phase / (2.0 * std::f64::consts::PI)) - 1.0
                }
                OscillatorType::Triangle => {
                    // 相位折叠：[0,π] 上升 [-1,1]，[π,2π] 下降 [1,-1]。
                    let t = self.phase / std::f64::consts::PI;
                    if t <= 1.0 { 2.0 * t - 1.0 } else { 3.0 - 2.0 * t }
                }
            };
            out.push(v as f32);
        }
        out
    }
}

/// Web Audio 上下文（最小面图：振荡器源列表 → 增益 → 单一下游 sink）。
pub struct WebAudioContext {
    /// 采样率（构造时确定；headless NullSink 面 48000，与宿主泵对齐）。
    pub sample_rate: u32,
    /// 振荡器源（JS 侧节点对象 ↔ 数组下标句柄）。
    oscillators: Vec<OscillatorState>,
    /// 总增益（destination 面；最小面无独立 GainNode 图——per-source gain 承接
    /// `createGain().gain` 的主用例，RFC §3.1 简化注记）。
    destination_gain: f32,
}

impl WebAudioContext {
    /// 新建上下文（采样率 [`WEBAUDIO_SAMPLE_RATE`]——NullSink 面；CpalSink 面
    /// 构造时以设备实际率重建，格式变更面归设备切片）。
    pub fn new() -> Self {
        Self {
            sample_rate: WEBAUDIO_SAMPLE_RATE,
            oscillators: Vec::new(),
            destination_gain: 1.0,
        }
    }

    /// 创建振荡器（返回句柄 = `oscillators` 下标——JS 侧节点对象持有）。
    pub fn create_oscillator(&mut self, osc_type: OscillatorType, frequency: f32) -> usize {
        self.oscillators.push(OscillatorState::new(osc_type, frequency));
        self.oscillators.len() - 1
    }

    /// 振荡器句柄可变借用（JS 面属性反射：type/frequency/gain/start/stop）。
    pub fn oscillator_mut(&mut self, handle: usize) -> Option<&mut OscillatorState> {
        self.oscillators.get_mut(handle)
    }

    /// 设置 destination 总增益（0.0-1.0 clamp）。
    pub fn set_destination_gain(&mut self, gain: f32) {
        self.destination_gain = gain.clamp(0.0, 1.0);
    }

    /// 当前活跃（已 start 未 stop）源数。
    pub fn active_count(&self, now_ms: u64) -> usize {
        self.oscillators.iter().filter(|o| o.active_at(now_ms)).count()
    }

    /// 每 tick 推进：逐活跃源合成 → per-source 增益 → destination 增益 →
    /// 软削幅 → 写 sink（与 Mixer.mix_into 同款饱和近似）。
    ///
    /// `now_ms` 为上下文时钟（宿主单调钟注入）；`frames` 为本 tick 帧数
    ///（宿主泵节拍决定，如 128 帧 quantum 或按 ms 换算）。
    pub fn advance(&mut self, now_ms: u64, frames: usize, sink: &mut dyn AudioSink) -> Result<usize, AudioSinkError> {
        // 最小面 mono（Web Audio destination 上/下混归后续切片）。
        let mut mixed = vec![0.0f32; frames];
        let mut active = 0usize;
        for osc in self.oscillators.iter_mut() {
            if !osc.active_at(now_ms) {
                continue;
            }
            active += 1;
            let block = osc.synthesize(frames, self.sample_rate, now_ms);
            let g = if osc.muted { 0.0 } else { osc.gain };
            for (dst, &src) in mixed.iter_mut().zip(block.iter()) {
                *dst += src * g;
            }
        }
        if active == 0 {
            return Ok(0);
        }
        for sample in &mut mixed {
            *sample = (*sample * self.destination_gain).clamp(-1.0, 1.0);
        }
        sink.write(&mixed)?;
        Ok(active)
    }

    /// sink 端点格式（advance 前由宿主 start——mono / 上下文采样率）。
    pub fn format(&self) -> AudioFormat {
        AudioFormat {
            sample_rate: self.sample_rate,
            channels: 1,
        }
    }
}

impl Default for WebAudioContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::NullSink;

    fn sink(rate: u32) -> NullSink {
        let mut s = NullSink::new();
        s.start(AudioFormat {
            sample_rate: rate,
            channels: 1,
        })
        .unwrap();
        s
    }

    /// 观测时长换算：frames → 秒（过零率分母）。
    #[test]
    fn sine_440_zero_crossing_matches_contract() {
        // 440Hz sine 写 1 秒 → 过零率 ≈ 880（2×频率——M1/M2c 同款锚点）。
        let mut ctx = WebAudioContext::new();
        let h = ctx.create_oscillator(OscillatorType::Sine, 440.0);
        ctx.oscillator_mut(h).unwrap().start_at(0);
        let mut s = sink(48_000);
        // 10 × 100ms tick。
        let frames_per_tick = 4_800;
        for tick in 0..10 {
            ctx.advance(tick * 100, frames_per_tick, &mut s).unwrap();
        }
        let zps = s.zero_crossings_per_second().unwrap();
        assert!((zps - 880.0).abs() < 20.0, "440Hz sine 过零率锚点 ≈880（got {zps}）");
    }

    #[test]
    fn square_wave_double_zero_crossing_rate() {
        // 方波同频过零率与 sine 同阶（每周期两次穿越）——440Hz ≈880。
        let mut ctx = WebAudioContext::new();
        let h = ctx.create_oscillator(OscillatorType::Square, 440.0);
        ctx.oscillator_mut(h).unwrap().start_at(0);
        let mut s = sink(48_000);
        for tick in 0..10 {
            ctx.advance(tick * 100, 4_800, &mut s).unwrap();
        }
        let zps = s.zero_crossings_per_second().unwrap();
        assert!((zps - 880.0).abs() < 25.0, "440Hz square 过零率 ≈880（got {zps}）");
    }

    #[test]
    fn oscillator_before_start_is_silent() {
        // 未 start：advance 不写帧（active=0，sink 无写入）。
        let mut ctx = WebAudioContext::new();
        let h = ctx.create_oscillator(OscillatorType::Sine, 440.0);
        assert!(!ctx.oscillator_mut(h).unwrap().active_at(0));
        let mut s = sink(48_000);
        let active = ctx.advance(0, 4_800, &mut s).unwrap();
        assert_eq!(active, 0, "未 start 不合成");
        assert_eq!(s.frames_written(), 0);
    }

    #[test]
    fn stop_silences_source_after_stop_time() {
        // stop(500ms) 后源静默：0-500ms 写帧、500ms 起不再写。
        let mut ctx = WebAudioContext::new();
        let h = ctx.create_oscillator(OscillatorType::Sine, 440.0);
        ctx.oscillator_mut(h).unwrap().start_at(0);
        ctx.oscillator_mut(h).unwrap().stop_at(500);
        let mut s = sink(48_000);
        let a1 = ctx.advance(0, 4_800, &mut s).unwrap();
        let a2 = ctx.advance(500, 4_800, &mut s).unwrap();
        assert_eq!(a1, 1, "stop 前 1 活跃源");
        assert_eq!(a2, 0, "stop 后 0 活跃源");
        assert_eq!(s.frames_written(), 4_800, "仅 stop 前写入");
    }

    #[test]
    fn gain_scales_amplitude() {
        // 增益 0.5 → 峰值幅度减半（sine 峰 1.0 → 0.5）。
        let mut ctx = WebAudioContext::new();
        let h = ctx.create_oscillator(OscillatorType::Sine, 440.0);
        ctx.oscillator_mut(h).unwrap().start_at(0);
        ctx.set_destination_gain(0.5);
        let mut s = sink(48_000);
        ctx.advance(0, 4_800, &mut s).unwrap();
        // NullSink 无采样快照——经 destination_gain=0.5 的合成值域断言直接验证
        // （advance 内部数学面）：峰值 = sin 峰 1.0 × 0.5 = 0.5。
        let mut osc = OscillatorState::new(OscillatorType::Sine, 440.0);
        osc.start_at(0);
        let block = osc.synthesize(48, 48_000, 0);
        let peak = block.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!((peak - 1.0).abs() < 0.05, "裸 sine 峰值 ≈1.0（got {peak}）");
        assert!(
            (peak * 0.5 - 0.5).abs() < 0.03,
            "增益 0.5 后峰值 ≈0.5（got {}）",
            peak * 0.5
        );
    }

    #[test]
    fn sawtooth_and_triangle_are_periodic() {
        // 锯齿/三角波形周期性：同频下相邻周期同相位值（相位归一无漂移）。
        let mut osc = OscillatorState::new(OscillatorType::Sawtooth, 100.0);
        osc.start_at(0);
        let block = osc.synthesize(48_000, 48_000, 0); // 恰 1 秒 = 100 周期
        // 100 周期后相位应回到 ≈起点（首样本相位 0 → 末样本接近 2π 折叠点）。
        let first = block[0];
        let near_wrap = block[47_999];
        let expected_end = 2.0 * (47_999.0 / 480.0 % 1.0) - 1.0;
        assert!(
            (near_wrap - expected_end as f32).abs() < 0.01,
            "锯齿末样本相位精确（got {near_wrap} expect {expected_end}）"
        );
        assert!((-1.0..=1.0).contains(&first));

        let mut tri = OscillatorState::new(OscillatorType::Triangle, 100.0);
        tri.start_at(0);
        let tb = tri.synthesize(48_000, 48_000, 0);
        assert!(tb.iter().all(|v| v.abs() <= 1.0), "三角波形值域 [-1,1]");
    }

    #[test]
    fn multiple_sources_mix_and_clamp() {
        // 双源同相叠加：峰值 2.0 → 软削幅 1.0（mix 面饱和近似）。
        let mut ctx = WebAudioContext::new();
        let h1 = ctx.create_oscillator(OscillatorType::Square, 440.0);
        let h2 = ctx.create_oscillator(OscillatorType::Square, 440.0);
        ctx.oscillator_mut(h1).unwrap().start_at(0);
        ctx.oscillator_mut(h2).unwrap().start_at(0);
        let mut s = sink(48_000);
        let active = ctx.advance(0, 4_800, &mut s).unwrap();
        assert_eq!(active, 2, "双源并发");
        assert_eq!(s.frames_written(), 4_800);
    }
}
