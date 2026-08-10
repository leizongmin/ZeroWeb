#!/usr/bin/env bash
# perf-gate.sh — 性能门禁比较（纯函数：读报告 + 读基线 → 判定 + 退出码）。
#
# 不包含任何测量逻辑（测量在 scripts/bench-report.sh，ZeroUI 同款纪律：measure 与 gate 分离）。
# 三层门禁（公式见 docs/specs/performance-and-resource-budget.md）：
#   Hard Gate:  page/*/total_ms p95 ≤ absolute_budget_ms（默认 2000ms，对齐「首屏 < 2s」）
#   Budget Gate: mb/*  p95 ≤ max(baseline * 1.35, baseline + 1.0ns)（基线 <10ns 时）
#                      否则 p95 ≤ baseline * 1.35（2026-08-10 校准：github 共享 runner 类
#                      实测方差 +27~31%，旧 1.20 因子余量不足；ns 级指标加绝对下限吸收抖动）
#                page/* p95 ≤ baseline * 1.15 + 40ms（+40 常数吸收调度抖动）
#                resource/peak_rss_mb ≤ baseline * 1.20 + 128MB
#   无基线条目（新指标）→ NEW/PASS 并记录趋势；schema_version / config_hash 不匹配 → exit 2
#   （配置变了，须重新 capture）；无该平台基线 → PASS+WARN 附 capture 指引。
#
# 用法：perf-gate.sh [--report <json>] [--baseline <json>]
#   默认 report = tests/benchmarks/results/ 最新；baseline = docs/perf/baselines/<platform_class>.json
# 退出码：0 全过（含全 NEW）/ 1 回归或测量失败 / 2 配置错误
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"
BASELINE_DIR="$PROJECT_ROOT/docs/perf/baselines"

REPORT=""
BASELINE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --report) REPORT="$2"; shift 2 ;;
        --baseline) BASELINE="$2"; shift 2 ;;
        *) echo "perf-gate: 未知参数 $1"; exit 2 ;;
    esac
done

if [ -z "$REPORT" ]; then
    REPORT=$(ls -t "$RESULTS_DIR"/benchmark_*.json 2>/dev/null | head -1 || true)
fi
if [ -z "$REPORT" ] || [ ! -f "$REPORT" ]; then
    echo "perf-gate: FAIL — 未找到 bench 报告（先运行 scripts/bench-report.sh）"
    exit 1
fi

# 测量侧失败防御：报告中含 error 条目（bench 执行失败）→ 直接 FAIL
if [ "$(jq '[.microbenches[] | select(.error != null)] | length' "$REPORT")" -gt 0 ]; then
    echo "perf-gate: FAIL — 报告含测量失败条目（bench 执行非零），门禁不判定"
    exit 1
fi

# suspect 报告（测量期间系统负载升高——另一条流 WPT/测试叠加）→ 不可信，不判定
if [ "$(jq -r '.suspect // false' "$REPORT")" = "true" ]; then
    echo "perf-gate: INCONCLUSIVE — 报告标记 suspect（测量期间系统负载升高，可能被另一条流污染），"
    echo "  结果不可信，请机器空闲时重跑（bench-report.sh 已带负载守卫自动重试提示）。"
    exit 3
fi

PLATFORM_CLASS=$(jq -r '.platform.platform_class' "$REPORT")
[ -n "$BASELINE" ] || BASELINE="$BASELINE_DIR/$PLATFORM_CLASS.json"

echo "perf-gate: report=$(basename "$REPORT") platform=$PLATFORM_CLASS"

HAS_BASELINE=0
test -f "$BASELINE" || HAS_BASELINE=1
case "$HAS_BASELINE" in
    1)
        echo "perf-gate: WARN — 无 $PLATFORM_CLASS 基线 ($BASELINE)"
        echo "perf-gate: 首次使用请 capture 基线："
        echo "  bash scripts/record-bench-baseline.sh --justification \"初始基线\""
        echo "perf-gate: 全部指标按 NEW/PASS 处理（门禁待基线就位后生效）"
        exit 0
        ;;
esac

# schema / config_hash 一致性（防跨配置比较——改了场景/迭代数必须先重新 capture）
REP_SV=$(jq -r '.schema_version' "$REPORT")
BASE_SV=$(jq -r '.schema_version' "$BASELINE")
if [ "$REP_SV" != "$BASE_SV" ] || [ "$REP_SV" != "1" ]; then
    echo "perf-gate: FAIL — schema_version 不匹配（report=$REP_SV baseline=$BASE_SV），需重新 capture 基线"
    exit 2
