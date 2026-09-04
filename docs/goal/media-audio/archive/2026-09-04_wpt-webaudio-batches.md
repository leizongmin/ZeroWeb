# WPT webaudio 批次过程记录归档（第 1~17 批明细）（只追加不修改）

**入口文档**: [../media-audio.md](../media-audio.md) | **控制面**: [../master.md](../master.md)
**归档日期**: 2026-09-04（治理切片——master.md 头链与当前状态块的批次明细按治理规则「持续演进、不无限增长」移入本档；最新批次（18）与累计口径保留在 master.md；证据 JSON 序列在 evidence/ 不动）

---

## 头链批次记录（第 17 批 → 第 4 批，倒序原文）

此前同日：**WPT webaudio 第十七批——MediaStreamAudioDestinationNode
语义面（38 用例 867P/0F = 100%）**：ctor-mediastreamaudiodestination 导入（全 task
零渲染）——shim part06 落 `_zwWABuildMediaStreamDestination` builder + 构造器
（ctx 校验 TypeError / options 非 object TypeError）+ createMediaStreamDestination
工厂（Audio/Offline 双 context）+ prototype 链——1 入 0 出 + channelCount 2 缺省、
mode 'explicit'、interpretation 'speakers'；options.channelCount **非固定**（=7 合法
——与 splitter/merger 固定通道面分流；[1,32] 外 NotSupportedError）+ mode 非
explicit InvalidStateError + interpretation 枚举 TypeError + stream 反射最小面
（readonly 占位——真 MediaStream 域排除）。**目录清点扩展**：the-mediastream-
audiosourcenode（getUserMedia 域 3 件）/ the-mediaelementaudiosourcenode（复评——
cors https+CGI、srcObject 互连、crash 系、setSinkId 真设备——全排除维持）/
scriptprocessor（废弃）/ audioworklet / processing-model / permission-policy——
webaudio the-audio-api **全接口目录含 MediaStream 邻域清点收束**。evidence：
`evidence/2026-09-04-webaudio-mediastreamdestination-batch17.json`。此前 2026-09-04：
**WPT webaudio 第十六批——convolver/analyser 零渲染候选
（36 用例 863P/0F）**：ctor-convolver（5 W3CTH task 全绿）/ convolver-setBuffer-null
/ convolver-setBuffer-already-has-value / realtimeanalyser-basic 四件导入——
**ConvolverNode 语义面落地**（shim part06：`_zwWABuildConvolver` builder +
`new ConvolverNode(ctx, options)` 构造器 + createConvolver 工厂——normalize 缺省
true（ctor options `disableNormalization` 反射）/ buffer 缺省 null + **sampleRate
不匹配 NotSupportedError** + 重复赋值/null 清空往返不抛 + channelCount [1,2] 界
（ctor dict 级 `_zwWACtorChannel12` + setter `_zwWAInstallChannel12Setter` 双面）+
mode 'clamped-max' 缺省（'max' NotSupportedError / 'foobar' TypeError）；Offline
共享补接 createConvolver——前置共享块在 prototype 赋值前执行拿 undefined，单测实证
后改挂后置块）；realtimeanalyser-basic 零新增缺口（Analyser 缺省面 ctor-analyser
已覆盖）。上游目录清点扩展：convolver/analyser 余件全 startRendering 渲染断言
（DSP 卷积/FFT 量化归 RFC §0）+ MediaElementAudioSource 全族（media 互连域）+
MediaStream 族（getUserMedia 域）+ ScriptProcessor（已废弃）+ worklet/processing-
model/permission-policy——**the-audio-api 各接口目录清点收束**）。evidence：
`evidence/2026-09-04-webaudio-convolver-batch16.json`；单测
`test_webaudio_convolver_face_m3xxxi`；make test 18866/0；testharness-media
598P/0F/24PF 零回归。此前 2026-09-04：**WPT webaudio 第十五批——batch14 回归勘误 + detached 面
导入（32 用例 836P/0F）**：① constructor-allowed-to-start 勘误移除——第十四批误入
清单（test_driver.bless 在 R142 unsupported 白名单外 → runner 判 Unsupported →
make testharness-webaudio exit 1；且其断言「构造后 suspended → onstatechange 异步
转 running」与 shim headless 恒 'running' 结构性互斥，bless stub 化后仍必 Fail）；
② promise-methods-after-discard 导入（3 subtest）——**AudioContext detached/
not-fully-active 语义面落地**：shim part05 IframeAudioContext 绑定构造器（R295/R373
同款——frame 移除后构造抛 InvalidStateError + 产物印记 `_zwFrameEntry`）+ part06
suspend/resume/close 的 destroyed reject 面（spec not-fully-active 三方法入口语义）
+ part01 `_zwRemoveIframeWindowClient` 的 destroyed 印记与 SW client 解挂解耦
（plain iframe 无 `_zwSwClientId`/无 SW host 桥同样置位——此前 host-fn 缺失时提前
return 使印记永不置位）；余件（suspend-resume 整文件 task3 startRendering /
playbackstats / rendersizehint / sinkid / not-fully-active 跨源 helper /
bfcache navigation / worklet）逐件核实全排除——**the-audiocontext-interface 目录
清点收束**）。evidence：`evidence/2026-09-04-webaudio-detached-face-batch15.json`；
单测 `test_webaudio_detached_iframe_context_m3xxx`；make test 18865/0；
testharness-media 598P/0F/24PF 零回归（part01 removeChild 挂钩共享面实证）。
此前 2026-09-04：**WPT webaudio 第十四批导入——the-audiocontext-interface
余件试导（31 用例 833P/0F）**：audiocontextoptions（latencyHint 基本档/数值档/
sampleRate 面——**AudioContext double latencyHint clamp 语义落地**：baseLatency =
hint clamp 到 [0.005, 0.4]，spec「设备支持范围内尽量接近 hint」headless 以
Chromium headless 观测近似——大 hint（×10/×20）clamp 0.4 上限使两 high-latency
上下文相等，latencyHint-double 断言面）/ constructor-allowed-to-start（第十五批
勘误移除——误入清单致 runner Unsupported exit 1）/ suspend-after-construct）。
此前 2026-09-03：**WPT webaudio 第十三批导入——headless 零渲染面饱和
（28 用例 787P/0F = 100%）**：audiocontext-getoutputtimestamp 导入（shim 补
`AudioContext.getOutputTimestamp`——AudioTimestamp {contextTime, performanceTime}
形状 + 有限非负面）；the-audiocontext-interface 余件逐件核实全排除（用户手势/
onstatechange 时序/iframe 跨源/真设备/startRendering）。**余下上游用例全部依赖
渲染量化/设备/手势面——接口语义族 headless 可导入面吃尽**，随渲染切片复评。
evidence：`evidence/2026-09-03-webaudio-getoutputtimestamp-batch13.json`；单测扩
断言组 9；make test 18826/0。此前同日：**第十二批导入——ctor-audiobuffersource
（27 用例 777P/0F = 100%）**：排除注记勘误——此前「末 task multiple contexts 依赖
startRendering」实为 ctor-audiobuffer.html 的排除理由（两文件混淆），
ctor-audiobuffersource 全 task 零渲染，实测核对后解除；shim 补 AudioBufferSourceNode
loopStart/loopEnd 反射（缺省 0）+ ctor options 扩 loopStart/loopEnd/playbackRate/
detune 四字段。**777P/0F**（+44 净涨零回归），evidence：
`evidence/2026-09-03-webaudio-ctor-buffersource-batch12.json`；单测扩断言组 8；
make test 18826/0（一次 transient flake 复跑全绿）。此前同日：**第十一批导入——
源节点语义面（26 用例 733P/0F = 100%）**：shim 落 AudioScheduledSourceNode 调度异常共享件
`_zwWAInstallSchedSource`（ConstantSource/AudioBufferSource 装载——非 finite
TypeError/负 RangeError/先 stop 或重复 start InvalidStateError/start 多参负
RangeError）+ AudioParam min/max float 界 fround（-3.4028234663852886e38 断言面）
+ StereoPanner 工厂路径补 `_zwWAInstallChannel12Setter`（channelCount [1,2] setter
+ mode 'max' NotSupportedError——扩 helper 双面）+ runner WEBAUDIO_SUPPORT_SCRIPTS
增 start-stop-exceptions.js。constant-source-basic / stereopannernode-basic /
audiobuffersource-basic 三件导入。**733P/0F**（+37 净涨零回归），evidence：
`evidence/2026-09-03-webaudio-source-semantics-batch11.json`；单测扩断言组 7；
make test 18826/0。此前同日：**第十批导入——零渲染候选复评（23 用例
696P/0F = 100%）**：biquadfilternode-basic 直接导入（type 八枚举 setter 面已由
ctor-biquadfilter 落 shim——零新增缺口）；ctor-offlineaudiocontext 导入前落 shim
OfflineAudioContext 构造器扩 OfflineAudioContextOptions dict（3-arg legacy +
1-arg dict 双形态、length/sampleRate required、numberOfChannels [1,32]/length ≥ 1/
sampleRate [8000,96000] 正义约束 NotSupportedError、destination 通道面
channelCount=numberOfChannels + mode 'explicit'）。audiocontext-suspend-resume
维持排除（task 3 硬依赖 startRendering）。**696P/0F**（+73 净涨零回归），evidence：
`evidence/2026-09-03-webaudio-zero-gap-review-batch10.json`；单测扩断言组 6；
make test 18826/0。此前同日：**第九批导入——处理类节点 ctor 族第二批
（21 用例 623P/0F = 100%）**：shim part06 落地 WaveShaper（curve set 拷贝存储 +
null 清空 + oversample enum）/ DynamicsCompressor（五 AudioParam 缺省 +
reduction number + channelCount [1,2] 界 ctor/setter 双面）/ Panner（六 AudioParam
float 舍入 + 距离模型/cone 属性 + RangeError/InvalidStateError 校验双面）+
AudioListener（ctx.listener 惰性单例，九 param 缺省 forwardZ=-1/upY=1，
Audio/Offline 双 context）+ IIRFilter（required/[1,20] 界/fb[0]≠0/非 finite 校验 +
getFrequencyResponse 异常面 + ctor 经 `_zwWANodeCtor` 统一 AudioNodeOptions 应用）。
ctor-waveshaper / ctor-dynamicscompressor / dynamicscompressor-basic / ctor-panner /
iirfilter-basic 五件导入（ctor-iirfilter 维持排除——Functional task 依赖
startRendering）。**623P/0F**（+239 净涨零回归），evidence：
`evidence/2026-09-03-webaudio-processing-family2-batch9.json`；单测
`test_webaudio_processing_ctor_family2_m3xxvi`（五断言组）；make test 18824/0。
上一轮 429 中断遗留修复：AudioContext.listener defineProperty 头行丢失（shim
语法损坏，拼接后 node --check 复现）补回。此前同日：**WPT webaudio 第八批导入——AudioNode 接口基本面
（18 用例 384P/0F = 100%）**：shim 落地跨 context connect/disconnect 校验
（节点/AudioParam 目标 `_zwCtx` 身份 → InvalidAccessError）+ connect output/
input 索引越界 IndexSizeError + AudioBufferSourceNode 接口最小面（0入1出 +
buffer/loop/playbackRate/detune 反射）+ `_zwWANode.prototype` 链接 EventTarget
（instanceof 断言面）+ AudioContext 3-arg legacy 拒收；AudioParam 工厂增 ctxId
身份（12 处 builder 调用点透传，createOscillator/createGain 作用域修正）。
audionode / different-contexts 导入。**384P/0F**（+6 净涨零回归），evidence：
`evidence/2026-09-03-webaudio-audionode-batch8.json`。此前同日：**第七批
导入——ChannelMerger/Splitter/ConstantSource ctor 族 + AudioBuffer getChannelData
same-object（16 用例 378P/0F = 100%）**：shim part06 落地 ChannelSplitterNode/ChannelMergerNode
（固定拓扑 1入N出/N入1出、channelCount 派生、mode 'explicit'、固定通道 setter
面——赋现值 no-op/它值 InvalidStateError、ctor options 固定值可过/非固定值抛）+
ConstantSourceNode（offset AudioParam 缺省 1 + 工厂 options 透传）+
`ctx.createBuffer` + `AudioBuffer.copyToChannel/copyFromChannel` 数据面 +
`_zwWANodeCtor` 分发三节点；runner WEBAUDIO_SUPPORT_SCRIPTS 增
audioparam-testing.js。**378P/0F = 100%**（+47 subtest 净涨零回归），
evidence：`evidence/2026-09-03-webaudio-node-family-batch7.json`；单测
`test_webaudio_merger_splitter_constant_source_m3xxiv`（固定拓扑/固定面/
ConstantSource/copy 数据面 4 断言组）；make test 18815/0。此前 2026-09-02：
**WPT webaudio 第四~六批导入——ctor 族 + AudioBuffer +
gain-basic（12 用例 331P/0F = 100%，接口语义族 headless 饱和）**。第四批：StereoPanner/Delay/BiquadFilter/Analyser 四节点构造器 +
createPeriodicWave 异常面 + audioparam-exceptional-values 全落 shim part06
（builder 族防工厂↔构造器互调递归），10 用例 **323P/0F = 100%**（+259 subtest
净涨零回归），evidence：`evidence/2026-09-02-webaudio-ctor-family.json`。此前
同日：**Web Audio 最小面切片 1+2 落地**——zero-media `webaudio`
模块（振荡器四型波形合成 + WebAudioContext 图推进 + 过零率锚点单测 7 件）+
shim `AudioContext` 门面（createOscillator/createGain/destination/start/stop +
connect 链式 + AudioParam 最小值面）+ webview `WebAudioRegistry` + `__zwWA*`
宿主桥 + tab_worker/webview 泵接线 + e2e（JS 全链 → NullSink 帧数/过零率可观测，
`webaudio_bridge_nullsink_observable_chain`）。此前 2026-09-01：**D1 获批（D-WA-1
批准 + D-WA-2 选先 NullSink）**——
Web Audio AudioContext 最小面实施开工（切片 1+2，NullSink 设备面挂真出声切片），
RFC 见 [../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md](../../specs/web-audio-audiocontext-minimal-face-spec-rfc.md)。
此前：D2 获批项闭环——libasound2-dev 在位，cpal 编译 + 39 测全绿 +
**CpalSink 真设备流冒烟通过**（Ok 分支：构造/start/write/pause/resume 全链——
[evidence](evidence/2026-09-01-cpalsink-device-smoke.md)）；M2 A/V 同步主体
由 media-playback 流切片 D+E 兑现；opus 解码面转正。本档余：Mixer 接线（挂真出声
切片，可选））


