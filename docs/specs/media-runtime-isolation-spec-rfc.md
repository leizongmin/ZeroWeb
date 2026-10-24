# RFC：平台媒体解码与媒体运行时隔离——方案/许可证/回滚

**版本**：v1.8-draft
**日期**：2026-10-24
**作者**：AI Assistant（Rally media-runtime-isolation M2）
**状态**：草稿——待用户评审决策（D-RFC-1~4；见 §10）
**需求权威**：[goal 契约](../goal/media-runtime-isolation.md) DC-1~DC-5；本文为设计交付，需求不在此重述

---

## 0. 执行摘要

- **一句话目标**：基于 M1 实测基线，比较「现状增强 / 平台硬解 / 独立媒体进程」三路线，给出许可证/SDK/分发/回滚核查与分批实施切点，供用户裁决媒体运行时架构。
- **本期范围**：RFC 文档本身；不改代码、不装 SDK、不换许可证。
- **明确排除**：HTMLMediaElement 语义重做（复用已归档 media-playback/media-audio/media-elements）；Android 宿主改造细节（仅列接口影响）。
- **核心约束**：现有软件路径全程保留为回滚点；新依赖/系统库需用户批复（GB-20260901 先例）；平台/许可外部事实未核查项全部降级为待验证。
- **推荐方案**：分批——A（现状增强，无架构投入）→ C（媒体独立进程，DC-3 的唯一达成路径，feature 门控默认关，Linux shm 先行）→ B（平台硬解按需，容量/功耗驱动）；详见 §5、§10。
- **首个落地步骤**：D-RFC-2 批复后启动批次 A1（时钟批内对齐，无架构含义）；D-RFC-1 批复后并行启动 C1（protocol 契约扩展 + `media-process` feature 骨架）。

## 1. 背景与目标

### 1.1 背景与依据

- 立项依据：2026-10-20 用户按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md) 立项，媒体进程是待证据支持的方案（goal §0）。
- M1 基线已实测（[2026-10-24 evidence](../goal/media-runtime-isolation/evidence/2026-10-24-m1-baseline/ANALYSIS.md)）：产能、复制链、权限表、平台核查、时钟三缺口全部量化。
- 前置 RFC 决策（[video-decode-playback-spec-rfc](video-decode-playback-spec-rfc.md)，2026-09-01 批复）：路线 C 纯 Rust VP9 先行；dav1d/H.264 feature 默认关、按需单独立项——本 RFC 不推翻这些已定决策。

### 1.2 目标

- 架构目标：为 DC-2（状态/时钟/seek 契约）、DC-3（解码故障隔离与恢复）、DC-4（能力真实与回退）选定可实施的架构路线。
- 决策目标：把许可证/SDK/分发的未核查项显式化为决策输入，用户拍板后 M3 才动代码。

### 1.3 范围边界

- **在范围内**：媒体解码与帧消费的进程/线程架构；平台解码来源比较；codec 许可与分发影响；回滚切点。
- **不在范围内**：音频输出设备策略（NullSink→真设备输出）的最终选型（列议题不决策）；网络媒体获取（source_provider 现状保留）。

## 2. 现状分析（M1 实测）

### 2.1 当前架构

解码在 renderer 进程内联：`apps/renderer/src/runtime.rs:4122` 主循环 16ms 节拍泵
`VideoPlayerRegistry::tick_all`（zero-webview）→ `VideoPlayer`（zero-media 纯 Rust 软解）
→ `ImageCache` → `painter` ImagePrimitive → CPU 光栅。无独立媒体进程/线程；IPC 面
（crates/protocol）今天没有任何媒体消息。

### 2.2 M1 量化事实（方案对比的输入）

| 事实 | 数值/状态 | 对方案的含义 |
|---|---|---|
| 软解产能（320×240@24fps） | 纯视频 ≈45× 实时 / A/V 双轨 ≈42× 实时，约 1 核 | 产能不是当前痛点；硬解无 CPU 收益证据（低分辨率档） |
| 泵节拍占用 | 帧 tick p95 ≈1.2–1.5ms ≪ 16ms | 主循环不被解码挤占；线程化收益存疑 |
| media clock 速率 | **1.45402**（音频 advance_to 整批推进，FINDING） | 时钟语义缺陷独立于架构，方案 A 可修 |
| A/V 漂移 | p50=11ms（≤1.5 帧间隔） | sync 机制健康；进程化须保住该真值契约（DC-2） |
| 帧复制链 | C1 解码转换 + C2 纯色扫描 + **C4 快照全缓存深拷贝** + 全文档重渲染 | 进程化叠加 IPC 拷贝前须先修 C4；Linux shm 可省一道 |
| seek | cue 路径精确（302ms 冷 / 1.1ms 热） | 进程化后 seek 契约必须保持精确语义 |
| 解码故障面 | 解码器 panic/UB（rusty_vp9/openh264/dav1d C 侧）直接带走 renderer | **DC-3 当前无法达成**——方案 C 的核心动机 |
| 平台能力 | 本平台无 /dev/dri：无任何硬解路径；生产 VP9-only；音频 NullSink | DC-4 的"无硬解→软件回退"即当前常态 |
| 资源边界 | MSE 256MB 封顶（整个 MSE 面在 `decode-h264` feature 门控下，默认关即无 MSE 路径）；源字节驻留无界；解码无 quota | 权限缺口（DC-4 证据面），方案 A 可补 |

### 2.3 语义缺口（M1 盘点，架构无关）

part06.js:10131（autoplay 绕桥，`ms.playing=true` 于 :10141）、part06.js:16407（死钩子）、
part03.js:23558/23610（play-only 桥武装）——页面可见时钟/状态契约（DC-2）的缺口，M3 切片
候选，任何方案下都须修（行号以 HEAD 56d499bb0 为准）。

### 2.4 相关代码

