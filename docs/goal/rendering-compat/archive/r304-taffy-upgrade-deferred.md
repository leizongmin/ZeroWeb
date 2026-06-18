# R304 — taffy 升级评估：DEFER（归档自 master.md）

> 归档说明：本文件为 master.md「最近轮次详细记录」中 R304 的逐轮详细记录，于 doc-maintenance 轮（2026-06-19）归档——master.md 最近轮次窗口收窄为最近 20 轮（R305–R324），R304 作为第 21 轮迁出。R304 的核心结论（taffy 0.11 升级 DEFER）仍以浓缩形式保留在 master.md「综合裁决」杠杆穷尽表（taffy 0.11 升级 | R304 | DEFER）。本归档仅为可追溯性保留，archive 区不修改。

---

**承接**：R300/R302/R303 将剩余结构性缺口归因为「taffy 限制（grid auto-track growth / flex intrinsic sizing）」。本轮 read-only 评估 taffy 升级能否解锁这两个具名缺口。

**当前 taffy 现状**：workspace 声明 `taffy = "0.7"`，经 `[patch.crates-io]` 重定向到 vendored `crates/taffy-local`（taffy 0.7.7 全量源码，git-tracked 61 文件，commit 9e5df18 R59 引入）。本地补丁 = `cached_baselines()` 访问器（Cache + TaffyTree 暴露内部 `LayoutOutput.first_baselines`），补丁面极小（仅 2 个 pub 方法 + 复用已 pub 的 first_baselines 字段）。

**上游版本演进**（crates.io API + GitHub release notes + CHANGELOG.md 实证）：0.7.7（2025-03-06，当前）→ 0.8.0（2025-04，calc() + tagged-pointer 尺寸类型）→ 0.9.x（2025-08~11，named grid lines，Style 泛型化 CheapCloneStr，grid 类型改名）→ 0.10.x（2026-03~04，native float/clear feature + direction/RTL + cache API &LayoutInput）→ **0.11.0（2026-06-12，最新，safe alignment enum variant→associated constant）**。共 4 个 breaking-change minor 版本。

**核心结论 1 — flex intrinsic sizing 升级零收益**：CHANGELOG 实证，所有 flex/grid intrinsic-sizing 修复均**早于 0.7.7，已在 vendored 副本中**：#624 grid growth limits（0.4.1）、#673 intrinsic main size vs child cross size（0.5.2）、#722/#723/#728 auto-fill/fit 计数+min-size intrinsic（0.6.1）、#522/#481 flexbox/grid intrinsic main size（0.3.13）、#388 % min-content（0.3.7）、#291 flex min-content constraint（0.3.0）。**flexbox-collapsed-item（R301 残余 15%）= ZeroWeb 自身 engine.rs:2649 浮动 shrink-to-fit 不遵循 flex-resolved 子项尺寸（min-content floor）的 post-processing 缺口，非 taffy 版本问题，升级不解锁**。

**核心结论 2 — grid auto-track growth 升级零收益**：vendored 0.7.7 **已有** `expand_flexible_tracks`（fr 吸收 free space）+ `maximise_tracks` + `#783 stretch auto tracks if content-align=stretch`（0.7.5 已含）。对比上游 main `expand_flexible_tracks`（track_sizing.rs:1179）= **实质相同**（仅 `.is_flexible()→.is_fr()` 改名 + `total_cmp` float 排序 + 注释），**auto 列仍不吸收 free space**（两版本一致）。R302 grid-calc-margin（w=0）的 auto-track-absorb-free-space 行为升级不变。新版唯一 grid 修复 = #946（0.10.1 auto-repeat 计数+min-size）+ #960（0.11 item % vs grid area），均**旁系**于 R302。

**核心结论 3 — 迁移成本 prohibitive**：`layout-engine` 内 **541 处 `taffy::` 引用 + 108 处 alignment enum（9 文件）**，跨 4 个 breaking 版本：① 0.8 tagged-pointer 改 `LengthPercentage`/`Dimension`/`MinTrackSizingFunction` 等构造（ZW 50+ 处）；② 0.9 `Style<CheapCloneStr>` 泛型化 + `TrackSizingFunction→GridTemplateComponent` 改名（ZW 20+ 处）；③ 0.10 cache `&LayoutInput`（本地 cached_baselines 补丁须在新 cache 结构上重新推导）；④ 0.11 `AlignContent::Start→AlignContent::START` 关联常量（108 处）。**最坏回归风险**：0.10 native `float_layout` feature 与 ZeroWeb ~6 轮手动 float 后处理（R108b/R127/R129/R145/R301）**冲突**——启用 native float 须退役/重写这些 pass，触及全量 layout 测试套件，难调试。

**真实但无关的升级收益**（不针对具名缺口）：calc()（0.8，关联 R97/R180 max-content→0 bug）、native float/clear（0.10）、direction/RTL（0.10，关联 writing-mode）、grid #960。

**决策：DEFER 升级**。两个具名结构性缺口均为 ZeroWeb 侧架构问题（engine.rs shrink-to-fit post-processing / Phase A IFC 统一），非 taffy 版本问题；升级对它们零收益，而迁移+回归成本 prohibitive 且 native-float 冲突风险高。**纠正 R302「③ 评估 taffy 升级」lever 期望**——升级非 clean unlock，应从优先级队列移除。

**对优先级队列影响**：taffy 升级评估完成 = ruled out（具名缺口零收益 + 成本 prohibitive）。剩余真实 lever 收敛为**纯 ZeroWeb 侧架构工作**：① Phase A IFC 统一（stored/paint 三路径 baseline 墙，spec-rfc 多轮）；② engine.rs 浮动/intrinsic-sizing post-processing 完整化（min-content floor，R97/R181 硬域）；③ 独立能力缺口（DC-13 产品 smoke 端到端证据 / DC-9 blend_mode backdrop）。next = 启动 Phase A IFC 统一的 spec-rfc 设计（最大结构性 lever，影响 large-font/multicol/IFC 度量整簇），或先做 DC-13 产品 smoke 端到端（非 taffy 阻塞、有明确验收）。read-only 调研，无代码/reftest 变更，基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。
