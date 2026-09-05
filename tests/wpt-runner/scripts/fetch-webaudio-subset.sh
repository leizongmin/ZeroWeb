#!/usr/bin/env bash
# Fetch the pinned subset of upstream WPT webaudio the-audio-api interface tests
# used by the media-audio goal M3 Web Audio minimal face
# (docs/goal/archive/media-audio/master.md, D1 批复切片 2; goal 完成归档 2026-09-05).
#
# Strategy: constructor/interface-semantic cases only — the shim's AudioContext
# face (shim part06) covers construction + node interface + connect semantics.
# Rendering-dependent cases (startRendering / worklet / OfflineAudioContext
# rendering) stay out — RFC §0 exclusion list.
#
# 与 fetch-media-subset.sh 同构（同 WPT_REV pin + raw 拉单文件）；wpt-data
# gitignored，用例按需 fetch、不入库。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"
RAW_ROOT="https://raw.githubusercontent.com/web-platform-tests/wpt/${WPT_REV}"

fetch_raw() {
  local relative="$1"
  local target="${WPT_DATA}/${relative}"
  if [[ -s "${target}" && "${FORCE:-0}" != "1" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "${target}")"
  local temporary="${target}.tmp"
  curl --fail --location --silent --show-error --retry 5 --retry-delay 3 \
    --connect-timeout 8 --max-time 30 \
    "${RAW_ROOT}/${relative}" -o "${temporary}"
  test -e "${temporary}"
  mv "${temporary}" "${target}"
}

WA_FILES=(
  "webaudio/the-audio-api/the-audionode-interface/audionode-connect-return-value.html"
  "webaudio/the-audio-api/the-destinationnode-interface/destination.html"
  # OscillatorNode 构造器面（audit.js 框架——runner 内联 webaudio/resources/*.js）。
  "webaudio/the-audio-api/the-oscillatornode-interface/ctor-oscillator.html"
  # ---- 第四批（2026-09-02）：处理类节点 ctor 族 + AudioParam 异常面 ----
  "webaudio/the-audio-api/the-gainnode-interface/ctor-gain.html"
  "webaudio/the-audio-api/the-stereopanner-interface/ctor-stereopanner.html"
  "webaudio/the-audio-api/the-delaynode-interface/ctor-delay.html"
  "webaudio/the-audio-api/the-biquadfilternode-interface/ctor-biquadfilter.html"
  "webaudio/the-audio-api/the-analysernode-interface/ctor-analyser.html"
  "webaudio/the-audio-api/the-periodicwave-interface/createPeriodicWaveInfiniteValuesThrows.html"
  "webaudio/the-audio-api/the-audioparam-interface/audioparam-exceptional-values.html"
  # ---- 第五批（2026-09-02）：AudioBuffer 构造/接口面 ----
  "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer.html"
  "webaudio/the-audio-api/the-gainnode-interface/gain-basic.html"
  # ---- 第七批（2026-09-03）：ChannelMerger/Splitter/ConstantSource ctor +
  # AudioBuffer getChannelData same-object 面（无渲染——数据面）。
  "webaudio/the-audio-api/the-channelmergernode-interface/ctor-channelmerger.html"
  "webaudio/the-audio-api/the-channelsplitternode-interface/ctor-channelsplitter.html"
  "webaudio/the-audio-api/the-constantsourcenode-interface/ctor-constantsource.html"
  "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer-getChannelData.html"
  # ---- 第八批（2026-09-03）：AudioNode 接口基本面（跨 context InvalidAccessError
  # + connect 索引越界 IndexSizeError + AudioBufferSourceNode 接口反射）。
  "webaudio/the-audio-api/the-audionode-interface/audionode.html"
  "webaudio/the-audio-api/the-audionode-interface/different-contexts.html"
  # ---- 第九批（2026-09-03）：处理类节点 ctor 第二批（WaveShaper/DynamicsCompressor
  # /Panner/IIRFilter——全部无渲染语义面）。
  "webaudio/the-audio-api/the-waveshapernode-interface/ctor-waveshaper.html"
  "webaudio/the-audio-api/the-dynamicscompressornode-interface/ctor-dynamicscompressor.html"
  "webaudio/the-audio-api/the-dynamicscompressornode-interface/dynamicscompressor-basic.html"
  "webaudio/the-audio-api/the-pannernode-interface/ctor-panner.html"
  "webaudio/the-audio-api/the-iirfilternode-interface/iirfilter-basic.html"
  # ---- 第十批（2026-09-03）：零新增缺口复评导入——biquadfilternode-basic
  #（type 八枚举 setter 面已由 ctor-biquadfilter 落 shim，断言 99 不生效）+
  # ctor-offlineaudiocontext（dict 构造/required/正义约束/destination 通道面——
  # shim OfflineAudioContext 构造器扩 OfflineAudioContextOptions 后导入）。
  "webaudio/the-audio-api/the-biquadfilternode-interface/biquadfilternode-basic.html"
  "webaudio/the-audio-api/the-offlineaudiocontext-interface/ctor-offlineaudiocontext.html"
  # ---- 第十一批（2026-09-03）：源节点语义面——constant-source-basic（offset
  # min/max float 界 + start/stop 调度异常 W3CTH）/ stereopannernode-basic
  #（pan AudioParam + channelCount [1,2] setter 面）/ audiobuffersource-basic
  #（start/stop 异常 audit 面）。配套 shim：AudioScheduledSourceNode 调度异常
  # 共享面（非 finite TypeError/负 RangeError/先 stop 或重复 start
  # InvalidStateError）。
  "webaudio/the-audio-api/the-constantsourcenode-interface/constant-source-basic.html"
  "webaudio/the-audio-api/the-stereopanner-interface/stereopannernode-basic.html"
  "webaudio/the-audio-api/the-audiobuffersourcenode-interface/audiobuffersource-basic.html"
  # ---- 第十二批（2026-09-03）：ctor-audiobuffersource（全 task 零渲染——
  # initializeContext 仅构造 OfflineAudioContext；ctor options 面 buffer/detune/
  # loop/loopEnd/loopStart/playbackRate 反射，shim 补 loopStart/loopEnd 后导入）。
  # 此前「末 task multiple contexts 依赖 startRendering」排除注记经实测核对失效
  #（该注记实为 ctor-audiobuffer.html 的排除理由，两文件混淆）。
  "webaudio/the-audio-api/the-audiobuffersourcenode-interface/ctor-audiobuffersource.html"
  # ---- 第十三批（2026-09-03）：audiocontext-getoutputtimestamp（AudioTimestamp
  # 初始值面——shim 补 getOutputTimestamp 后导入）。不导入：constructor-allowed-to-start
  #（test_driver.bless 用户手势 + onstatechange 时序）、suspend-after-construct
  #（onstatechange 事件计数）、audiocontext-not-fully-active（iframe 跨源 helper
  # ——超出 webaudio runner 能力）；其余 audiocontext-* 依赖真设备 sinkid/渲染。
  "webaudio/the-audio-api/the-audiocontext-interface/audiocontext-getoutputtimestamp.html"
  # ---- 第十四批（2026-09-04）：the-audiocontext-interface 余件试导。不导入
  # constructor-allowed-to-start（test_driver.bless + 「构造后 suspended → 异步
  # running」断言——shim headless 恒 running 结构性互斥，误入清单曾使 runner
  # Unsupported exit 1，第十五批勘误移除）。
  "webaudio/the-audio-api/the-audiocontext-interface/audiocontextoptions.html"
  "webaudio/the-audio-api/the-audiocontext-interface/suspend-after-construct.html"
  # ---- 第十五批（2026-09-04）：promise-methods-after-discard（iframe realm
  # AudioContext + detached 后 suspend/resume/close reject InvalidStateError——
  # shim part05 IframeAudioContext 绑定构造器 + part06 detached reject 面 +
  # part01 _zwRemoveIframeWindowClient destroyed 印记解耦 SW client）。
  "webaudio/the-audio-api/the-audiocontext-interface/promise-methods-after-discard.html"
  # ---- 第十六批（2026-09-04）：convolver/analyser 零渲染候选——ctor-convolver
  #（构造/属性/ AudioNodeOptions/buffer 校验全 W3CTH 面）/ convolver-setBuffer-null
  # / convolver-setBuffer-already-has-value（buffer setter 重复赋值 + null 清空
  # audit 面）/ realtimeanalyser-basic（Analyser 1入1出 + min/maxDecibels/
  # smoothingTimeConstant 缺省与可写面）。配套 shim：ConvolverNode builder +
  # createConvolver 工厂（normalize/buffer/[1,2] 界 + sampleRate 校验）。
  "webaudio/the-audio-api/the-convolvernode-interface/ctor-convolver.html"
  "webaudio/the-audio-api/the-convolvernode-interface/convolver-setBuffer-null.html"
  "webaudio/the-audio-api/the-convolvernode-interface/convolver-setBuffer-already-has-value.html"
  "webaudio/the-audio-api/the-analysernode-interface/realtimeanalyser-basic.html"
  # ---- 第十七批（2026-09-04）：MediaStreamAudioDestinationNode 语义面——ctor 全
  # task 零渲染（TypeError 三态 + 1入0出 + channelCount 2 缺省/mode explicit/
  # interpretation speakers + options channelCount=7 非固定面）。配套 shim：
  # _zwWABuildMediaStreamDestination builder + 构造器/工厂（stream 反射最小面）。
  "webaudio/the-audio-api/the-mediastreamaudiodestinationnode-interface/ctor-mediastreamaudiodestination.html"
  # ---- 第十八批续（2026-09-04）：OfflineAudioContext detached execution context——
  # createElementNS(HTMLNS,'iframe') 形态 contentWindow gate 修复（part04 _nsHandles
  # localName 回落）后解除排除；decodeAudioData destroyed reject 面（part06）+
  # IframeOfflineAudioContext 绑定构造器（part05）。
  "webaudio/the-audio-api/the-offlineaudiocontext-interface/offlineaudiocontext-detached-execution-context.html"
  # ---- 第十九批（2026-09-05，media-audio D3 获批窄授权——offline 渲染路径）----
  # startRendering 最小面落地（shim 侧 JS 波形合成：四型振荡器 + custom periodic
  # wave spec 归一化 + 线性 gain 链解析）后解除排除导入。
  "webaudio/the-audio-api/the-oscillatornode-interface/osc-basic-waveform.html"
  # ---- 第二十批（D3 第三片——splitter/merger 通道路由图推进）：gain.html
  #（11 note 增益衰减渲染对比——通道 0/1 = gain 缩放、2/3 = 源直通，逐通道 SNR）。
  "webaudio/the-audio-api/the-gainnode-interface/gain.html"
  # ---- 第二十一批（D3 第三片续——AudioParam automation timeline）：增益包络
  # 调度（setValueAtTime/linearRamp/exponentialRamp/setTargetAtTime 事件表 +
  # startRendering 逐采样求值）。
  "webaudio/the-audio-api/the-audioparam-interface/audioparam-method-chaining.html"
  # ---- 第二十二批（D3 第五增量——AudioParam nominal range 界表）：每节点每 param
  # 的 spec min/max（_zwApplyParamLimits 反射 + value clamp + 只读）+ 原型参数发现
  #（_zwRegisterNodeParam + WeakMap 原型访问器）。
  "webaudio/the-audio-api/the-audioparam-interface/audioparam-nominal-range.html"
  # ---- 第二十三批（D3 第五增量续）：audiobuffer-copy-channel——copyFrom/copyTo
  # Channel 数据面（shim 第七批已落；原「startRendering 后段不可分割」注记失效——
  # 该文件无 startRendering 引用，纯 AudioBuffer 数据面）。
  "webaudio/the-audio-api/the-audiobuffer-interface/audiobuffer-copy-channel.html"
  # ---- 第二十四批（D3 第六增量续）：ctor-audiobuffer——AudioBuffer ctor 全族
  #（invalid ctor/required options/values/numberOfChannels/getChannelData 界）+
  # multiple contexts 渲染对比（双 OfflineAudioContext 共享 buffer，各 startRendering
  # 输出对拍）。原「末 task 依赖 startRendering」注记随图推进框架落地解除。
  "webaudio/the-audio-api/the-audiobuffer-interface/ctor-audiobuffer.html"
  # ---- 第二十五批（D3 第七增量续）：ctor-iirfilter——ctor 校验全族 + Functional
  # 渲染对比（ctor vs 工厂同系数双通道对拍——图推进 pass-through 等价面，两条
  # 链路同构即可精确相等，无 IIR DSP 需求）。原排除注记解除。
  "webaudio/the-audio-api/the-iirfilternode-interface/ctor-iirfilter.html"
  # ---- 第二十六批（D3 第九增量）：periodicWave.html（迁址路径 the-periodicwave-
  # interface/——Blink maxAbsSum 归一化公式修正 + createPeriodicWave 最小长度 2 +
  # OscillatorNode.setPeriodicWave）。原 404 排除注记解除。
  "webaudio/the-audio-api/the-periodicwave-interface/periodicWave.html"
  # ---- 第二十七批（D3 第十增量）：detune 耦合两件——振荡器 computed frequency
  #（frequency · 2^(detune/1200)）+ ≥Nyquist 精确静默 + detune automation（linearRamp
  # 逐采样耦合频率）。原「automation 精化切片」排除注记解除——静态耦合两 task + ramp
  # 耦合一 task 均在 D3 已批 offline 渲染路径内。
  "webaudio/the-audio-api/the-oscillatornode-interface/detune-limiting.html"
  "webaudio/the-audio-api/the-oscillatornode-interface/detune-overflow.html"
  # ---- 第二十九批（D3 第十二增量）：sub-sample-start——亚帧起点 ceil 语义。
  "webaudio/the-audio-api/the-oscillatornode-interface/sub-sample-start.html"
  # audioparam 四型 ramp 对拍（setValueAtTime/linearRamp/exponentialRamp/
  # setTargetAtTime）维持排除——上游坏 helper：audioparam-testing.js
  # verifyDiscontinuities 引用 createAudioGraphAndTest 形参 numberOfTests（非模块
  # 作用域）→ oncomplete 回调必抛 ReferenceError，上游 Chromium 亦红（master 同字节）。
  # shim OfflineAudioContext.oncomplete 已作为资产落地，待上游修复后复评。
  # 排除收束（更新）：pinned rev webaudio/the-audio-api 全部可执行用例均已导入或
  # 定性（剩余为 worklet/媒体互连/手势域 + 上游坏 helper 四件）。

)

# audit.js 框架（runner inline_extras 内联——用例以绝对路径引用）。
for f in audit.js audit-util.js audionodeoptions.js audioparam-testing.js; do
  fetch_raw "webaudio/resources/${f}"
done

for relative in "${WA_FILES[@]}"; do
  fetch_raw "${relative}"
done

echo "Web Audio testharness subset ready (${#WA_FILES[@]} files + audit framework, WPT ${WPT_REV})"
