# 媒体运行时隔离控制面

**日期**：2026-10-25
**状态**：Done（2026-10-25 收口归档；**D-RFC-1~4 已批复**——方案 C 批准 / A 批次随批复即启 / B 核查暂缓 / decode-h264 维持默认关，RFC v1.9 §10 批复记录；M1 基线 + M2 RFC 两项交付 + M3 前置设计八片齐备，[m3-prep-design DESIGN](evidence/2026-10-24-m3-prep-design/DESIGN.md)；**A1–A4 + C1 + C2 切片 1/2 + C3 + C4 已落地**——A 批次四片（rate 1.00000 / 三缺口闭合 / 内存收敛 / seek 冷路径 O(GOP)）+ C 批次隔离面（协议消息族与 media-host 骨架 / 解码内核与自走时钟 / registry 开态客户端切换 / 崩溃检测与退避重启）+ **C4 配对验收**（三态 M1 零回归 + 帧摘要三态全等 + 代际风暴零泄漏 + 能力真值三对照 + 30min 资源审计；配对抓出并订正两缺陷：A1 解码超前 pts 锚定回归 → 样本锚定、宿主 replay 流末残留位置拖拽 → replay 跳过位置 seek，feature 默认关），偏差记录 DESIGN §2.4/§3.5/§6.5/§8.6/§9.6/§10.7/§10.8/§10.9，[切片 2 证据](evidence/2026-10-25-c2-slice2/ANALYSIS.md)，[C3 证据](evidence/2026-10-25-c3-crash-recovery/ANALYSIS.md)，[C4 证据](evidence/2026-10-25-c4-paired-acceptance/ANALYSIS.md)；**DC-5 收口审计完成**——RFC §10 批复落地对照 + DC-1~4 逐项证据复核 + 最终支持范围 + 常驻回归清单，[审计](evidence/2026-10-25-dc5-final-audit/ANALYSIS.md)）。
**当前卡点**：无。**goal 已收口**（2026-10-25）——DC-1~5 全部有证闭合；`media-process` 启用维持默认关默认态（启用须用户授权，见缺口与待决表）。
**下一步**：无。goal 完成归档；重开条件 = 用户授权 `media-process` 默认启用（另启产品切片）或新媒体架构议题。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | **已测**（60s 基线 + 复制链 C1–C5 + 权限表，evidence 2026-10-24） |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | **RFC v1.9 已批复**（三路线对比 + §9 IF 级接口契约；方案 C 立项） |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | **已闭合**（C3 SIGKILL 注入契约 8/8 + 退避重启 + reload 恢复 + shim march Error face 页面事件通路） |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | **已裁决**（D-RFC-1 方案 C 批准立项 / D-RFC-3 平台硬解暂缓 / D-RFC-4 decode-h264 维持默认关；M1 核查：本平台 /dev/dri 缺失，无硬解路径） |
| `media-process` feature 启用时机（RFC §10「C4/M4 后另议」） | 默认态 = 维持默认关（启用属产品默认面变更，须用户点名授权方动——rule 11；无授权即现状，D-RFC-4 decode-h264 同款先例）。双侧事实存 [审计 §三](evidence/2026-10-25-dc5-final-audit/ANALYSIS.md) | **无行动待决**（重开条件 = 用户授权启用，另启产品切片，不等同本 goal 重开） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | **完成**（测量面成立，收口待评审） | [2026-10-24 基线](evidence/2026-10-24-m1-baseline/ANALYSIS.md) |
| M2 | 方案/许可证/回滚 RFC + protocol 接口契约 | **v1.9 已批复**（D-RFC-1~4 用户裁决） | [media-runtime-isolation-spec-rfc](../../../specs/media-runtime-isolation-spec-rfc.md) |
| M3 | 已确认平台与隔离切片 | **完成**（A1–A4 + C1–C4 全部落地并通过 M4 配对验收——rate≡1.0 + 三缺口闭合（bg02-media 基线 v2 复评再确认）+ 内存面收敛 + seek O(GOP)；protocol 17 变体 + media-host 骨架 + registry 开态客户端切换（HostMediaFacade push 镜像/21 面分流/MSE 内联豁免/IF-006 不回落）+ 崩溃检测双路 + 状态机退避重启 + SIGKILL 注入契约 8/8；`media-process` feature 默认关） | [DESIGN.md](evidence/2026-10-24-m3-prep-design/DESIGN.md) |
| M4 | 配对媒体、故障和资源审计 | **完成**（三态配对矩阵 + 帧一致性 + 代际风暴 + 能力真值 + 30min 资源审计全过；抓出并订正 A1 解码超前回归与宿主 replay 拖拽两缺陷） | [2026-10-25 C4](evidence/2026-10-25-c4-paired-acceptance/ANALYSIS.md) |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | MEDIA-01-cpu-copy-and-capability | **已测成立**（容量/复制链/权限/能力对照；C4 补能力真值三对照 + 配额端到端 QuotaExceeded；D4 对齐——宿主补申报 ogg-vorbis/wav + aac/webm-opus 主张实解验证，8/8 基础主张真值全验；收口待用户评审） | evidence/2026-10-24-m1-baseline + 2026-10-25-c4-paired-acceptance + 2026-10-25-d4-capability-alignment |
| DC-2 | MEDIA-02-clock-seek-and-state | 部分（media 层时钟 rate FINDING 已修：A1 落地 rate=1.00000；cue seek 精确、漂移 p50=11ms；seek 冷路径 A4 窄化 O(GOP)——mid-GOP 302ms→18ms 且不随 target 线性增长；页面可见时钟缺口已修：A2 三缺口闭合 + **bg02-media 基线 v2 复评通过**——4/4 腿音频/视频桥循环健康，三根因修复（loop attr 推送/march 边沿拍/音频重复登记幂等）+ 2026-10-23「显式 play 未武装」误读订正；C4 补帧摘要三态全等 + 代际风暴零泄漏 + 三态 M1 零回归） | evidence/2026-10-24-m1-baseline + m3-prep-design（含 m1baseline-a1/a4-post.log）+ 2026-10-25-c4-paired-acceptance + 2026-10-25-bg02-media-v3 |
| DC-3 | MEDIA-03-decoder-crash-recovery | 部分（C3 崩溃检测/重启面落地——SIGKILL 注入契约 8 场景全绿：error 镜像可观察/renderer 存活/重启重喂可播/重启耗尽 Failed 无内联回落/reload 恢复；C4 补宿主 replay 拖拽缺陷订正（常驻单测门）+ 30 轮重播审计单 host 零崩溃；shim march Error face 已接线——桥 isErrored 真值 → 元素 error 事件（常驻门 test_media_march_error_face_dispatches_element_error_c3）；页面级端到端终验未做） | evidence/2026-10-25-c3-crash-recovery + 2026-10-25-c4-paired-acceptance |
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

### 2026-10-24 M3 前置设计轮 II（A2 + GOP 抽样）

