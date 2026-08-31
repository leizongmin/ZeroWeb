# 媒体元素 — 运行时控制面板（master.md）

**入口文档**: [../media-elements.md](../media-elements.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-31（M1 完成——WPT 首批导入 + 基线 46.5% + 失败聚类 F1~F6）

---

## 当前状态

**专项定位**：媒体方向三拆之一（可立即启动）。HTMLMediaElement 非解码语义面（状态机/
事件序列/API 行为），WPT media-elements 真实用例驱动。**不被解码选型 RFC 阻塞**——
headless 近似驱动先行，兄弟目标建成后替换驱动源。

**M1 已完成（2026-08-31）**：`make testharness-media` 基础设施落地（fetch-media-subset.sh
+ runner `testharness-media` 子命令 + Makefile 目标）；首批 30 用例 / 245 subtest 基线
**46.5%**（114P/77F/13T/41PF）；失败聚类 F1~F6 成文（见 evidence/2026-08-31-media-baseline.md）。

**与兄弟 goal 的边界**：
- media-playback — 解码/帧渲染归其管（RFC 门控）；本目标的 readyState 真实驱动源由其
  供给（接口契约记录于两流 master.md）
- media-audio — 音频输出归其管；volume/muted 本目标只做语义（真增益归其接线）
- js-dom — 媒体反射段（part01.js R3040）共享；该流已归档（2026-08-31 核对），媒体段
  近 14 天无并发编辑，可安全作业

## 实测基线（2026-08-31 首批跑通后）

### 现有实现（修正后的真实面）

- ✅ 属性反射：R3040 autoplay/controls/loop/muted/playsInline + 布局占位渲染
- ✅ FR-009 资源 settle：audio/video/source/track 的 fetch 状态提交 + load/error 事件
  （error/readyState/networkState 初值、src 反射、historical 面均绿——**修正 2026-08-17
  立项时低估**：初值面与状态常量并非全缺）
- ✅ R2835 四方法：play()/pause()/load()/canPlayType()（canPlayType 恒 ''——合法保守值，
  能力表为空）
- ⚠️ 媒体专有 IDL 属性面缺失：currentTime/duration(NaN)/playbackRate/preload/
  crossOrigin 归一/seeking/paused 等（F2）
- ⚠️ `<track>` 反射 + TextTrack/TextTrackList/addTextTrack/textTracks 全缺（F1/F3）
- ⚠️ load 算法事件序列（loadstart/canplay/loadedmetadata/...）未派发——FR-009 只有
  load/error 两事件（F4）
- ⚠️ track.src 未按 URL 属性解析（同 R2838 a.href 模式未接）（F6）

## 缺口清单（含失败聚类映射）

| # | 缺口 | 状态 | 失败聚类 |
|---|------|------|----------|
| M1g | WPT media-elements 用例覆盖 | ✅ 首批 30 用例已导入，基线 46.5% | — |
| M2g | load 算法 + 状态机（事件序列派发） | ⬜ M2 | F4（11 case Timeout） |
| M3g | 事件序列 headless 近似驱动 | ⬜ M2 | F4 |
| M4g-a | 媒体元数据 IDL 反射（初值面，spec 无解码器时合法值） | 🔄 **下一轮首选** | F2（9 Fail） |
| M4g-b | `<track>` 反射 + TextTrack 最小面 | ⬜ M3 | F1+F3（59 Fail） |
| M4g-c | track.src URL 解析 + \0 剥离 | 🔄 **下一轮（随 M4g-a）** | F6（6 Fail） |
| M4g-d | canPlayType 能力表（空表→选型面更新） | ⬜ 等 media-playback M0 选型 | F5（41 PF，非 bug） |

## 下一步计划

1. **M1 切片 3（下一轮首选）**：F2 元数据 IDL 初值面——currentTime=0 / duration=NaN /
   playbackRate=1 / defaultPlaybackRate=1 / volume=1 / preload 缺省映射表 / crossOrigin
   null·anonymous·use-credentials 归一 + seeking=false / paused=true 初值 + F6 track.src
   绝对 URL 解析（`__zw_parse_url`，同 R2838）。落点 part03 媒体段（R2835 四方法旁），
   碰撞面已核对安全。验收 = F2/F6 的 15 Fail → 0。
2. **M2**：load 算法骨架 + readyState/networkState 推进 + 事件序列 headless 近似驱动
   （宿主 FR-009 settle 扩展 media 事件序列——loadstart/progress/suspend/loadedmetadata/
   loadeddata/canplay/canplaythrough；autoplay 挂 play/playing）→ F4 的 11 case Timeout
   转绿。之后追加 event_* 族用例进 MEDIA_TEST_FILES。
3. **M3**：F1 track 反射（kind 归一表/subtitles 缺省/metadata invalid 映射）+ TextTrack
   最小接口 + F3 addTextTrack/textTracks 集合面。

**碰撞管理**：js-dom 流已归档；媒体段（part01b.js 常量 + part03 方法段 + part06 settle）
近 14 天仅 sw 流 iframe 改动（1b6c87303），与本 goal 无重叠。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 摸底 | ✅ 完成（2026-08-31，46.5% 基线 + F1~F6 聚类） |
| M2 — 状态机与事件序列 | ⬜ 下一切片：M1 切片 3（F2/F6）先行 |
| M3 — API 语义 + track 面 + 播放层衔接 | ⬜ |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | canPlayType 能力表（依赖 media-playback M0 解码选型——选型决定支持面） | ⬜ 跨 goal 联动，media-playback master.md D1 同源 |

## 验证基线

- 基线时点：2026-08-31，WPT rev `3159769338`；**subtest 114/245 = 46.5%**（Fail 77 /
  Timeout 13 / PF 41）
- 入口：`make testharness-media`（FILTER 透传，`--json` 捕获 evidence）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- evidence：`evidence/2026-08-31-media-baseline.md`（+ 同名 .json 机读版）
