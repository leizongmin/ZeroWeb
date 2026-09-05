# media-audio M0 切片 1 — 音频环境验证报告

**日期**: 2026-09-01
**探测环境**: Linux WSL2（x86_64，Debian 13 用户态），本仓开发/CI 主环境
**探测目的**: M0 门控「音频环境验证与验证策略成文」——确定音频输出路线在本环境的
可编译面/可运行面，以及 headless 场景的验证策略。

## 1. 环境实测数据

| 项 | 状态 | 明细 |
|---|---|---|
| 内核层音频硬件 | ✅ 存在 | HDA Intel PCH（`/dev/snd/pcmC0D0*`），aplay 可枚举 playback 设备 |
| ALSA userspace 库 | ✅ 运行库在 | `libasound2` 1.2.14 |
| **ALSA 开发头文件** | ❌ **缺失** | `libasound2-dev` 未安装；`/usr/include/alsa/` 不存在；`pkg-config alsa` 失败 |
| PulseAudio 客户端库 | ✅ dev 在 | `libpulse-dev` 17.0 |
| PulseAudio 服务 | ❌ 不可用 | `pactl info` → Connection refused；`PULSE_SERVER` 指向的 `unix:/run/catlink/pulse/native` 不存在 |
| cpal 依赖 | ❌ 未引入 | 全工作区无 cpal/音频输出依赖（Cargo.toml 零音频栈） |
| ffmpeg/ffprobe | ✅ 可用 | 7.1.5——fixture 生成与解码验证的工具面（非浏览器内解码） |

## 2. 对候选技术路线的影响

**cpal（Rust 音频输出事实标准）**：
- Linux 默认 host = ALSA → **编译期即需要 `libasound2-dev`**，本环境不可编译。
- 安装 `libasound2-dev` 是系统级 apt 变更，超出 rally 自主范围（无人值守安全约束）→
  **须用户批准后方可进行**（已列入 master.md 待用户决策）。
- cpal 无内置 null/dummy host（Windows WASAPI 有 loopback 思路但 Linux 无 null 设备
  抽象）→ headless/CI 场景无法直接 cpal 出声，**应用层须自行做 sink 抽象**。

**PulseAudio 直连（libpulse 绑定，如 libpulse-binding）**：
- dev 库在，可编译；但 server 不可用（WSLg 音频桥未接通）→ 可编译不可验证。
- 引入即绑 Pulse 生态（非 Windows/macOS 原生），作为唯一后端不可行；作为 Linux 侧
  可选后端存疑（用户环境差异大）。

**复用现有多进程先例（image-decoder 模式）+ trait 抽象 sink**：
- 与进程边界选型解耦——无论解码在进程内（crate）还是独立进程，音频输出端点都是
  PCM 流消费者，trait 抽象先行不返工。

## 3. headless 验证策略（M0 切片 2 产出）

**核心设计：`AudioSink` trait 抽象 + 双实现**

```
trait AudioSink {
    fn start(&mut self, format: AudioFormat) -> Result<()>;
    fn write(&mut self, samples: &[f32]) -> Result<()>;   // PCM f32 交错帧
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn underrun_count(&self) -> u64;                       // 可观测性
}
```

- **`CpalSink`**（真实设备，feature-gated `audio-cpal`）：cpal 路径，桌面环境启用。
  编译面受 `libasound2-dev` 门控——未装则 feature 关闭，零影响主构建。
- **`NullSink`**（headless/CI 默认）：吞掉 PCM 但**可观测**——统计写入帧数、最后写入
  时刻、（可选）环形缓冲末端样本值。e2e 断言形态：
  - 「play() 后 NullSink 收到 ≥N 帧 PCM」→ 播放驱动的可观测等价物；
  - 「混音总线末端 440Hz 正弦波过零率 ≈ 440」→ 解码+混音正确性的频域代理断言
    （过零率无需 FFT，O(n) 可算）。
- 特性开关策略：CI 与 WPT 环境强制 NullSink；`audio-cpal` feature 打开时优先真实
  设备，设备枚举失败自动回落 NullSink（run-rules 的环境自适应先例同款）。

**验证策略成文结论**：M1「首个声音输出」的验收 = NullSink 可观测断言（CI 可常驻）
+ CpalSink 人工冒烟（可选，依赖设备环境）。不依赖真实出声即可常驻 CI——
headless 验证策略成立，**media-audio M0 的验证策略门控解除**。

## 4. M0 收口判定

- [x] 切片 1 — 本地音频设备探测：完成（本报告 §1）
- [x] 切片 2 — headless 验证策略设计成文：完成（本报告 §3）
- [x] 切片 3 — 与 media-playback M0 选型的依赖对齐记录：见 master.md（解码面依赖
      其 RFC 选型；输出面 AudioSink trait 与选型解耦，可先行）
- [ ] cpal 编译验证：**阻塞于 libasound2-dev 缺失**（须用户批准装包；不阻塞
      AudioSink trait + NullSink 设计与实施——那属 M1，另有双门控条款约束）

**M0 完成**（cpal 实测面留待装包后补录，不影响门控判定——验证策略已成立）。