- DESIGN.md 追加 §6/§7（同日第二轮，仍零代码改动）：
  - **缺口定性修正**：`_zwMediaTimeMarchesOn` 在 runner 面（testharness.rs:9290 泵 tick）有调用，死的是**生产 renderer 面**（媒体泵不调）；M1 ANALYSIS §7 订正注追加。缺口 1+3 确认同面：`_zwMediaAutoplay` 挂 load 序列末（part06.js:10273）= 上游移交定位的「load 时武装环节」。
  - **Fix 1 设计**：play 内核提取 `_zwMediaPlaySteps`，IDL play 与 autoplay 共用（spec#ready-states potential autoplay 分支原义）；风险 = 事件时序切 play 算法派发面（WPT event 族门禁）；收益 = bridgeOn/重试/pending-seek/rate-gain-loop 同步/ended-loop 全面继承。
  - **Fix 2 设计**：any_playing 门控 + `submit_script_priority` 裸 march 脚本（rAF 泵 runtime.rs:4152 同通道先例）；不加 page_hidden（播放推进不随可见性暂停）；否决 shim 自泵/getter 镜像两备选并记录理由。
  - **WPT 上游 GOP 抽样**：4 webm 全量（white 2s 规则 GOP / test、movie_5 近全 intra ~20-23ms / movie_5 仅 1 cue 点）——A4 待验证项闭合，movie_5 稀疏 cue 形态证实 A4 全量前向回退分支为必要路径。
- RFC v1.3：A2 行补设计稿指引 + 回滚格（Fix 1/Fix 2 独立）；无决策/门禁/批次范围变化。

### 2026-10-24 M3 前置设计轮 III（A3）

- DESIGN.md 追加 §8（同日第三轮，仍零代码改动）——**A 批次四片设计稿齐备（A1/A2/A3/A4）**：
  - **C4 快照 Arc 化**：`duplicate_for_snapshot`（image_cache.rs:1340，唯一调用方 webview.rs:1713 → runtime.rs:1443 每帧发布）逐条深拷贝全缓存；「像素插入后不可变」（image_cache.rs:31 既有 R3254-M2 不变量）使 `CacheEntry.data` Arc 化语义安全（hit-test 缓存 runtime.rs:1453 与 svg_source 先例同面）；快照本体从不出进程（跨进程只传 IpcImagePayload），深拷贝纯浪费。
  - **C2 扫描跳过**：视频帧 `ImageData::from_rgba`（video_registry.rs:1188/1204）每帧 2 次全帧读（纯色检测 + content_hash）；`from_rgba_variable` 跳纯色检测，图片解码路径保持扫描。
  - **源驻留封顶**：`sources`/`av_sources` 无总量上限（M1 §5 缺口）；**av_sources 有 seek 重建功能性依赖**（seek :945-959 字节源）——设计为 256MB 总预算 + 非在播 LRU 驱逐（av_sources 随 player 存活保留）+ 准入拒绝走既有错误面。
  - 回滚：三子项各自单提交。
- RFC v1.4：A3 行补设计稿指引；无决策/门禁/批次范围变化。

### 2026-10-24 M3 前置设计轮 IV（C1）

- DESIGN.md 追加 §9（同日第四轮，仍零代码改动）——**C 批次首片实施级设计就绪**：
  - **命名冲突排查闭合**：`ProcessRole`（channel.rs:20-31）无 Media 变体、`IpcMessageKind` 无任何
    Media* 变体（最近邻 ImageDecode* :119/:121、CompositorFrame :125）——IF-002~006 全族 14 个拟
    新增变体命名零冲突。
  - **wire format 草案**：控制面 7 变体（Register/DataChunk/Play/Pause/Seek{request_id}/Release/
    Shutdown）+ host 面 7 变体（Ready/SourceReady/DataAck/StateUpdate{epoch}/SeekResult/Frame/Error）
    + MediaPlaybackState/MediaErrorCode 伴生枚举；全走 IpcMessage 信封 + bincode，受 64MiB 上限
    （DataChunk 分片、Frame 超限 → FrameTooLarge）。
  - **骨架边界**：C1 = `apps/media-host`（run_role 先例）+ `--type=media` 接线 + 懒 spawn/
    SPAWN_FAILED 永久内联先例镜像 + 空源生命周期（保契约可测）；解码内核（携 A1 修复后实现）
    C2 接；`zeroweb-media-` shm 前缀 + `ZW_MEDIA_SHM=0` 关断镜像 frame_shm 先例；feature
    `media-process` 默认关 = D-RFC-1 门。
  - 契约测试计划：bincode 往返/超限拒绝/`--type=media` 映射单测 + 内联回落与 kill-switch 集成
    （`#[ignore]` + test-guard 先例）。
- RFC v1.5：C1 行补设计稿指引；无决策/门禁/批次范围变化。

### 2026-10-24 M3 前置设计轮 V（C2）

- DESIGN.md 追加 §10（同日第五轮，仍零代码改动）：
  - **C1 wire format 缺口发现并闭合**：桥面 setRate/setGain/setLoop（video_registry.rs:1711-1713，
    起播同步面 part03.js:23633/23639/23647）在轮 IV 草案控制面无对应消息——拟新增 3 变体
    （MediaSetRate/MediaSetLoop/MediaSetGain，全族 14→17）+ MediaSourceReady 扩 mediaMeta 三元组。
  - **host 自走时钟设计**：audio cursor 主时钟（携 A1 BatchSampleQueue 同构语义），事件循环按
    (音频采样截止 | 帧演示截止) 等待，无 60Hz 轮询；推送校准闭合 IF-003「≥100ms」实施值——
    跃迁即时 / media_time 随帧（pts_ms 自带）/ 纯音频 250ms 心跳（与 shim timeupdate 节流
    part06.js:16463 同值，页面可见节拍零变化）。
  - **renderer 变换**：march 页面语义原样保留（250ms 节流/played ranges/ended/loop 全是 DOM 面，
    宿主不接管）；仅供数方式变——bridge getter backing 转 push 镜像；生产泵媒体分支转被动
    （帧队列排空 → ImageCache 注入 → interest 标脏，rAF 泵先例通道）。
- RFC v1.6：C2 行补设计稿指引 + C1 行 14→17 修订 + §13 历史行；无决策/门禁/批次范围变化。

### 2026-10-24 M3 前置设计轮 VI（C3）

