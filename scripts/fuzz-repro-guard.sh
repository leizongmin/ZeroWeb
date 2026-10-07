#!/usr/bin/env bash
# fuzz-repro-guard.sh — 病态输入/fuzz 复现的安全包裹器（2026-10-07 OOM 事故产物）
#
# 背景：fuzz 崩溃工件（timeout-*/OOM-*）的复现与最小化极易触发指数放大——
# 2026-10-07 两个 ~35GB 的最小化进程把 46GB 机器拖到系统 OOM，连坐杀死
# oom_score_adj=500 的远程入口服务，用户会话断线 16 分钟。病态输入实验
# 无论有人值守与否，一律走本脚本，禁止裸跑。
#
# 用法：
#   scripts/fuzz-repro-guard.sh [--mem-mb N] [--time-s S] \
#     [--build "-p <pkg> --test <test_target>"] -- <测试二进制> [参数...]
#
#   --mem-mb N     地址空间上限 MiB（默认 8192；ulimit -v 按虚拟内存计）
#   --time-s S     墙钟超时秒（默认 30）
#   --build "..."  先无限制预编译（cargo test <...> --no-run）。ulimit -v 会把
#                  链接器一起掐死（LLVM 地址空间映射大），故预编译必须在本
#                  脚本设限之前完成——这也是裸跑容易漏掉的一步。
#   -- 之后是待执行的测试二进制与参数；环境变量（如 PROBE_INPUT）原样透传。
#
# 示例（复现 css-parser fuzz 工件）：
#   scripts/fuzz-repro-guard.sh \
#     --build "-p zero-css-parser --test my_probe" -- \
#     target/debug/deps/my_probe-<hash> p_probe --nocapture --test-threads=1
#   （配 PROBE_INPUT=/path/to/timeout-xxx 环境变量传入病态输入）

set -euo pipefail

MEM_MB=8192
TIME_S=30
BUILD_ARGS=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mem-mb) MEM_MB="$2"; shift 2 ;;
    --time-s) TIME_S="$2"; shift 2 ;;
    --build) BUILD_ARGS="$2"; shift 2 ;;
    --) shift; break ;;
    *) echo "未知参数: $1（-- 之后才是二进制与参数）" >&2; exit 2 ;;
  esac
done

if [[ $# -eq 0 ]]; then
  echo "用法: $0 [--mem-mb N] [--time-s S] [--build \"<cargo test 参数>\"] -- <测试二进制> [参数...]" >&2
  exit 2
fi

BIN="$1"; shift

# 预编译阶段：不设 ulimit（链接器需要大地址空间），编译失败直接退出。
if [[ -n "$BUILD_ARGS" ]]; then
  # shellcheck disable=SC2086
  cargo test $BUILD_ARGS --no-run
fi

# 执行阶段：内存 + 墙钟双包裹。ulimit -v 限地址空间（比 MemoryMax 严格但对
# "指数放大" 类失控足够；systemd-run 需 polkit 授权非通用）。
exec bash -c "
  ulimit -v $((MEM_MB * 1024))
  exec timeout --signal=KILL ${TIME_S}s \"\$@\"
" _ "$BIN" "$@"