- 解码底座：`crates/media`（feature：`decode-av1`=dav1d 0.11、`decode-h264`=openh264 0.9 source、`audio-cpal`=cpal 0.16，全默认关）
- 播放/注册表/桥：`crates/webview/src/video_registry.rs`（`__zw_video_*` 回调族、MSE_FEED_MAX_BYTES=256MB）
- 帧消费：`crates/render-foundation/src/image_cache.rs`、`crates/engine/src/paint/painter/text.rs:820`
- IPC 基建（方案 C 复用面）：`crates/protocol/src/process.rs`（ProcessManager/RendererState 进程监理、`child_process_args` 角色参数约定）、`frame_shm.rs`（POSIX shm 帧传输，Linux 默认开、非 Linux 内联回落）、`channel/message/transport`
- **解码进程隔离同构先例（D1）**：`apps/image-decoder`（角色循环 `run_role`）+ `crates/webview/src/image_decoder.rs`（renderer 侧代理：懒惰 spawn、邻位二进制发现、spawn 失败永久错误面短路（SPAWN_FAILED，v1.7 订正——注释称「进程内解码」与代码不符，以代码为准）、运行期失败不绕过隔离边界）+ `IpcMessageKind::ImageDecodeRequest/Result`（消息形态先例）

---

## 3. 方案空间与对比

三条路线不互斥，按批次组合；对比基准 = 满足 goal DC-1~DC-5。

### 3.1 方案定义

| 方案 | 内容 | 实现来源 |
|---|---|---|
| **A 现状增强** | 保持进程内软解架构；修 M1 实测缺陷：时钟批内对齐（rate 1.454→≈1.0）、三缺口（§2.3）、C4 快照深拷贝、C2 纯色扫描跳过、源字节驻留封顶 | 仓内自实现：`video_registry.rs` advance_to/shim part03/part06、`image_cache.rs`、webview publish 路径 |
| **B 平台硬解** | 按平台接入系统解码：macOS VideoToolbox、Linux VAAPI、Windows Media Foundation/D3D11VA、Android MediaCodec；feature 门控，软解保底 | 新增系统框架绑定（拟新增 `zero-media` 平台 backend trait + 各平台 impl）；**API/ABI/许可均待核查（TBD-2）** |
| **C 独立媒体进程** | 解码搬入 media host 进程，renderer 经 zero-protocol 控制面 + 帧传输面交互；解码器崩溃不带走页面（DC-3） | 复用 `protocol` ProcessManager/frame_shm/channel 先例 + 拟新增媒体 IPC 消息族与 host 可执行/线程模型 |

### 3.2 对比表

| 维度 | A 现状增强 | B 平台硬解 | C 独立媒体进程 |
|---|---|---|---|
| DC-3（故障隔离）达成 | 🔴 不可达（进程内无隔离边界） | 🔴 不可达（硬解仍在 renderer 进程内） | 🟢 唯一可达路径 |
| DC-2（时钟/状态契约） | 🟢 直修 | 🟡 须桥接平台时钟 | 🟡 须跨进程时钟/状态代理（契约可测） |
| 产能/CPU（低分辨率实测档） | 🟢 修缺陷后更优 | 🟢 优（但无实测痛点） | 🟡 IPC+帧传输开销，Linux shm 可压平 |
| 实现复杂度 | 🟢 低（仓内既有面） | 🔴 高（4 平台 × 各自 API/构建/CI） | 🟡 中（进程模型 + 协议契约，基建有先例） |
| 跨平台成本 | 🟢 零新增 | 🔴 每平台独立核查/适配 | 🟡 Linux shm 先行，其余内联回落（有先例形态） |
| 许可/分发影响 | 🟢 无 | 🟡 系统框架待核查（§4） | 🟢 无新依赖 |
| 回滚难度 | 🟢 单提交级 | 🟢 feature 关 | 🟢 feature 关（默认关上线） |
| 触发新用户决策 | 是（D-RFC-2，轻量：随批/逐片） | 是（D-RFC-3） | 是（D-RFC-1） |

### 3.3 详细对比

| 方案 | 优点 | 缺点 |
|---|---|---|
| A | 直接消除 M1 实测缺陷；无架构风险；是 B/C 的共同前置（复制链与时钟语义不修，任何架构都继承） | 不改变故障面：解码器 panic/UB 仍带走 renderer，DC-3 永远 NOT_RUN |
| B | 真机硬解的功耗/高分辨率/codec 覆盖上限；H.264 现网兼容价值的潜在路径（也可经既有 openh264 软解部分达成，无需 B） | M1 证明低分辨率档产能富余 ≈42–45×，当前无 CPU 痛点；4 平台核查与适配成本最高；许可核查未启动（TBD-2）；本平台无 /dev/dri，连验证环境都不具备（macOS/Windows 机器待用户侧确认，TBD-4） |
| C | DC-3 达成的唯一路径；故障半径从页面级缩到元素级（可观察播放错误 + 重载恢复）；权限/quota 面收敛到 host 进程一处；Ladybird 同构先例 | 跨进程时钟/状态/seek 契约复杂度（DC-2 风险面）；帧传输在非 Linux 平台暂无 shm（内联拷贝成本，M1 复制链放大）；MSE 字节流需经 IPC 喂给 host；两套媒体路径并存期的维护成本 |

**推荐**：分批 A → C，B 缓办（详见 §5.1 理由）。A 不需要架构决策，但其切片归 M3，与 C 的批复解耦。

## 4. 许可证 / SDK / 分发核查

### 4.1 仓内已核事实（既往批复，本 RFC 不改变；「前 RFC」= [video-decode-playback-spec-rfc](video-decode-playback-spec-rfc.md)）

| 依赖 | 许可 | 状态 |
|---|---|---|
| rusty_vp9 0.1 / matroska-demuxer 0.8 / symphonia 0.6 / opus-decoder 0.1.1 | 纯 Rust crate（许可随 workspace 汇总） | 在用（VP9 面 2026-09-01 前 RFC D-RFC-1 路线 C 批复） |
| openh264 0.9（`decode-h264`，默认关） | 绑定 Apache-2.0 + C 源 BSD-2-Clause，构建期源码编译（Cisco AVC Patent Trust License 2.0 框架下的 openh264 分发） | 前 RFC D-RFC-3 已批复单独立项，feature 未启用 |
| dav1d 0.11（`decode-av1`，默认关） | 绑定安全 Rust；链接系统 libdav1d（安装需批复，GB-20260901 已批） | 前 RFC D-RFC-2 已批复，feature 未启用 |
| cpal 0.16（`audio-cpal`，默认关） | 双许可 crate（许可面仓内未核——待核查） | 在用（可选 feature，无架构门禁） |