- DESIGN.md 追加 §11（同日第六轮，仍零代码改动）——**C 批次 C3 设计稿就绪**：
  - **降级纪律先例订正（RFC v1.7 触发点）**：image_decoder.rs 注释称「spawn 失败→直接进程内解码」，
    **代码为准**——SPAWN_FAILED 置位后 `decode_image` 直接 Err（:205-206），三调用方
    （async_load.rs:1125/:1213 落 Outcome::Error+FailedResource、webview.rs:1509 warn+continue）
    均无内联回落；RFC v1.6 IF-001/IF-006 与本文 §9 的「永久进程内直调」引用失实，§2.2/§9.0/
    IF-001/IF-006 全部改写，IF-006 纪律两条合一（spawn/运行期同归错误面）；失实注释订正归 C1 顺带。
  - **崩溃检测双路**：send 失败即时检测（broken pipe）+ 泵周期 `try_wait` 兜底（先例
    protocol/process.rs:441-459 `exit_reason`、:623-643 check_crashes）；proxy per-webview
    所有权（registry 持有），不采 image-decoder 全局 static PROXY 模式。
  - **状态机与重启**：Idle→Healthy→Restarting（退避 1s/2s/4s，n≤3，握手成功重置）→Healthy|
    Failed（耗尽=永久错误面，不内联）；重启不自动重播/重注册（epoch 重单调），reload 走现有
    导航释放面回 Idle=DC-3 重载恢复路径。
  - **DC-3 契约测试**：真实 SIGKILL 注入 5 场景（error 事件可见/renderer 存活/kill 后 play 重启/
    3 连 kill 永久错误+无回落断言/reload 恢复），`#[ignore]`+test-guard 先例。
- RFC v1.7：C3 行补设计稿指引 + 降级纪律订正（见上）+ §13 历史行；无决策/门禁/批次范围变化。

### 2026-10-24 M3 前置设计轮 VII（C4）

- DESIGN.md 追加 §12（同日第七轮，仍零代码改动）——**C4/M4 验收实施级方案就绪，全部批次设计稿齐备（A1–A4 + C1–C4）**：
  - **三态配对矩阵**：feature 关（内联基线）/ 开 + shm / 开 + `ZW_MEDIA_SHM=0`（非 Linux 形态
    代理），固定输入 = M1 两枚 60s fixture，同 commit 同机器，报告头必含版本与来源（DC-1）。
  - **仪器复用**：M1 harness 五场景场景体与 M1BASELINE 输出行原样保留，仅构造入口按态切换
    （直调 vs proxy）；帧一致性 = 三态逐帧 RGBA 哈希序列比对（§8 双路径行集成化）。
  - **判据表**：rate [0.9,1.1]（A1 同带）、seek resolved_ms 三态相等、漂移 p50 内联+5ms、
    像素全等、开态墙钟 ≤ 内联 ×1.5（IPC 税上界）、旧 epoch 零泄漏——拟新增值 C4 定稿可调，
    语义带绑定既有契约；不满按域回修重跑，不改判据凑数。
  - **资源审计**：每 renderer 恰 1 host、host RSS、shm 槽位有界、源驻留 ≤256MB（A3 复测闭环
    M1 §5 缺口）、暂停态 host CPU≈0（自走时钟验证）、C1–C5 复制链开/关态增量表。
  - **能力三对照**（DC-4）：capabilities vs canPlayType vs 实解一致 + QuotaExceeded 面；
    本机无 /dev/dri 即软解如实报用例。
- RFC v1.8：C4 行补设计稿指引 + §13 历史行；无决策/门禁/批次范围变化。**设计前置至此全部完成，
  后续推进唯一入口 = 用户批复 D-RFC-1~4。**

### 2026-10-24 D-RFC 批复轮（M3 实施启动）

- **用户四项裁决**（AskUserQuestion 逐项确认）：D-RFC-1 **批准方案 C**（独立媒体进程立项）/ D-RFC-2 **随批复即启**（A1–A4 按序推进）/ D-RFC-3 **暂缓**（方案 B 核查不启动）/ D-RFC-4 **维持默认关**（decode-h264 不启用）。
- RFC v1.9：状态 草稿→已批复；§10 决策表增批复列；§13 历史行。批复前禁令解除；`media-process` feature 仍默认关（启用时机归 C4/M4 后另议）。
- master.md 状态面同步（状态/卡点/缺口表/M2/M3 里程碑）。本轮纯文档；A1 实施随后同日另轮记录。

### 2026-10-24 A1 时钟批内对齐实施轮（M3 首切片落地）

- **A1 修复落地**（`crates/webview/src/video_registry.rs` 单提交回滚点）：新增私有 `BatchSampleQueue`（样本级队列：交错样本 pending + 格式登记 + 毫秒→帧定点换算余数 `frac_rem` 跨 tick 累积），三入口（AudioEntry / WebmAudioEntry / MseAudioEntry）advance_to 同构改造——解码批入队（解码超前有界 ~1 批）、按 tick 目标帧数精确写出、全额写出时 `cursor_ms = target`（rate ≡ 1.0 由构造保证）；EOF/loop/错误停止分支先全量冲队再停（原行为末批整批入 sink 不丢样）；5 个 registry 级流位重置点（audio_seek、seek/play 的 MSE 与 av 复活分支）补 `queue.clear()`。
- **实施偏差三处**记入 [DESIGN.md §2.4](evidence/2026-10-24-m3-prep-design/DESIGN.md)：填充条件改 crossing 语义（`decoded_end < target`——原设计 `pending < need` 会跳过 EOF None 拉）；解码超前/时钟推进耦合不变式（游标只随消费推进，trim 变体失败实证：解耦后 60s 流在 42s 处提前 EOF 丢 18s）；EOF 检测晚一拍（生产 rAF 泵无条件节拍无影响，测试泵条件改 `is_any_playing`——M2 切片 F 用例与 M1 harness 同步修正）。
- **验收**（[m1baseline-a1-post.log](evidence/2026-10-24-m1-baseline/m1baseline-a1-post.log)，release + test-guard）：T3 rate **1.00000**（修复前 1.45402，带宽 [0.9,1.1]）；T5 漂移 p50=11.0ms 与基线持平；T2 av-pair 容量 1206ms/60s（≈50× 实时，基线 1422ms）无回退；T4 cue seek 325ms/1ms/870ms 与基线同量级。lib 786 测试全绿；workspace make test、clippy -D warnings、fmt 过。
- 本轮仅动 video_registry.rs + M1 测试断言收带 + 两文档；zero-media/engine/protocol 零改动。

### 2026-10-24 A2 三缺口修复轮（M3 第二切片，Fix 1/Fix 2 各自单提交）

