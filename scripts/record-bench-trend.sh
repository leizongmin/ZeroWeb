#!/usr/bin/env bash
# record-bench-trend.sh — 性能基准趋势记录（镜像 scripts/record-wpt-trend.sh）。
#
# 把一份 bench 报告（默认最新）追加到
#   docs/perf/trends/benchmark-trend.csv（每指标一行）
# 并保留 JSON 快照（docs/perf/trends/<date>-<platform_class>.json），形成可回溯趋势。
#
# --auto-tighten（weekly CI 用）：实测 p95 低于基线 → 就地更新基线（仅收紧，无需 justification；
# 收紧永远合法）。门禁公式中的 factor/constant 不变。
#
# 用法：
#   bash scripts/record-bench-trend.sh                     # 记录最新报告
#   bash scripts/record-bench-trend.sh --report <json> --note "R21xx 优化后"
#   bash scripts/record-bench-trend.sh --auto-tighten      # 记录 + 收紧基线
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"
TREND_DIR="$PROJECT_ROOT/docs/perf/trends"
BASELINE_DIR="$PROJECT_ROOT/docs/perf/baselines"
TREND_CSV="$TREND_DIR/benchmark-trend.csv"

REPORT=""
NOTE=""
AUTO_TIGHTEN=0
while [ $# -gt 0 ]; do
    case "$1" in
        --report) REPORT="$2"; shift 2 ;;
        --note) NOTE="$2"; shift 2 ;;
        --auto-tighten) AUTO_TIGHTEN=1; shift ;;
        *) echo "record-bench-trend: 未知参数 $1"; exit 2 ;;
    esac
done

if [ -z "$REPORT" ]; then
    REPORT=$(ls -t "$RESULTS_DIR"/benchmark_*.json 2>/dev/null | head -1 || true)
fi
if [ -z "$REPORT" ] || [ ! -f "$REPORT" ]; then
    echo "record-bench-trend: FAIL — 未找到 bench 报告（先运行 scripts/bench-report.sh）"
    exit 1
fi

DATE=$(date +%F)
PLATFORM_CLASS=$(jq -r '.platform.platform_class' "$REPORT")
SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)
NOTE_SAFE=$(echo "$NOTE" | sed 's/,/，/g')  # CSV 逗号转义

mkdir -p "$TREND_DIR"
if [ ! -f "$TREND_CSV" ]; then
    echo "# 性能基准趋势（per-metric p50/p95/max；门禁公式见 docs/specs/performance-and-resource-budget.md）" > "$TREND_CSV"
    echo "# date,platform_class,metric_id,p50,p95,max,unit,git_sha,note" >> "$TREND_CSV"
fi

# 展开为 per-metric 行（p50/p95/max；resource/startup 单值 → p95 即本身，p50/max 空）
ROWS=$(jq -c '
    [ (.microbenches // [])[] | select(.p95 != null) |
        {id: ("mb/" + .id), p50: .p50, p95: .p95, max: .max, unit: .unit} ] +
    [ (.pages // [])[] | .scenario as $s |
        (.stages | to_entries[] | select(.value.p95 != null) |
         {id: ("page/" + $s + "/" + .key), p50: .value.p50, p95: .value.p95, max: .value.max, unit: "ms"}) ] +
    [ (.pages // [])[] | .scenario as $s | select(.first_paint_wall_ms.p95 != null) |
        {id: ("page/" + $s + "/first_paint_wall_ms"), p50: .first_paint_wall_ms.p50,
         p95: .first_paint_wall_ms.p95, max: .first_paint_wall_ms.max, unit: "ms"} ] +
    [ (.resource // {}) | select(.peak_rss_mb != null) |
        {id: "resource/peak_rss_mb", p50: null, p95: .peak_rss_mb, max: null, unit: "MB"} ] +
    [ {id: "startup_ms", p50: null, p95: .startup_ms, max: null, unit: "ms"} ]
' "$REPORT")

# CSV：单值指标 p50/max 留空
echo "$ROWS" | jq -r --arg date "$DATE" --arg pc "$PLATFORM_CLASS" --arg sha "$SHA" --arg note "$NOTE_SAFE" '
    .[] | "\($date),\($pc),\(.id),\(.p50 // ""),\(.p95 // ""),\(.max // ""),\(.unit),\($sha),\($note)"
' >> "$TREND_CSV"

# JSON 快照（整份报告，可回溯；带时间戳命名——同一天多次运行各留一份，
# 2026-08-08 曾因定向报告覆盖全量报告快照丢失 93 个指标）
SNAPSHOT="$TREND_DIR/${DATE}-$(date +%H%M%S)-${PLATFORM_CLASS}.json"
cp "$REPORT" "$SNAPSHOT"

echo "record-bench-trend: 已记录 $(echo "$ROWS" | jq 'length') 个指标 → $TREND_CSV / $SNAPSHOT"

# --auto-tighten：实测 p95 < 基线 → 收紧基线（min 合并；仅收紧）
if [ "$AUTO_TIGHTEN" = "1" ]; then
    BASELINE="$BASELINE_DIR/$PLATFORM_CLASS.json"
    if [ ! -f "$BASELINE" ]; then
        echo "record-bench-trend: WARN — 无基线，跳过 auto-tighten（先 record-bench-baseline.sh）"
    else
        jq --slurpfile trend <(echo "$ROWS") '
            .metrics = ([ .metrics[] | . as $m |
                (($trend[0] // []) | map(select(.id == $m.id)) | .[0]) as $t |
                if $t != null and $t.p95 != null and $t.p95 < $m.p95
                then $m + {p95: $t.p95, tightened_at: now | todateiso8601}
                else $m end ])
        ' "$BASELINE" > "$BASELINE.tmp" && mv "$BASELINE.tmp" "$BASELINE"
        TIGHTENED=$(jq '[.metrics[] | select(.tightened_at != null)] | length' "$BASELINE")
        echo "record-bench-trend: auto-tighten 收紧 $TIGHTENED 个指标（基线 $BASELINE）"
    fi
fi
