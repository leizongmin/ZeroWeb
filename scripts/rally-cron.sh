#!/usr/bin/env bash
# 启动 rally cron 调度器（docs/rally/jobs.yaml）。
#
# 前台进程运行，按 cron 触发 docs-maintenance / monthly-report-finalize /
# goal-blockers-nightly 三个定时任务（claude-dsflash，晚 8 点-早 6 点便宜窗口）。
# Ctrl-C 或 SIGTERM 退出。
#
# 用法：
#   bash scripts/rally-cron.sh               # 启动调度器（前台）
#   bash scripts/rally-cron.sh --dry-run     # 只校验配置并打印下次触发时间
#   bash scripts/rally-cron.sh --no-rules    # 单次禁用 run-rules.md 注入（一般不需要）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ "${1:-}" = "--dry-run" ]; then
    exec rally cron run -w "$PROJECT_ROOT" --dry-run
fi

# 无人值守安全（docs/rally/oom-guard.md）：长驻调度器建议放限内存单元内。
# 若在 systemd 下运行，可改用：systemd-run --user --scope -p MemoryMax=4G \
#   bash scripts/rally-cron.sh
exec rally cron run -w "$PROJECT_ROOT" "$@"
