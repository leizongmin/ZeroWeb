# 媒体音频 — 音频输出与环境验证目标（门控）

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active（**启动门控**——M0 音频环境验证须完成且解码选型（media-playback M0）落地后才动源码；M0 前的调研/环境探测可自主）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 3「`<video>`/`<audio>` 播放」的音频面）

> **说明**
> 本文档是 ZeroWeb「媒体音频」专项目标执行契约。目标是为 `<video>`/`<audio>` 补上音频
> 输出：音频解码 → 混音 → 音频设备输出（cpal 或等价），`volume/muted` 控制、音视频时钟
> 同步（A/V sync）、`AudioContext`（Web Audio）最小面评估。**音频输出依赖物理音频设备
> 环境（headless CI 无声卡）与解码选型（跟随 media-playback M0 路线），双重门控**——M0
> 环境验证 + 选型对齐先行。本文定义 Mission、边界、Done Criteria、执行协议和文档治理
> 规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新
> 写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：媒体方向三拆之三（门控最深的一个）。理由：
> ① 音频是媒体体验的另一半（静音视频只算半成品），但**依赖链最长**——解码选型
> （media-playback M0）+ 音频设备环境（本目标 M0）双重前置；② 独立成流让依赖链显式化，
> 避免阻塞 media-elements（语义面）/ media-playback（视频面）先行收益；③ cpal（CPAL）
> 是 Rust 音频输出的标准选择（winit 生态同源），环境探测与 dummy 设备回退是首要课题
> （headless CI 的音频验证策略须先行设计）。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **零音频能力**：Cargo.toml 无 cpal/音频依赖；无音频解码/混音/输出管线。
> - **媒体底座**：`<video>`/`<audio>` 布局占位 + 属性反射（R3040）；muted/volume 反射
>   已有（真控制未接）。
> - **headless 现实**：CI/headless 无声卡——**验证策略**（dummy 设备 + 混音总线可观测
>   断言）是 M0 首要课题；本地有声卡环境做真输出抽验。
> - **时钟底座**：rAF 帧驱动时钟（P1a）已有——音频时钟对齐可挂；视频时钟（media-playback
   M2）未建。

---

## Mission

为媒体播放补上音频输出：音频解码（跟随 media-playback 选型）→ 混音（多源 + volume/
muted）→ 设备输出（cpal，dummy 回退）；音视频时钟同步（audio clock 为主时钟）；
`volume/muted` 真控制；`AudioContext` 最小面可行性评估。分阶段里程碑：

| 阶段 | 目标 | 说明 |
|---|---|---|
| M0（门控） | **环境验证 + 验证策略** | 本地音频设备探测、cpal 集成 PoC（dummy 回退）、headless 验证策略（混音总线可观测断言）设计成文 |
| 第一阶段 | **首个声音输出** | 选型格式音频解码 → 混音 → 设备输出（本地有声卡环境 e2e） |
| 中期 | **A/V 同步 + 控制** | audio clock 主时钟对齐视频帧、volume/muted 真控制、多源混音 |
| 长期 | **`<audio>` 元素全路径 + AudioContext 评估** | 纯音频播放全路径；Web Audio 最小面 RFC（须用户批准再实施） |

**关键约束**：
- **验证策略先行**：headless CI 无声卡——音频正确性的常驻断言须基于**混音总线的可观测
  输出**（采样数据/事件），真设备输出只在本地有声卡环境抽验（evidence 记录环境）。
- **选型跟随**：音频解码路线与 media-playback M0 的容器/编解码选型绑定（同源 demux）；
  其 RFC 未批前本目标不动解码源码。

覆盖范围：

1. **音频管线** — 音频解码（选型面内编解码：AAC/Opus/Vorbis 视选型）→ 重采样 →
   混音总线（f32 帧、多源叠加）→ 设备输出（cpal 流；dummy 设备回退）
2. **A/V 同步** — audio clock 为主时钟：视频帧调度对齐音频时间戳（drift 校正策略）；
   `currentTime` 由组合时钟驱动
3. **控制语义** — `volume`（0.0-1.0 clamp + InvalidStateError 语义）/`muted` 真控制
   （混音增益）、`defaultMuted` 反射
4. **多源混音** — 多个媒体元素并发播放的混音与各自 volume/muted
5. **`<audio>` 元素全路径** — 无视频面的纯音频播放（复用同管线 + 元素占位渲染已有）
6. **Web Audio 评估**（M3）— `AudioContext` 最小面（createOscillator/GainNode/
   destination）可行性 RFC——**独立 Mission 级扩展，须用户批准**

