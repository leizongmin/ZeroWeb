**最后更新**: 2026-09-05（**D3 第九增量完成——periodicWave.html 导入（47 用例
1399P/0F = 100%），webaudio 排除面清零**：迁址路径 the-periodicwave-interface/
恢复获取（原 404 注记解除）。配套 shim 三处：① Blink maxAbsSum 归一化公式
（1/max(Σ(|real[n]|+|imag[n]|), 0.5)——periodicWave 输出对拍面：real=[0,2,…,3]
→ maxAbsSum 5 → 峰值恰 1.0，替换此前不正确的 0.6324·sqrt(Σ) 公式）；②
createPeriodicWave/PeriodicWave ctor 同正义约束（real/imag 同长 + 最小长度 2 →
IndexSizeError；ctor 缺省成员以另一方长度补零——ctor-oscillator 无 imag 断言面
兼容）；③ OscillatorNode.setPeriodicWave（custom 波形存储 + type='custom'）。
**至此 pinned rev webaudio/the-audio-api 全部可执行用例均已导入或定性（47 用例
1399 subtest 全绿；剩余为 worklet/媒体互连/手势域不在离线渲染面）**。evidence
刷新 47 用例 1399 subtest。）
此前 2026-09-05：**D3 获批落地——GB-20260904 待决策征询跟进**：
OfflineAudioContext.startRendering 扩界**获批（窄授权）**——仅 offline 渲染路径
+ shim↔宿主桥扩展；RFC §0 实时设备输出量化面维持排除。征询凭据 msg
`om_x100b669923cd64a4c3e335615ed3d9f`。无代码变更。）
**（注：下方「最后更新」2026-09-04 巡检/第十八批收口块为前轮记录，保留作历史）**
**最后更新**: 2026-09-04（**WPT webaudio 第十八批收口——驱动用例解除排除导入
（39 用例 874P/0F = 100%）**：① `decodeAudioData` 入口语义面（spec BaseAudioContext
——detached 上下文（`_zwFrameEntry._zwSwDestroyed` 印记）→ InvalidStateError reject
优先；缺参/非 ArrayBuffer → TypeError；headless 无宿主音频解码器 → EncodingError
诚实 stub）+ Audio/Offline 双 prototype 共享；② part05 IframeOfflineAudioContext
**绑定构造器**（AudioContext 同款）；③ **createElementNS(HTMLNS,'iframe') 双 gate
修复（根因勘误）**——前段注记「_zwMEl plain-object 路径无 host handle」不准：ns
产物有 host handle（`__nN`），真缺口是 host tag store 对 ns handle 为空（探针实证
`__zw_get_tag_handle('__n0')` 返 ''）→ part04 contentWindow get-trap 与 part01
frame 移除链两个 `==='IFRAME'` gate 恒 miss（前者 contentWindow undefined、后者
destroyed 印记不置位使 decodeAudioData 走 EncodingError 分支）。修复：两 gate 各
inline `_nsHandles` HTML-ns localName 回落（ASCII 大写比对；helper 提取跨 part
复用探针实证不可达后改双点 inline，注记互指）——零 host 变更、零 _zwMEl 路径变更。
单测 `test_webaudio_decode_audio_data_face_m3xviii`（两面）+ 沙箱探针全链实证
（destroyed=true → dad:InvalidStateError）。evidence：
`evidence/2026-09-04-webaudio-detached-offline-batch18.json`。

（**第 1~17 批过程记录**——每批 shim 面明细/单测/排除注记——已归档至
  [archive/2026-09-04_wpt-webaudio-batches.md](archive/2026-09-04_wpt-webaudio-batches.md)，
  证据 JSON 序列见验证基线；累计口径 39 用例 874P/0F = 100%。）
## 当前状态

**专项定位**：媒体方向三拆之三（门控最深）。音频输出（解码→混音→设备）+ A/V 同步 +
volume/muted 真控制。**双重启动门控均已解除**：① M0 音频环境验证与验证策略成文
（2026-09-01 完成）；② media-playback M0 解码选型 RFC 获批（2026-09-01，路线 C）。