## 当前状态块的批次明细（第 4~13 批，原文）

- **WPT webaudio 第四批导入（同日，处理类节点 ctor 族 + AudioParam 异常面）**：
  - **shim part06 扩展**：① 处理类节点 builder 族
    `_zwWABuildStereoPanner/_zwWABuildDelay/_zwWABuildBiquadFilter/_zwWABuildAnalyser`
    ——工厂 `createX()` 与构造器 `new X(ctx, options)` 共用 builder（**构造器经
    `_zwWANodeCtor` 分发 builder、工厂直调 builder，两向终结于 builder 防互调
    递归**——首版工厂调 `new X()` 触发 Maximum call stack 教训）；per-kind 专属
    选项在 builder 内应用（StereoPanner mode 缺省 'clamped-max' + channelCount
    [1,2] 界 / Delay maxDelayTime 缺省 1.0 + maxValue 反射 + delayTime clamp /
    BiquadFilter type 八枚举 + Q/detune/frequency/gain 缺省 1/0/350/0 / Analyser
    fftSize 幂界 [32,32768] + frequencyBinCount 反射 + min/maxDecibels 交叉校验
    （ctor 选项路径延后统一校验 `_armed` 门——中间态不抛）+ smoothingTimeConstant
    [0,1]）。`_zwWANode` 处理类节点 numberOfInputs 1（oscillator 保持 0）。
    ② AudioParam 调度方法异常面：value/time 非 finite → TypeError、负时间/
    timeConstant ≤ 0/duration ≤ 0 → RangeError、exponentialRamp |v| ≤ 1e-100 →
    RangeError、setValueCurve 曲线项非 finite → TypeError。
    ③ createPeriodicWave 非 finite → TypeError + real/imag 等长 IndexSizeError。
  - **导入 7 用例**：ctor-gain（W3CTH 4 subtest）/ ctor-stereopanner（audit 51）/
    ctor-delay（audit 53）/ ctor-biquadfilter（W3CTH 5）/ ctor-analyser（audit 78）/
    createPeriodicWaveInfiniteValuesThrows（2）/ audioparam-exceptional-values
    （audit 66）。**10 用例 323P/0F = 100%**（+259 净涨零回归——ctor-oscillator
    62P 口径核对一致）。
  - 单测 `test_webaudio_node_ctor_and_param_exception_face_m3w4`（五断言组：
    缺省面/选项反射/ctor 异常六态/AudioParam 调度异常七态/createPeriodicWave+
    工厂同对象）。
  - evidence：`evidence/2026-09-02-webaudio-ctor-family.json`。
  - **排除注记**：audioparam-method-chaining / audioparam-nominal-range（依赖
    startRendering 渲染断言——RFC §0 不做清单）。
