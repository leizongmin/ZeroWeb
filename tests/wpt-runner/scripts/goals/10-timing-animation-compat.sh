#!/usr/bin/env bash
# Goal: timing-animation-compat（编号 10——推进顺序见同目录 README.md）
# 轻量热身切片（纯 JS API 面）
# M1 资产预置：fetch hr-time performance-timeline user-timing web-animations 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/timing-animation-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="timing-animation-compat"
# 顶层四 corpus + web-animations 深面子目录（M1 rally 轮追加，2026-09-27）：
# - interfaces/{8 子目录} = WAAPI API 面（Animation/KeyframeEffect/Animatable/…）
# - timing-model/{4 子目录} = 时序模型语义（currentTime/phases/playbackRate/…）
# lib.sh fetch_dir_html 只递归一层（防 API 限流），三级目录照 fetch-dom-subset.sh
# SUBDIRS 先例显式追加；runner 侧 testharness.rs SUBDIRS 常量与此清单同域。
DIRS=( "hr-time" "performance-timeline" "user-timing" "web-animations"
  "web-animations/interfaces/Animatable"
  "web-animations/interfaces/Animation"
  "web-animations/interfaces/AnimationEffect"
  "web-animations/interfaces/AnimationPlaybackEvent"
  "web-animations/interfaces/Document"
  "web-animations/interfaces/DocumentTimeline"
  "web-animations/interfaces/KeyframeEffect"
  "web-animations/interfaces/TimelineTrigger"
  "web-animations/timing-model/animation-effects"
  "web-animations/timing-model/animations"
  "web-animations/timing-model/time-transformations"
  "web-animations/timing-model/timelines"
)

goals_fetch_all

# M1 rally 轮扩展：三计时 corpus 的 .any.js 用例（hr-time 2 + performance-timeline 20 +
# user-timing 20 = 42 案）。lib.sh fetch_dir_html 排除 .any.js（注释：wrapper harness
# 形态「runner 形态支持由各 goal M1 rally 轮评估」——本 goal runner 通道经
# any_js_window_wrapper 以 window 变体执行，wasm/fs/indexeddb 先例，故显式清单补拉）。
# 排除：idlharness.any.js ×3——依赖 /resources/WebIDLParser.js，该文件是上游
# resources/webidl2 的 build 期生成资产（repo 内不存在，wpt serve 时构建），repo
# raw 拉不到（实测 404），window 可执行面之外（clipboard idlharness 先例）。
ANY_JS=(
  "hr-time/basic.any.js"
  "hr-time/monotonic-clock.any.js"
  "performance-timeline/buffered-flag-after-timeout.any.js"
  "performance-timeline/buffered-flag-observer.any.js"
  "performance-timeline/buffered-flag-with-entryTypes-observer.tentative.any.js"
  "performance-timeline/case-sensitivity.any.js"
  "performance-timeline/droppedentriescount.any.js"
  "performance-timeline/multiple-buffered-flag-observers.any.js"
  "performance-timeline/observer-buffered-false.any.js"
  "performance-timeline/performanceentry-tojson.any.js"
  "performance-timeline/po-callback-mutate.any.js"
  "performance-timeline/po-disconnect-removes-observed-types.any.js"
  "performance-timeline/po-disconnect.any.js"
  "performance-timeline/po-entries-sort.any.js"
  "performance-timeline/po-getentries.any.js"
  "performance-timeline/po-mark-measure.any.js"
  "performance-timeline/po-observe-repeated-type.any.js"
  "performance-timeline/po-observe-type.any.js"
  "performance-timeline/po-observe.any.js"
  "performance-timeline/po-takeRecords.any.js"
  "performance-timeline/supportedEntryTypes.any.js"
  "performance-timeline/webtiming-resolution.any.js"
  "user-timing/buffered-flag.any.js"
  "user-timing/case-sensitivity.any.js"
  "user-timing/clear_all_marks.any.js"
  "user-timing/clear_all_measures.any.js"
  "user-timing/clear_non_existent_mark.any.js"
  "user-timing/clear_non_existent_measure.any.js"
  "user-timing/clear_one_mark.any.js"
  "user-timing/clear_one_measure.any.js"
  "user-timing/entry_type.any.js"
  "user-timing/mark-entry-constructor.any.js"
  "user-timing/mark-errors.any.js"
  "user-timing/mark-l3.any.js"
  "user-timing/mark-measure-return-objects.any.js"
  "user-timing/mark.any.js"
  "user-timing/measure-l3.any.js"
  "user-timing/measure-with-dict.any.js"
  "user-timing/measure_syntax_err.any.js"
  "user-timing/structured-serialize-detail.any.js"
  "user-timing/supported-usertiming-types.any.js"
  "user-timing/user_timing_exists.any.js"
  "common/performance-timeline-utils.js"
)
echo "== fetch .any.js window 变体 + /resources/ 依赖（${#ANY_JS[@]} 文件）=="
for f in "${ANY_JS[@]}"; do
  fetch_raw "${f}" || echo "  fetch failed: ${f}（重跑脚本幂等续拉）" >&2
done

goals_inventory
goals_next_steps "${GOAL}"
