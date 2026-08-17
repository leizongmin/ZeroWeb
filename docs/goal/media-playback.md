# 媒体播放 — 视频解码与帧渲染目标（选型 RFC 门控）

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active（**启动门控**——M0 解码器选型 RFC 须用户批准后方可动源码；M0 本身可自主推进）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 3「`<video>`/`<audio>` 播放 + Media Source Extensions」的第一阶段）

> **说明**
> 本文档是 ZeroWeb「媒体播放」专项目标执行契约。目标是让 `<video>` 从布局占位变为**能播放**：
> 选定视频解码技术路线（独立进程 vs Rust crate），实现「无音频的帧解码 + 按帧率驱动 + 帧图元
> 渲染 + currentTime 推进 + 基础 controls 行为」，以真实视频文件的帧输出验证。**解码器选型是
> Mission 级决策（专利/依赖/架构三重风险），按 run-rules rule 11 须 RFC + 用户批准后才动
> 源码**。本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，供后续
> `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：媒体方向三拆之二（两个门控流之一）。理由：
> ① 「视频能放」是媒体方向收益最肉眼可见的一跳（占位框 → 播放）；② **解码器选型是全方向
> 最大的架构决策**——H.264/H.265 专利池 + 纯 Rust 解码器不成熟 + ffmpeg C 依赖体量，
> 三条路线（ffmpeg 外部进程 / rust crate 组合 / 限定 VP9/AV1 开源编解码先行）各有重大
> 权衡，必须用户拍板（对齐 P1b「先 RFC 后实施」的先例）；③ 架构上有现成先例可复用——
> image-decoder 已是独立进程（D1 交付物），视频解码器走同款进程边界模式是候选之一；
> ④ 与 media-elements（语义面）分层清晰：本目标提供 readyState 真实驱动源，其语义层
> 不返工。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **`<video>`/`<audio>` 现状**：布局占位 + 属性反射（R3040）——无解码、无帧输出、
>   无 currentTime 真值。
> - **架构先例**：`apps/image-decoder`（图像解码独立进程，D1）+ `zero-protocol` IPC +
>   `apps/compositor`（C2 合成器进程）——多进程基建已在产，视频解码进程可复用同款边界。
> - **渲染通路**：canvas 像素桥接页面图元的链路（R3268）已验证「解码位图 → 页面图元」
>   可行——视频帧可走同款通路。
> - **解码依赖现状**：Cargo.toml 无任何视频解码依赖；纯 Rust 生态（rav1e/decoder 类）
>   成熟度有限；ffmpeg sys 绑定未评估。
> - **WPT 面**：真实播放时序用例（`media/` 大目录）大量依赖音频同步与真实 seek——
>   验收以**本地真实文件帧输出 e2e** 为主 + 上游可执行子集。

---

## Mission

让 `<video>` 能播放真实视频文件：选定解码路线 → 帧解码 → 按帧率驱动 → 帧图元渲染 →
currentTime 真值推进 → 基础 controls 行为（play/pause/seek/ended）。分阶段里程碑：

| 阶段 | 目标 | 说明 |
|---|---|---|
| M0（门控） | **解码器选型 RFC** | 三路线对比（ffmpeg 外部进程 / Rust crate 组合 / 开源编解码先行）——**须用户批准** |
| 第一阶段 | **首个视频帧上屏** | RFC 批准后：解码选定格式（如 VP9/AV1 或 mp4/H264 视选型）→ 首帧 → canvas 通路式图元渲染 |
| 中期 | **连续播放** | 帧率驱动（rAF/时钟）+ currentTime 推进 + play/pause/seek/ended + readyState 真值驱动 |
| 长期 | **多格式 + controls + 稳定** | 选型面内的多容器/编解码、`controls` UI 行为、真实文件 e2e 常驻 |

**关键约束**：
- **音频不同步**（首期）：无音频输出的视频播放（静音播放）——音频归兄弟目标 media-audio；
  currentTime 由视频时钟驱动（简化记录）。
- **验收以本地真实文件为主**：上游 WPT media 用例大量依赖音频同步/真 seek 精度/HTTPS
  服务环境；本目标验收 = 真实视频文件（仓内 fixture）帧输出 e2e + 上游可执行子集
  （skip list 注明依赖项）。

覆盖范围：

1. **解码管线** — 选型路线的解码器集成（进程边界/Crate API）、容器 demux（mp4/webm）、
   帧格式转换（NV12/YUV → RGBA，进渲染管线既有通路）
2. **播放驱动** — 帧率时钟（与 rAF 帧驱动的 event loop 集成——P1a 底座）、pause/resume、
   seek（关键帧粒度起步）、ended、playbackRate
3. **渲染集成** — 帧位图 → 页面图元（canvas 像素桥接同款通路 R3268）；video 元素盒
   （object-fit 已有）
4. **语义层驱动** — readyState/duration/buffered 的真实值化（喂给兄弟目标 media-elements
   的状态机——替换其 headless 近似驱动）
5. **资源生命周期** — 解码进程/资源释放（导航离开/元素移除不泄漏）

执行方式：**门控推进** — M0 RFC 批准前不动源码（调研/RFC/fixture 准备可自主）；批准后转
轻量修复优先（每片 kill-switch + A/B 零回归）。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 选型 RFC | 解码路线对比与推荐（M0，须用户批准） | 专利（H.264/H.265 专利池 vs VP9/AV1 免费）/ 依赖（ffmpeg C vs 纯 Rust 成熟度）/ 架构（独立进程复用 image-decoder 先例 vs 进程内 crate）三维评估 |
| 解码管线 | demux + 解码 + 帧转换 | 按 RFC 选型实施 |
| 播放驱动 | 帧率时钟/seek/ended/playbackRate | 与 event loop rAF 帧驱动集成（P1a 底座） |
| 渲染集成 | 帧 → 图元通路 | canvas 像素桥接（R3268）同款 |
| 语义驱动 | readyState/duration 真值化 | 接口对齐 media-elements（驱动源替换不返工） |
| e2e 资产 | 真实视频 fixture + 帧输出断言 | 本地常驻 e2e（CLAUDE.md 测试资产化——本地等价断言） |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **音频输出/同步** — 兄弟目标 `media-audio.md`（本目标静音播放）
- **HTMLMediaElement 语义面**（事件序列/canPlayType/track）— 兄弟目标 `media-elements.md`
- **Media Source Extensions（MSE）** — 播放基础稳定后的后续目标
- **EME/DRM** — 远期非目标
- **WebCodecs API**（VideoDecoder JS API）— 播放基础稳定后的后续（但 M0 选型会预留
  解码器层复用性）
- **画中画/媒体会话/全屏媒体 UI** — UI 域远期
- **直播流（HLS/DASH）** — 远期

### 依赖约束

- **启动门控（M0）**：解码器选型涉及专利/重依赖/架构三重 Mission 级决策，按 run-rules
  rule 11，**RFC 须用户批准后才动源码**。M0 期间的调研、RFC 起草、视频 fixture 准备、
  渲染通路 PoC 分析可自主推进。
- **与 media-elements 的接口契约**：本目标产出「解码事件 → 状态推进」的驱动接口；其
  M2 状态机的 headless 近似驱动在接口就绪后替换。两流对齐接口须在 master.md 记录。
- **与 js-dom 流碰撞管理**：video 元素宿主段（js_dom_shim part01 媒体段）共享；
  `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对后再动。