fi
REP_HASH=$(jq -r '.run_config.config_hash' "$REPORT")
BASE_HASH=$(jq -r '.run_config.config_hash' "$BASELINE")
if [ "$REP_HASH" != "$BASE_HASH" ]; then
    echo "perf-gate: FAIL — run_config 不匹配（测量配置已变更），需重新 capture 基线"
    echo "  report:    ${REP_HASH:0:16}…"
    echo "  baseline:  ${BASE_HASH:0:16}…"
    exit 2
fi

RESULT=$(jq -n --slurpfile rep "$REPORT" --slurpfile base "$BASELINE" '
    def family($id):
        if ($id | startswith("mb/")) then "mb"
        elif ($id | startswith("page/")) then "page"
        elif ($id | startswith("resource/")) then "rss"
        else "other" end;
    def budget_for($b; $fam; $bud):
        if $b.tier == "hard" then ($b.absolute_budget_ms // $bud.hard_total_ms)
        # mb/*（2026-08-10 校准）：1.35 因子吸收共享 runner 类方差（两轮实测 3 指标
        # +27~31% 且代码零改动）；基线 <10ns 的 ns 级指标绝对抖动主导，取
        # max(因子预算, 基线 + microbench_floor_ns)（旧基线无 floor 键 → 仅因子）。
        elif $fam == "mb" then
            ($b.p95 * $bud.microbench_factor) as $f
            | if $b.p95 < 10 then
                (($b.p95 + ($bud.microbench_floor_ns // 0.0)) as $with_floor
                 | if $f > $with_floor then $f else $with_floor end)
              else $f end
        elif $fam == "page" then $b.p95 * $bud.page_factor + $bud.page_constant_ms
        elif $fam == "rss" then $b.p95 * $bud.rss_factor + $bud.rss_constant_mb
        else $b.p95 * 1.35 end;
    ($rep[0]) as $r | ($base[0]) as $b0 |
    # 注意：`X as $r | BODY` 里隐式 `.` 仍指向原始输入（-n 下为 null），
    # 顶层字段必须显式 `$r.` 前缀（仅管道内的逐元素 `.` 才是当前元素）。
    ([ ($r.microbenches // [])[] | {id: ("mb/" + .id), p95: .p95, unit: .unit} ] +
     [ ($r.pages // [])[] | .scenario as $s |
        (.stages | to_entries[] | {id: ("page/" + $s + "/" + .key), p95: .value.p95, unit: "ms"}) ] +
     [ ($r.pages // [])[] | .scenario as $s |
        {id: ("page/" + $s + "/first_paint_wall_ms"), p95: .first_paint_wall_ms.p95, unit: "ms"} ] +
     [ {id: "resource/peak_rss_mb", p95: (($r.resource // {}).peak_rss_mb // null), unit: "MB"} ])
    | map(. as $m |
        (($b0.metrics // []) | map(select(.id == $m.id)) | .[0]) as $b |
        if $m.p95 == null then {id: $m.id, verdict: "SKIP", measured: null, baseline: null, budget: null, unit: $m.unit}
        elif $b == null then {id: $m.id, verdict: "NEW", measured: $m.p95, baseline: null, budget: null, unit: $m.unit}
        else ($m.id | family(.)) as $fam |
             (budget_for($b; $fam; $b0.budgets)) as $budget |
             {id: $m.id, verdict: (if ($m.p95 > $budget) then "FAIL" else "PASS" end),
              measured: $m.p95, baseline: $b.p95, budget: $budget, unit: $m.unit}
        end)
')

echo ""
printf "%-8s %-58s %10s %10s %12s %s\n" "verdict" "metric" "measured" "baseline" "budget" "unit"
echo "$RESULT" | jq -r '.[] |
    def fmt($v): if $v == null then "-" else (($v * 100 | round) / 100) end;
    "\(.verdict)\t\(.id)\t\(fmt(.measured))\t\(fmt(.baseline))\t\(fmt(.budget))\t\(.unit)"
' | while IFS=$'\t' read -r v id m b bud u; do
    printf "%-8s %-58s %10s %10s %12s %s\n" "$v" "$id" "$m" "$b" "$bud" "$u"
done

FAILS=$(echo "$RESULT" | jq -r '[.[] | select(.verdict == "FAIL")] | length')
NEWS=$(echo "$RESULT" | jq -r '[.[] | select(.verdict == "NEW")] | length')
CHECKS=$(echo "$RESULT" | jq -r '[.[] | select(.verdict == "PASS" or .verdict == "FAIL")] | length')

echo ""
if [ "$FAILS" -gt 0 ]; then
    echo "perf-gate: GATE FAIL — $FAILS 个指标超预算（已检查 $CHECKS，NEW=$NEWS）"
    echo "perf-gate: 修复回归，或确认为合法变化后显式更新基线（record-bench-baseline.sh --relax --justification \"...\"）"
    exit 1
fi
echo "perf-gate: GATE PASS — 已检查 $CHECKS 个指标（NEW=$NEWS），全部在预算内"
