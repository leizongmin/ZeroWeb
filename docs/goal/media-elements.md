# 媒体元素 — WPT 驱动的 HTMLMediaElement 语义正确性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 3「`<video>`/`<audio>` 播放」方向的第一块；非目标注记「首期只处理布局占位」的历史裁决已随 Tier 1 收口由用户 2026-08-17 拆分决策解除）

> **说明**
> 本文档是 ZeroWeb「媒体元素」专项目标执行契约。目标是把 `<video>`/`<audio>` 从属性反射
> （R3040：autoplay/controls/loop/muted/playsInline 反射 + 布局占位）深化为
> HTMLMediaElement 的完整**非解码语义面**：readyState/networkState 状态机、canPlayType、
> load() 算法、事件序列（loadstart/progress/canplay 等）、play/pause 语义、duration/
> currentTime 元数据面、track 面（`<track>` 反射）。**不含解码与像素/音频输出**（归兄弟
> 目标 media-playback / media-audio）。以 WPT `html/semantics/embedded-content/media-0`
> 真实用例通过率为验证标准。本文定义 Mission、边界、Done Criteria、执行协议和文档治理
> 规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新
> 写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：媒体方向三拆之一（可立即启动的两个之一）。
> 理由：① HTMLMediaElement 的语义面（状态机/事件/API）**独立于解码器存在**——WPT
> media-0 目录大量用例只断言 JS 可观察行为（canPlayType 返回值、readyState 推进、事件
> 序、异常），不依赖真播放；② 属性反射底座已有（R3040），渐进自然；③ 与解码器选型
> （media-playback 的门控项）解耦——本目标不被 RFC 阻塞，立即可开工；④ 兄弟目标的播放
> 语义建成后会**反向喂给本目标**（readyState HAVE_METADATA 等真值化），三者分层清晰。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **属性反射**：R3040——autoplay/controls/loop/muted/playsInline（part01.js:405
>   HTMLMediaElement 段）已有；`<video>`/`<audio>` 布局占位渲染已有。
> - **状态机/事件**：readyState/networkState/loadstart/canplay 等事件序列——未实现
>   （无 load 算法）；canPlayType 未核实（M1 摸底）。
> - **play/pause/duration/currentTime**：元数据面未实现（无解码器时这些值无法真化——
>   本目标先建 headless 语义桩：状态与事件正确、值为容器元数据近似或 stub 有记录）。
> - **WPT 面**：`html/semantics/embedded-content/media-0` 未导入（canvas 流已在
>   CANVAS_TEST_SUBDIRS 注释中提及 video 媒体面不在其范围），无基线。

---

## Mission

以 **WPT `html/semantics/embedded-content/media-0` 真实用例通过率为验证标准**，把
HTMLMediaElement 的非解码语义面（状态机/事件序列/API 行为/异常）对齐到 Chromium 水平。
分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入 media-0 范围内用例 + 通过率基线（当前无基线） |
| 中期 | **状态机与事件 60%+** | load 算法/readyState/networkState 推进/事件序列/canPlayType |
| 长期 | **80%+（可校准）** | play/pause/异常语义/track 面/与 media-playback 播放层的真值衔接 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d /
form-validation——不允许手写 inline 用例替代或充数）。依赖真解码输出的用例（像素/音频
断言、真实 seek 精度）入 skip list 并注明归兄弟目标。

覆盖范围：

1. **load 算法与状态机** — `load()`（resource selection algorithm：source 子元素/attr）、
   readyState（HAVE_NOTHING→HAVE_METADATA→…）/networkState（NETWORK_EMPTY→LOADING→…）
   推进、src/currentSrc 解析
2. **事件序列** — loadstart/progress/suspend/abort/error/emptied/stalled/loadedmetadata/
   loadeddata/canplay/canplaythrough/playing/waiting/seeking/seeked/ended/durationchange/
   timeupdate/play/pause/ratechange/volumechange——派发顺序与时机（headless 元数据近似
   有记录）
3. **API 行为与异常** — `canPlayType`（'probably'/'maybe'/'' 返回语义——与兄弟目标的
  解码器能力表衔接，MVP 空/近似表有记录）、`play()`/`pause()`（Promise 化的 play 返回值
   ——NotAllowedError 等）、`load()` 重置语义、快进 seek 的异常（InvalidStateError）
4. **元数据面** — duration/currentTime/defaultPlaybackRate/playbackRate/volume/muted 的
   getter/setter 语义（无解码器时值为 stub/近似——**headless 简化逐项记录**）
5. **track 面** — `<track>` 元素反射（kind/src/srclang/label/default）、textTracks
   集合的最小面