- **Fix 1（缺口 1+3，autoplay 走 play 算法）**：play 算法内核自 part03 IDL play 提取为 part06 顶层 `_zwMediaPlaySteps`（257 行原文迁移去层，行为面不变），IDL play 薄包装转发；`_zwMediaAutoplay` 删手写置态 + 同步三事件、改调内核（autoplay 不暴露 promise）——bridgeOn/退避重试/pending-seek 补推/rate-gain-loop 同步/ended-loop 全继承，**生产 autoplay 元素首次可达 `__zwVideoBridge.play`**（缺口 1 的 shim load 武装环节 + 缺口 3 audio 形态同面闭合）。元素身份留档（`_zwSel`/`_zwHandle`）内移内核（偏差记 [DESIGN §6.5](evidence/2026-10-24-m3-prep-design/DESIGN.md)）。
- **Fix 2（缺口 2，生产 renderer march 接线）**：`apps/renderer/src/runtime.rs` 媒体泵 any_playing 块增 `submit_script_priority` 裸 march 提交（rAF 泵同通道同形态；门控 any_playing + javascript_enabled，**不加 page_hidden**——播放推进不随可见性暂停）——timeupdate 250ms 节流/cue active/ended 事件面生产首启。
- **新增语义测试**：`test_media_autoplay_walks_play_steps_a2fix1`（autoplay 元素加载序列末 → 桥 play 同步命中绝对 URL + paused=false + play/playing/timeupdate queued 泵后派发锁新时序 + pause 走桥证 bridgeOn）。
- **验证**：engine lib 3473 全绿（既有 media event 族 + 新测试）；wpt-runner 225 全绿；make test、clippy -D warnings、fmt 过。
- **限制**：`testharness-media` 门禁本环境未跑——外网不可达（fetch curl 超时），且该子集首批本就不含 ready-states/autoplay/event_* 真解码族（后置声明）；断言面由 engine 桥测试承载，有网环境补跑。
- 共享面占用：engine shim（part03/part06）+ bridge 测试、renderer runtime.rs 一处提交；runner 面行为随 shim 语义同迁（门禁族全绿）。bg02-media 基线 v2 回补（audio/autoplay INCONCLUSIVE 复评）仍归后续验证切片。

### 2026-10-24 A3 内存面收敛轮（M3 第三切片，C4/C2/封顶各自单提交）

- **C4 快照 Arc 化**（`2003c55dc`，render-foundation）：`CacheEntry.data: Arc<ImageData>`——页面隐藏快照与源缓存共享像素句柄零拷贝（R3254-M2 像素不可变前提），`snapshot_entries` 深拷贝面不变；新增 Arc 共享断言单测。
- **C2 视频帧扫描跳过**（`aa540dc4d`）：`ImageData::from_rgba_variable` 视频帧专用（`solid_color: None`，content_hash 照算），registry 两处 tick_all 帧插入切换；图片解码路径保持 `from_rgba` 扫描。
- **源驻留封顶**（`340fcde8b`，webview）：`sources + av_sources` 256MB 总量准入（M1 §5 权限表缺口闭合）——超限驱逐非在播 sources 键，再驱逐无存活 player 的 av_sources 孤儿键，仍超拒绝登记走既有 play 失败面；在播 player/audio entry 键永不驱逐；play 字节搬移总量守恒不加检；驱逐 `_with_budget` 参数化 4 单测。
- **偏差四处**记入 [DESIGN §8.6](evidence/2026-10-24-m3-prep-design/DESIGN.md)（LRU→迭代序任一 / play 搬移不加检 / audio 准入失败 plain return / 预算参数化）。
- **验收**：M1 harness 5/5（[m1baseline-a3-post.log](evidence/2026-10-24-m3-prep-design/m1baseline-a3-post.log)：rate 1.00000、漂移逐位持平、seek 同量级；容量在噪声带内，pure-video tick p50 411→401us——§8.5 墙钟预期下降未显现，C2 单帧省读低于噪声，C4 快照路径无触发，如实记录）；bench-gate 定向 2/2 PASS；render-foundation 116 + webview lib 790 全绿；workspace clippy -D warnings、make test、fmt 过。
- 共享面占用：render-foundation image_cache + webview video_registry 两文件；zero-media/engine/protocol 零改动。

### 2026-10-24 A4 cue 锚定窄化 seek 轮（M3 第四切片，A 批次收官）

- **A4 落地**（`crates/media` 单提交回滚点）：`VideoDecoder` 开流时对源字节平扫 Cues（EBML 三层遍历 Segment→Cues→CuePoint/CueTime，零新依赖，`cue_times_ms` 锚点集缓存）；`seek_to_ms` 插入 cue 锚定窄化回退——≤ target 最近 cue 重 seek（簇首 keyframe 起步）前向解码至 ≥ target，O(target)→O(GOP)；锚块非 keyframe（稀疏/relative-position 形态）落既有流首全量前向，错误面不新增。偏差三处记 [DESIGN §3.5](evidence/2026-10-24-m3-prep-design/DESIGN.md)（锚点只存 CueTime / 多轨重复不过滤 / 非 keyframe 锚回落 ③）。
- **验收**（[m1baseline-a4-post.log](evidence/2026-10-24-m3-prep-design/m1baseline-a4-post.log)，release + test-guard）：mid-GOP 冷 seek 15s **302ms→18.3ms**（16×）、45s **1000ms→18.4ms**（54×），15s/45s 差 62µs——线性增长断言 + GOP 上界断言入 harness T4 常驻；30s 直落 1.0ms 不变；drift/rate/容量逐位或噪声带内零回归。单测三新增（cue 平扫真值 / 窄化精确 / 稀疏回退），media lib 54 + webview lib 790 全绿；workspace clippy -D warnings（含 quickjs 组合）、make test、fmt 过。期间修复 rebase 带入的 s68（R5275）`collapsible_if` lint——quickjs feature 组合 clippy 挂（extract.rs 机械坍缩，独立提交非本切片代码）。
- **限制**：CueRelativePosition 产物流抽样复核待验证（本环境无 ffmpeg 抽样面，归后续验证切片）；§7 movie_5 稀疏形态以等价 fixture 覆盖（wpt-data gitignore 不可作单测依赖）。
- 共享面占用：zero-media decode.rs + tests.rs + webview M1 harness T4 扩展；registry/engine/protocol 零改动。

### 2026-10-24 C1 协议与骨架实施轮（C 批次首片落地）

