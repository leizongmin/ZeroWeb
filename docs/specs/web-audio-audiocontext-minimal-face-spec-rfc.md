# Spec RFC：Web Audio 最小面（AudioContext / Oscillator / GainNode 可行性评估）

**版本**：v1.0
**日期**：2026-09-01
**状态**：Proposed（media-audio M3 门控项——**实施与否待用户批准**（goal 文档 DC-4：
RFC 完成即满足，不批准不影响该 goal DONE））
**关联**：[media-audio goal](../goal/archive/media-audio.md)（M3 门控项；goal 已完成，2026-09-05 归档）·
[media-playback goal](../goal/archive/media-playback/master.md)（解码/播放面已落地）·
[video-decode-playback-spec-rfc.md](video-decode-playback-spec-rfc.md)（路线 C 先例）

---

## 0. 执行摘要

- **目标**：评估 `AudioContext`（Web Audio API）最小面（`createOscillator` /
  `GainNode` / `destination`）在 ZeroWeb 现有音频底座上的可行性与工程量，形成
  是否实施的决策依据。
- **结论**：**可行，工程量小**。`zero-media` 已具备全部底层组件——合成源
  （Oscillator = Rust 侧正弦/波形发生器，零解码）、增益（`Mixer` per-source
  volume/muted）、输出（`AudioSink` trait：`NullSink` 可观测 / `CpalSink` 设备）。
  最小面 ≈ 一个 `AudioContext` 宿主对象（js_dom_shim 桥 + Rust 侧图管理） +
  三个节点类型的桥接，预估 **1~2 个切片**（≈300~500 行 Rust + ≈200 行 shim）。
- **推荐**：**批准实施最小面**（渲染/合成域高频依赖：游戏/可视化/提示音）；
  或**维持不实施**（当前 WPT/产品面无 Web Audio 需求驱动）。两项均不阻塞
  media-audio goal DONE。
- **不做**：AudioWorklet、ConvolverNode/PannerNode（空间音频）、MediaStreamAudioSource
  （getUserMedia 域）、OfflineAudioContext、AudioBuffer 解码复用（解码面已有独立路径）、
  完整 Web Audio 图调度（拉取式 subgraph 调度为最小面简化）。

---

## 1. 背景与现状

### 1.1 规范面（Web Audio API）

最小可观测面 = 用户可验证的 JS 语义闭环：

| API | 语义 | WPT 断言面（`webaudio/the-audio-api`） |
|-----|------|------|
| `new AudioContext()` | 构造（state 恒 `'running'` headless；sampleRate 真值） | 构造/state/sampleRate 属性 |
| `ctx.destination` | 输出端点节点（`AudioDestinationNode`） | identity/存在性 |
| `ctx.sampleRate` | 输出采样率（device 或 NullSink 固定值） | 数值断言 |
| `ctx.createOscillator()` | 振荡器源（`type` sine/square/sawtooth/triangle + `frequency`） | 属性反射 + start/stop 时序 |
| `ctx.createGain()` | 增益节点（`gain` AudioParam） | 属性反射 + 透传 |
| `osc.connect(node).connect(ctx.destination)` | 图连接（返回目标节点——链式） | connect 返回值 |
| `osc.start(t)` / `osc.stop(t)` | 播放控制 | 事件/状态面 |

### 1.2 ZeroWeb 现状（2026-09-01 实测）

- **零 Web Audio 面**：js_dom_shim 无 `AudioContext`/`BaseAudioContext` 任何分支
  （`grep AudioContext` 零命中）。
- **底层组件已齐**（`zero-media`，media-audio M0/M1 交付）：
  - `AudioSink` trait（start/write f32 交错/pause/resume/underrun）+ `NullSink`
    （headless 可观测：frames_written/zero_crossings）+ `CpalSink`（设备面，
    feature-gated）；
  - `Mixer`（N 源 → 1 sink，per-source volume/muted 增益，软削幅）——7 单测常驻；
  - 播放管线宿主接线（tab_worker 音频泵 `audio_advance_all` 同节拍模式）。
- **缺**：振荡器合成源（Rust 侧波形发生器——无解码依赖，纯函数）；Web Audio 图
  的 JS 宿主面与 Rust 侧节点管理。

---

## 2. 方案对比

| 维度 | A. 最小面（推荐范围） | B. 完整 Web Audio 图 | C. 不实施 |
|------|------|------|------|
| JS 面 | AudioContext + Oscillator/Gain/Destination + connect/start/stop | 全节点族（DynamicsCompressor/BiquadFilter/Delay…） | 无 |
| Rust 面 | 简单图（源列表 + 增益 + 单 sink），每音频 tick 合成→增益→写 sink | 拉取式子图调度（topo 遍历 + 每节点 processing quantum 128 帧） | — |
| 工程量 | 1~2 切片（≈500~700 行） | 专项立项（数周量级） | 0 |
| 覆盖需求 | 提示音/游戏音效/可视化演示（createOscillator 主导） | 专业音频应用 | — |
| 风险 | 低（复用既有 sink/mixer/pump 底座；headless NullSink 可观测断言） | 中（调度正确性 + realtime quantum 语义） | WPT webaudio 全零、产品域空缺 |
| 模拟退让 | NullSink 阶段值域断言（过零率锚点——与 M1 契约同款） | 同 | — |