- **WPT webaudio 第九批导入（2026-09-03，处理类节点 ctor 族第二批）**：
  - **shim part06 扩展**：`_zwWABuildWaveShaper/_zwWABuildDynamicsCompressor/
    _zwWABuildPanner/_zwWABuildIIRFilter` 四 builder + 共享件
    `_zwWACtorChannel12`（[1,2] ctor 界）+ `_zwWAInstallChannel12Setter`
    （[1,2] setter 面——越界 NotSupportedError 保留旧值）；AudioListener 惰性单例
    （Audio/Offline 双 context defineProperty）。IIRFilter ctor 经 `_zwWANodeCtor`
    分发——AudioNodeOptions dict 基类界统一应用（{channelCount:17} 反射）。
  - **导入 5 用例**：ctor-waveshaper（audit 54）/ ctor-dynamicscompressor（4）/
    dynamicscompressor-basic（13）/ ctor-panner（125）/ iirfilter-basic（43）。
    **21 用例 623P/0F = 100%**（+239 净涨零回归——384→623）。
  - 单测 `test_webaudio_processing_ctor_family2_m3xxvi`（五断言组：WaveShaper
    curve 拷贝+enum / DC 参数面+[1,2] 界双面 / Panner 13 属性+校验四面+
    float 舍入 / listener 惰性单例+双 context / IIRFilter 八异常面+工厂）。
  - evidence：`evidence/2026-09-03-webaudio-processing-family2-batch9.json`。
  - **排除注记**：ctor-iirfilter（Functional task 依赖 startRendering 渲染对比
    ——RFC §0 不做清单；语义面 AudioNodeOptions 已落 shim，随渲染切片复评）。
  - **429 中断遗留修复**：上一轮写入的 AudioContext.listener defineProperty
    头行丢失（shim 语法损坏）——本轮拼接全部分片 `node --check` 复现后补回。
