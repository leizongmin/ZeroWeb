#!/usr/bin/env bash
# dom-ops-gate.sh — dom-ops 基准门禁编排（M4 消费接口）。
#
# 三段分离与既有 bench-report.sh/perf-gate.sh 同款：measure（tests/benchmarks/dom-ops/
# run-dom-ops.mjs）/ convert（to-bench-report.mjs）/ gate（scripts/perf-gate.sh）。
# 本脚本只编排「转换 + 基线录制/比较」，判定逻辑零新增——比较原样委托 perf-gate.sh，
# 退出码透传（0 全过/1 回归/2 配置错误/3 suspect）。
#
# 基线文件独立于共享准则基线（docs/perf/baselines/linux-x86_64.json）：
#   domops-<platform_class>.json（如 domops-linux-x64.json）。config_hash 按测量协议
#   隔离——混写会破坏既有准则基线的 config_hash 一致性，禁止。
#
# 用法：
#   dom-ops-gate.sh --report <dom-ops results/<ts>/report.json>
#   dom-ops-gate.sh --report <...> --record-baseline "理由"   # 录制/重录基线（理由必填）
# 退出码：录制模式 0 成功 / 2 失败；比较模式 = perf-gate.sh 语义。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
BASELINE_DIR="$REPO/docs/perf/baselines"
RESULTS_DIR="$REPO/tests/benchmarks/results"
SHARED_BASELINE="$BASELINE_DIR/linux-x86_64.json"
CONVERTER="$REPO/tests/benchmarks/dom-ops/to-bench-report.mjs"
PERF_GATE="$SCRIPT_DIR/perf-gate.sh"

REPORT=""
JUSTIFICATION=""
while [ $# -gt 0 ]; do
    case "$1" in
        --report) REPORT="$2"; shift 2 ;;
        --record-baseline) JUSTIFICATION="$2"; shift 2 ;;
        *) echo "dom-ops-gate: 未知参数 $1"; exit 2 ;;
    esac
done

command -v jq >/dev/null || { echo "dom-ops-gate: FAIL — 需要 jq"; exit 2; }
if [ -z "$REPORT" ] || [ ! -f "$REPORT" ]; then
    echo "dom-ops-gate: FAIL — --report <dom-ops report.json> 必填（先跑 run-dom-ops.mjs）"
    exit 2
fi

# 转换（转换器校验 schema/config_hash/入选行完整性，失败即停——门禁不在坏样本上判定）。
STAMP="$(date +%Y%m%d_%H%M%S)"
CONVERTED="$RESULTS_DIR/domops_bench_$STAMP.json"
node "$CONVERTER" --report "$REPORT" --out "$CONVERTED"

PLATFORM_CLASS=$(jq -r '.platform.platform_class' "$CONVERTED")
BASELINE="$BASELINE_DIR/domops-$PLATFORM_CLASS.json"

if [ -z "$JUSTIFICATION" ]; then
    # ── 比较模式 ──
    if [ ! -f "$BASELINE" ]; then
        echo "dom-ops-gate: WARN — 无 domops 基线（$BASELINE）"
        echo "dom-ops-gate: 首次使用请录制基线："
        echo "  bash scripts/dom-ops-gate.sh --report \"$REPORT\" --record-baseline \"初始基线\""
        echo "dom-ops-gate: perf-gate 将按 NEW/PASS 处理（门禁待基线就位后生效）"
    fi
    exec bash "$PERF_GATE" --report "$CONVERTED" --baseline "$BASELINE"
fi

# ── 录制模式 ──
if [ -f "$BASELINE" ]; then
    echo "dom-ops-gate: 重录既有基线 $BASELINE"
    echo "  旧 config_hash: $(jq -r '.run_config.config_hash' "$BASELINE" | cut -c1-16)…"
    echo "  新 config_hash: $(jq -r '.run_config.config_hash' "$CONVERTED" | cut -c1-16)…"
else
    echo "dom-ops-gate: 录制新基线 $BASELINE"
fi

NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
# 预算沿用既有准则基线的公式值（1.35 因子 / floor 1ns / page +40ms / rss +128MB），
# 共享基线缺失时回落到同一组默认——两处数值必须保持同源，改任何一处需同步。
jq -n \
    --slurpfile conv "$CONVERTED" \
    --slurpfile shared "$SHARED_BASELINE" \
    --arg just "$JUSTIFICATION" \
    --arg now "$NOW" '
    ($conv[0]) as $r
    | (($shared[0].budgets // null)
        // {microbench_factor: 1.35, microbench_floor_ns: 1.0,
            page_factor: 1.15, page_constant_ms: 40.0,
            rss_factor: 1.2, rss_constant_mb: 128.0, hard_total_ms: 2000.0}) as $bud
    | {
        schema_version: 1,
        kind: "dom-ops-baseline",
        platform_class: $r.platform.platform_class,
        cpu_model: $r.platform.cpu_model,
        cpu_cores: $r.platform.cpu_cores,
        os: $r.platform.os,
        recorded_at: $now,
        git_sha: $r.git_sha,
        git_dirty: $r.git_dirty,
        run_config: $r.run_config,
        justification: $just,
        budgets: $bud,
        metrics: [ $r.microbenches[] | {id: ("mb/" + .id), p95: (.p95 | round), tier: "budget"} ]
    }' > "$BASELINE.tmp"
mv "$BASELINE.tmp" "$BASELINE"

echo "dom-ops-gate: OK — $(jq '.metrics | length' "$BASELINE") 条指标（ recorded_at=$NOW justification=$JUSTIFICATION）"
jq -r '.metrics[] | "  \(.id)  p95=\(.p95)ns"' "$BASELINE"
echo "dom-ops-gate: 基线为入库产物（docs/perf/baselines/），随本次修改一起提交"