---

## 当前能力/缺口基线

**详见** [media-playback/master.md](media-playback/master.md)（运行时控制面板，唯一真实
状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **架构先例**：image-decoder 独立进程（D1）+ zero-protocol IPC + compositor（C2）——
  多进程边界可复用
- ✅ **渲染通路**：canvas 像素 → 页面图元桥接（R3268）已验证
- ✅ **event loop 帧驱动**：rAF 帧驱动（P1a）已有——播放时钟可挂
- ⚠️ **缺口 1 — 零解码能力**：无任何视频解码依赖与管线
- ⚠️ **缺口 2 — 选型未定**（M0 门控项）：专利/依赖/架构三维未评估成文
- ⚠️ **缺口 3 — currentTime/readyState 无真值源**（media-elements 的近似驱动待替换）
- ⚠️ **缺口 4 — 无播放 e2e 资产**：无视频 fixture + 帧输出断言

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 选型 RFC 已批准并落地

- [ ] 解码路线 RFC 完成（三维对比：专利/依赖/架构 + 推荐 + 风险 + 回滚）并经用户批准
- [ ] 实现与 RFC 一致；偏离处记录原因

### DC-2: 首个视频端到端播放

- [ ] RFC 选定格式：真实视频文件 → demux → 解码 → 首帧上屏（本地 e2e 常驻）
- [ ] 连续播放：帧率驱动 + currentTime 推进 + play/pause/seek/ended
- [ ] 帧渲染走页面图元通路（与 canvas 桥接一致）

