# DC-5 收口审计——RFC §10 批复与落地对照（2026-10-25）

审计对象：DC-5「RFC 决策与最终支持范围审计」。方法：RFC v1.9 §10 决策表逐项对照
仓内实况（feature 表、代码面、常驻回归面、证据面，本日 grep/清单核实），不以文档
记载自证。核对基线 git 9c6652fa3。

## 一、D-RFC-1~4 批复落地对照

| ID | 批复（2026-10-24） | 落地实况（本日核实） | 判定 |
|---|---|---|---|
| D-RFC-1 | 批准方案 C（独立媒体进程） | C1–C4 全落地：protocol 媒体消息族 17 变体 + media-host 骨架与解码内核/自走时钟 + registry 开态客户端切换（HostMediaFacade push 镜像/21 面分流）+ 崩溃检测双路与退避重启；M4 三态配对验收过（三态 M1 零回归、帧摘要全等、代际风暴零泄漏、30min 资源审计）。`media-process` feature **默认关**（`crates/webview/Cargo.toml` features `default = ["v8"]`，`media-process = []` 纯门） | **落地**（开关态=待决项，见下三） |
| D-RFC-2 | A 批次随批复即启 | A1–A4 落地：时钟 rate 1.00000（修复前 1.45402）；A2 三缺口闭合——本轮 bg02-media 基线 v2 复评再确认并修复三产品根因（loop attr 推送/march 边沿拍/音频重复登记幂等）；A3 源驻留内存收敛（C2 切片 2：测试进程 cores 1.001→0.012、tick p50 431ns→3.7µs）；A4 seek 冷路径 O(GOP)（15s 302ms→18.3ms、45s 1000ms→18.4ms） | **落地** |
| D-RFC-3 | 方案 B（平台硬解）暂缓 | 零平台 SDK 代码（`crates/media/src/`、`apps/media-host/src/` grep VideoToolbox/MediaCodec/AVFoundation/windows-media 无命中）；TBD-2 维持未验证；能力表无平台硬解主张 | **遵守** |
| D-RFC-4 | decode-h264 维持默认关 | `decode-h264 = ["zero-media/decode-h264"]` 非 default（webview/media-host 两处核实）；CI 主线零 openh264 工具链；能力真值以 `probe_ok = cfg!(feature)` 如实表达（D4 instrument 断言 av1/h264 默认不在列） | **遵守** |

## 二、DC-1~4 闭合证据复核（不豁免、逐项有证）

| DC | 闭合证据 |
|---|---|
| DC-1 基线与能力 | M1 基线（[2026-10-24](../2026-10-24-m1-baseline/ANALYSIS.md)）+ 能力真值三对照（C4）+ D4 欠申报订正（8/8「在列 ⇔ 实解」双断言） |
| DC-2 状态/时钟/seek 契约 | IF-003 契约测试（epoch 代际/±1ms 容差）+ C4 配对验收 + bg02-media 基线 v2 复评闭合（[2026-10-25](../2026-10-25-bg02-media-v3/ANALYSIS.md)，「拟新增代理/共享时钟测试」落在 media_host_contract 8 场景） |
| DC-3 故障注入与恢复 | C3 SIGKILL 注入契约 8/8（kill host → error 广播 → 退避重启 → reload 恢复）+ shim march Error face 页面级通路（`test_media_march_error_face_dispatches_element_error_c3`） |
| DC-4 权限不足与回退 | 能力不虚报（C4/D4 双断言 + handshake 常驻在列断言）+ feature 关路径 M1 release 零回归（C2 切片 2 20.0s 面）+ IF-006 不回落纪律（spawn 失败归错误面，与 image-decoder 先例一致） |

## 三、「启用时机归 C4/M4 另议」——已到期，移交用户裁决

RFC §10 附注：`media-process` 启用时机归 C4/M4 验收后另议。C4/M4 已完成，另议到期。
**此为产品开关裁决，agent 不自行决定**，移交时附双侧事实：

- 支持默认开：C4 三态配对零回归；崩溃隔离（解码进程 SIGKILL 不连累 renderer，退避重启恢复）；内存收敛（cores 1.001→0.012）；Linux shm 帧路径。
- 支持维持默认关：非 Linux 帧路径仍内联（RFC §5.1A 已知缺口）；验收平台仅 Linux x86_64（TBD-4 真机环境未核）；A 批次全部收益在默认路径已生效——开 feature 的边际收益集中在崩溃隔离与内存面；双路径长期维护成本。

## 四、最终支持范围（审计结论）

1. **默认构建**（所有用户，feature 全关）：方案 A 强化内联路径——A1–A4 全部生效；
   能力主张 = symphonia 软解 8 项基础（mp3 / ogg-opus / ogg-vorbis / wav / mp4-aac /
   webm-vorbis / webm-opus / webm-vp9 视频面）+ 2 项 feature 门控（av1/h264）如实申报。
2. **`media-process` 显式开启**：C 批次托管路径——IF-001~006 契约、自走时钟 push
   镜像、崩溃退避重启、Linux shm 帧路径；验收平台 Linux x86_64。
3. **平台硬解**：未启动（D-RFC-3 暂缓）；**H.264/AV1**：feature 门控默认关（D-RFC-4）。
4. 音频真设备输出（TBD-3，NullSink→cpal）：可选未排期。

## 五、常驻回归与适用门禁清单

| 批次/面 | 常驻测试 | 门禁 |
|---|---|---|
| A1/A4 时钟与 seek | `media_m1_baseline` 5 项（T1–T5，release harness） | M1 harness rate/漂移/容量界 |
| A2 页面语义 | engine `js_dom_bridge_tests/part09` 28 项 media（march/error face/loop 推送）+ renderer `media_march_tests` 4 项 | make test |
| A3/C2 内存与内核 | zero-media 解码/webm 族 + `video_registry` 单测（loop 回卷/登记幂等/mp4-aac） | make test + bench（源驻留预算） |
| C1/C3 契约与故障 | `media_host_contract` 8 场景（含 kill-switch、feature 关回归、SIGKILL 注入） | make test |
| C4/D4 验收与能力 | `media_c4_paired` 6 项（`--features media-process` 显式运行） | 提交批内显式跑 |
| 资源面 | bg02-media 基线 v2（2 场景 n=2，`probe_travel` 回卷安全界，[基线](../../../../perf/baselines/background-resource-linux-x86_64-v2.json)） | background-resource-gate check |
| 工作区 | make test 87 套件 + guarded-clippy | 本轮（2026-10-25）exit 0 全绿 |

## 六、遗留与移交

- **用户裁决**：`media-process` feature 默认开关（上文三，双侧事实已附）——本 goal
  唯一待决项，记入 master 缺口与待决表。
- TBD-2（平台解码核查）随 D-RFC-3 暂缓；TBD-4（真机环境）跨平台 C 收口时再启；
  两者不阻塞本 goal。
- 控制面过时项随本轮订正：缺口表「故障注入」行（NOT_STARTED → C3 已落地）、
  「架构投入」行（待决 → D-RFC-1 已批复）、M3 行（收尾状态 → 完成）。
- 全部 DC 有证：若用户对开关裁决选「维持默认关」或移交产品线，goal 即可 DONE；
  若选「默认开」，开关切换按产品变更另启切片（不等同本 goal 重开）。