- **protocol 媒体消息族**（`crates/protocol` 单提交回滚点）：新增 `media.rs` wire format 模块——`IpcMessageKind` 追加 17 变体（控制面 10：Register/DataChunk/Play/Pause/Seek{request_id}/Release/Shutdown/SetRate/SetLoop/SetGain；host 面 7：Ready/SourceReady/DataAck/StateUpdate/SeekResult/Frame/Error）+ 伴生枚举 `MediaPlaybackState`/`MediaErrorCode` + 常量（`MEDIA_PROTOCOL_VERSION=1`/`MEDIA_SOURCE_MAX_BYTES=256MiB`/`MEDIA_SHM_PREFIX="zeroweb-media-"`，kill-switch `ZW_MEDIA_SHM` 复用 frame_shm `env_linux_default_on`）；`ProcessRole::Media` + `--type=media` 映射；变体**追加在枚举尾**（bincode 1 追加式纪律）。protocol 单测 6 新增（17 变体往返/伴生枚举/Frame 双路径/常量面），331 全绿。
- **media-host 骨架**（`apps/media-host` 新 member，单提交）：`run_role` 镜像 image-decoder 先例——首发 `MediaReady{version, capabilities: []}` 握手（DC-4 不虚报 codec 能力）+ 空源生命周期（register 校验 total_len 上限 → SourceReady(None 元数据)；DataChunk 实收封顶超限摘源 → DataAck/QuotaExceeded）+ 播放控制如实报 `HostInternal`（C2 前无解码内核不虚报状态）+ Release/Shutdown fire-and-forget/退出。单测 4（握手序/生命周期契约含信封 id 配对/配额双面/断连退出）。
- **webview proxy + feature 门**（`crates/webview` 单提交）：`media_proxy.rs` `MediaHostClient`（懒惰 spawn + 5s 握手超时 + 二进制解析链 `ZW_MEDIA_HOST_BIN`→`CARGO_BIN_EXE`→exe 上溯→PATH）——读线程 + mpsc split transport（`try_recv` 不支持管道非阻塞；C2 泵排空队列前置），非匹配消息独立 pending 队列；**IF-006 降级纪律**：传输级失败（spawn/握手/读写/超时）置永久错误面不重试不内联回落，host `MediaError`（配额/播放）为播放级不置永久失败。feature `media-process` 默认关 = registry 直调现状零变化（C1 不改任何调用点）。契约测试 2（`#[ignore]` + test-guard 真实二进制）：全生命周期 + spawn 失败 latch 证明（坏路径失败→恢复合法路径仍失败）。
- **顺带订正与配套**：image_decoder.rs:178 挂名注释订正（「spawn 失败→进程内解码」为失实，实际错误面，RFC v1.7/§11.1 归 C1 项，仅注释）；renderer runtime.rs 收消息 match 补 17 变体臂（媒体走独立 stdio 通道不经主进程 IPC，防 E0004）；Makefile 四处 image-decoder 构建行同列 `-p zero-media-host`。偏差九处记 [DESIGN §9.6](evidence/2026-10-24-m3-prep-design/DESIGN.md)（含 shm kill-switch 集成测试延后 C2：C1 无 Frame 流可驱动）。
- **验收**：protocol 331 + media-host 4 + webview feature-on 定向（media_proxy 3 单测 + 契约 2）全绿；全量 `make test` 过（含 quickjs clippy 组合）；`cargo check -p zero-webview`（默认关，media_proxy 不编译）过。首轮 make test 暴露 renderer E0004（17 变体非穷尽 match）——已修后复跑绿。
- 共享面占用：protocol（消息族）+ webview（feature 门/proxy/注释）+ renderer（match 臂）+ Makefile；registry/engine/media 零改动，默认关路径行为零变化。

### 2026-10-25 C2 切片 1 实施轮（解码内核 + 自走时钟 + 事件面落地）

- **音频管线六类型迁零-media**（`524c487c0`）：`AudioStreamDecoder`/`BatchSampleQueue`/`AudioEntry`/`WebmAudioTrackKind`/`WebmAudioEntry`/`MseAudioEntry` 自 video_registry.rs（2780→2209 行）逐字迁入 `crates/media/src/pipeline.rs`（A1 时钟语义单一真值，内核与 registry 共享；`#[cfg(decode-h264)]` 门随迁）；registry 改 `use zero_media::` 引用，行为零变化。
- **解码内核 + 自走时钟事件循环**（`423c11f44`）：`apps/media-host` 增 `kernel.rs` `MediaKernel` 纯函数面（`handle`/`advance`/`next_wake_ms` 无 IO）——registry 同构播放语义（A/V 音频主钟 `sync_to_media_time`、纯视频墙钟、纯音频游标；seek/replay 重建链同序；懒建解码器 + `apply_stored_controls` 补存 loop/gain）+ 推送面（跃迁即时 / 帧携 pts / 250ms 心跳 / epoch 代际 DC-2 / 配额 256MiB 双面）；`lib.rs` `RoleIo{next_message(timeout), send_reply}` + `run_role_feed`（唤醒截止 = 音频队列排空时刻，纯视频 16ms 泵量子 FIXME 标注待 player next_pts API；空闲无限期阻塞，无 60Hz 轮询）；`main.rs` stdio 拆两半（stdin 读线程 → mpsc + `recv_timeout`，`IpcChannel` 阻塞 recv 无超时是拆分根因）。虚拟时钟单测 15（真实 fixture 全管线）+ lib 5。
- **proxy 事件面**（`4d830f30e`）：`MediaHostEvent{State/Frame/SourceReady/Error}` + `try_recv_event` 非阻塞排空（暂存+入站按到达序合并，推送面出队为事件、响应面留队供 `wait_for` 配对；未连接不触发懒 spawn；传输失效置永久错误面 IF-006，C3 换重启面）。单测 7。
- **真实二进制契约 + 回执 id 配对修正**（`6320292fe`）：内核 play 即时 `Playing` 推送与 pause `Paused` 回执携请求信封 id（**IF-002 闭合**——原 id 0 使 `client.play` 成功路径必超时）；client 增 `pause`/`set_loop`（set 面 fire-and-forget 对齐内核无回执成功面）；契约测试更名 `media_host_contract.rs`（C1「play/seek 必错」断言对 C2 内核失效）扩 4 场景（视频 play→元数据/状态/帧→seek 代际+1→暂停冻结 / A/V / 纯音频 set_loop 回卷代际+1 / 控制面错误面），全绿 3.4s。
- **偏差八处**记 [DESIGN §10.7](evidence/2026-10-24-m3-prep-design/DESIGN.md)（回执 id 配对、纯视频回卷 renderer 驱动、A/V Ended 静音伴音轨、±1ms 位置容差、host 常驻源字节、SourceReady 元数据推送时机=play 后、真实进程首帧 pts≠0、能力真值）；RFC v1.11 记 IF-002 成功面语义与 set 面 fire-and-forget 扩围。
- **门红修复**（既有非本切片，`guarded-clippy` -D warnings 暴露）：`zero-media` `cue_times_ms` 收窄 `#[cfg(test)]`（A4 起唯一调用方在测试面）+ tests.rs 冗余 `mut` + `media_m1_baseline.rs` 文档列表缩进（doc_lazy_continuation）。
- **验收**：media-host 15+5（decode-h264 组合同绿）+ webview media_proxy 7 + 契约 4 全绿；workspace `guarded-clippy` -D warnings 过；全量 `make guarded-test` 过（exit 0）。registry 未切换（`media-process` 仍默认关），默认路径行为零变化。
- 共享面占用：zero-media（pipeline.rs 新增 + 门红两行）+ webview（video_registry 迁出 + media_proxy + 契约测试）+ media-host + protocol transport（MAX_FRAME_SIZE pub）+ RFC/DESIGN/master 文档；engine/renderer/registry 调用点零改动。

### 2026-10-25 C2 切片 2 实施轮（registry 客户端切换 + 宿主面端到端）