### 4.2 平台系统框架（方案 B 前置核查，全部**待验证** → TBD-2）

以下均为外部事实，未经本仓验证，**不得作为实施依据**；若 D-RFC-3 批准，M3 首批切片先做核查并落 evidence：

| 平台 | 框架/API | 待核查点 |
|---|---|---|
| macOS | VideoToolbox（VTDecompressionSession） | 框架随系统分发（无再分发件）；绑定形态（objc2/sys crate 是否维护）；CI 无 macOS runner 的验证路径 |
| Linux | VAAPI（libva）+ Mesa 状态跟踪器 | 依赖驱动/Mesa 环境（本平台 /dev/dri 缺失即不可用）；libva LGPL 动态链接的合规形态 |
| Windows | Media Foundation / DXVA（D3D11VA） | MF 与 D3D11VA 的能力边界（H.264/VP9/AV1 覆盖）；绑定 crate 生态成熟度 |
| Android | MediaCodec（Java/JNI 或 NDK AMediaCodec） | 与 Kotlin 宿主 + Rust JNI 桥的分工；Surface 输出与现有 ImagePrimitive 帧管线的对接成本 |

### 4.3 分发影响结论

- 方案 A/C：零新增分发件、零许可变更。
- 方案 B：各平台不引入再分发件（系统框架动态链接）前提下，许可风险低，但**逐平台核查结论未产生**（TBD-2）；openh264 源码编译路线（方案无关，既有 feature）的分发面维持前 RFC D-RFC-3 批复形态不变。
- 任何系统库安装沿用 GB-20260901 先例：用户批复后进环境，不自动改变许可证或引入新 codec（goal 边界）。

---

## 5. 推荐方案与目标状态

### 5.1 推荐与理由

**推荐**：批次 A（D-RFC-2 随批门）→ 批次 C（D-RFC-1）→ 批次 B（D-RFC-3，暂缓）。

1. A 的每一项都是 M1 实测缺陷修复，任何架构选择下都需要；且是 C 的前置（复制链不修，IPC 帧传输放大成本；时钟语义不修，跨进程代理把 bug 复杂化）。
2. C 是 DC-3 唯一达成路径；goal 的收口（DONE）在用户不调整 DC 的前提下依赖 C。基建先例（ProcessManager、shm 帧传输）摊薄部分成本（诚实边界见 §5.1A）。
3. B 无实测痛点驱动（产能富余 ≈42–45×）且核查/适配成本最高、本平台无验证环境——暂缓不阻塞 DC-2/3/4；H.264 兼容缺口可独立经既有 openh264 feature（前 RFC D-RFC-3 批复面）部分缓解。

### 5.1A 方案 C 成本口径（诚实边界）

基建先例摊薄的比初稿估计更多：**image-decoder 独立进程（D1）是解码器漏洞隔离的同构先例**——
renderer 侧懒惰 spawn 子进程（`--type=image-decoder` 约定、二进制约等于可执行文件邻位发现、
stdio 管道 + PipeTransport bincode）、角色循环 `run_role` 直至对端断开、启动失败永久回落
进程内、运行期失败不绕过隔离边界（返回错误）——进程生命周期/降级纪律/消息封装形态全部有
模板。ProcessManager 另供 spawn/状态机/崩溃检测；frame_shm 供 Linux 帧零拷贝先例。

真正拟新增的成本面：**流式控制契约**（play/pause/seek 会话 vs image-decoder 的一次性
request/response）、**跨进程时钟/状态镜像**（DC-2 契约核心，C2 切片）、**视频帧率的持续
帧传输**（image-decoder 是低频单帧响应）。重启/退避监理拟新增（C3）。详见 §9 接口契约。

### 5.2 方案 C 目标架构（示意）

```
┌─ renderer 进程 ──────────────────────┐      ┌─ media host 进程（拟新增）─────┐
│ HTMLMediaElement 语义 / JS 桥        │      │ VideoPlayerRegistry（移入）    │
│   状态代理镜像 ←── MediaState 消息 ──────────→ decode/demux（zero-media）   │
│   时钟代理（audio cursor 契约）      │      │ 音频游标/事件源               │
│ ImageCache ←── frame shm / 内联 ───────────→ 帧发布（C1 转换后直接入 shm）  │
│ painter → 光栅                       │      │ 解码 quota/资源上限执行点      │
│ source_provider 字节流 ──MediaData────────→ MSE buffer 镜像（若启用 MSE 面）│
└──────────────────────────────────────┘      └────────────────────────────────┘
        控制面：zero-protocol IpcMessage 拟新增媒体消息族（play/pause/seek/…）
        监理面：复用 ProcessManager 先例（spawn/状态机/崩溃检测）；重启/退避拟新增（C3）
        帧面：Linux frame_shm 先例形态；非 Linux 先内联 rgba，量产后再议平台 shm
```

关键变更：

- **拟新增** `zero-protocol` 媒体消息族（控制/状态/帧/字节流四类）与 media host 进程模型；契约同步主进程与 renderer 两端（架构边界条款）。
- `zero-webview::video_registry` 的 registry 键面/JS 桥不变（页面契约面不动，DC-2 验收面不变）；registry 实现从直调 zero-media 切到 IPC 客户端（feature 门控，默认关时直调路径保留）。
- feature `media-process`（拟新增，默认关）：关闭 = 现状（方案 A 终态），开启 = 进程隔离路径。软件回退始终在位。

### 5.3 方案 A 终态（无新架构）

缺陷清零后的现状：时钟 rate≈1.0、三缺口修复（bg02-media 基线 v2 可回补）、C4/C2 拷贝消除、源驻留封顶。DC-3 保持 NOT_RUN——需要用户按 DC-5 认可延期/调整后才可收口。

## 6. 安全考虑