**决策点**：最小面已覆盖 createOscillator 主导的常见用例；完整图（滤波器/压缩器/
worklet）属专业音频域，按需另行立项。

---

## 3. 最小面技术设计（路线 A）

### 3.1 架构

```
JS（js_dom_shim）                    Rust（zero-media 新模块 webaudio）
┌────────────────────┐   宿主桥    ┌──────────────────────────────┐
│ AudioContext 门面   │ ────────→ │ WebAudioContext              │
│  createOscillator  │  __zwWA*   │  sources: Vec<OscillatorState>│
│  createGain        │            │  gain: f32 / muted: bool      │
│  destination       │            │  sample_rate: u32             │
│  sampleRate/state  │            │  advance(now, &mut dyn AudioSink)│
└────────────────────┘            └──────────────────────────────┘
                                          │ 每 tick 合成 → 增益 → 写
                                          ▼
                                   NullSink（CI 可观测）/ CpalSink（设备）
```

- **JS 侧**（镜像 `__zwVideoBridge` 同款 feature-detect 单点模式）：
  `AudioContext` 工厂对象（`createOscillator`/`createGain`/`destination`/`sampleRate`/
  `state`）+ 节点对象（`connect` 链式返回、`frequency`/`gain` 参数反射、`start`/`stop`）。
- **Rust 侧**：`zero-media::webaudio`——`OscillatorState`（type/frequency/start/stop
  时序 + 波形函数 f32 采样生成）、`WebAudioContext::advance(now_ms, sink)`（宿主
  音频泵每 tick：逐活跃源合成 → Mixer 增益 → sink.write）。
- **接线**：复用 tab_worker 音频泵节拍（`is_any_playing` 快速门 + `audio_advance_all`
  同款）；`setVideoPlayers` 同款 late-injection 注入上下文句柄。

### 3.2 时序与语义锚点（headless 简化记录）

- `state`：headless 恒 `'running'`（无 autoplay 政策域——浏览器 `suspended` 态归
  用户手势策略，远程面不模拟，注释记录）。
- `start(when)`：`when` 为相对 AudioContext 时钟秒；headless 用注入单调时钟换算
  （与 VideoPlayer tick 同款可测试性注入）；`stop(when)` 后源静默。
- 波形：sine/square/sawtooth/triangle 四型纯函数采样（`2π·f·t` 相位累积防 alias）。
- 可观测断言：NullSink 过零率锚点（440Hz sine ≈880——与 M1/M2c 契约同款）。

### 3.3 切片划分

1. **切片 1（zero-media）**：`webaudio` 模块——OscillatorState 合成 + WebAudioContext
   advance + NullSink 过零率单测（≈250 行）。
2. **切片 2（shim + webview 接线）**：JS 门面 + 宿主桥 + 音频泵挂接 + e2e
   （`createOscillator → destination` 全链 NullSink 可观测；WPT webaudio 可执行
   子集评估导入）（≈300 行 shim + ≈100 行 webview）。

### 3.4 测试与质量

- 每切片带单测（CLAUDE.md 测试资产化）；上游 WPT `webaudio/the-audio-api/
  the-audiocontext-interface` 属性面用例经 `make import-wpt` 流程评估导入
  （依赖真音频时序的入 skip list 注明）。
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings`
  + `make test` 全绿；路线 C 保持（零 C 依赖——纯 Rust 合成，无新外部依赖）。

---

## 4. 风险与回滚

| 风险 | 缓解 |
|------|------|
| 实时设备输出延迟/爆音（CpalSink 面） | 最小面先 NullSink 可观测；设备面沿用 M1 CpalSink 既有流控 |
| shim 面膨胀（js_dom_shim 已 ~12000 行/part03） | 节点门面独立 part 段 + 行数预算关注；不超 2000 行/文件红线（现未触） |
| headless 与浏览器语义漂移（suspended 态/autoplay 政策） | 逐项记录于 master.md（headless 简化清单模式） |
| 无需求驱动的过早建设 | 本 RFC 即决策闸门——不批准则不实施，goal DONE 不受影响 |

---

## 5. 待用户决策点

| # | 决策 | 选项 | 推荐 |
|---|------|------|------|
| D-WA-1 | **是否实施 Web Audio 最小面** | 批准切片 1+2 / 暂缓（记录后关闭） | 批准（渲染/合成域收益；工程量小；底座零新增依赖） |
| D-WA-2 | （若批准）设备面时点：最小面随切片 2 接 CpalSink，还是先 NullSink、设备面挂 media-audio M1 CpalSink 真出声切片 | 先 NullSink（推荐）/ 一并接设备 | 先 NullSink（与「Mixer 接线挂真出声切片」决策注记同构——CI 可验面先行） |