### DC-3: 语义驱动真值化

- [ ] readyState/duration/buffered 由解码层真实驱动（media-elements 接口替换验证）

### DC-4: 多格式（选型面内）+ 稳定性

- [ ] RFC 选型面内的容器/编解码组合有 e2e 覆盖
- [ ] 资源生命周期：导航/元素移除后解码资源释放（无泄漏断言）
- [ ] 上游 WPT 可执行子集导入 + 通过率（skip list 注明音频同步/HTTPS 依赖项）

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + e2e/用例资产化

---

## 活跃里程碑

### M0 — 解码器选型 RFC（门控，当前活跃）

**目标**：三路线 RFC 起草并获批。

**切片建议**：
1. 渲染通路 PoC 分析：canvas 桥接（R3268）复用面 / image-decoder 进程边界复用面（纸面）
2. 三路线调研：ffmpeg 外部进程（专利/体积/维护）/ Rust crate 组合（mp4 demux + 解码器
   成熟度盘点）/ VP9+AV1 开源先行（webm 面 / 生态覆盖度）——各自工程量/风险/格式覆盖
3. 视频 fixture 准备（小体积多格式样本 + 版权清白的生成/来源记录）
4. RFC 起草 → 提交用户审批（**停止源码改动，记「待用户决策」**）

### M1 — 首个视频帧上屏（RFC 批准后）

**目标**：选定格式 demux + 解码 + 首帧 → 图元渲染（本地 e2e）。

### M2 — 连续播放 + 语义驱动

**目标**：帧率时钟 + play/pause/seek/ended + readyState 真值驱动（接口替换）。

### M3 — 多格式 + 稳定 + 收尾

**目标**：选型面多格式 e2e、资源生命周期、WPT 可执行子集。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用（M0 等用户审批不是 BLOCK——记待决策后转 fixture/调研等零碰撞面） |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；本地 e2e 基于真实视频文件帧输出（非 mock 充数）；上游
用例子集真实导入；`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部
自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**（M0 期间）：渲染通路/进程边界复用面、三路线调研、fixture 准备
2. **自主起草**选型 RFC（对比 + 推荐 + 风险 + 回滚）
3. RFC 批准后：**自主实现/测试/验证**，每片 kill-switch + net≥0 即 land
4. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **门控纪律**：M0 RFC 未批不动源码；等待期间转零碰撞面（调研/fixture/通路分析）。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过。
3. **碰撞管理**：碰 js-dom 共享面（媒体反射段）或 media-elements 接口面前先核对两流
  master.md 活跃记录。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **播放缺陷分析**：每个 e2e 失败必须定位根因（demux？解码？时钟？渲染？语义驱动？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/media-playback/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/media-playback/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/media-playback/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
