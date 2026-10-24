# 媒体运行时隔离控制面

**日期**：2026-10-24
**状态**：Active（**M1 基线测量完成**——见 [2026-10-24 evidence](evidence/2026-10-24-m1-baseline/ANALYSIS.md)；**M2 RFC 草案完成待用户评审**——[docs/specs/media-runtime-isolation-spec-rfc.md](../../../specs/media-runtime-isolation-spec-rfc.md) D-RFC-1~4 决策点）。
**当前卡点**：用户评审/批复 RFC（D-RFC-1~4：方案 C 立项 / A 批次随批 / 方案 B 核查 / decode-h264 启用）。批复前不改代码、不装 SDK、不变 feature 表。M1 待决输入（时钟 rate 1.454、NullSink）已入 RFC 方案 A 批次与 TBD-3。
**下一步**：用户批复后按 RFC §7 批次推进——D-RFC-2 随批即启 A1（时钟批内对齐）；D-RFC-1 批准则并行 C1（protocol 媒体消息族 + `media-process` feature 骨架）。若改选 A-only，DC-3 按 DC-5 由用户认可延期/调整后收口。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | **已测**（60s 基线 + 复制链 C1–C5 + 权限表，evidence 2026-10-24） |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | **RFC 草案完成**（三路线对比 + 许可/回滚，待用户批复） |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | NOT_STARTED |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | 待决（M1 核查：本平台 /dev/dri 缺失，无硬解路径） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | **完成**（测量面成立，收口待评审） | [2026-10-24 基线](evidence/2026-10-24-m1-baseline/ANALYSIS.md) |
| M2 | 方案/许可证/回滚 RFC | **草案完成待批复**（决策门=用户） | [media-runtime-isolation-spec-rfc](../../../specs/media-runtime-isolation-spec-rfc.md) |
| M3 | 已确认平台与隔离切片 | NOT_STARTED（候选清单已备） | 无 |
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
- 三缺口盘点在位核对：part06.js:10084（autoplay 绕桥）、part06.js:16350（死钩子）、part03.js:23558/23610（play-only 武装）。
- 守卫与证据：执行阶段 test-guard（900s/4GB）；原始输出 m1baseline-raw.log；无残留进程。本轮未安装 SDK，未改 feature 门。

### 2026-10-24 M2 RFC 轮

- 新增 `docs/specs/media-runtime-isolation-spec-rfc.md`（设计交付，需求权威=goal DC 表）：三路线对比（A 现状增强 / B 平台硬解 / C 独立媒体进程）、许可/SDK/分发核查（仓内已批 vs 待验证分表）、分批实施与回滚切点（A1–A3/C1–C4/B1–B2+）、D-RFC-1~4 决策点。推荐 A→C、B 暂缓；C 为 DC-3 唯一达成路径。
- 独立评审（lei-spec-rfc §4 独立视角）：0 blocker/2 major/5 minor 全部修复——D-RFC ID 与前 RFC 撞名（§9 消歧 + 全文标「前 RFC」）、§2.2 数字与 raw log 对齐；MSE 面属 `decode-h264` feature 门控事实补入 ANALYSIS §5。
- Lint：26 Pass/1 Warning/0 Fail/3 Skip/4 PendingReview/5 N/A；决策草案就绪、实施未就绪（PendingReview 门）。
- 本轮零代码改动；无 SDK 安装、无 feature 表变更。
