# 媒体运行时隔离控制面

**日期**：2026-10-24
**状态**：Active（**M1 基线测量完成**——见 [2026-10-24 evidence](evidence/2026-10-24-m1-baseline/ANALYSIS.md)；**M2 两项交付齐备 + M3 前置设计就绪待用户评审**——[docs/specs/media-runtime-isolation-spec-rfc.md](../../../specs/media-runtime-isolation-spec-rfc.md) v1.2：三路线对比 + §9 接口契约 + A4 seek 窄化批次；A1/A4 设计稿见 [m3-prep-design DESIGN](evidence/2026-10-24-m3-prep-design/DESIGN.md)；D-RFC-1~4 决策点）。
**当前卡点**：用户评审/批复 RFC（D-RFC-1~4：方案 C 立项 / A 批次随批 / 方案 B 核查 / decode-h264 启用）。批复前不改代码、不装 SDK、不变 feature 表。M1 待决输入（时钟 rate 1.454、NullSink）已入 RFC 方案 A 批次与 TBD-3。
**下一步**：用户批复后按 RFC §7 批次推进——D-RFC-2 随批即启 A1（时钟批内对齐）；D-RFC-1 批准则并行 C1（protocol 媒体消息族 + `media-process` feature 骨架）。若改选 A-only，DC-3 按 DC-5 由用户认可延期/调整后收口。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | **已测**（60s 基线 + 复制链 C1–C5 + 权限表，evidence 2026-10-24） |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | **RFC v1.1 完成**（三路线对比 + §9 IF 级接口契约，待用户批复） |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | NOT_STARTED |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | 待决（M1 核查：本平台 /dev/dri 缺失，无硬解路径） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | **完成**（测量面成立，收口待评审） | [2026-10-24 基线](evidence/2026-10-24-m1-baseline/ANALYSIS.md) |
| M2 | 方案/许可证/回滚 RFC + protocol 接口契约 | **v1.2 完成待批复**（决策门=用户） | [media-runtime-isolation-spec-rfc](../../../specs/media-runtime-isolation-spec-rfc.md) |
| M3 | 已确认平台与隔离切片 | NOT_STARTED（A1/A4 设计稿已备，待 D-RFC-2） | [DESIGN.md](evidence/2026-10-24-m3-prep-design/DESIGN.md) |
| M4 | 配对媒体、故障和资源审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | MEDIA-01-cpu-copy-and-capability | **已测成立**（容量/复制链/权限/能力对照；收口待用户评审） | evidence/2026-10-24-m1-baseline |
| DC-2 | MEDIA-02-clock-seek-and-state | 部分（media 层时钟/seek/漂移已测：rate 1.454 FINDING、cue seek 精确、漂移 p50=11ms；页面可见时钟缺口=三缺口未修） | evidence/2026-10-24-m1-baseline |
| DC-3 | MEDIA-03-decoder-crash-recovery | NOT_RUN | 无 |
| DC-4 | MEDIA-04-platform-fallback-and-permissions | 部分（平台核查：无 /dev/dri、NullSink、VP9-only 生产面；故障注入未做） | evidence/2026-10-24-m1-baseline |
| DC-5 | RFC 与最终支持范围审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：media/engine/webview/protocol/renderer/compositor 与 Android。本轮实际占用：**仅新增独立测试**（crates/webview/tests/media_m1_baseline.rs）、fixture 与文档；zero-media/render-foundation/runtime 源码零改动，无共享面冲突。后续切片启动前按 run-rules §9–11 核对。

### 2026-10-24 M1 基线轮

- 新增 `scripts/make-long-video-fixture.py` + 2 个 60s fixture（母本簇重复 ×30，来源清白，README 有声明与重现命令）。
- 新增 `crates/webview/tests/media_m1_baseline.rs` 5 场景（`#[ignore]` 显式运行，release + test-guard 全过）。
- 关键测量：纯视频 60s 解码 1339ms（≈45× 实时，~1 核）；A/V 双轨 1422ms（≈42×）；空 tick p50≈431ns；cue seek 精确（302ms 冷/1.1ms 热/1000ms）；漂移 p50=11ms。**FINDING**：media clock rate=1.45402（音频批量化），断言收 sanity 带，修复归 M3。（数字已按 raw log 订正，ANALYSIS 内有订正注）
- 三缺口盘点在位核对：part06.js:10131（autoplay 绕桥）、part06.js:16407（死钩子）、part03.js:23558/23610（play-only 武装）——行号按 dd17e2854 后 HEAD 订正，语义核对不变。
- 守卫与证据：执行阶段 test-guard（900s/4GB）；原始输出 m1baseline-raw.log；无残留进程。本轮未安装 SDK，未改 feature 门。