执行方式：**门控推进** — M0 环境验证与策略成文 + media-playback 选型落地前不动源码；
之后轻量修复优先（每片 kill-switch + A/B 零回归）。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 环境验证 | cpal 探测/dummy 回退/headless 验证策略（M0） | 策略成文是解锁条件 |
| 音频管线 | 解码→重采样→混音→输出 | 选型跟随 media-playback |
| A/V 同步 | audio clock 主时钟 + drift 校正 | 与 media-playback M2 时钟集成 |
| 控制语义 | volume/muted 真控制 | 反射已有（R3040），接真增益 |
| 多源混音 | 多元素并发 | 混音总线设计目标 |
| e2e 资产 | 本地有声卡真输出 + headless 混音总线断言 | 双轨验证（evidence 记录环境） |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **视频解码/帧渲染** — 兄弟目标 `media-playback.md`
- **HTMLMediaElement 语义面** — 兄弟目标 `media-elements.md`
- **Web Audio 完整 API**（AudioWorklet/ConvolverNode/PannerNode 空间音频等）— M3 只做
  最小面可行性 RFC，实施是批准后的后续目标
- **媒体流 getUserMedia（麦克风/摄像头）** — 权限模型域（M13），远期
- **Speech synthesis/recognition** — 远期
- **系统音量/OS 混音控制** — 平台域非目标

### 依赖约束

- **双重启动门控**：① M0 音频环境验证与验证策略成文（本目标自持）；② media-playback
  M0 解码选型 RFC 获批（音频解码路线绑定其选型）。两者齐备后才动源码。
- **A/V 同步接口**：与 media-playback M2 的视频时钟接口对齐（audio clock 主时钟）——
  接口契约记录于两流 master.md。
- **与 js-dom 流碰撞管理**：volume/muted 反射段共享；`git log` 核对后再动。

---

## 当前能力/缺口基线

**详见** [media-audio/master.md](media-audio/master.md)（运行时控制面板，唯一真实状态
来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **反射底座**：muted/volume 属性反射（R3040，真控制未接）
- ✅ **时钟底座**：rAF 帧驱动（P1a）——音频时钟对齐可挂
- ⚠️ **缺口 1 — 零音频管线**：无解码/混音/输出依赖与代码
- ⚠️ **缺口 2 — 环境未验证**：cpal 可用性/dummy 回退/headless 验证策略未探测成文（M0）
- ⚠️ **缺口 3 — 选型未对齐**：音频解码路线待 media-playback M0 落地
- ⚠️ **缺口 4 — A/V 同步机制缺失**（依赖 media-playback M2 视频时钟）
- ⚠️ **缺口 5 — 无音频 e2e 资产**（真输出 + 混音总线断言双轨）

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 环境验证与验证策略成文

- [ ] cpal 集成 PoC（本地有声卡环境）+ dummy 设备回退验证
- [ ] headless 验证策略（混音总线可观测断言）设计成文并落入 e2e 资产

### DC-2: 音频管线端到端

- [ ] 选型面内音频格式：解码 → 重采样 → 混音 → 设备输出（本地真输出 e2e 常驻）
- [ ] headless 混音总线断言常驻（CI 可跑）

### DC-3: 同步与控制

- [ ] A/V 同步（audio clock 主时钟 + drift 校正）——与 media-playback e2e 联合断言
- [ ] volume/muted 真控制 + 多源混音

### DC-4: `<audio>` 全路径 + Web Audio 评估

- [ ] `<audio>` 元素纯音频播放全路径（e2e）
- [ ] `AudioContext` 最小面可行性 RFC 完成（**实施与否待用户批准——不批准不影响本目标
      DONE**）

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + e2e 资产化

---

## 活跃里程碑

### M0 — 环境验证 + 验证策略（门控，当前活跃）

**目标**：音频环境探测与验证策略成文；解锁条件清单就绪。

**切片建议**：
1. 本地音频设备探测 + cpal PoC（播放正弦波；无声卡/dummy 回退行为记录）
2. headless 验证策略设计：混音总线可观测断言形态（采样快照/事件计数）成文
3. 与 media-playback M0 选型的依赖对齐记录（解锁条件清单更新）

### M1 — 首个声音输出（双门控解除后）

**目标**：选型格式音频解码 → 混音 → 设备输出（本地 e2e + headless 总线断言）。

### M2 — A/V 同步 + 控制

**目标**：audio clock 主时钟 + volume/muted 真控制 + 多源混音。

### M3 — `<audio>` 全路径 + Web Audio 评估

**目标**：纯音频播放 e2e + AudioContext 最小面 RFC（提交用户决策）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 目标音频能力全依赖物理设备且本机亦无声卡时可判 BLOCK（先穷尽 dummy/软件回退） |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；真输出 e2e 在有声卡环境验证（evidence 记录环境）+ headless
总线断言常驻；`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，
archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**（M0 期间）：音频设备环境探测、cpal PoC、验证策略设计
2. 双门控解除后：**自主实现/测试/验证**，每片 kill-switch + net≥0 即 land
3. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **门控纪律**：M0 未成文 + media-playback 选型未落地前不动源码；等待期间转调研/
   策略设计/PoC。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过。
3. **碰撞管理**：与 media-playback 的时钟接口对齐先行；volume/muted 反射段 `git log`
   核对后再动。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **音频缺陷分析**：每个 e2e 失败必须定位根因（解码？重采样？混音？设备？同步？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/media-audio/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/media-audio/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/media-audio/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