- **信任边界**：媒体字节是不可信输入（网络/页面源）。方案 C 把解码移出 renderer 后，host 进程成为病态字节的第一现场——解码 quota（内存/时长上限，M1 权限缺口）应在 host 侧执行；fuzz 复现入口沿用 `scripts/fuzz-repro-guard.sh`，不因进程化改变。
- **权限面收敛**：MSE 封顶（256MB）与源驻留上限在 host 执行后，页面侧只持契约镜像；`__zw_video_*` 桥面与 feature-detect 回落路径不变。
- **进程监理**：host 崩溃由 ProcessManager 模式监理（复用 renderer 先例），重启上限/退避策略为 M3 切片内容；不自动放大权限（host 不持网络面，字节仍由 renderer source_provider 提供）。
- **IPC 面**：媒体消息族纳入 `zero-protocol` 现有校验/序列化纪律；帧 shm 沿用 frame_shm 的 Linux 默认开 + 环境变量关断形态。

---

## 7. 实施计划与回滚切点（获批后执行，本 RFC 不实施）

| 批次 | 范围 | 门禁 | 验证 | 回滚切点 |
|---|---|---|---|---|
| A1 | 时钟批内对齐（advance_to 插值/目标对齐，rate→≈1.0） | D-RFC-2（随批） | 扩展 `media_m1_baseline.rs` T3 断言收 0.9–1.1；时钟契约测试 | 单提交回退 |
| A2 | 三缺口修复（autoplay 绕桥 / 死钩子接线 / load 时桥武装；设计稿见 goal evidence DESIGN.md §6——缺口 1+3 同面 = autoplay 走 play 算法，缺口 2 = 生产 renderer march 接线（runner 已接线，生产面缺失）） | D-RFC-2（随批） | 拟新增页面语义测试 + bg02-media 基线 v2 回补 | 单提交回退（Fix 1/Fix 2 各自独立） |
| A3 | C4 快照深拷贝消除（publish 路径引用化/写时复制）、C2 扫描跳过、源驻留封顶（设计稿见 goal evidence DESIGN.md §8——C4 = `CacheEntry.data` Arc 化（像素不可变既有不变量），C2 = 视频帧跳纯色扫描，封顶 = 256MB 总预算 + 非在播 LRU 驱逐 + 准入拒绝错误面） | D-RFC-2（随批；性能关键面） | `make bench-gate` 定向（media/render 域）+ M1 harness 复测 | 单提交回退（三子项互相独立） |
| A4 | seek 定位窄化：phase-② 全量前向（O(target)）→ cue 锚定 fallback（O(GOP)，`decode.rs` 面；归因证据见 goal evidence 2026-10-24-m3-prep-design DESIGN.md §3） | D-RFC-2（随批） | 扩展 `m1_seek_cue_path`：mid-GOP 目标耗时不随 target 线性增长；15/30/45 三点不回退 | 单提交回退 |
| C1 | `zero-protocol` 媒体消息族契约（§9 IF-001/002/005/006）+ host 进程骨架（feature `media-process` 默认关；设计稿见 goal evidence DESIGN.md §9——17 拟新增变体 wire format（§9.2 基础 14 + §10.2 补 setRate/setGain/setLoop；命名零冲突已核：ProcessRole 无 Media、IpcMessageKind 无 Media*）+ `ProcessRole::Media`/`--type=media` 接线 + media-host 骨架（C1 只立契约与空源生命周期，解码内核 C2 接）+ `zeroweb-media-` shm 前缀 / `ZW_MEDIA_SHM=0` 关断） | D-RFC-1 | 契约测试（两端同步核对）；默认关路径全量 `make test` 绿 | feature 关即现状 |
| C2 | registry 客户端切换 + 状态/时钟代理（§9 IF-003/004；DC-2 契约测试；设计稿见 goal evidence DESIGN.md §10——host 自走时钟（音频截止/帧演示截止等待，无 60Hz 轮询）+ push 校准闭合（跃迁即时/随帧/250ms 心跳）+ epoch 代际 + march 页面语义原样保留（getter backing 转 push 镜像，泵媒体分支转被动）） | D-RFC-1 | 拟新增跨进程时钟/状态/seek 契约测试；A/V 漂移真值复测 | feature 关 |
| C3 | 故障注入验收（DC-3）：host 终止 → 可观察错误 → 重载恢复（设计稿见 goal evidence DESIGN.md §11——崩溃检测双路（send 失败即时 + 泵周期 `try_wait` 兜底）、状态机 Idle/Healthy/Restarting（退避 1s/2s/4s，n≤3）/Failed、重启不自动重播重注册（epoch 重单调）、SIGKILL 注入 5 场景契约测试） | D-RFC-1 | 拟新增 DC-3 场景测试（真实进程终止） | feature 关 |
| C4 | M4 配对数据 + 资源审计（DC-1/DC-4 收口证据；设计稿见 goal evidence DESIGN.md §12——三态配对矩阵（feature 关 / shm 开 / `ZW_MEDIA_SHM=0`）、M1 harness 场景体复用、判据表（rate 带/seek 相等/漂移带/像素全等/IPC 税上界/代际零泄漏）、资源审计清单（进程树/RSS/shm 槽位/源驻留 256MB 复测/空闲 CPU）、capabilities 三对照） | D-RFC-1 | M1 harness 对照复测（开/关 feature 配对） | feature 关 |
| B1 | 平台框架逐平台核查（§4.2 表落 evidence） | D-RFC-3 | 核查报告 + 可用平台清单 | 纯文档 |
| B2+ | 首个平台 backend（依核查结果定） | D-RFC-3 + 用户点名平台 | capability 真值测试 + 软解配对 | feature 关 |

依赖：A1–A4 与 C1 并行可行（不同文件面；A4 在 media 层 `decode.rs`，A1 在 webview 层 registry）；C2 依赖 A1（时钟语义先修）；C3 依赖 C2；B 依赖 B1 核查。

### 7.1 代码变更边界