**M3 Web Audio 切片 1+2 已落地（2026-09-02，D1 批复 / D-WA-2 NullSink 先行）**：
- **切片 1（zero-media `webaudio` 模块）**：`OscillatorState`（sine/square/sawtooth/
  triangle 四型纯函数合成——相位累积 mod 2π 防 alias/长流漂移；start/stop 时序
  gate）+ `WebAudioContext`（源列表 → per-source 增益 → destination 总增益 →
  软削幅 → 下游 sink 写入；调用方注入单调钟——VideoPlayer 同款可测试性）；
  单测 7 件常驻（440Hz sine 过零率 ≈880 M1 契约锚点 / 方波同阶 / 未 start 静默 /
  stop 后静默 / 增益幅度 / 锯齿三角周期精确 / 双源并发）。
- **切片 2（shim + webview 接线）**：
  - shim part06 `AudioContext` 构造器（无 new TypeError；state 恒 'running'、
    sampleRate 48000、destination/currentTime 面）+ `createOscillator`（type 枚举
    归一 setter + frequency/detune AudioParam 最小值面 + start/stop 桥推）+
    `createGain`（gain AudioParam）+ connect 链式返回（spec connect 返回目标节点）；
    OfflineAudioContext 占位（illegal——RFC §0 不做清单）。宿主桥未注册时语义面
    完整不产声（headless 近似零回归面）。
  - webview `webaudio_registry`：`WebAudioRegistry`（ctx + NullSink 端点 + JS 节点
    句柄映射 + epoch 时钟）+ `register_webaudio_bridge_callbacks`（`__zwWA*` 五回调
    ——create_osc/start/stop/set_freq/active，字符串契约同 __zwVideoBridge 模式）；
    WebView 持 `webaudio()` Arc 句柄。
  - 生产接线：tab_worker `SetWebAudio` 命令 + worker 注入 `__zwWA*`（SetVideoPlayers
    同款 late-injection）；音频泵同 1ms 节拍推进（`wa.advance(now_ms)`，无活跃源
    快速门零开销）。**renderer 多进程路径桥面对齐（2026-09-02 补）**：renderer
    js_worker `SetWebAudio` + runtime 注入（镜像 tab 路径——`__zwWA*` 面两路径
    一致）；renderer 泵缺口（主循环无节拍驱动 advance）记录于 media-playback
    master.md 深结构缺口块（架构决策域待用户点名）。
  - e2e：`webaudio_bridge_nullsink_observable_chain`——JS AudioContext → shim →
    __zwWA* 桥 → Rust 合成 → NullSink 帧数（≈48000/秒）+ 过零率 ≈880 断言 +
    stop 后活跃源清零。media 45 单测、webview 679 全绿、browser xvfb 411 全绿、
    clippy（media/webview/browser）零警告。
- **WPT webaudio 可执行子集首批导入（同日）**：`webaudio/the-audio-api` 2 用例
  （audionode-connect-return-value + destination）全绿——配套 shim 面扩展：
  AudioNode 接口反射（numberOfInputs/numberOfOutputs/channelCount(2)/
  maxChannelCount(32)/channelCountMode/channelInterpretation）+ channelCount
  setter 语义（0 → NotSupportedError / >max → IndexSizeError——destination 断言面）+
  connect 非法目标 TypeError（audionode 断言面）+ OfflineAudioContext 构造兼容面
  （构造/length/sampleRate 反射 + startRendering rejected promise——RFC §0 简化
  记录，无离线渲染）。runner 新 `testharness-webaudio` 子命令（WEBAUDIO_TEST_FILES
  白名单 + make 目标 + fetch-webaudio-subset.sh 拉取脚本）。
  evidence：`evidence/2026-09-02-webaudio-wpt-subset.json`（2P/0F）。
