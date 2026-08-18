#!/usr/bin/env bash
# target-disk-guard.sh — target/ 磁盘占用守卫（磁盘无限增长防护，见 docs/rally/oom-guard.md）。
#
# 背景：长时间 rally 循环中 target/ 会持续膨胀（多 feature 组合产物、incremental
# 缓存、旧 dep-info 累积），曾把整块磁盘跑满。本守卫在重型 make 入口（build/test/
# reftest/bench 家族）前置执行：占用超阈值时自动清理并继续，未超时零开销放行。
#
# 清理策略（三级，由轻到重）：
#   1. 阈值内 → 直接放行（du 一次，~1-2s/10GB 量级）
#   2. 超阈值 → 先只删 target/*/incremental/（纯编译缓存：cargo 按内容哈希
#      重建，删除绝对安全，且通常是最大头——多 feature 组合的 incremental
#      可达数十 GB）。复测降到阈值内则继续，保住已编译产物避免 30 分钟级
#      冷启动重编译。
#   3. 仍超 → 全量删除 target/。不按 mtime 挑产物目录删的原因：partial
#      清理易破坏 cargo 增量编译一致性，下次构建反而触发更多重编译；全量
#      清后从头重建的产物集是已知可控的。磁盘满的代价（连丢多次 rally
#      轮次）远高于一次全量重编译。
#
# 顺带清理仓库根的 core.* 转储（历次 OOM 尸体，单个可达数百 MB，git 不追踪，
# 无人清理会一直堆积）——每次执行都清，不等超阈值事件。
#
# 用法：scripts/target-disk-guard.sh [target_dir]   （默认 <repo>/target）
# 环境变量：
#   ZW_TARGET_DISK_LIMIT_GB  阈值（GB，默认 100；CI/磁盘小的机器可调小）
#   ZW_TARGET_DISK_GUARD=0   跳过守卫（紧急放行用，不推荐）
# 退出码：0 放行（含清理后放行）/ 1 阈值配置非法
set -euo pipefail

LIMIT_GB=${ZW_TARGET_DISK_LIMIT_GB:-100}
if ! [[ "$LIMIT_GB" =~ ^[1-9][0-9]*$ ]]; then
    echo "target-disk-guard: invalid ZW_TARGET_DISK_LIMIT_GB=$LIMIT_GB"
    exit 1
fi
LIMIT_KB=$((LIMIT_GB * 1024 * 1024))

if [ "${ZW_TARGET_DISK_GUARD:-1}" = "0" ]; then
    echo "target-disk-guard: skipped (ZW_TARGET_DISK_GUARD=0)"
    exit 0
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${1:-$PROJECT_ROOT/target}"

# 仓库根 core.* 转储清理（每次执行；OOM 尸体 git 不追踪，堆积无上限）
shopt -s nullglob
core_files=("$PROJECT_ROOT"/core.*)
if [ ${#core_files[@]} -gt 0 ]; then
    # shellcheck disable=SC2312：du 仅供展示，失败不阻塞
    core_size=$(du -sch "${core_files[@]}" 2>/dev/null | tail -1 | cut -f1)
    echo "target-disk-guard: 清理仓库根 core.* 转储 ×${#core_files[@]}（${core_size:-?}）"
    rm -f "${core_files[@]}"
fi

# target/ 不存在（fresh clone）→ 无需测量
if [ ! -d "$TARGET_DIR" ]; then
    exit 0
fi

# du 测量失败（权限等）→ 放行：守卫自身故障不应阻塞开发流程。
# 管道退出码取 du 的（pipefail），非数字结果一并放行。
USED_KB=$(du -sk "$TARGET_DIR" 2>/dev/null | cut -f1) || exit 0
case "$USED_KB" in ''|*[!0-9]*) exit 0 ;; esac
USED_GB=$((USED_KB / 1024 / 1024))

if [ "$USED_KB" -le "$LIMIT_KB" ]; then
    exit 0
fi

echo "target-disk-guard: target/ 占用 ${USED_GB}GB 超过阈值 ${LIMIT_GB}GB，开始分级清理 …"

# 第一级：只删 incremental 缓存（纯缓存，cargo 按内容哈希重建，删除安全）。
# 覆盖 target/incremental/ 与带 --target 的 target/<triple>/incremental/。
inc_dirs=("$TARGET_DIR"/incremental "$TARGET_DIR"/*/incremental)
removed=0
for inc in "${inc_dirs[@]}"; do
    [ -d "$inc" ] || continue
    inc_kb=$(du -sk "$inc" 2>/dev/null | cut -f1) || continue
    rm -rf "$inc"
    removed=$((removed + inc_kb))
    echo "target-disk-guard: 已删增量缓存 $inc（$((inc_kb / 1024 / 1024))GB）"
done

# 复测：降到阈值内 → 保住已编译产物，避免 30 分钟级冷启动
USED_KB=$(du -sk "$TARGET_DIR" 2>/dev/null | cut -f1) || USED_KB=0
case "$USED_KB" in ''|*[!0-9]*) USED_KB=0 ;; esac
if [ "$USED_KB" -le "$LIMIT_KB" ]; then
    echo "target-disk-guard: 分级清理后 $((USED_KB / 1024 / 1024))GB ≤ 阈值 ${LIMIT_GB}GB，保留已编译产物，继续执行原命令。"
    exit 0
fi

# 第二级：仍超 → 全量删除（等价 cargo clean，但自定义 CARGO_TARGET_DIR 时
# cargo clean 会清错对象；且 cargo clean 在无 Cargo.toml 的上下文里不可用）
rm -rf "$TARGET_DIR"
echo "target-disk-guard: 删增量缓存后仍超阈值，已全量清空 target/，继续执行原命令。"