- **允许修改**：§7 批次涉及面——`crates/media`、`crates/webview`（registry/publish/测试）、`crates/protocol`（媒体消息族，契约两端同步）、`crates/render-foundation`（C4/C2）、`crates/engine` shim 媒体三缺口面、`tests/fixtures/media`、各 goal 文档；每步变化可追溯到批次。
- **禁止修改**：架构边界条款面（engine/webview 职责不混写、`zero-webview` 稳定嵌入边界语义）；HTMLMediaElement 页面语义重做（复用已归档 goal）；feature 默认值变更（`decode-h264` 默认态→本 RFC D-RFC-4；`media-process` 默认态→本 RFC D-RFC-1；`decode-av1` 维持前 RFC D-RFC-2 已批形态、启用时机另行评估；`audio-cpal` 无架构门禁，默认关维持）；大规模重构（goal「默认不大规模重构」边界）。
- **既有约定**：页面契约面（`__zw_video_*` 桥、registry 键面）在方案 C 切换期不变（DC-2 验收面）。

## 8. 测试策略

| 需求/风险 | 层级/位置 | 用例与断言 | 依赖/数据 | 覆盖与缺口 |
|---|---|---|---|---|
| DC-1（已测） | 集成 `crates/webview/tests/media_m1_baseline.rs`（在用） | 容量/时钟/seek/漂移 M1BASELINE 量化行 | 长 fixture（在仓） | 已覆盖；A3/C4 后复测配对 |
| DC-2 契约（拟新增） | 集成（C2 批次） | play/pause/seek/换轨后镜像状态与时钟真值一致；不返回旧代际数据 | 长 fixture + 进程模型 | 方案 C 关键风险面，须先于 C3 |
| DC-3（拟新增） | 集成（C3 批次） | 终止 media host → 播放错误事件可见 → renderer 可用 → 重载恢复 | 长 fixture + 进程终止注入 | 现架构无法测试（方案 C 动机） |
| DC-4（部分已测） | 平台核查（已落 evidence）+ 拟新增 quota 测试 | 能力查询真值；无硬解→软解回退；超限拒绝 | 环境事实 | 跨平台核查缺口 = TBD-2/TBD-4 |
| 时钟缺陷回归 | 扩展 T3 断言（A1） | rate 收 0.9–1.1 带 | AV 长 fixture | 现 sanity 带 0.5–3.0 防回退后收紧 |
| seek 冷路径回归（A4，v1.2 增补） | 扩展 `m1_seek_cue_path` | mid-GOP 目标（15s/45s）seek 耗时 ≤ GOP 前向上界且不随 target 线性增长；keyframe 对齐点不回退 | 长 fixture（GOP=2s 受控形态） | WPT 上游流 GOP 抽样 = 实施期待验证项 |
| 帧传输双路径（C2/C4） | 集成（拟新增） | `media-process` 开/关配对：同一 fixture 逐帧像素一致（shm 开与 `ZW_`关断、内联回落各一档） | 长 fixture + 环境变量翻转 | Linux shm/内联两路径均显式覆盖；非 Linux 平台实测缺口 = TBD-4 |
| 帧拷贝回归 | `make bench-gate` 定向（A3） | media/render 域微基准不回退 | 基线在库 | 全量 capture 合入前补跑 |

命令纪律：测试一律经 `make test` / test-guard；性能测量走既有 M1 harness 显式入口（release + guard）。

---

## 9. 媒体 host 协议接口契约（IF 级，M2 交付第二项）

本节是 D-RFC-1 批准后 C1/C2 批次要实现的接口设计主定义区（拟新增名称以「拟新增」标注，
现有名称为真名）。设计沿 §5.2 目标架构；不复述需求（goal DC 权威）。

### 9.0 先例映射（复用 vs 拟新增）

| 契约面 | 复用先例（真名） | 拟新增 |
|---|---|---|
| 进程 spawn/角色参数/二进制约等于发现/stdio 管道 | `image_decoder.rs` spawn、`child_process_args(ProcessRole, id)`（process.rs:143）、Windows `CREATE_NO_WINDOW` | `ProcessRole::Media`（拟新增变体 + `--type=media`）；`media-host` 二进制（拟新增 workspace member，Makefile 随 `zero-image-decoder` 行同列） |
| 消息封装/传输 | `IpcMessage{id, kind}` 信封、`PipeTransport`（4B LE 长度前缀 + bincode，64MiB 帧上限）、共享 `IpcMessageKind` 枚举 | `IpcMessageKind` 媒体消息族变体（拟新增） |
| 角色主循环 | `apps/image-decoder` `run_role<C: IpcChannel>`（对端断开退出） | `apps/media-host`（拟新增）同形角色循环 |
| 帧零拷贝 | `frame_shm.rs`（`/dev/shm/zeroweb-cmp-*`、env 关断、非 Linux 内联） | media 专属 shm 段（拟新增，前缀 `zeroweb-media-*`，`ZW_MEDIA_SHM=0` 关断） |
| 崩溃检测 | `RendererHandle::exit_reason`/`check_crashes` 状态机 | media host 句柄（拟新增，同形）+ 重启/退避（C3） |
| 降级纪律 | image_decoder.rs 代码为准（v1.7 订正）：隔离开启后 spawn 失败（SPAWN_FAILED 短路，image_decoder.rs:205）与运行期失败同归错误面，无任何层内联回落；源码注释「spawn 失败→进程内解码」与代码不符，注释订正归 C1 顺带 | 同纪律直接沿用（见 9.6） |

### IF-001 会话与生命周期

- **类型**：系统集成（renderer ↔ media host 子进程）。
- **拓扑**：每个 renderer 至多一个 media host（懒惰 spawn——首个媒体源 play 时；先例
  image_decoder.rs:101 懒惰 spawn）；host 生命周期 = renderer 生命周期；源状态以
  registry 键（去 query/fragment）寻址，导航 release 即 `MediaReleaseSource`。
- **握手**：连接后 host 首发拟新增 `MediaReady{protocol_version, capabilities}`
  （capabilities 列 codec 面，真值随 zero-media feature 门控——与页面 canPlayType 同源，
  不虚报）；renderer 版本/能力不符按 spawn 失败处理（9.6 降级）。
- **关闭**：renderer 退出前发拟新增 `MediaShutdown` 并等子进程退出（对齐
  `RendererHandle::shutdown` 语义）；host 收到对端断开（stdin EOF）自行退出（`run_role`
  先例）。
- **错误处理**：spawn 失败（二进制缺失/握手超时）→ 永久错误面 `MediaError{HostInternal}`
  （本 webview/renderer 生命周期不重试、不内联——SPAWN_FAILED 短路先例 image_decoder.rs:205，
  v1.7 订正：先例实际为错误面而非「进程内直调」）；运行期崩溃 → 9.6 错误面，不静默回退。