- **WPT webaudio audit.js 框架接入 + OscillatorNode ctor 面导入（同日第二批）**：
  runner `run_webaudio_cases` 增 inline_extras 机制内联 webaudio/resources/*.js
  （audit.js/audit-util.js/audionodeoptions.js——canvas-tests.js 同款 vendored
  框架），`ctor-oscillator.html` **62 subtest 全绿**（runner 计数口径；含框架
  task 行时 64）。配套 shim 面扩展（part06）：
  - `AudioParam` 接口（Illegal constructor + `_zwMakeAudioParam` 工厂——
    instanceof 面 + 非 finite value TypeError + 调度方法链式 + defaultValue/
    minValue/maxValue）；frequency/detune/gain 三 param 换用工厂（value setter
    同步宿主桥 set_freq）。
  - `OscillatorNode`/`GainNode` 节点构造器（`new X(ctx, options)`——ctx 校验
    TypeError + AudioNodeOptions dict 校验：channelCount 0/>32 →
    NotSupportedError、channelCountMode/channelInterpretation enum invalid →
    TypeError；type='custom' 单给 → InvalidStateError + periodicWave 类型校验
    TypeError）+ `PeriodicWave` 接口（构造 + real/imag/disableNormalization
    存储——无 FFT 合成，RFC §0 简化记录）。
  - AudioNodeOptions 反射面：channelCount setter 0 → NotSupportedError、
    >32 → IndexSizeError（源节点与 destination 同界）；channelCountMode/
    channelInterpretation 可写枚举（invalid 静默保留——setter 面与 ctor dict
    面严格度分离，spec WebIDL enum 惯例）。
  evidence：`evidence/2026-09-02-webaudio-ctor-oscillator.json`（64P/0F）。
- **AudioContextOptions + 生命周期面落地（同日第三批评估）**：shim part06
  `AudioContext(options)` 扩 AudioContextOptions dict（latencyHint enum 三值/
  double——enum invalid → TypeError、非 dict 对象 → TypeError、non-finite double
  → TypeError）+ `sampleRate` 选项（[3000,768000] 外 → NotSupportedError、范围
  内反射——headless 合成面以该率运行）+ `baseLatency` 反射（enum 档 5/15/40ms、
  double 档 = hint 值）+ `close()`（state → 'closed' + settled Promise）/
  `suspend()`/`resume()`（closed 后 reject InvalidStateError）。
  **audiocontextoptions.html 不导入**——latencyHint double 档断言 Chromium 设备
  专用 baseLatency 档位值（playbackLatency×10 → 0.8 恒等——Linux Chromium 实测
  档），headless 无设备延迟模型不可复现；语义面已落 shim，随设备面（CpalSink
  真出声切片）复评。
（**第 4~13 批 + 第五/六/七批**——处理类节点 ctor 族两批 / AudioNode 接口面 /
  源节点语义面 / AudioBuffer 构造与 getChannelData / ctor-audiobuffersource /
  getOutputTimestamp 饱和等明细——已归档至
  [archive/2026-09-04_wpt-webaudio-batches.md](archive/2026-09-04_wpt-webaudio-batches.md)，
  累计口径见验证基线。）

- **余项**：createGain 的 per-node 桥推（当前 per-osc gain 由 WebAudioContext 承接，
  gain 节点 → 桥映射挂真出声/设备切片）；WPT 余面（osc-basic-waveform/
  sub-sample-start/detune-* 依赖渲染量化面——startRendering，随渲染面评估）。

**M0 已收口（2026-09-01）**：
- 环境实测：内核层 HDA 声卡在；**ALSA dev 头缺失（libasound2-dev 未装）→ cpal 默认
  ALSA host 无法编译**；PulseAudio dev 在但 server 不可用（Connection refused）。
- 验证策略成立：**`AudioSink` trait 抽象 + 双实现**——`CpalSink`（feature-gated
  `audio-cpal`，设备面）+ `NullSink`（headless/CI 默认，可观测：写入帧数 + 过零率
  频域代理断言）。M1 验收 = NullSink 可观测断言（CI 常驻）+ CpalSink 人工冒烟（可选）。
- cpal 编译实测须装 `libasound2-dev`（系统级变更 → 待用户决策 D2；不阻塞 trait/
  NullSink 层设计与实施）。
- 证据：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)

**与兄弟 goal 的边界**：
- media-playback — 视频/解码选型归其管；A/V 同步接口对齐（audio clock 主时钟——契约记录
  于两流 master.md）；**解码面依赖其 RFC 选型，输出面（AudioSink trait）与选型解耦可先行**
- media-elements — 语义面归其管；volume/muted 本目标接真增益（IDL 语义已由其 M3 扩批
  III 落地：非有限 TypeError/同值短路/queued volumechange/load 清 pending）
- js-dom — volume/muted 反射段共享，`git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时 + 2026-09-01 M0 探测 / M1 切片 1 更新）

### 现有实现

- ✅ 反射底座：muted/volume 属性反射（R3040 + M3 扩批 III IDL 语义全对齐）
- ✅ 时钟底座：rAF 帧驱动（P1a）——音频时钟对齐可挂
- ✅ 环境/策略底座：M0 收口（AudioSink trait + NullSink 验证策略成文）
- ✅ **输出面（M1 切片 1）**：`zero-media::audio`——`AudioSink` trait（start/
  write f32 交错/pause/resume/underrun_count）+ `NullSink`（写入帧数 + 过零率
  频域代理（2×频率锚定）+ 暂停拒写计 underrun）；单测 5 件常驻（启动前拒写/
  暂停门控/非整帧拒收/440Hz 过零率/重启累计语义）
- ✅ **设备面（M1 切片 2）**：`CpalSink`（feature-gated `audio-cpal`，cpal 0.16 入
  workspace optional 依赖）——f32 原生采样直通设备流、回调队列饿死计 underrun、
  pause/resume 流控（write 拒收 + 流暂停双闸）、格式变更须重建（显式报错防
  流错配）；环境自适应冒烟常驻（无设备/格式不支持 → 构造报错回落 NullSink，
  本环境实测构造成功 + start/pause/resume 全通）
- ✅ **混音面（M1 切片 3）**：`Mixer`——attach/detach 源句柄（资源生命周期面）+
  per-source volume/muted 增益 + mix_into 软削幅（clamp [-1,1]）写下游 sink；
  短源补零不断流；单测 7 件常驻
- ✅ **解码面（M2c，跨 goal：media-playback 流落地）**：`zero-media::audio_decode::
  AudioDecoder`（symphonia 0.6：mp3 + ogg/vorbis）——f32 交错 PCM 输出直写
  AudioSink 契约；全链 e2e 双件常驻（mp3 + vorbis fixture → NullSink 过零率 ≈880
  = 2×440Hz，本档 NullSink 断言契约首次真值实证）
- ✅ **播放管线宿主侧接线（M2c 后续切片 A/B，跨 goal：media-playback 流落地）**：
  `<audio>` settle → `VideoPlayerRegistry.register_audio_source` → 宿主桥 play →
  音频泵（tab_worker `audio_advance_all`，实时节奏逐包解码）→ NullSink 写入；
  volume/muted 增益联动（media-elements IDL setter 桥推 `setGain` + play 起播同步）；
  seek 追赶区静默（skip_until_ms 丢弃线）；导航离开 `clear()` 释放（DC-4）；
  tabworker/renderer 双路径对齐（SetVideoPlayers 注入）。e2e 三面常驻
  （webm video / mp3 audio / oga-opus 不登记负例）
- ✅ opus 解码面（2026-09-01 M2c opus 接线落地，跨 goal：media-playback 流）——
  `opus-decoder 0.1.1` 纯 Rust（RFC 6716/8251，零 unsafe 零 FFI）补齐 symphonia 缺位；
  `zero-media::opus_decode::open_ogg_opus`（symphonia ogg 容器 + OpusHead 解析 +
  pre-skip 丢弃）；`sample-ogg-opus.oga` 转正可播（registry 双面回落登记 + 泵推进）
- ⚠️ 重采样/混音接线未实施——播放管线把解码帧喂 Mixer 的宿主侧接线待做
- ✅ 选型已对齐（media-playback RFC 获批：路线 C，symphonia 音频解码面归 M2c）
- ✅ 音频 e2e 资产：`tests/fixtures/media/`（sample-mp3.mp3 / sample-ogg-opus.oga，
  ffmpeg 生成、来源清白、生成命令记录于该目录 README）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| A1 | 音频环境验证 + headless 验证策略 | ✅ M0 收口（2026-09-01） |
| A2 | 解码选型未对齐（外部门控：media-playback M0） | ✅ 已对齐（RFC 获批，2026-09-01） |
| A3 | 零音频管线（解码/重采样/混音/输出） | 🔄 M1 切片 1-3 + M2c 解码面 + M2c 后续宿主接线落地（NullSink/CpalSink/Mixer/symphonia + 播放管线增益联动）；余重采样与 Mixer 多源接线（当前注册表内 NullSink 直连） |
| A4 | A/V 同步机制缺失 | 🔄 M2 主体落地（2026-09-01，media-playback 流切片 D+E）——audio clock 主时钟（webm 双轨伴生音频解码 + 视频帧调度对齐音频游标 + drift 构造校正 + currentTime 组合时钟 + seek 双轨对齐）；余设备面真输出（CpalSink 冒烟，可选） |
| A5 | 音频 e2e 资产 | ✅ 真解码链 e2e 落地（mp3 + vorbis fixture → NullSink 过零率锚点常驻）+ 合成源面 |

## DC 达成审计（2026-09-02，对照入口文档 Done Criteria 逐项核验）

**DC-1（环境验证与验证策略成文）✅**：M0 收口——cpal 集成 PoC（本机 HDA 声卡 +
D2 获批装 libasound2-dev 后编译/枚举/CpalSink 真设备流冒烟全链通过，
[evidence](evidence/2026-09-01-cpalsink-device-smoke.md)）；dummy 回退验证 =
NullSink 双实现策略成文 + e2e 资产化（过零率锚点常驻）。

**DC-2（音频管线端到端）🔄（余设备真输出常驻）**：解码 → 重采样 → 混音 →
设备输出——解码（symphonia mp3/vorbis + opus-decoder）✅、混音（Mixer 组件 +
per-entry 增益联动 ✅）、重采样（symphonia 输出面覆盖；独立重采样器未实施——
fixture 采样率与 sink 匹配场景零需求，记录注记）、**设备输出 e2e 留桌面环境**
（CpalSink 冒烟已过但非常驻——headless CI 无声卡的结构性边界，goal 契约允许
「真输出只在本地有声卡环境抽验」）。headless 总线断言常驻 ✅（NullSink 帧数 +
过零率 = CI 可跑面）。

**DC-3（同步与控制）✅**：A/V 同步 audio clock 主时钟 + drift 构造校正 +
seek 双轨对齐（media-playback 流切片 D+E 兑现，联合 e2e
`registry_av_pair_reaches_ended_after_audio_exhausted` 常驻）；volume/muted
真控制（增益联动全接——IDL setter 桥推 + play 起播同步）；多源混音
（per-entry NullSink 直连覆盖并发语义面；Mixer N→1 接线挂设备切片——决策
注记成文）。

**DC-4（`<audio>` 全路径 + Web Audio 评估）✅**：`<audio>` 纯音频播放全路径
e2e 常驻（M2c 后续切片 A/B：settle → 桥 play → 泵推进 + mp3/oga 负例三面）；
AudioContext 最小面 RFC 完成 **且已获批实施**（D1，2026-09-01）——切片 1+2
落地（zero-media webaudio 模块 + shim 门面 + 宿主桥 + 泵接线 + e2e）+ WPT
webaudio 子集六批导入（2 + 62 + 259 + 1 + 7 subtest，合计 12 用例 331P/0F）。

**DC-5（测试与质量不可退让）✅**：make test 18705 全绿（2026-09-02 组合树实测）、
clippy 零警告、每切片带单测 + e2e 资产化（webaudio 7 单测 + NullSink 链 e2e +
桥 roundtrip + ctor 族契约单测 m3w4 六断言组）。

**结论**：DC-1/3/4/5 满足；DC-2 余设备真输出常驻面（结构性边界：headless CI
无声卡——goal 契约的双轨验证形态下，本地抽验 evidence 已在档，**不构成
DONE 阻塞**；Mixer/重采样接线随设备切片可选推进）。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | AudioContext（Web Audio）最小面可行性 RFC → 是否实施 | ✅ 获批（2026-09-01，GB-20260901 批复）——D-WA-1 批准切片 1+2；D-WA-2 选**先 NullSink**（设备面挂 media-audio M1 CpalSink 真出声切片）。RFC：[../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md](../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md) |
| D2 | 安装 `libasound2-dev`（系统级 apt 变更）以解锁 cpal 编译验证 | ✅ 获批（2026-09-01）——装包后补 cpal 编译实测 |
| D3 | **OfflineAudioContext.startRendering 渲染量化面（RFC §0 扩界申请）**——接口语义族已 18 批 874P/0F 吃尽（2026-09-04），余下上游用例（osc-basic-waveform / gain.html / periodicWave / ctor-iirfilter 渲染对比 / audioparam-method-chaining / convolver DSP 卷积等 ~15 件）全部卡在 startRendering 渲染断言。**实施基础已在**：zero-media `WebAudioContext` 振荡器四型合成 + 图推进（切片 1 落地，NullSink 可观测）；缺的是 offline 渲染路径（长度采样离线合成 → AudioBuffer 产物 → promise resolve）+ shim↔宿主桥扩展。为何需用户：主 RFC §0 显式排除「渲染量化面（不做清单）」——扩界属 Mission 级范围变更（run-rules rule 11），agent 不可代判；不批准亦请明示「维持排除」以便归档 | ✅ **获批（2026-09-05，窄授权）**——批准 **仅 offline 路径**（长度采样离线合成 → AudioBuffer → promise resolve + shim↔宿主桥扩展）；RFC §0 的**实时设备输出量化面维持排除**（本批复未解除）。解锁余下 ~15 件 startRendering 卡点用例的导入评估。征询凭据：msg `om_x100b669923cd64a4c3e335615ed3d9f`（GB-20260904 巡检合并征询）；批复来源：session 对话（GB-20260904 待决策征询跟进） |

## 下一步计划

1. ~~**Web Audio 最小面实施（D1 已批准）**~~ 🔄 切片 1+2 ✅ 2026-09-02 落地
   （zero-media webaudio 模块 + shim AudioContext 门面 + __zwWA* 宿主桥 + 泵接线 +
   e2e——见当前状态）；WPT webaudio 子集导入 ✅ 十八批（39 用例 874P/0F = 100%，
   2026-09-04 收口——含 MediaStreamAudioDestinationNode ctor 面 +
   OfflineAudioContext detached execution context 面，the-audio-api 全接口目录含 MediaStream 邻域清点收束——含 audit.js 框架接入 + ctor 全族 + AudioParam 异常面 +
   AudioBuffer/OfflineAudioContext 构造面 + AudioScheduledSourceNode 调度异常 +
   getOutputTimestamp + AudioContextOptions + detached/not-fully-active 面 +
   ConvolverNode 语义面 + batch14 constructor-allowed-to-start 勘误移除）；余：
   设备面挂 M1 CpalSink 真出声切片（D-WA-2）。
   **接口语义族 headless 可导入面已吃尽（第十六批 the-audio-api 各接口目录
   清点收束）**——余下用例全部依赖 startRendering 渲染量化面 / DSP 合成（卷积/
   FFT）/ AudioBufferSourceNode 播放推进面 / copyToChannel 数据面 / worklet /
   用户手势 / onstatechange 时序 / 真设备 sinkid / 跨源 iframe helper / bfcache
   导航 / getUserMedia 域 / MediaElement 互连域，随渲染切片或设备切片复评。
2. **M1 收口评估（余项收窄）**：Mixer 多源混音接线**决策注记（2026-09-01）**——
   现播放管线 per-entry NullSink 直连已覆盖多源并发语义面（per-source 增益/独立
   解码流/并发泵）；Mixer（M1 切片 3 组件，7 单测常驻）的价值在**单设备输出流的
   N→1 合流**——即 CpalSink 真设备输出时的前置组件。NullSink 阶段接 Mixer 只添
   无行为变化的中间层（且破坏 per-entry sink 可观测断言面）。**结论：Mixer 接线
   挂到 CpalSink 真出声切片**（可选/桌面环境），M1 的 CI 可验面已收口。
   CpalSink 真出声冒烟仍留桌面环境（编译/枚举面 D2 已验证）。
2. **M2**：~~A/V 同步接口对齐~~ ✅ 2026-09-01 主体兑现（media-playback 流切片
   D+E：audio clock 主时钟 + 组合时钟 + seek 双轨对齐）；A/V pair ended 面回归
   守卫落地（webview `registry_av_pair_reaches_ended_after_audio_exhausted`——
   伴音流末 video player 走到 Ended、泵停）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 环境验证 + 验证策略（门控） | ✅ 完成（2026-09-01，含 D2 后 cpal 编译实测补录） |
| M1 — 首个声音输出 | 🔄 切片 1-3 + 解码链 e2e + 播放管线宿主接线落地（2026-09-01）；**D2 获批项闭环（2026-09-01）**——cpal 编译 + 39 测全绿 + CpalSink 真设备流冒烟通过（Ok 分支：构造/start/write/pause/resume 全链，[evidence](evidence/2026-09-01-cpalsink-device-smoke.md)）；余 Mixer 接线（挂真出声切片，可选） |
| M2 — A/V 同步 + 控制 | 🔄 主体落地（2026-09-01，media-playback 流切片 D+E：audio clock 主时钟 + 组合时钟 + seek 对齐）；余 CpalSink 真出声冒烟（可选） |
| M3 — `<audio>` 全路径 + Web Audio 评估 | 🔄 `<audio>` 纯音频播放全路径 e2e 已常驻（M2c 后续切片 A/B：settle → 桥 play → 泵推进）；Web Audio 最小面 **D1 已批准**（2026-09-01，D-WA-2 选先 NullSink）——切片 1+2 待实施 |

## 验证基线

- 测试基线：`make test` 全绿 18866（2026-09-04 组合树实测）；clippy 零警告
  （default 与 `--features audio-cpal` 双配置）
- WPT webaudio：**47 用例 1399P/0F = 100%**（2026-09-05 二十六批累计——第二十六批
  periodicWave（迁址路径 + Blink 归一化 + setPeriodicWave）31 subtest + 第二十五批
  ctor-iirfilter（ctor 校验 + Functional pass-through 等价对拍）6 subtest + 第二十四批
  ctor-audiobuffer（ctor 全族 + multiple contexts 渲染对比）62 subtest + 第二十三批
  audiobuffer-copy-channel（copy 语义精化）62 subtest + 第二十二批
  nominal-range（nominal range 界表）~327 subtest + 第二十一批
  method-chaining 3 + 第二十批 gain.html 7 + 第十九批
  osc-basic-waveform 33 + 第八批~第十八批 39 用例 874P；原第二十批前口径——
  42 用例 911P 面——
  osc-basic-waveform（D3 offline 渲染最小面）33 subtest + 第二十批 gain.html（通道
  图推进）7 subtest + 第二十一批 audioparam-method-chaining（automation timeline）
  3 subtest；第八批~第十八批 39 用例 874P 面——connect 返回值 +
  destination + ctor-oscillator 62 + ctor-gain/stereopanner/delay/biquadfilter/
  analyser + createPeriodicWave 异常面 + audioparam-exceptional-values 66 +
  audiobuffer 面 + 第七批 ctor-channelmerger/channelsplitter/constantsource +
  audiobuffer-getChannelData + 第八批 audionode/different-contexts +
  第九批 ctor-waveshaper/ctor-dynamicscompressor/dynamicscompressor-basic/
  ctor-panner/iirfilter-basic + 第十批 biquadfilternode-basic/
  ctor-offlineaudiocontext + 第十一批 constant-source-basic/
  stereopannernode-basic/audiobuffersource-basic + 第十二批
  ctor-audiobuffersource + 第十三批 audiocontext-getoutputtimestamp +
  第十四批 audiocontextoptions/suspend-after-construct + 第十五批
  promise-methods-after-discard（constructor-allowed-to-start 第十五批勘误
  移除）+ **第十六批 ctor-convolver/convolver-setBuffer-null/
  convolver-setBuffer-already-has-value/realtimeanalyser-basic** +
  **第十七批 ctor-mediastreamaudiodestination**（MediaStreamAudioDestinationNode
  语义面）+ **第十八批 offlineaudiocontext-detached-execution-context**
  （decodeAudioData 入口语义 + IframeOfflineAudioContext 绑定 +
  createElementNS iframe 双 gate 修复）——the-audio-api 全接口目录含
  MediaStream 邻域清点收束；
  the-audiocontext-interface + the-audio-api 各接口目录清点收束）；
  evidence：`evidence/2026-09-05-offline-render-d3.json`（第十九批）、
  `evidence/2026-09-02-webaudio-wpt-subset.json`（首批）、
  `evidence/2026-09-02-webaudio-ctor-oscillator.json`（第二批）、
  `evidence/2026-09-02-webaudio-ctor-family.json`（第四批）、
  `evidence/2026-09-02-webaudio-audiobuffer.json`（第五批）、
  `evidence/2026-09-02-webaudio-gain-basic.json`（第六批）、
  `evidence/2026-09-03-webaudio-node-family-batch7.json`（第七批）、
  `evidence/2026-09-03-webaudio-audionode-batch8.json`（第八批）、
  `evidence/2026-09-03-webaudio-processing-family2-batch9.json`（第九批）、
  `evidence/2026-09-03-webaudio-zero-gap-review-batch10.json`（第十批）、
  `evidence/2026-09-03-webaudio-source-semantics-batch11.json`（第十一批）、
  `evidence/2026-09-03-webaudio-ctor-buffersource-batch12.json`（第十二批）、
  `evidence/2026-09-03-webaudio-getoutputtimestamp-batch13.json`（第十三批）、
  `evidence/2026-09-04-webaudio-detached-face-batch15.json`（第十五批）、
  `evidence/2026-09-04-webaudio-convolver-batch16.json`（第十六批）
- NullSink 可观测锚点：440Hz 正弦 @48kHz 过零率 ≈880（2×频率；修正 M0 evidence
  的 ≈440 笔误——evidence 只追加不修改，以代码与本档为事实源）；暂停拒写计
  underrun；非整帧写入拒收
- 复核（2026-09-04 治理整固后终审）：fresh 跑 874P/0F（39 文件）与本档记录
  一致；验证基线 evidence 链 16 文件全在盘。
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- perf-gate（2026-09-04 全量复核：webaudio 第 14~18 批 shim 积累后）——GATE FAIL
  32/113 指标超预算，失败集横跨 12 crate（style-system/script-sandbox/webview/
  browser-shell/protocol/net 等，无一在本次变更触达路径）。三层判据（2026-09-03
  learning 同法）：① 失败集与 09-03 run A/B 轮换特征同族；② 隔离复测三样本全部
  回预算内（cascade/500 76.2→37.8µs、sandbox_creation 1.53→0.93ms、
  tab_creation_100 7.78→5.76µs，余量 1.1~2×）；③ 代码窗口核对——本会话变更
  （shim 执行期注入/registry loop 标志/tab_worker 泵钟/computed_style video 分支）
  均不触达失败指标敏感路径 → 判 ZRG 负载噪声（同族签名），不动基线不 relax。
  此前（2026-09-03 本轮 shim 代码量积累后定向核查）：两轮全量 GATE FAIL 失败
  指标集完全轮换（零交集）+ 全部失败指标隔离复测 1.5~3.3× 余量回预算内 + 本轮
  变更不触达失败 crate 敏感路径（shim 仅 V8 执行期注入）→ 判 ZRG 负载噪声签名
  （ZRG-2026-08-22/23/24-01 同族），不动基线不 relax——判据沉淀
  [docs/learnings/performance/2026-09/2026-09-03-bench-gate-rotating-fail-noise-signature.md](../../learnings/performance/2026-09/2026-09-03-bench-gate-rotating-fail-noise-signature.md)
- evidence：[evidence/2026-09-01-m0-environment-probe.md](evidence/2026-09-01-m0-environment-probe.md)

## 归档

- [archive/2026-09-01_m0-m2-and-d2-closure.md](archive/2026-09-01_m0-m2-and-d2-closure.md) —
  M0/M1/M2 过程与 D2 获批项闭环记录（只追加不修改；本控制面保留最新态）。
- [archive/2026-09-04_wpt-webaudio-batches.md](archive/2026-09-04_wpt-webaudio-batches.md) —
  WPT webaudio 第 1~17 批过程记录归档（2026-09-04 治理切片——头链批次明细 +
  当前状态块第 4~13 批与第五/六/七批明细原文移入；本控制面保留第 18 批与
  累计口径 39 用例 874P/0F = 100%）。
