#!/usr/bin/env bash
# Linux 本地模拟器/真机安装冒烟——忠实移植自 scripts/android/install-smoke.ps1
# （M0 Windows 路径）。流程：装 APK → 启动 → 断言四进程拓扑与 UID 隔离 →
# 断言 socket probe 日志。失败一律退出 2（与其他门禁一致）。
#
# 用法: install-smoke.sh <apk-path> [--require-renderer]
#   --require-renderer  额外断言 renderer socket 连接与页面帧就绪（renderer APK 用）
set -euo pipefail

ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
ADB="$ANDROID_HOME/platform-tools/adb"
apk=""
require_renderer=0
for arg in "$@"; do
  case "$arg" in
    --require-renderer) require_renderer=1 ;;
    *) apk="$arg" ;;
  esac
done

fail() { echo "FAIL: $1" >&2; exit 2; }
[[ -n "$apk" ]] || { echo "usage: $0 <apk-path> [--require-renderer]" >&2; exit 2; }
[[ -x "$ADB" ]] || fail "adb not found under ANDROID_HOME: $ANDROID_HOME"
[[ -f "$apk" ]] || fail "APK does not exist: $apk"

if [[ "$require_renderer" == 1 ]] && ! tar -tf "$apk" | grep -q "lib/[^/]*/libc\+\+_shared\.so"; then
  fail "Renderer-enabled APK must package libc++_shared.so"
fi

"$ADB" wait-for-device
"$ADB" logcat -c
"$ADB" install -r "$apk"
"$ADB" shell am start -W -n com.leizm.zeroweb/.MainActivity
sleep 2

processes=$("$ADB" shell ps -A -o USER,NAME | grep "com\.leizm\.zeroweb" || true)
browser_line=$(grep -E " com\.leizm\.zeroweb$" <<<"$processes" | head -1 || true)
renderer_line=$(grep -E "RendererService0$" <<<"$processes" | head -1 || true)
compositor_line=$(grep -E "com\.leizm\.zeroweb:compositor$" <<<"$processes" | head -1 || true)
decoder_line=$(grep -E "ImageDecoderService$" <<<"$processes" | head -1 || true)

if [[ -z "$browser_line" || -z "$renderer_line" || -z "$compositor_line" || -z "$decoder_line" ]]; then
  fail "Expected browser, renderer, compositor, and image-decoder process roles"
fi

# FR-008 进程隔离：renderer/decoder 走 isolated UID，compositor 与 browser 同 UID
browser_uid=${browser_line%% *}
renderer_uid=${renderer_line%% *}
compositor_uid=${compositor_line%% *}
decoder_uid=${decoder_line%% *}
if [[ "$renderer_uid" == "$browser_uid" || "$decoder_uid" == "$browser_uid" || "$compositor_uid" != "$browser_uid" ]]; then
  fail "Android process UID isolation does not match the required browser/renderer/compositor/decoder topology"
fi

echo "$processes"
probes=$("$ADB" logcat -d -t 500)
for probe in "decoder probe succeeded" "compositor bridge ready"; do
  grep -q "$probe" <<<"$probes" || fail "Android socket probe did not report success: $probe"
done
if [[ "$require_renderer" == 1 ]]; then
  grep -q "renderer socket connected" <<<"$probes" || fail "Renderer-enabled APK did not connect its native renderer socket"
  grep -q "renderer page frame ready" <<<"$probes" || fail "Renderer-enabled APK did not produce a page frame"
fi

echo "install-smoke PASS"