- **默认动作**：`media-process` feature 默认关 = 不 spawn，registry 直调 zero-media（现状）。

### IF-002 控制面（renderer → host）

| 消息（拟新增） | 载荷 | 语义 | 响应 |
|---|---|---|---|
| `MediaRegisterSource` | source 键、总长、mime | 登记源；**超 per-source 字节上限 → `MediaError{QuotaExceeded}`**（上限值 C1 定，对齐 MSE 封顶精神） | `MediaSourceReady` |
| `MediaDataChunk` | source、seq、字节 | 源字节流式喂入（64MiB 帧上限内分片） | 累积回执（bounded） |
| `MediaPlay` | source、position_ms | 起播/重播（Ended 后 = 重头，registry 现语义） | `MediaStateUpdate` |
| `MediaPause` | source | 暂停（时钟冻结） | `MediaStateUpdate` |
| `MediaSeek` | source、target_ms、request_id | 精确 seek（cue 路径语义跨进程保持，M1 §3.4 契约） | `MediaSeekResult{resolved_ms}` |
| `MediaReleaseSource` | source | 释放源与播放器（导航/`release` 面） | — |
| `MediaShutdown` | — | 优雅退出 | — |

请求/响应以信封 `id` 匹配（现有 `IpcMessage.id` 先例）；fire-and-forget 仅限无回执行
（`MediaReleaseSource`）。

### IF-003 状态/时钟面（host → renderer，DC-2 契约核心）

- 拟新增 `MediaStateUpdate{source, epoch, state}`：state ∈ `Playing{media_time_ms}` /
  `Paused{media_time_ms}` / `Ended` / `Error{code}`。
- **权威与时钟**：host 持媒体时钟真值（audio cursor 主时钟，M1 §3.3 漂移契约），自走墙钟
  推进（play 后自节拍，不再依赖 renderer 每 16ms 轮询——省 60Hz IPC 往返）；pause 冻结、
  seek 重锚。
- **推送时机**：状态跃迁即时推；`media_time_ms` 变化 ≥100ms 推一次（对应页面 timeupdate
  节拍上限，具体值 C2 按现 shim 事件面校准——拟新增实现细节，不在此定死）。
- **世代规则（DC-2「不返回旧代际数据」）**：每个源持单调 `epoch`；seek/replay/换源均
  递增；renderer 侧收到旧 epoch 的状态或帧一律丢弃。镜像状态（页面可见 playing/
  currentTime/duration/ended）只由最新 epoch 消息建立。
- **错误处理**：renderer 对 `MediaStateUpdate` 的类型/键不识别（版本漂移）→ 按 9.6 运行期
  错误处理，不猜测。

### IF-004 帧面（host → renderer）

- 拟新增 `MediaFrame{source, epoch, pts_ms, key}`：像素经 shm 段（`zeroweb-media-*`，
  Linux 默认开、`ZW_MEDIA_SHM=0` 关断）或内联 `Vec<rgba>`（非 Linux/关断回落；单帧超
  64MiB 帧上限 → `MediaError{FrameTooLarge}`——4K RGBA ≈33MB 在界内）。
- **背压**：latest-wins——renderer 按 (source, epoch) 只保留最新帧注入 `ImageCache`
  （现 `try_republish_cached` 路径不动）；host 侧检测未取走旧段即复用/关闭（ shm 槽位
  有界，C1 定深度，拟新增）。**不阻塞**：视频帧按节拍丢弃，不倒灌泵。
- **渲染节拍**：帧到达 → renderer 现有发布路径（ImageCache → repaint）触发；泵媒体分支
  从主动轮询转被动消费（runtime.rs 媒体 tick 面 C2 简化，行为面 DC-2 测试守）。

### IF-005 字节流与 MSE 扩展点

- 文件/URL 源：字节仍由 renderer `source_provider` 获取（host **不持网络面**，§6 安全
  边界），经 IF-002 `MediaRegisterSource`/`MediaDataChunk` 喂入。
- MSE 面：整个 MSE 现状在 `decode-h264` feature 门控（video_registry.rs:476）；其 IPC 化
  仅在该 feature 启用后作为扩展点（`MediaDataChunk` 复用），C1 不实现——**依赖本 RFC
  D-RFC-4**，未批不进范围。

### IF-006 错误与故障面（DC-3 契约）

- 拟新增 `MediaError{source, epoch, code, message}`；code 枚举（拟新增）：
  `DecodeFailure` / `SourceCorrupt` / `QuotaExceeded` / `FrameTooLarge` / `HostInternal`。
- **崩溃检测**：renderer 侧 host 句柄按 `exit_reason` 先例监测；检测到 → 对全部活跃源发
  `MediaStateUpdate{Error{HostInternal}}`（页面可观察播放错误，DC-3 第一段）→ 页面重载
  走现有导航恢复面（DC-3 第二段）；**renderer 自身不随宿主崩溃**（隔离目的）。
- **重启**：C3 批次实现（退避/上限策略拟新增）；C2 阶段崩溃后不再自动重播，仅报错。
- **降级纪律**（v1.7 合一并订正——image-decoder 代码为准，启动失败与运行期失败本就同面）：
  隔离开启后**一切失败（spawn/握手/运行期崩溃/协议错误）同归 `MediaError` 错误面**，
  **不得静默回进程内直调**（隔离边界不可绕过——DC-3 验收前提；spawn 失败短路为本 webview
  生命周期一次性，后续 play 立即 `MediaError{HostInternal}`）；feature 默认关 = 内联直调，
  正常路径不受影响。「spawn 失败 → 进程内回退」不可能发生（image_decoder.rs:205 先例即错误面）。

### 9.7 与 DC 的映射

| DC | 依赖接口 | 验证入口（§8 已列，此处挂接点） |
|---|---|---|
| DC-2 | IF-003 epoch/镜像规则 + IF-002 seek 回执 | C2 契约测试（旧代际丢弃断言） |
| DC-3 | IF-006 崩溃检测/错误面/降级纪律 2 | C3 进程终止注入测试 |
| DC-4 | IF-001 capabilities 真值 + IF-002 QuotaExceeded | capability 测试 + quota 测试 |