- **WPT webaudio 第十批导入（2026-09-03，零渲染候选复评）**：
  - **零新增缺口直接导入**：biquadfilternode-basic（audit 29——type 八枚举 setter
    面已由 ctor-biquadfilter 落 shim，断言 99 不生效）。
  - **缺口补落后导入**：ctor-offlineaudiocontext（audit 44）——暴露 shim 真缺口
    （OfflineAudioContext 仅 3-arg 形态），落地 OfflineAudioContextOptions dict
    构造（双形态/required/正义约束三面/destination 通道面）后导入全绿。
  - **维持排除**：audiocontext-suspend-resume（task 3 硬依赖 startRendering）。
  - **23 用例 696P/0F = 100%**（+73 净涨零回归——623→696）。
  - 单测 `test_webaudio_processing_ctor_family2_m3xxvi` 扩断言组 6
    （OfflineAudioContext 构造面九态）。
  - evidence：`evidence/2026-09-03-webaudio-zero-gap-review-batch10.json`。
- **WPT webaudio 第十一批导入（2026-09-03，源节点语义面）**：
  - **shim 扩展**：`_zwWAInstallSchedSource` 调度异常共享件（start/stop 非 finite
    TypeError/负 RangeError/先 stop 或重复 start InvalidStateError/start 多参负
    RangeError）装载 ConstantSource + AudioBufferSource；AudioParam min/max
    float 界 fround；StereoPanner 工厂路径补 [1,2] setter + mode 'max' 拒绝
    （`_zwWAInstallChannel12Setter` 扩 channelCountMode setter 双面）。
  - **runner**：WEBAUDIO_SUPPORT_SCRIPTS 增 start-stop-exceptions.js。
  - **导入 3 用例**：constant-source-basic / stereopannernode-basic /
    audiobuffersource-basic。**26 用例 733P/0F = 100%**（+37 净涨零回归）。
  - 单测扩断言组 7（调度异常八态 + float 界 + StereoPanner 工厂面）。
  - evidence：`evidence/2026-09-03-webaudio-source-semantics-batch11.json`。
