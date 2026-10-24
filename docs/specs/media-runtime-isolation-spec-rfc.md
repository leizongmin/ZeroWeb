# RFC：平台媒体解码与媒体运行时隔离——方案/许可证/回滚

**版本**：v1.0-draft
**日期**：2026-10-24
**作者**：AI Assistant（Rally media-runtime-isolation M2）
**状态**：草稿——待用户评审决策（D-RFC-1~4；见 §9）
**需求权威**：[goal 契约](../goal/media-runtime-isolation.md) DC-1~DC-5；本文为设计交付，需求不在此重述

---

## 0. 执行摘要

- **一句话目标**：基于 M1 实测基线，比较「现状增强 / 平台硬解 / 独立媒体进程」三路线，给出许可证/SDK/分发/回滚核查与分批实施切点，供用户裁决媒体运行时架构。
- **本期范围**：RFC 文档本身；不改代码、不装 SDK、不换许可证。
- **明确排除**：HTMLMediaElement 语义重做（复用已归档 media-playback/media-audio/media-elements）；Android 宿主改造细节（仅列接口影响）。
- **核心约束**：现有软件路径全程保留为回滚点；新依赖/系统库需用户批复（GB-20260901 先例）；平台/许可外部事实未核查项全部降级为待验证。
- **推荐方案**：分批——A（现状增强，无架构投入）→ C（媒体独立进程，DC-3 的唯一达成路径，feature 门控默认关，Linux shm 先行）→ B（平台硬解按需，容量/功耗驱动）；详见 §5、§9。
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

part06.js:10084（autoplay 绕桥）、part06.js:16350（死钩子）、part03.js:23558/23610
（play-only 桥武装）——页面可见时钟/状态契约（DC-2）的缺口，M3 切片候选，任何方案下都须修。

### 2.4 相关代码

- 解码底座：`crates/media`（feature：`decode-av1`=dav1d 0.11、`decode-h264`=openh264 0.9 source、`audio-cpal`=cpal 0.16，全默认关）
- 播放/注册表/桥：`crates/webview/src/video_registry.rs`（`__zw_video_*` 回调族、MSE_FEED_MAX_BYTES=256MB）
- 帧消费：`crates/render-foundation/src/image_cache.rs`、`crates/engine/src/paint/painter/text.rs:820`
- IPC 基建（方案 C 复用面）：`crates/protocol/src/process.rs`（ProcessManager/RendererState 进程监理）、`frame_shm.rs`（POSIX shm 帧传输，Linux 默认开、非 Linux 内联回落）、`channel/message/transport`

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

基建先例摊薄的是**部分**成本：ProcessManager 提供 spawn/状态机/崩溃检测先例，**重启/退避监理是拟新增**（C3 切片）；shm 帧传输是 Linux 先例，非 Linux 先内联。C 的主要新增成本在媒体消息族契约与跨进程时钟/状态代理（C2），无先例可复用。

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
| A2 | 三缺口修复（autoplay 绕桥 / 死钩子接线 / load 时桥武装） | D-RFC-2（随批） | 拟新增页面语义测试 + bg02-media 基线 v2 回补 | 单提交回退 |
| A3 | C4 快照深拷贝消除（publish 路径引用化/写时复制）、C2 扫描跳过、源驻留封顶 | D-RFC-2（随批；性能关键面） | `make bench-gate` 定向（media/render 域）+ M1 harness 复测 | 单提交回退 |
| C1 | `zero-protocol` 媒体消息族契约 + host 进程骨架（feature `media-process` 默认关） | D-RFC-1 | 契约测试（两端同步核对）；默认关路径全量 `make test` 绿 | feature 关即现状 |
| C2 | registry 客户端切换 + 状态/时钟代理（DC-2 契约测试） | D-RFC-1 | 拟新增跨进程时钟/状态/seek 契约测试；A/V 漂移真值复测 | feature 关 |
| C3 | 故障注入验收（DC-3）：host 终止 → 可观察错误 → 重载恢复 | D-RFC-1 | 拟新增 DC-3 场景测试（真实进程终止） | feature 关 |
| C4 | M4 配对数据 + 资源审计（DC-1/DC-4 收口证据） | D-RFC-1 | M1 harness 对照复测（开/关 feature 配对） | feature 关 |
| B1 | 平台框架逐平台核查（§4.2 表落 evidence） | D-RFC-3 | 核查报告 + 可用平台清单 | 纯文档 |
| B2+ | 首个平台 backend（依核查结果定） | D-RFC-3 + 用户点名平台 | capability 真值测试 + 软解配对 | feature 关 |

依赖：A1–A3 与 C1 并行可行（不同文件面）；C2 依赖 A1（时钟语义先修）；C3 依赖 C2；B 依赖 B1 核查。

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
| 帧传输双路径（C2/C4） | 集成（拟新增） | `media-process` 开/关配对：同一 fixture 逐帧像素一致（shm 开与 `ZW_`关断、内联回落各一档） | 长 fixture + 环境变量翻转 | Linux shm/内联两路径均显式覆盖；非 Linux 平台实测缺口 = TBD-4 |
| 帧拷贝回归 | `make bench-gate` 定向（A3） | media/render 域微基准不回退 | 基线在库 | 全量 capture 合入前补跑 |