> 本节为设计契约（IF 主定义区）；实现批次、门禁与回滚见 §7，架构决策见 §10。批复前不实现。

## 10. 决策点（用户批复表）

> **ID 消歧**：本表 D-RFC-1~4 是**本 RFC** 的决策点，与 2026-09-01 已批复的
> [video-decode-playback-spec-rfc](video-decode-playback-spec-rfc.md) 中同名 D-RFC-1~3 **无关**；
> 本文引用后者时一律标「前 RFC」（形态沿其先例）。

| ID | 决策点 | 选项 | 建议批复 |
|---|---|---|---|
| D-RFC-1 | 是否批准方案 C（独立媒体进程）立项，解锁批次 C1 起 | 批准 / 改选 A-only（DC-3 须按 DC-5 认可延期或调整）/ 暂缓 | 批准（理由 §5.1） |
| D-RFC-2 | 方案 A 批次（A1–A3）是否随本 RFC 批复直接进入 M3 切片 | 随批 / 逐片另批 | 随批（均为实测缺陷修复，无架构含义） |
| D-RFC-3 | 方案 B（平台硬解）是否启动逐平台核查（B1 纯文档） | 启动核查 / 暂缓 | 暂缓（无实测痛点，核查成本高） |
| D-RFC-4 | H.264 兼容缺口是否启用既有 `decode-h264`（openh264）feature | 启用（生产 feature 表变更）/ 维持默认关 | 暂缓（与架构解耦；现网需求出现时再议） |

> 批复前：不改代码、不装 SDK、不变更 feature 表。D-RFC-1 未批时，goal 收口按 DC-5 走「用户认可延期/调整 DC」路径。

## 11. Spec Lint 报告

**汇总**：26 Pass / 1 Warning / 0 Fail / 3 Skip（TBD-2/TBD-4 外部事实专项，skip≠pass）/ 4 PendingReview（D-RFC-1~4）/ 5 N/A（32 条规则表 + 决策/外部事实专项；门禁判定见末行；v1.1 增补 §9 的专项检查见下方「v1.1 增补检查」）。

- **结论**：决策草案就绪，可交付用户评审（PendingReview: D-RFC-1~4）；**实施未就绪**——D-RFC 批复前不存在可实施合约，不动代码。
- **结构检查（关键项）**：执行摘要存在（§0，Pass）；场景存在性 N/A（仅设计交付，需求权威 = goal DC-1~5 表，验收映射在 §8 逐行给验证方法，测试绑定 Pass——§7 每批次带验证列）；实施交接完备（§7 表 + §7.1 边界，首步可执行——§0 首个落地步骤指向 A1/C1 两门禁分支，Pass）。
- **语义评审（关键项）**：原始意图与授权一致（Pass——推荐不采纳，§10 决策表留用户，加粗声明「批复前不改代码/不装 SDK/不变 feature 表」）；外部事实保守化（Pass——§4.2 平台框架全表标待验证 → TBD-2，未写入任何 FR/已定决策）；实现来源闭合（Pass——§3.1 来源列：A 仓内自实现、C 复用 protocol 先例 + 拟新增、B 拟新增平台 backend）；章节引用（Pass——§10→§5.1、§4.2→TBD-2、§5.2→C3、§9→§7/§10、§5.1A→§9、§11→§0/§3.1/§4.2/§7/§8/§10 均实指）；清单/依赖计数一致（Pass——新增依赖仅 §3.1/§5.2 所列拟新增项，无未列依赖）。
- **独立评审记录（同日）**：独立视角复核（完整文档 + 原始需求 + 代码锚点逐项核实）返回 0 blocker / 2 major / 5 minor，全部已修复——major①D-RFC ID 与前 RFC 撞名（§10 消歧注 + 全文标「前 RFC」）；major②§2.2 产能数字与入库 raw log 不一致（已对齐 42–45× 口径，evidence 侧同步订正）；minor 含 §7 门禁列、MSE feature 门控限定、ProcessManager 无重启先例的诚实边界（§5.1A）、§4.1 许可待核查标注。
- **v1.1 增补检查（§9 接口契约）**：结构（IF-001~006 各带错误处理/默认动作，先例映射表 9.0 逐行区分真名/拟新增——实现来源闭合 Pass）；语义（§5.1A「无先例可复用」旧表述经 image-decoder 源码核实后修正为部分先例——外部事实保守化 Pass，进程模型事实全部锚定 `apps/image-decoder`、`image_decoder.rs`、`process.rs:143`、`frame_shm.rs` 真名）；引用（§9→§7/§10/§8 实指）；新拟新增名称（`ProcessRole::Media`、`MediaReady`/`MediaStateUpdate`/`MediaFrame`/`MediaError` 族、`zeroweb-media-*` shm 前缀）全部标注拟新增且不进入任何已定决策——仍受 D-RFC-1 门禁。
- **Warning ×1**：M1 seek 冷路径（15s/45s 慢点）未归因——不阻塞架构决策，留 A 批次观察。**v1.2 已归因**（fixture 块级探针 + matroska-demuxer 源码 + `decode.rs` 判定三方交叉：mid-GOP 目标触发 phase-② 全量前向解码，耗时 ∝ target，证据见 goal evidence DESIGN.md §3），拟新增 A4 批次收口，实施期窄化后复核。
- **Skip ×3**：平台 API 可用性、许可核查结论、真机环境（均属 TBD-2/TBD-4 外部事实，未验证 ≠ 通过；不阻塞 A/C）。
- **PendingReview ×4**：D-RFC-1~4（决策点，呈现用户）。
- **N/A ×5**：场景存在性、UI 对齐、CLI 语义、默认动作闭合、优先级完备（无 UI/CLI 接口需求；无 FR/NFR 新定义——设计交付引用 goal 权威）。
- **未决与阻塞**：TBD-1（PendingReview）阻塞实施、不阻塞草案评审；TBD-2/TBD-4 仅阻塞方案 B 与跨平台收口。
- **已接受风险**：方案 C 非 Linux 帧传输先内联（性能缺口已知，双路径配对测试见 §8）；seek 冷路径方差已归因（v1.2，收口待 A4）；决策前 main 保持纯方案 A 能力。
- **v1.2 增补检查（A4 批次 + 归因）**：结构（§7 A4 行带门禁/验证/回滚列，与 A1–A3 同构——实施交接完备维持 Pass）；语义（归因证据三方交叉为一手事实——demuxer 源码为 cargo registry 本地只读、fixture 探针为自建产物分析、`decode.rs` 判定为仓内源码；「cue 访问 API 不存在」经 matroska-demuxer 0.8.1 源码核实（`CuePoint` 私有），A4 的 Cues 自解析实现来源闭合（仓内自实现，fixture 生成器同构先例）——外部事实保守化 Pass）；引用（A4 行 → goal evidence DESIGN.md §3 实指；A4 仍受 D-RFC-2 门禁，未进入任何已定决策）。汇总计数不变（增补项全部 Pass，无新增 Warning/Fail）。