- **WPT webaudio 第十二批导入（2026-09-03，ctor-audiobuffersource）**：
  - **排除注记勘误**：此前批五注记「ctor-audiobuffer（末 task multiple contexts
    依赖 startRendering）」与 ctor-**audio buffersource**.html 混淆——后者全 task
    零渲染（initializeContext 仅构造 OfflineAudioContext），实测核对后解除排除。
  - **shim 补缺**：AudioBufferSourceNode loopStart/loopEnd 反射（缺省 0）+
    ctor options 扩 loopStart/loopEnd/playbackRate/detune。
  - **27 用例 777P/0F = 100%**（+44 净涨零回归——733→777）。
  - 单测扩断言组 8（ctor options 全字段反射等价工厂路径）。
  - evidence：`evidence/2026-09-03-webaudio-ctor-buffersource-batch12.json`。
- **WPT webaudio 第十三批导入（2026-09-03，headless 零渲染面饱和）**：
  - **shim 补缺**：`AudioContext.getOutputTimestamp`（AudioTimestamp
    {contextTime, performanceTime} 形状 + 有限非负面——headless 无音频钟，
    真值挂 audio clock 主时钟切片）。
  - **导入 1 用例**：audiocontext-getoutputtimestamp（audit 10 subtest）。
    **28 用例 787P/0F = 100%**（+10 净涨零回归——777→787）。
  - **饱和结论**：the-audiocontext-interface 余件逐件核实全排除
    （constructor-allowed-to-start 用户手势/suspend-after-construct onstatechange
    计时/not-fully-active iframe 跨源/sinkid 真设备域/startRendering）；
    nan-param、setValueCurve-exceptions 等 Audioparam 余件断言在渲染管线内
    （RFC §0）。**接口语义族 headless 可导入面吃尽**——余下用例随渲染切片或
    设备切片复评。
  - 单测扩断言组 9（AudioTimestamp 形状）。
  - evidence：`evidence/2026-09-03-webaudio-getoutputtimestamp-batch13.json`。