命令纪律：测试一律经 `make test` / test-guard；性能测量走既有 M1 harness 显式入口（release + guard）。

---

## 9. 决策点（用户批复表）

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

## 10. Spec Lint 报告

**汇总**：26 Pass / 1 Warning / 0 Fail / 3 Skip（TBD-2/TBD-4 外部事实专项，skip≠pass）/ 4 PendingReview（D-RFC-1~4）/ 5 N/A（32 条规则表 + 决策/外部事实专项；门禁判定见末行）。

- **结论**：决策草案就绪，可交付用户评审（PendingReview: D-RFC-1~4）；**实施未就绪**——D-RFC 批复前不存在可实施合约，不动代码。
- **结构检查（关键项）**：执行摘要存在（§0，Pass）；场景存在性 N/A（仅设计交付，需求权威 = goal DC-1~5 表，验收映射在 §8 逐行给验证方法，测试绑定 Pass——§7 每批次带验证列）；实施交接完备（§7 表 + §7.1 边界，首步可执行——§0 首个落地步骤指向 A1/C1 两门禁分支，Pass）。
- **语义评审（关键项）**：原始意图与授权一致（Pass——推荐不采纳，§9 决策表留用户，加粗声明「批复前不改代码/不装 SDK/不变 feature 表」）；外部事实保守化（Pass——§4.2 平台框架全表标待验证 → TBD-2，未写入任何 FR/已定决策）；实现来源闭合（Pass——§3.1 来源列：A 仓内自实现、C 复用 protocol 先例 + 拟新增、B 拟新增平台 backend）；章节引用（Pass——§9→§5.1、§4.2→TBD-2、§5.2→C3、§10→§0/§3.1/§4.2/§7/§8/§9 均实指）；清单/依赖计数一致（Pass——新增依赖仅 §3.1/§5.2 所列拟新增项，无未列依赖）。
- **独立评审记录（同日）**：独立视角复核（完整文档 + 原始需求 + 代码锚点逐项核实）返回 0 blocker / 2 major / 5 minor，全部已修复——major①D-RFC ID 与前 RFC 撞名（§9 消歧注 + 全文标「前 RFC」）；major②§2.2 产能数字与入库 raw log 不一致（已对齐 42–45× 口径，evidence 侧同步订正）；minor 含 §7 门禁列、MSE feature 门控限定、ProcessManager 无重启先例的诚实边界（§5.1A）、§4.1 许可待核查标注。
- **Warning ×1**：M1 seek 冷路径（15s/45s 慢点）未归因——不阻塞架构决策，留 A 批次观察（见已接受风险）。
- **Skip ×3**：平台 API 可用性、许可核查结论、真机环境（均属 TBD-2/TBD-4 外部事实，未验证 ≠ 通过；不阻塞 A/C）。
- **PendingReview ×4**：D-RFC-1~4（决策点，呈现用户）。
- **N/A ×5**：场景存在性、UI 对齐、CLI 语义、默认动作闭合、优先级完备（无 UI/CLI 接口需求；无 FR/NFR 新定义——设计交付引用 goal 权威）。
- **未决与阻塞**：TBD-1（PendingReview）阻塞实施、不阻塞草案评审；TBD-2/TBD-4 仅阻塞方案 B 与跨平台收口。
- **已接受风险**：方案 C 非 Linux 帧传输先内联（性能缺口已知，双路径配对测试见 §8）；seek 冷路径方差未归因；决策前 main 保持纯方案 A 能力。

## 11. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | D-RFC-1~4 用户批复 | PendingReview（阻塞实施、不阻塞评审） | 用户决策 | 用户评审本 RFC；批复后按 §7 批次推进 |
| TBD-2 | 平台解码框架核查（4 平台 API/绑定/许可） | 重要（仅方案 B） | 外部事实未验证 | 若 D-RFC-3 批复，M3 首批核查落 evidence |
| TBD-3 | 音频真设备输出选型（NullSink→cpal） | 可选 | 用户对音频体验的要求未表 | 列 M3 议题，不阻塞 |
| TBD-4 | macOS/Windows/Android 真机验证环境 | 重要（仅方案 B/跨平台 C 收口） | 机器可用性与 CI runner 现状 | 与用户确认可用环境清单 |

## 12. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v1.0-draft | 2026-10-24 | 初稿：M1 基线（27d45b8c8）输入下的三路线对比、许可/分发核查、分批实施与回滚切点、D-RFC-1~4 决策点。同日独立评审修复（0 blocker/2 major/5 minor，见 §10 评审记录）；evidence 数字同步订正对齐 raw log。待用户评审 |