## 12. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | D-RFC-1~4 用户批复 | PendingReview（阻塞实施、不阻塞评审） | 用户决策 | 用户评审本 RFC；批复后按 §7 批次推进 |
| TBD-2 | 平台解码框架核查（4 平台 API/绑定/许可） | 重要（仅方案 B） | 外部事实未验证 | 若 D-RFC-3 批复，M3 首批核查落 evidence |
| TBD-3 | 音频真设备输出选型（NullSink→cpal） | 可选 | 用户对音频体验的要求未表 | 列 M3 议题，不阻塞 |
| TBD-4 | macOS/Windows/Android 真机验证环境 | 重要（仅方案 B/跨平台 C 收口） | 机器可用性与 CI runner 现状 | 与用户确认可用环境清单 |

## 13. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v1.0-draft | 2026-10-24 | 初稿：M1 基线（27d45b8c8）输入下的三路线对比、许可/分发核查、分批实施与回滚切点、D-RFC-1~4 决策点。同日独立评审修复（0 blocker/2 major/5 minor，见 §11 评审记录）；evidence 数字同步订正对齐 raw log。待用户评审 |
| v1.1-draft | 2026-10-24 | 补齐 M2 交付第二项「protocol 与媒体 host 接口」：新增 §9 IF 级契约（会话/控制/状态时钟/帧/字节流/错误六面 + 先例映射），image-decoder 独立进程先例核实后修正 §5.1A 成本口径；§10 决策点、门禁与 D-RFC 语义不变。零代码改动 |
| v1.2-draft | 2026-10-24 | M3 前置设计轮：§7 新增 A4（seek 定位窄化，O(target)→O(GOP)）+ §8 seek 回归行；M1 seek 冷路径 Warning 经三方证据归因解除（demuxer 源码 + fixture 块级探针 + decode.rs 判定，详见 goal evidence 2026-10-24-m3-prep-design/DESIGN.md）；A1 时钟批内对齐设计稿落 DESIGN.md §2。零代码改动，D-RFC 门禁不变 |
| v1.3-draft | 2026-10-24 | A2 行补设计稿指引（DESIGN.md §6：缺口 1+3 同面 = autoplay 走 play 算法；缺口 2 = 生产 renderer march 接线——runner 已接线、生产面缺失，M1「零调用方」表述按 crates/ 域订正）；WPT 上游 GOP 抽样闭合 A4 待验证项（DESIGN.md §7）。无决策/门禁/批次范围变化，零代码改动 |
| v1.4-draft | 2026-10-24 | A3 行补设计稿指引（DESIGN.md §8：C4 = CacheEntry Arc 化——「像素插入后不可变」既有不变量使 Arc 共享语义安全，快照本体从不出进程；C2 = from_rgba_variable 跳纯色扫描；源驻留 = 256MB 总预算 + 非在播 LRU + 准入拒绝，av_sources seek 依赖受保护）。无决策/门禁/批次范围变化，零代码改动 |
| v1.5-draft | 2026-10-24 | C1 行补设计稿指引（DESIGN.md §9：媒体消息族 14 拟新增变体 wire format 草案——控制面 7 + 状态/帧/错误面 7 + MediaPlaybackState/MediaErrorCode 伴生枚举；命名冲突排查（ProcessRole 无 Media 变体、IpcMessageKind 无 Media* 变体）；media-host 骨架边界（C1 = 契约 + 握手 + 空源生命周期，解码内核 C2 接）；契约测试计划含 kill-switch 与 feature 关回归）。无决策/门禁/批次范围变化，零代码改动 |
| v1.6-draft | 2026-10-24 | C2 行补设计稿指引（DESIGN.md §10：host 自走时钟 + push 供数 renderer 镜像——march 页面语义原样保留、getter backing 转 push 镜像、250ms 心跳与 shim timeupdate 节流同值校准闭合 IF-003「≥100ms」实施值）；**C1 wire format 缺口修订**：桥面 setRate/setGain/setLoop（video_registry.rs:1711-1713 同步面）在 C1 草案无对应消息，§10.2 拟新增 3 变体（全族 14→17）+ MediaSourceReady 载荷扩 mediaMeta 三元组。无决策/门禁/批次范围变化，零代码改动 |
| v1.7-draft | 2026-10-24 | C3 行补设计稿指引（DESIGN.md §11：崩溃检测双路 + 状态机/退避 + SIGKILL 注入 5 场景）；**降级纪律先例订正**：image-decoder spawn 失败实为永久错误面（SPAWN_FAILED 短路 image_decoder.rs:205，三调用方均落错误面、无内联回落），源码注释「spawn 失败→进程内解码」与代码不符——§2.2/§9.0/IF-001/IF-006 全部以代码为准改写，IF-006 纪律两条合一（spawn/运行期同归错误面），失实注释订正归 C1 顺带。无决策/门禁/批次范围变化，零代码改动 |
| v1.8-draft | 2026-10-24 | C4 行补设计稿指引（DESIGN.md §12：M4 三态配对矩阵 + M1 harness 复用 + 判据表 + 资源审计清单 + capabilities 三对照）——**A1–A4 + C1–C4 全部批次设计稿齐备**。无决策/门禁/批次范围变化，零代码改动 |