- **client 面载荷化 + set 面补齐**（`22c2ab10f`）：`MediaHostClient::play`/`pause` 回执改返 `MediaStateUpdateParams` 载荷（wire 不变，IF-002 id 配对消费面）；新增 `set_rate`/`set_gain` fire-and-forget。
- **桥拆分 + registry 客户端切换**（`0e6ca7aa5`）：`video_bridge_facade_script` + `register_video_bridge_callbacks` + bridge_tests 迁 `video_registry_bridge.rs`（532 行）；registry 2212→1817 行（行数纪律），外部 `zero_webview::video_registry::*` 路径经再导出不变；桥私有字段访问改 `contains_source` pub API。新 `video_registry_host.rs` `HostMediaFacade`（push 镜像 DC-2 epoch 门控/未知键丢弃；8MiB 分片 settle；纯视频 Ended+loop 回卷待办 + flush 镜像归零防 seek 回流末；play 回执后即时排空元数据——`SourceReady` 同批出队使 JS 立读 duration 有真值，排空窗帧缓冲不丢首帧）；video_registry 21 面 `hosts(key)` 早退/并集分流（登记不落内联字节 IF-006；MSE 键天然内联豁免；`tick_all` 并集形态保 MSE 内联 player）；lib.rs 两模块 cfg 门控。
- **测试**：facade 合成事件单测 7 + 契约第 5 场景 `hosted_registry_end_to_end`（登记→play→帧入 ImageCache→元数据升级→seek resolved→暂停冻结→纯视频 loop 回卷观测→clear）+ M1 harness 时钟分叉（开态真实节拍/关态合成钟原样）+ A4 断言 feature 分叉（release 内核冷 seek 19.1ms≈内联 18.2ms 并源；宿主暖对 30s/45s 非线断言；30s 收全量条件化）。
- **验收**：M1 release 宿主面 5/5（226.6s，[证据](evidence/2026-10-25-c2-slice2/ANALYSIS.md)——测试进程 cores 1.001→0.012、tick p50 431ns→3.7µs、时钟 rate 0.99827、1440/1440 帧零丢）+ feature-off release 5/5（20.0s 零回归）+ 契约 5 场景全绿（5.0s）+ `make guarded-clippy` 过 + 全量 `make guarded-test` 过。
- **偏差四处**记 [DESIGN §10.8](evidence/2026-10-24-m3-prep-design/DESIGN.md)（play 回执后即时排空/元数据升级时点、M1 时钟分叉 + A4 feature 界、泵保持 16ms 被动排空、host 不落内联字节 + MSE 内联豁免）。
- 共享面占用：webview（video_registry/video_registry_host/video_registry_bridge/media_proxy/lib.rs/两测试文件）+ 文档三处；renderer runtime.rs 泵调用点零改动（`tick_all` 排空语义内聚）、engine/protocol/media-host 零改动。

### 2026-10-25 C3 实施轮（崩溃检测/重启退避 + SIGKILL 注入契约）

- **状态机 + 双路检测**（media_proxy.rs）：`HostLifecycle{Idle/Healthy/Restarting/Failed}` 替换 failed 布尔——运行期连接失效（send 失败即时 + 泵周期 `poll_health` try_wait 兜底，facade `tick_into` 每 tick 调用）转退避重启（1s/2s/4s n≤3，重连成功重置计数），初始 spawn/握手失败保持直达 Failed（§11.1 纪律）；`shutdown` 回 Idle（reload 恢复）。失效弃置前取 exit status（`exit_reason` 同构 renderer 先例）+ stderr tail（16KiB 有界读线程，host stderr inherit→piped）入 tracing、status 入消息面。
- **崩溃通知 one-shot + 镜像广播**：命令路径检测存 pending 由 poll 下发（播放级错误与传输失效同为 Err 面，广播只认崩溃通知防误播）；poll try_wait 路径同步返回清 pending（不二次投递）；facade 收通知全活跃镜像广播 `Error{HostInternal}`（playing 清除/回卷待办作废/epoch 不变）。真值口：facade `is_errored` → registry `is_errored`（pub）→ 桥 `__zwVideoBridge.isErrored`；`media_host_pid` 诊断/注入面。
- **重启语义**：全新 host 源全丢失、不自动重播/重注册（重喂前 play 报 unknown source）；Restarting 期间命令恒错误面；`clear()` 从「连接保持」改 teardown 回 Idle（§11.4 reload 恢复路径，Failed 随导航清除）。
- **测试**：SIGKILL 注入契约 3 新场景（崩溃广播+隔离 / 退避重连新 pid+重喂可播 / 假二进制驱动三次重试全败达 Failed+reload 恢复）——契约套件 8/8（16.2s）；单测 facade 崩溃广播 + proxy spawn 失败短路 + tail 有界（feature-on 17/17）；registry 内联 24/24（feature-off）。
- **验收**：契约 8/8 全绿（16.2s，[证据](evidence/2026-10-25-c3-crash-recovery/ANALYSIS.md)）+ M1 宿主面 release 复跑 5/5（226.9s 零回归——rate 0.99827、冷 seek 19.0ms、1440/1440 帧零丢）+ `make guarded-clippy` 过 + 全量 `make guarded-test` 过（exit 0）。
- **偏差五处**记 [DESIGN §10.9](evidence/2026-10-24-m3-prep-design/DESIGN.md)（场景 4 到达路径=重启尝试耗尽、one-shot 双路投递语义、stderr tail 本地实现、页面 error 事件链止步桥 getter、clear 改 teardown）。
- 共享面占用：webview（media_proxy/video_registry_host/video_registry/video_registry_bridge/契约测试）+ 文档三处；engine/protocol/media-host/renderer 零改动。

### 2026-10-25 C4 配对验收轮（A1 解码超前订正 + 宿主 replay 订正，代码 `4ea7f9e26`）

- 新增 `crates/webview/tests/media_c4_paired.rs`（验收 instrument，`#[ignore]` 显式运行 + test-guard 包裹，M1 先例）：帧一致性摘要（逐帧 content_hash 序列 fold，参照=解码器直出同管道）、代际风暴（9 跨 seek + 30 帧邻域断言）、能力真值三对照（host 协议实报 vs 实解 vs engine 静态表）、配额端到端、30min 资源审计（30 轮完整重播 + 暂停态空闲探针 + 孤儿核对）。
- **配对抓出缺陷 1——A1 解码超前 pts 锚定回归**：A1（`3f98c4332`）把 decoded_end 锚 batch pts，长 fixture vorbis pts 步长 ≈ 样本步长/1.44 → EOF 提前至 ~70% 样本时刻，AV 轨 1013/1440 帧、视频随 audio cursor 同步冻结；`pipeline.rs` 三入口订正样本锚定（cursor+queued 基面），pts 连续轨零变化。M1 原判据只数纯视频呈现帧故三批未觉——帧摘要配对首次现形，回归自 A1 起潜伏（a1-post 证据已是 1013）。
- **配对抓出缺陷 2——宿主 replay 流末残留位置拖拽**：renderer 镜像 Ended 后保留末次 Playing 读数（流末），facade play 原样回传 wire，内核 replay 重建后照单 seek 拖回流末 → 重启即再 Ended（审计重播轮 2 tick 坍缩）；`kernel.rs` replay 面跳过位置 seek 链（spec「ended playback」步 6.4），新增常驻单测（内核 16/16、契约 8/8）。
- **三态 M1 零回归**：内联 5/5 20.41s / 宿主 5/5 226.33s / 宿主+shm=0 5/5 226.31s——1440/1440 帧、rate≥0.998、宿主客户端 CPU ≤0.013 核、seek 不劣于内联 +10ms 带。
- **帧一致性/风暴**：digest `3f970f43b2bfb694` 三态全等（presented=reference=1440）；风暴 30 帧邻域零泄漏（匹配器最近邻订正——48 帧内容周期致最低命中假回跳）。
- **能力真值/配额**：6 项实报 4 验证 2 无 fixture；欠申报 ogg-vorbis/wav（可解未列）；配额 QuotaExceeded 端到端。
- **30min 审计**（1807s）：30/30 轮完整重播单 host 零 respawn（IF-001）、RSS 峰值 18.4MB 平台化、暂停态空闲 5s CPU 0ms、shm 0 段、退出无孤儿。
- **偏差 D1–D7**（shm 传输面未实施第三态同传输 / 容量判据等价解释 / 风暴匹配器周期处理 / 能力清单偏差 / 审计协议时钟面二订 / 复制链增量表代理 / renderer 驻留代理）记 [C4 ANALYSIS §六](evidence/2026-10-25-c4-paired-acceptance/ANALYSIS.md)。
- 门禁：guarded-clippy 过 + 全量 guarded-test 过（exit 0）+ 提交门禁 PASS；证据 13 log + ANALYSIS 同批入库（[ANALYSIS](evidence/2026-10-25-c4-paired-acceptance/ANALYSIS.md)）。
- 共享面占用：zero-media（pipeline 解码超前锚定）+ media-host（kernel replay 面）+ webview（新增测试文件）+ goal 文档；其余 member 零改动。

