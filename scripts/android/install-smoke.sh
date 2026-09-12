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
# 模拟器 logcat 噪声 ~1000 行/s（CI 实证 8529 行/8.2s），默认 ~256KB 主环仅存 ~2s——
# 扩环 + 按 tag 过滤读（-s），避免 probe 行被噪声冲出读取窗口（34707541575 实证误报）
"$ADB" logcat -G 4M
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
# probe 日志晚于 am start -W 返回（软件渲染冷启动首帧 4s+，probe 在服务连接后台线程），
# 固定窗口会误报——轮询等待，上限 60s；读法按 tag 过滤（见上方 -G/-s 注记）
probe_deadline=$((SECONDS + 60))
probes=""
while (( SECONDS < probe_deadline )); do
  probes=$("$ADB" logcat -d -s ZeroWebRole:*)
  if grep -q "decoder probe succeeded" <<<"$probes" && grep -q "compositor bridge ready" <<<"$probes"; then
    break
  fi
  sleep 2
done
for probe in "decoder probe succeeded" "compositor bridge ready"; do
  grep -q "$probe" <<<"$probes" || fail "Android socket probe did not report success: $probe"
done
if [[ "$require_renderer" == 1 ]]; then
  while (( SECONDS < probe_deadline )); do
    probes=$("$ADB" logcat -d -s ZeroWebRole:*)
    if grep -q "renderer socket connected" <<<"$probes" && grep -q "renderer page frame ready" <<<"$probes"; then
      break
    fi
    sleep 2
  done
  grep -q "renderer socket connected" <<<"$probes" || fail "Renderer-enabled APK did not connect its native renderer socket"
  grep -q "renderer page frame ready" <<<"$probes" || fail "Renderer-enabled APK did not produce a page frame"
fi

echo "install-smoke PASS"
