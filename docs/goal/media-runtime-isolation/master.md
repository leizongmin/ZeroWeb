# 媒体运行时隔离控制面

**日期**：2026-10-24
**状态**：Active（**D-RFC-1~4 已批复**——方案 C 批准 / A 批次随批复即启 / B 核查暂缓 / decode-h264 维持默认关，RFC v1.9 §10 批复记录；M1 基线 + M2 RFC 两项交付 + M3 前置设计八片齐备，[m3-prep-design DESIGN](evidence/2026-10-24-m3-prep-design/DESIGN.md)；**A1/A2 已落地**——rate 1.00000 + 三缺口闭合，偏差记录 DESIGN §2.4/§6.5）。
**当前卡点**：无。A3/A4 待启动。
**下一步**：A3（C4 快照 Arc 化 + C2 扫描跳过 + 源驻留封顶，DESIGN §8）/ A4（cue 锚定窄化 seek，DESIGN §3.3）；C1（protocol 媒体消息族 17 变体 + `media-process` feature 骨架，feature 默认关）并行启动。
**入口**：[目标契约](../media-runtime-isolation.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| CPU/帧复制/A-V 同步与权限 | 固定长视频、版本与真实帧消费链 | **已测**（60s 基线 + 复制链 C1–C5 + 权限表，evidence 2026-10-24） |
| 同步状态代理/时钟/seek 的进程契约 | 比较现状、平台解码、独立运行时 | **RFC v1.9 已批复**（三路线对比 + §9 IF 级接口契约；方案 C 立项） |
| 故障、权限不足与软件回退 | 真实隔离实例注入并验证浏览器恢复 | NOT_STARTED |
| 平台 SDK/codec 许可/分发与架构投入 | M1/M2 证据与 RFC 后决定，不自动换许可证 | 待决（M1 核查：本平台 /dev/dri 缺失，无硬解路径） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 生产资源、能力与权限基线 | **完成**（测量面成立，收口待评审） | [2026-10-24 基线](evidence/2026-10-24-m1-baseline/ANALYSIS.md) |
| M2 | 方案/许可证/回滚 RFC + protocol 接口契约 | **v1.9 已批复**（D-RFC-1~4 用户裁决） | [media-runtime-isolation-spec-rfc](../../../specs/media-runtime-isolation-spec-rfc.md) |
| M3 | 已确认平台与隔离切片 | **IN_PROGRESS**（A1/A2 已落地——rate≡1.0 + 三缺口闭合；A3/A4 待实施；C1–C4 设计稿齐备，D-RFC-1/2 已批） | [DESIGN.md](evidence/2026-10-24-m3-prep-design/DESIGN.md) |
| M4 | 配对媒体、故障和资源审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | MEDIA-01-cpu-copy-and-capability | **已测成立**（容量/复制链/权限/能力对照；收口待用户评审） | evidence/2026-10-24-m1-baseline |
| DC-2 | MEDIA-02-clock-seek-and-state | 部分（media 层时钟 rate FINDING 已修：A1 落地 rate=1.00000；cue seek 精确、漂移 p50=11ms；页面可见时钟缺口已修（A2 三缺口闭合，bg02-media v2 复评归后续验证切片）） | evidence/2026-10-24-m1-baseline（含 m1baseline-a1-post.log） |
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