### 2026-10-25 shim march Error face 接线轮（C3 链页面级收口）

- engine shim `_zwMediaTimeMarchesOn` 每 tick 新增 error 面（M3 扩批 XXXV，DESIGN §10.9-16 接线）：桥 `__zwVideoBridge.isErrored` 真值翻转 → `playing=false` + 元素 `error` 事件单次派发（`_zwMediaFire` 标准面、不冒泡、one-shot `_zwErrorDispatched`，错误态保持到下次 load，spec #concept-media-error）。置于 ended 面（XVI）之前——error 先翻 `playing=false`，同 tick 抑制假 ended。
- 常驻回归门 `test_media_march_error_face_dispatches_element_error_c3`（js_dom_bridge_tests/part09.rs）：play 后桥假值 march 无事件 → 真值翻转 march 派 error + paused 置位 → 再 march 不重复派。
- 生产链路补齐：runtime 每 tick march（A2 Fix 2）× registry 镜像 error 态（C3）× 桥 isErrored 真值口（C3）→ **页面 error 事件**——C3 契约场景 1「kill host → 页面 error 事件可见」的页面级断言通路。dormant：feature-off registry 无 error → 桥假值不触发；桥缺 isErrored（如 WPT runner）→ 面不触发。边界：仅 playing 时间轴错误投递（march 受 any_playing 门控），暂停时错误投递超出本切片。
- 门禁：新测试过 + guarded-clippy 过 + 全量 guarded-test 过（exit 0，87 套件零失败）。首轮门禁与 integration 套件单独复跑各遇负载型 flake（`details_summary_activation_and_cancellation_conform_across_hosts` 三宿主事件序 / `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight` 并发，逐轮失败面不同），干净树 stash+重编译对照同样复现——既有 flake 与本改动无关，全量复跑即绿；flake 根修另立工作项（test-guard 包裹复跑 `zero-integration-tests` 套件可复现）。
- 共享面占用：engine（js_dom_shim/part06.js +22 行 + js_dom_bridge_tests/part09.rs +79 行）——zero-web stream 面；其余 member 零改动。

### 2026-10-25 D4 能力清单对齐轮（宿主欠申报订正 + 无 fixture 主张实解验证）

- 宿主 `codec_capabilities`（apps/media-host/src/kernel.rs）补申报 `audio/ogg; codecs=vorbis` + `audio/wav`——内核路由面（`AudioDecoder::open` symphonia probe）早已可解（C4 ANALYSIS §四欠申报两条目），申报与路由同源，「能播却报不支持」可播性收窄风险消除；engine 静态表 `_cptTable` 两侧真值一致。
- C4 instrument 能力真值段对齐（media_c4_paired.rs）：原「无 fixture 未验」两主张实解落位——`audio/mp4; codecs=aac` 以 sample-mp4-h264.mp4 的 AAC LC 音轨验证（symphonia isomp4+aac 路由，README 生成记录在案），`audio/webm; codecs=opus` 以 wpt-data 既有资产 movie_5.webm 直解验证（open_webm_opus_audio_track，与 zero-media 常驻单测同源；fetch 白名单按需不入库）。不新增 fixture、不破坏 fixtures 目录来源清白声明。8 项基础主张全部「在列 ⇔ 实解」双断言；原未验记录段/欠申报探针段移除。
- 宿主单测常驻断言（lib.rs handshake 测试）：ogg-vorbis/wav 必须在列——欠申报回归默认构建即红，不依赖 instrument 显式运行。
- 验证：capability instrument 过（8/8 capability-verified + QuotaExceeded 不变，[证据](evidence/2026-10-25-d4-capability-alignment/ANALYSIS.md)）+ 宿主 handshake 单测过；全量 make test 过（与 bg02 复评轮同批门禁，见该轮执行记录）。
- 共享面占用：media-host（kernel.rs 能力表 + lib.rs 单测）+ webview（instrument）+ goal 文档；零 protocol/engine 改动。

### 2026-10-25 bg02-media 基线 v2 复评轮（三根因修复 + 误读订正 + 基线 v2）