### 2026-10-24 M2 RFC 轮

- 新增 `docs/specs/media-runtime-isolation-spec-rfc.md`（设计交付，需求权威=goal DC 表）：三路线对比（A 现状增强 / B 平台硬解 / C 独立媒体进程）、许可/SDK/分发核查（仓内已批 vs 待验证分表）、分批实施与回滚切点（A1–A3/C1–C4/B1–B2+）、D-RFC-1~4 决策点。推荐 A→C、B 暂缓；C 为 DC-3 唯一达成路径。
- 独立评审（lei-spec-rfc §4 独立视角）：0 blocker/2 major/5 minor 全部修复——D-RFC ID 与前 RFC 撞名（§9 消歧 + 全文标「前 RFC」）、§2.2 数字与 raw log 对齐；MSE 面属 `decode-h264` feature 门控事实补入 ANALYSIS §5。
- Lint：26 Pass/1 Warning/0 Fail/3 Skip/4 PendingReview/5 N/A；决策草案就绪、实施未就绪（PendingReview 门）。
- 本轮零代码改动；无 SDK 安装、无 feature 表变更。

### 2026-10-24 M2 RFC v1.1 轮（接口契约）

- RFC 新增 §9 媒体 host 协议接口契约（IF 级，M2 交付第二项「protocol 与媒体 host 接口」）：先例映射表 + IF-001~006 六面（会话生命周期 / 控制面 / 状态时钟面含 epoch 代际 / 帧面含 shm 双路径 / 字节流含 MSE 扩展点 / 错误面含降级纪律），§9.7 挂接 DC-2/3/4；原 §9~12 顺延为 §10~13，交叉引用同步。
- 成本口径修正：§5.1A 原写「无先例可复用」，经 `apps/image-decoder` 源码核实（`run_role` 循环、懒惰 spawn `image_decoder.rs:211`、SPAWN_FAILED 永久回落、`child_process_args` role 映射、PipeTransport/frame_shm 复用面）改为**部分先例成立**；真正拟新增成本收敛为流式控制契约 + 跨进程时钟镜像 + 持续帧传输三点。
- 帧传输双路径与 64MiB 帧上限核过：4K RGBA ≈33MB < `MAX_FRAME_SIZE`；超限走 `FrameTooLarge` 错误码。
- Lint 增补 v1.1 专项检查（结构/语义/引用/拟新增命名隔离四项均 Pass）；D-RFC-1~4 门禁、批复前禁令（不改代码/不装 SDK/不变 feature 表）不变。
- 本轮零代码改动；修订历史 v1.1-draft 行已记。

### 2026-10-24 M3 前置设计轮（A1 + seek 归因）

- 新增 [DESIGN.md](evidence/2026-10-24-m3-prep-design/DESIGN.md)（设计准备，零代码改动）：
  - **A1 时钟批内对齐设计稿**：缺陷机制源码级确认（三入口同构，批长 ~23.3ms > 16ms 泵节拍 → 每 tick 整批消费 → rate=B/16≈1.45）；修复 = 样本级队列（`BatchSampleQueue` 批内精确消费，EOF/loop/seek/skip 语义不变），验收 rate 收 [0.9,1.1]、漂移重测钉带。
  - **seek 冷路径归因闭合**（M1 Warning 解除）：三方交叉证据——fixture 块级探针（1440 块仅 30 keyframe 全在偶秒；tc=15000/45000 非 keyframe、30000 keyframe）+ matroska-demuxer 0.8.1 源码（broad phase cue 二分落簇首、narrow phase 落首个 ≥ target 块、`CuePoint` 私有无 API）+ `decode.rs:258-304` 判定。因果链：mid-GOP 目标 → phase-① 判定失败 → phase-② 全量前向解码（302ms@15s、1000ms@45s，∝ target；30s keyframe 对齐 1.1ms）。M1 ANALYSIS §3.4「冷 I/O」归因就此修正（解码器重建降为次项）。
  - **A4 修复设计**：cue 锚定窄化 fallback（Cues 自解析 ~60 行 + ≤ target 最近 cue 重 seek + 前向解码 ≤ GOP），O(target)→O(GOP)；无 Cues 流保留全量前向为最终回退。
- RFC v1.2：§7 新增 A4 行 + 依赖行更新 + §8 seek 回归行 + §11 Warning 标注已归因 + §13 v1.2 行；D-RFC 门禁与批复前禁令不变。
- 本轮探针/源码查阅均为只读（/tmp 脚本不入仓，cargo registry 只读）；未改代码、未装 SDK、未变 feature 表。
