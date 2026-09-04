#!/usr/bin/env bash
# Fetch the pinned subset of upstream WPT webaudio the-audio-api interface tests
# used by the media-audio goal M3 Web Audio minimal face
# (docs/goal/media-audio/master.md, D1 批复切片 2).
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
  # 不导入：audioparam-method-chaining / audioparam-nominal-range（startRendering
  # 渲染断言——RFC §0 不做清单）；ctor-audiobuffer（末 task multiple contexts 依赖
  # startRendering——audit runner 整文件跑）；periodicWave.html（startRendering）；
  # ctor-iirfilter（Functional task 依赖 startRendering 渲染对比——语义面
  # AudioNodeOptions 已落 shim，随渲染切片复评）。
)

# audit.js 框架（runner inline_extras 内联——用例以绝对路径引用）。
for f in audit.js audit-util.js audionodeoptions.js audioparam-testing.js; do
  fetch_raw "webaudio/resources/${f}"
done

for relative in "${WA_FILES[@]}"; do
  fetch_raw "${relative}"
done

echo "Web Audio testharness subset ready (${#WA_FILES[@]} files + audit framework, WPT ${WPT_REV})"