- **A2 三缺口闭合判定 PASS**：autoplay 腿音频桥武装 + 循环健康（audio-clock 1.59→0.63→1.67 跨流末回卷）；显式 play 腿 shim 埋点实测两轮 load `#a` 均 `hit=true`——2026-10-23「显式 play audio 桥武装失败」系终态误读（单遍播完 → EOF 停播 → 游标复位 0 → shim playing 卡 true，5s 首采样全落 EOF 后），已订正入 [ANALYSIS §三](evidence/2026-10-25-bg02-media-v3/ANALYSIS.md)。
- **根因修复一（shim play 内核）**：setLoop 推送裸读 `_pMs.loop` 漏解析属性形态（`<audio loop>` 静态 HTML）→ registry loop 关 → 音频 entry 流末停播、视频 Ended 冻结。推送读与 IDL getter/march 同源（dirty 镜像优先 + attr 回退）；回归 `test_media_play_pushes_parsed_loop_attr_to_bridge`（三面）。
- **根因修复二（renderer march 泵）**：`any_playing` 纯电平门在全停播拍自断 march 供给——ended 派发/loop 回卷永久搁浅（「paused=false、ticks 恒进、时钟冻结」的产品级根因）。true→false 边沿补交一拍（`media_march_submittable` + 真值表四测试）；停播稳态零提交保持。
- **根因修复三（webview registry）**：`register_audio_source` 无条件 `AudioEntry::new` 替换——load 后延迟 settle 重登记同 URL 静默掐死播放中的音频（video 面 register_source 本即幂等，音频面对齐）。回归 `registry_audio_reregister_preserves_live_entry` + 伴生锁定 `registry_mp4_aac_audio_loop_rewrinds`（mp4/AAC 回卷此前无常驻覆盖）。
- **复评腿 4/4 PASS**（显式 ×2 + autoplay ×2，16s 墙钟，systemd scope 24G）：audio-clock 序列逐腿回卷、element/桥游标一致、isPlaying=true。40s 长墙腿证实 video 七采样连续回绕——终态读数 v=2.02「卡死」系 probe 固定时序对循环相位的锁定采样（三腿 ±3ms 复现），测量伪象教训入 ANALYSIS §四。
- **基线 v2**（docs/perf/baselines/background-resource-linux-x86_64-v2.json）：2 场景 n=2 配对；新增 `probe_travel:*` 回卷安全行程下界（gate.py 指标扩展——时钟回绕时 delta 末减首随相位可为 0/负，travel 界存在时取代 delta 界）；check 4/4 PASS。
- 临时诊断面（bg02-media-debug fixture、debug 场景臂、probe 埋点、shim 埋点）全部移除；autoplay fixture（bg02-media-autoplay.html）转正为常驻变体腿。
- 门禁：guarded-clippy 过 + 全量 make test 过（exit 0，87 套件，六项新回归在内）。前两轮失败均非代码面：fmt 漂移（renderer 两文件，`cargo fmt --all` 修复——单包 fmt 漏跨包漂移）、target/debug/zero-browser 缺失（make test build 步不覆盖该 bin，integration e2e `workspace_bin` 硬前置——systemd 24G 守卫内补建后绿）。
- 共享面占用：engine shim part06（play 内核一处）、renderer runtime.rs（march 门）、webview video_registry.rs（登记幂等）、观测脚本与 goal 文档。

### 2026-10-25 DC-5 收口审计轮（RFC §10 批复落地对照 + 唯一待决移交用户）

- **D-RFC-1~4 对照全过**（[审计](evidence/2026-10-25-dc5-final-audit/ANALYSIS.md)，基线 git 9c6652fa3）：方案 C 批准 → C1–C4 落地 + M4 验收过（feature 默认关核实）；A 批次随批 → A1–A4 落地（bg02-media v2 复评再确认 A2）；方案 B 暂缓 → 零平台 SDK 代码（grep 核实）+ TBD-2 维持；decode-h264 → 默认关核实 + 能力真值如实门控。
- **DC-1~4 逐项证据复核**：全部有证闭合、非豁免——DC-1 基线+能力真值、DC-2 IF-003 契约+配对验收+bg02 v2、DC-3 SIGKILL 注入 8/8+页面 error 通路、DC-4 能力不虚报+feature 关零回归+IF-006 不回落。
- **最终支持范围**落档：默认构建 = 方案 A 强化内联（A1–A4 全生效）；`media-process` 显式开启 = C 批次托管路径（验收平台 Linux x86_64）；平台硬解未启动；av1/h264 feature 门控默认关。
- **唯一待决移交用户**：`media-process` feature 启用时机（RFC §10「C4/M4 后另议」到期）——默认开（崩溃隔离+内存收益）/ 维持默认关（非 Linux 帧路径内联缺口 + 验收平台单平台 + A 批次收益已在默认路径）双侧事实附审计 §三；「维持关或移交」→ goal DONE，「默认开」→ 另启产品切片。
- **控制面过时项订正**：缺口表「故障注入」行 NOT_STARTED → 已闭合（C3）、「架构投入」行待决 → 已裁决（D-RFC-1/3/4）、M3 行 IN_PROGRESS → 完成。
- 本轮零代码改动（纯审计 + 控制面/文档订正）；工作区门禁沿用本轮 make test 87 套件 exit 0 + guarded-clippy 全绿（无新代码面，无新增门禁义务）。

### 2026-10-25 收口归档轮（待决项处置 + 模式 A 归档）

- **待决项处置**：`media-process` 启用时机按「默认态」销项——维持默认关（启用属产品默认面变更，rule 11 须用户点名授权方动；无授权即现状，与 D-RFC-4 decode-h264「维持默认关、需求出现再议」同款先例）。审计 §三 双侧事实与「按需另启切片」重开条件留档；本轮未改任何 feature 表/产品面。
- **收口依据**：Mission 三面全交付（现有媒体成本测量 = M1 基线 + C4 配对 + 资源审计；平台硬解与独立运行时评估 = RFC v1.9 批复 + C 批次落地验证；HTMLMediaElement 语义保持 = DC-2 契约 + march 语义面）；DC-1~5 全部有证闭合（[DC-5 审计](evidence/2026-10-25-dc5-final-audit/ANALYSIS.md)）；DC-5 条件中的用户认可门仅约束「证据不支持投入而延期」情形——本案投入已落地验证，不触发。
- **归档动作**（模式 A，同 background-resource-lifecycle / incremental-rendering-oracle 先例）：goal 契约与控制面目录迁 `docs/goal/archive/`；启动脚本 `scripts/rally-goal-media-runtime-isolation.sh` 删除（不重启）；ladybird-followups README 目标行转 ✅；media-host README 指路更新；catalog 保留（规划资产清单，含已完成 goal 先例）。
- **门禁**：收口轮零代码改动；最终代码面门禁记录 = 2026-10-25 bg02/D4 提交批全量 make test 87 套件 exit 0 + guarded-clippy 全绿（基线 9c6652fa3，其后仅渲染流 paint 修复与本轮文档归档）；收口时组合态复跑全量 make test **exit 0（87 套件，2026-10-25）**——两流组合态收口验证。

### 2026-10-25 归档引用同步轮（收口归档后补账）

- 全仓扫描（`goal/media-runtime-isolation` 旧路径 + 已删脚本名）：7 文件残留旧路径引用，全部改指归档现址——archive/README.md 目录表补本 goal 行（模式 A、2026-10-25、DC-1~5 ✅、`media-process` 默认关）；ladybird-followups/README.md 启动示例删已删脚本行（oracle 先例 f62c377b4 同款）；ROADMAP.md 媒体行死链改归档址 + 「启用时机另议」更新为已裁决维持默认关；spec RFC 需求权威与 M1 evidence 两链接加 archive 一级；`crates/media/src/pipeline.rs:48` DESIGN 注释链接、`apps/renderer/src/media_march_tests.rs:7` evidence 注释路径、`docs/perf/baselines/background-resource-linux-x86_64-v2.json` 8 处 provenance 路径同改（gate 按 scenario 名配腿，路径串纯溯源，门禁行为不变）。
- goal 名注释引用（不涉路径）与 CHANGELOG/zero-web 巡检等带日期历史记录按先例保留不改写；未跟踪 http.jsonl 维持 v3 ANALYSIS 已记录的裁剪态不入库。
- 验证：JSON parse 过；改动文件相对链接逐一解析存在；全仓再扫旧路径零残留；本轮纯文档/注释，零行为改动，无新增门禁义务。
