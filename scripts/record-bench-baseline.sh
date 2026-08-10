#!/usr/bin/env bash
# record-bench-baseline.sh — 记录/更新性能基线（docs/perf/baselines/<platform_class>.json）。
#
# 纪律（对齐 ZeroUI，见 docs/specs/performance-and-resource-budget.md）：
#   - 基线硬件固定：本地 dev box（linux-x86_64）与 CI（github-ubuntu-latest）分开记录
#   - 收紧优先：新 p95 ≥ 旧 p95 × 1.005 且未显式 --relax → 拒绝（防悄悄放宽掩盖回归）
#   - --justification 必填（基线变更须有理由，收紧同样建议填写）
#   - 放宽 = --relax + --justification + 政策文档记录（禁用无声逃生舱）
#
# 用法：
#   bash scripts/record-bench-baseline.sh --justification "初始基线"
#   bash scripts/record-bench-baseline.sh --report <json> --justification "..."
#   bash scripts/record-bench-baseline.sh --justification "..." --relax   # 显式放宽
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"
BASELINE_DIR="$PROJECT_ROOT/docs/perf/baselines"

REPORT=""
JUSTIFICATION=""
RELAX=0
while [ $# -gt 0 ]; do
    case "$1" in
        --report) REPORT="$2"; shift 2 ;;
        --justification) JUSTIFICATION="$2"; shift 2 ;;
        --relax) RELAX=1; shift ;;
        *) echo "record-bench-baseline: 未知参数 $1"; exit 2 ;;
    esac
done

if [ -z "$JUSTIFICATION" ]; then
    echo "record-bench-baseline: FAIL — --justification 必填（基线变更须有理由）"
    echo "  用法: bash scripts/record-bench-baseline.sh --justification \"初始基线\""
    exit 2
fi

if [ -z "$REPORT" ]; then
    REPORT=$(ls -t "$RESULTS_DIR"/benchmark_*.json 2>/dev/null | head -1 || true)
fi
if [ -z "$REPORT" ] || [ ! -f "$REPORT" ]; then
    echo "record-bench-baseline: FAIL — 未找到 bench 报告（先运行 scripts/bench-report.sh）"
    exit 1
fi

PLATFORM_CLASS=$(jq -r '.platform.platform_class' "$REPORT")
BASELINE="$BASELINE_DIR/$PLATFORM_CLASS.json"
mkdir -p "$BASELINE_DIR"
DATE=$(date +%F)
SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)

# 报告 → 指标数组（tier：page total_ms = hard 绝对预算，其余 budget）
METRICS_JSON=$(jq -c '
    [ (.microbenches // [])[] | select(.p95 != null) | {id: ("mb/" + .id), p95: .p95, tier: "budget"} ] +
    [ (.pages // [])[] | .scenario as $s |
        (.stages | to_entries[] | select(.value.p95 != null) |
         (if .key == "total_ms" then {tier: "hard", absolute_budget_ms: 2000.0} else {tier: "budget"} end) as $t |
         {id: ("page/" + $s + "/" + .key), p95: .value.p95} + $t) ] +
    [ (.pages // [])[] | .scenario as $s | select(.first_paint_wall_ms.p95 != null) |
        {id: ("page/" + $s + "/first_paint_wall_ms"), p95: .first_paint_wall_ms.p95, tier: "budget"} ] +
    [ (.resource // {}) | select(.peak_rss_mb != null) |
        {id: "resource/peak_rss_mb", p95: .peak_rss_mb, tier: "budget"} ]
' "$REPORT")

# 收紧优先守卫：存在旧基线且任何指标 p95 ≥ 旧值×1.005 → 拒绝（除非 --relax）
if [ -f "$BASELINE" ] && [ "$RELAX" != "1" ]; then
    VIOLS=$(jq -n --argjson m "$METRICS_JSON" --slurpfile base "$BASELINE" '
        [ $m[] | . as $n |
          (($base[0].metrics // []) | map(select(.id == $n.id)) | .[0]) as $b |
          if $b != null and $n.p95 >= ($b.p95 * 1.005)
          then {id: $n.id, old: $b.p95, new: $n.p95}
          else empty end ]')
    if [ "$VIOLS" != "[]" ]; then
        echo "record-bench-baseline: REFUSE — 以下指标较旧基线劣化 ≥0.5%，收紧优先，拒绝覆盖："
        echo "$VIOLS" | jq -r '.[] | "  \(.id): old=\(.old) new=\(.new) (+\(((.new / .old - 1) * 100) | . * 100 | round / 100)%)"'
        echo "  确认为合法变化请显式放宽：--relax --justification \"...\"（须在政策文档记录理由）"
        exit 1
    fi
fi

# 写基线
jq -n \
    --argjson metrics "$METRICS_JSON" \
    --slurpfile rep "$REPORT" \
    --arg platform_class "$PLATFORM_CLASS" \
    --arg recorded_at "$DATE" \
    --arg git_sha "$SHA" \
    --arg justification "$JUSTIFICATION" \
    '{schema_version: 1, kind: "perf-baseline",
      platform_class: $platform_class,
      cpu_model: $rep[0].platform.cpu_model, cpu_cores: $rep[0].platform.cpu_cores, os: $rep[0].platform.os,
      recorded_at: $recorded_at, git_sha: $git_sha,
      run_config: {config_hash: $rep[0].run_config.config_hash, profile: $rep[0].run_config.profile},
      justification: $justification,
      budgets: {microbench_factor: 1.35, microbench_floor_ns: 1.0, page_factor: 1.15,
                page_constant_ms: 40.0, rss_factor: 1.20, rss_constant_mb: 128.0,
                hard_total_ms: 2000.0},
      metrics: $metrics}' \
    > "$BASELINE"

COUNT=$(echo "$METRICS_JSON" | jq 'length')
echo "record-bench-baseline: OK — 已写入 $BASELINE（$COUNT 个指标，$([ "$RELAX" = "1" ] && echo "relax 模式" || echo "正常模式")）"
echo "  justification: $JUSTIFICATION"