## 当前状态块的批次明细（第五/六/七批 + 排除注记 + 第七批更新，原文）

- **WPT webaudio 第五批导入（同日，AudioBuffer 构造/接口面）**：
  - **shim part06 扩展**：`AudioBuffer` 构造器（独立构造不依赖 ctx）——
    length/sampleRate required 缺失 → TypeError（WebIDL dict required）；
    numberOfChannels [1,32] / length ≥ 1 / sampleRate [8000,96000] 正义约束 →
    NotSupportedError；`duration = length/sampleRate` 反射；`getChannelData(i)`
    返回 Float32Array（零填充通道存储面）+ 越界 → IndexSizeError。
    audiobuffer.html（W3CTH）1P 全绿——11 用例 324P/0F。
- **WPT webaudio 第六批导入（同日，gain-basic 单件）**：`gain-basic.html`（audit 单
  task——`gainNode.gain instanceof AudioParam` 断言，无渲染）7 subtest 全绿——
  **12 用例 331P/0F = 100%**；evidence：`evidence/2026-09-02-webaudio-gain-basic.json`。
  **排除注记**：no-dezippering 四件（gain/stereopanner/delay/biquadfilter）+
  gain.html（全部 startRendering 渲染断言——RFC §0）。
    单测 m3w4 扩断言组 6（AudioBuffer 必选项/正义约束/反射/getChannelData 面）。
  - evidence：`evidence/2026-09-02-webaudio-audiobuffer.json`。
  - **排除注记**：ctor-audiobuffer.html（末 task multiple contexts 依赖
    startRendering——audit runner 整文件跑，前段构造面无法单独导入）；
    audiobuffer-getChannelData / audiobuffer-copy-channel（copyToChannel/
    copyFromChannel 数据面随播放切片）；periodicWave.html（startRendering）。
  **第七批更新（2026-09-03）**：audiobuffer-getChannelData 解除排除导入
 （same-object 断言面无渲染——copyTo/copyFrom 数据面语义同步落 shim）；
  audiobuffer-copy-channel 维持排除（同文件 startRendering 后段不可分割——
