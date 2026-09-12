#!/usr/bin/env bash
# M5 chaos 冒烟——角色进程 kill → 断连恢复重绑（RFC M3 death recovery 语义执行验证）。
# 前置：应用已安装并处于运行态（先跑 install-smoke.sh）。kill 应用子进程需 root，
# google_apis 非 playstore 系统镜像 adb root 可用（脚本内自取）。
#
# 流程：逐角色 kill（compositor → decoder → renderer0），断言：
#   1) BIND_AUTO_CREATE 拉起新进程（同名、pid 变化）
#   2) compositor/decoder 重连协议重新执行（ZeroWebRole 成功日志再现；
#      每轮 kill 前清空 logcat，再现即为重连证据）
# renderer-less 构建（emulator flavor）renderer 只断言进程重启，socket/attach
# 路径在 nativeRendererLinked()=false 时按设计旁路。
# 失败一律退出 2（与其他门禁一致）。
#
# 用法: chaos-smoke.sh
set -euo pipefail

ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
ADB="$ANDROID_HOME/platform-tools/adb"

fail() { echo "FAIL: $1" >&2; exit 2; }
[[ -x "$ADB" ]] || fail "adb not found under ANDROID_HOME: $ANDROID_HOME"

# kill 应用子进程需 root；adbd 重启后重新 wait-for-device
"$ADB" root >/dev/null 2>&1 || true
"$ADB" wait-for-device
# 与 install-smoke 同款抗噪（M3 切片 16）：扩主环防 ~1000 行/s 噪声冲掉恢复日志
"$ADB" logcat -G 4M

role_pid() {
  "$ADB" shell pidof "$1" 2>/dev/null | tr -d '\r' | awk '{print $1}'
}

await_new_pid() { # <name> <old_pid> <timeout_seconds>
  local name="$1" old="$2" deadline=$((SECONDS + $3)) new=""
  while (( SECONDS < deadline )); do
    new=$(role_pid "$name" || true)
    if [[ -n "$new" && "$new" != "$old" ]]; then
      echo "$new"
      return 0
    fi
    sleep 2
  done
  return 1
}

await_log() { # <text> <timeout_seconds>
  local text="$1" deadline=$((SECONDS + $2)) probes=""
  while (( SECONDS < deadline )); do
    probes=$("$ADB" logcat -d -s ZeroWebRole:* 2>/dev/null || true)
    if grep -q "$text" <<<"$probes"; then
      return 0
    fi
    sleep 2
  done
  return 1
}

kill_role_cycle() { # <role_process_name> <success_log_or_empty>
  local name="$1" success_log="$2" old new kill_out
  "$ADB" logcat -c
  old=$(role_pid "$name" || true)
  [[ -n "$old" ]] || fail "role process not found before kill: $name"
  echo "chaos: killing $name (pid $old)"
  kill_out=$("$ADB" shell kill "$old" 2>&1 || true)
  if grep -qiE "not permitted|permission denied" <<<"$kill_out"; then
    fail "adb root required to kill role processes: $kill_out"
  fi
  new=$(await_new_pid "$name" "$old" 60) || fail "role process did not restart after kill: $name"
  echo "chaos: $name restarted with new pid $new"
  if [[ -n "$success_log" ]]; then
    await_log "$success_log" 60 || fail "re-connection log missing after restart: $success_log"
    echo "chaos: re-connection confirmed: $success_log"
  fi
}

kill_role_cycle "com.leizm.zeroweb:compositor" "compositor bridge ready"
kill_role_cycle "com.leizm.zeroweb:com.leizm.zeroweb.ImageDecoderService" "decoder probe succeeded"
kill_role_cycle "com.leizm.zeroweb:com.leizm.zeroweb.RendererService0" ""

echo "chaos-smoke PASS"