6. **属性反射深化** — preload/controlsList/crossOrigin、buffered/seekable TimeRanges
   形状

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 状态机与事件 | load 算法/readyState/networkState/事件序列 | engine 媒体元素宿主逻辑（js_dom_shim part01 媒体段 + host 命令） |
| API 语义 | canPlayType/play/pause/seek 异常/元数据 getter/setter | 以 WPT 用例为准；headless stub 值逐项记录 |
| track 反射 | `<track>` 属性 + textTracks 最小面 | 深化（字幕渲染归远期） |
| WPT 基础设施 | media-0 用例导入、testharness 执行、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **解码与帧输出**（video 像素渲染）— 兄弟目标 `media-playback.md`（带选型 RFC 门控）
- **音频输出**（audio 设备集成）— 兄弟目标 `media-audio.md`（带环境验证门控）
- **Media Source Extensions（MSE）** — Tier 3 远期（依赖播放基础先行）
- **EME（加密媒体）** — 远期非目标
- **字幕渲染**（track cue 的视觉呈现）— 渲染域远期；本目标只做反射与集合面
- **画中画/Picture-in-Picture、全屏媒体控件** — UI 域远期

### 依赖约束

- **与 media-playback 的衔接**：本目标的状态机是接口层——readyState 推进的**真实驱动**来自
  解码层（兄弟目标 RFC 批准后建设）。MVP 期间本目标用 headless 近似驱动（如 canplaythrough
  在 load 后异步派发），兄弟目标建成后替换驱动源——语义层代码不返工。
- **canPlayType 能力表**：与兄弟目标的解码器选型联动（选型决定支持容器/编解码面）；MVP
  用显式近似表（记录于 master.md），选型 land 后更新。
- **与 js-dom 流碰撞管理**：媒体元素反射段（part01.js R3040）若该流活跃，先
  `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对；活跃则先做
  零碰撞面（WPT 导入、事件序列 host 层设计、状态机单测）。

---

## 当前能力/缺口基线

**详见** [media-elements/master.md](media-elements/master.md)（运行时控制面板，唯一真实
状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **属性反射**：R3040 autoplay/controls/loop/muted/playsInline + 布局占位渲染
- ⚠️ **缺口 1 — load 算法与状态机缺失**：readyState/networkState 不推进、事件序列未实现
- ⚠️ **缺口 2 — canPlayType 未核实**（M1 摸底）
- ⚠️ **缺口 3 — 元数据面未实现**：duration/currentTime/volume 等为 stub 或缺失
- ⚠️ **缺口 4 — track 面缺失**
- ⚠️ **缺口 5 — WPT 覆盖为零**：media-0 未导入，无基线

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT media-0 用例导入与通过率基线

- [ ] 从上游 WPT 仓库导入 `html/semantics/embedded-content/media-0` 范围内真实用例
      （依赖真解码输出的入 skip list 并注明归兄弟目标）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/media-elements/evidence/`，历史可追溯

### DC-2: 状态机与事件序列

- [ ] load 算法 + readyState/networkState 推进 + 事件派发顺序与 spec 一致（WPT 为准；
      headless 近似驱动逐项记录）

### DC-3: API 语义

- [ ] canPlayType 三值语义（能力表显式可查）、play/pause 语义与异常、元数据
      getter/setter、`<track>` 反射 + textTracks 最小面

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT media-0 基线建立 + 现状摸底

**目标**：导入用例记录基线；摸清 canPlayType/事件面现状。

**切片建议**：
1. 用例导入 + 分类通过率报告（零源码改动，纯资产）
2. 失败聚类 → 反射面/状态机/事件面的已有 vs 缺失清单
3. canPlayType + 基础反射深化（preload/crossOrigin/controlsList）

### M2 — 状态机与事件序列

**目标**：load 算法骨架 + readyState/networkState 推进 + 事件序列（headless 近似驱动）。

### M3 — API 语义 + track 面 + 与播放层衔接

**目标**：play/pause/异常/元数据面/track 反射；与兄弟目标接口对齐（驱动源可替换）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前媒体元素反射面（R3040）与语义面差距
2. **自主导入** WPT media-0 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（反射缺失？状态机？事件序？异常？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（js_dom_shim 媒体反射段）前先 `git log` 核对；有活跃
   编辑则转零碰撞面（WPT 导入、状态机 host 层、单测）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **用例失败分析**：每个失败 case 必须分析根因（反射？状态机？事件？异常语义？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由（headless 近似逐项记录）。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/media-elements/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/media-elements/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/media-elements/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
