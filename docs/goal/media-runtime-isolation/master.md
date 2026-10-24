# 媒体运行时隔离控制面

**日期**：2026-10-24
**状态**：Active（**M1 基线测量完成**——长视频 60s fixture、帧复制链、权限表、平台核查、三缺口盘点齐备，见 [2026-10-24 evidence](evidence/2026-10-24-m1-baseline/ANALYSIS.md)；M2 RFC 架构门仍未解锁，不随 M1 自动解锁）。
**当前卡点**：M2 RFC 架构门待用户解锁。M1 产出两个待决输入：media clock 量化缺陷（rate≈1.454，音频批粒度对 16ms 泵每拍平均 +batch/2 超越）与音频输出 NullSink 生产行为；均归 M3/RFC 议题，本轮不修。
**下一步**：待用户解锁 M2 后，以 M1 证据（容量 34–56× 实时、拷贝面 C1–C5、权限表、平台无硬解）为输入写方案 RFC（读 lei-spec-rfc）；M3 切片候选清单已备（时钟批内对齐、三缺口修复、C2/C4 拷贝优化）。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | **已测**（60s 基线 + 复制链 C1–C5 + 权限表，evidence 2026-10-24） |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | NOT_STARTED（M1 已测 media 层时钟/seek 真值，进程契约归 M2 RFC） |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | NOT_STARTED |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | 待决（M1 核查：本平台 /dev/dri 缺失，无硬解路径） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | **完成**（测量面成立，收口待评审） | [2026-10-24 基线](evidence/2026-10-24-m1-baseline/ANALYSIS.md) |
| M2 | 方案/许可证/回滚 RFC | NOT_STARTED（架构门待解锁） | 无 |
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
- 关键测量：纯视频 60s 解码 1076ms（≈56× 实时，~1 核）；A/V 双轨 1733ms；空 tick p50=400ns；cue seek 精确（300ms 冷/1.1ms 热/932ms）；漂移 p50=11ms。**FINDING**：media clock rate=1.45402（音频批量化），断言收 sanity 带，修复归 M3。
- 三缺口盘点在位核对：part06.js:10084（autoplay 绕桥）、part06.js:16350（死钩子）、part03.js:23558/23610（play-only 武装）。
- 守卫与证据：执行阶段 test-guard（900s/4GB）；原始输出 m1baseline-raw.log；无残留进程。本轮未安装 SDK，未改 feature 门。
