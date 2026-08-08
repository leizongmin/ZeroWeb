#!/usr/bin/env bash
# bench-report.sh — 全量性能测量管线（性能门禁体系，见 docs/specs/performance-and-resource-budget.md）。
#
# 测量三面：
#   1. criterion 微基准（16 crate × [[bench]]，target/criterion/**/new/sample.json 原始样本
#      → 逐迭代 ns = times[i]/iters[i]，自算 p50/p95/max——criterion 0.5.1 无 --output-format json）
#   2. 页面级首屏基准（zero-wpt-runner perf：welcome/medium/morning 三 fixture，
#      各阶段耗时 parse/style/layout/paint/total + 墙钟首屏，每场景 14 个样本）
#   3. 进程资源（峰值 RSS：Linux VmHWM / macOS getrusage / 其他平台 null + startup_ms）
#
# 输出：tests/benchmarks/results/benchmark_${DATE}.json（机器可读，供 perf-gate.sh /
#       record-bench-trend.sh 消费）+ benchmark_${DATE}.txt（人读摘要）。
# 纪律（对齐 ZeroUI）：测量与本脚本、比较（perf-gate.sh）分离，永不融合；
# 任一 crate 基准执行失败 → 整体 exit 1（修复 run-benchmarks.sh 时代"失败总 exit 0"的 bug）。
# ZERO_WEB_BENCH_QUICK=1 → 仅 --no-run 编译检查（PR CI 用，不测量）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/tests/benchmarks/results"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$RESULTS_DIR"

DATE=$(date +%Y%m%d_%H%M%S)
REPORT_JSON="$RESULTS_DIR/benchmark_${DATE}.json"
REPORT_TXT="$RESULTS_DIR/benchmark_${DATE}.txt"

# 所有带 [[bench]] 的 crate 及其 bench 文件名（taffy-local 的 dummy bench 会 exit 1，
# 必须走显式清单，不能裸跑 cargo bench --workspace）。
declare -A BENCH_MAP=(
    [zero-css-parser]="css_bench"
    [zero-dom]="dom_bench"
    [zero-style-system]="style_bench"
    [zero-layout-engine]="layout_bench"
    [zero-engine]="engine_bench"
    [zero-canvas]="canvas_bench"
    [zero-render-foundation]="render_bench"
    [zero-host-runtime]="host_runtime_bench"
    [zero-webview]="webview_bench"
    [zero-net]="net_bench"
    [zero-protocol]="protocol_bench"
    [zero-security]="security_bench"
    [zero-storage]="storage_bench"
    [zero-wasm-sandbox]="wasm_bench"
    [zero-browser-shell]="browser_shell_bench"
    [zero-script-sandbox]="script_sandbox_bench"
)

# 页面级场景：id:fixture:base_dir（base_dir 恒传 morning 目录——welcome/medium 自包含不受影响，
# 保证 run_config 跨轮次一致）。顺序仅作显示，config_hash 用排序后的规范化串。
declare -a PAGE_SCENARIOS=(
    "welcome:apps/browser/assets/welcome.html:"
    "medium:tests/benchmarks/fixtures/medium.html:"
    "morning:apps/browser/assets/morning-work/article.html:apps/browser/assets/morning-work"
)
PAGE_BASE_DIR="apps/browser/assets/morning-work"
PAGE_ITERATIONS=15

QUICK_MODE=0
if [ "${ZERO_WEB_BENCH_QUICK:-}" = "1" ]; then
    QUICK_MODE=1
fi

# 负载守卫（2026-08-08：本机为双流共享，另一条流会不定期跑 WPT 全量——重 CPU 负载下
# µs 级微基准集体超标、报告不可信。loadavg 1min 超阈值（默认 = 逻辑核数 × 0.75）时
# 快速失败并提示重试，避免产出垃圾报告。ZW_BENCH_ALLOW_BUSY=1 强制跳过守卫）。
BUSY_THRESHOLD=${ZERO_WEB_BENCH_BUSY_THRESHOLD:-$(($(nproc 2>/dev/null || echo 16) * 3 / 4))}
# CI（GITHUB_ACTIONS）为专用 runner，无另一条流的 WPT/测试干扰——负载守卫只在
# 本地共享机器有价值（2026-08-08：CI 4 核 runner loadavg 波动易超阈值 3 误伤）
if [ "$QUICK_MODE" != "1" ] && [ "${ZW_BENCH_ALLOW_BUSY:-}" != "1" ] && [ "${GITHUB_ACTIONS:-}" != "true" ]; then
    LOAD1=$(cut -d' ' -f1 /proc/loadavg 2>/dev/null | cut -d. -f1 || echo 0)
    if [ -n "$LOAD1" ] && [ "$LOAD1" -gt "$BUSY_THRESHOLD" ]; then
        echo "bench-report: ABORT — 系统繁忙（loadavg 1min=$LOAD1 > 阈值 $BUSY_THRESHOLD，可能另一条流正在跑 WPT/测试），"
        echo "  测量结果不可信，稍后重试。强制运行：ZW_BENCH_ALLOW_BUSY=1（不推荐，产出不可信报告）。"
        exit 3
    fi
fi

# ---------- 元数据 ----------
GIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)
if git -C "$PROJECT_ROOT" status --porcelain | grep -q .; then
    GIT_DIRTY="true"
else
    GIT_DIRTY="false"
fi
# 平台类：CI（GitHub Actions ubuntu-latest）与本地 dev box 分基线（硬件固定，见 policy 文档）
if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
    PLATFORM_CLASS="github-ubuntu-latest"
else
    PLATFORM_CLASS="$(uname -s | tr 'A-Z' 'a-z')-$(uname -m)"
fi
OS_NAME=$(uname -s | tr 'A-Z' 'a-z')

# 规范化 run_config → config_hash（sha256，不含 git_sha：配置 = 怎么测，不是测了什么）
BENCH_LIST_SORTED=$(printf '%s\n' "${!BENCH_MAP[@]}" | sort | while read -r c; do echo "$c:${BENCH_MAP[$c]}"; done)
SCENARIOS_SORTED=$(printf '%s\n' "${PAGE_SCENARIOS[@]}" | sed 's/:.*//' | sort | paste -sd, - | tr -d '\n')
FIXTURES_SORTED=$(printf '%s\n' "${PAGE_SCENARIOS[@]}" | sed 's/^[^:]*:\([^:]*\):.*/\1/' | sort | paste -sd, - | tr -d '\n')
# criterion 测量上限（2026-08-08：css_parse_by_size 5000 规则档 O(n²) 解析 ~14s/迭代，
# 而 sample-size 是采样下限（且 criterion 强制 ≥10）——慢档会强制采满 sample-size 次：
# sample-size 20 时 5000 档 ~5 分钟封顶，全套 ~12 分钟，可接受。快档受 3s 时间上限
# 约束采样量不变。参数入 config_hash（改测量配置必须重新 capture 基线）。
# 详见 docs/learnings/performance/css-parser-quadratic-scaling.md）
CRITERION_FLAGS="--warm-up-time 1 --measurement-time 3 --sample-size 20 --noplot"
CONFIG_HASH=$(printf 'profile=release;criterion=%s;benches=%s;scenarios=%s;viewport=800x600;iterations=%s;fixtures=%s;base_dir=%s' \
    "$CRITERION_FLAGS" "$(echo "$BENCH_LIST_SORTED" | paste -sd, -)" "$SCENARIOS_SORTED" "$PAGE_ITERATIONS" "$FIXTURES_SORTED" "$PAGE_BASE_DIR" \
    | sha256sum | cut -d' ' -f1)

echo "=== ZeroWeb Benchmarks ===" | tee "$REPORT_TXT"
echo "Date: $(date)" | tee -a "$REPORT_TXT"
echo "Commit: $GIT_SHA (dirty=$GIT_DIRTY)" | tee -a "$REPORT_TXT"
echo "Platform: $PLATFORM_CLASS" | tee -a "$REPORT_TXT"
echo "Crates: ${#BENCH_MAP[@]} | Pages: $(printf '%s\n' "${PAGE_SCENARIOS[@]}" | sed 's/:.*//' | paste -sd, -)" | tee -a "$REPORT_TXT"
echo "Config hash: ${CONFIG_HASH:0:16}…" | tee -a "$REPORT_TXT"
if [ "$QUICK_MODE" = "1" ]; then
    echo "Mode: quick compile check" | tee -a "$REPORT_TXT"
fi
echo "" | tee -a "$REPORT_TXT"

PASSED=()
FAILED=()
MICROBENCHES_JSONL="$TMP_DIR/microbenches.jsonl"
: > "$MICROBENCHES_JSONL"

# 定向测量（ZERO_WEB_BENCH_CRATES=zero-css-parser,zero-dom — 局部优化验证 / 忙时小窗口测量）
CRATES_FILTER=""
if [ -n "${ZERO_WEB_BENCH_CRATES:-}" ]; then
    CRATES_FILTER=",$ZERO_WEB_BENCH_CRATES,"
fi
BENCH_CRATES=()
for crate in $(printf '%s\n' "${!BENCH_MAP[@]}" | sort); do
    if [ -z "$CRATES_FILTER" ] || [[ "$CRATES_FILTER" == *",$crate,"* ]]; then
        BENCH_CRATES+=("$crate")
    fi
done

# ---------- 1. criterion 微基准 ----------
if [ "$QUICK_MODE" = "1" ]; then
    # 编译检查：16 个 bench 全部 --no-run 通过才算过
    for crate in "${BENCH_CRATES[@]}"; do
        bench_name="${BENCH_MAP[$crate]}"
        echo "--- $crate ($bench_name) --no-run ---" | tee -a "$REPORT_TXT"
        if cargo bench -p "$crate" --bench "$bench_name" --no-run > "$TMP_DIR/bench.log" 2>&1; then
            PASSED+=("$crate")
        else
            FAILED+=("$crate")
            echo "[WARN] $crate compile-check failed" | tee -a "$REPORT_TXT"
            tail -5 "$TMP_DIR/bench.log" | tee -a "$REPORT_TXT"
        fi
    done
else
    # 真实测量：每次跑前快照 target/criterion 已有 sample.json，跑后 diff 出新文件
    #（同一 group 的旧数据在下一 crate 跑时会被 criterion 覆写，comm 只取本次新增）。
    rm -rf "$PROJECT_ROOT/target/criterion"
    for crate in "${BENCH_CRATES[@]}"; do
        bench_name="${BENCH_MAP[$crate]}"
        echo "--- $crate ($bench_name) ---" | tee -a "$REPORT_TXT"
        # 只取 new/（当前运行）——criterion 默认 Baseline::Save 会把上次运行留在 base/
        before=$(find "$PROJECT_ROOT/target/criterion" -path '*/new/sample.json' 2>/dev/null | sort || true)
        # shellcheck disable=SC2086：CRITERION_FLAGS 有意分词
        bench_rc=0
        if ! cargo bench -p "$crate" --bench "$bench_name" -- $CRITERION_FLAGS > "$TMP_DIR/bench.log" 2>&1; then
            bench_rc=$?
            FAILED+=("$crate")
            echo "[WARN] $crate benchmarks failed (exit $bench_rc)" | tee -a "$REPORT_TXT"
            tail -5 "$TMP_DIR/bench.log" | tee -a "$REPORT_TXT"
            echo "{\"crate\":\"$crate\",\"bench\":\"$bench_name\",\"error\":\"bench-exit-nonzero\"}" >> "$MICROBENCHES_JSONL"
            continue
        fi
        PASSED+=("$crate")
        after=$(find "$PROJECT_ROOT/target/criterion" -path '*/new/sample.json' 2>/dev/null | sort || true)
        # comm 需要排序输入（find 已 sort）；无新文件时 comm 退出 1，|| true 兜底
        new_files=$(comm -13 <(printf '%s\n' "$before") <(printf '%s\n' "$after") || true)
        for sample in $new_files; do
            dir=$(dirname "$sample")
            full_id=$(jq -r '.full_id // .title // "unknown"' "$dir/benchmark.json" 2>/dev/null || echo unknown)
            # 逐迭代 ns = times[i]/iters[i]；pct(p) 为分位索引取整（与 criterion 文本报告
            # 的 median 同量级，首跑会做 sanity-check）
            jq -c --arg id "$crate/$full_id" --arg crate "$crate" --arg bench "$bench_name" '
                def pct(p): (sort | .[(((p/100.0) * (length - 1)) | floor)]);
                # 转置后每元素为 [iters_i, times_i]；逐迭代 ns = times_i / iters_i = .[1] / .[0]
                [.iters, .times] | transpose | map(.[1] / .[0])
                | {id: $id, crate: $crate, bench: $bench, unit: "ns",
                   p50: pct(50), p95: pct(95), max: max, samples: length}
            ' "$sample" >> "$MICROBENCHES_JSONL"
        done
    done
fi

# ---------- 2/3. 页面级首屏 + 进程资源（仅真实测量模式） ----------
PAGES_JSON="null"
RESOURCE_JSON="null"
STARTUP_MS="null"
if [ "$QUICK_MODE" != "1" ]; then
    echo "--- page scenarios ---" | tee -a "$REPORT_TXT"
    perf_args=()
    for spec in "${PAGE_SCENARIOS[@]}"; do
        id="${spec%%:*}"
        path="${spec#*:}"; path="${path%%:*}"
        perf_args+=(--scenario "$id:$path")
    done
    if ! cargo run --release --quiet --bin zero-wpt-runner -- perf \
        "${perf_args[@]}" --base-dir "$PAGE_BASE_DIR" \
        --iterations "$PAGE_ITERATIONS" --width 800 --height 600 \
        > "$TMP_DIR/pages.json" 2> "$TMP_DIR/perf.log"; then
        echo "[WARN] page benchmark failed" | tee -a "$REPORT_TXT"
        tail -10 "$TMP_DIR/perf.log" | tee -a "$REPORT_TXT"
        FAILED+=("pages")
    else
        PAGES_JSON=$(jq -c '{scenarios: [.scenarios[] | {
                scenario: .id, fixture: .fixture, viewport: .viewport, samples: (.samples | length),
                stages: {
                    parse_ms: [.samples[].parse_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max},
                    style_ms: [.samples[].style_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max},
                    layout_ms: [.samples[].layout_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max},
                    paint_ms: [.samples[].paint_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max},
                    total_ms: [.samples[].total_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max}
                },
                first_paint_wall_ms: [.samples[].wall_ms] | {p50: (sort | .[((0.50 * (length - 1)) | floor)]), p95: (sort | .[((0.95 * (length - 1)) | floor)]), max: max}
            }],
            resource: .resource, startup_ms: .startup_ms,
            cpu_model: .cpu_model, cpu_cores: .cpu_cores, os: .os}' "$TMP_DIR/pages.json")
        RESOURCE_JSON=$(echo "$PAGES_JSON" | jq -c '.resource')
        STARTUP_MS=$(echo "$PAGES_JSON" | jq -c '.startup_ms')
        CPU_MODEL=$(echo "$PAGES_JSON" | jq -r '.cpu_model')
        CPU_CORES=$(echo "$PAGES_JSON" | jq -r '.cpu_cores')
        echo "  $(echo "$PAGES_JSON" | jq -r '[.scenarios[] | "\(.scenario): total p95 \(.stages.total_ms.p95)ms"] | join(", ")')" | tee -a "$REPORT_TXT"
        echo "  resource: peak_rss=$(echo "$RESOURCE_JSON" | jq -r '.peak_rss_mb')MB startup=$(echo "$STARTUP_MS")ms" | tee -a "$REPORT_TXT"
    fi
else
    CPU_MODEL="unknown"
    CPU_CORES="unknown"
fi

# 测量后负载校验（2026-08-08：共享机器上另一条流的 WPT 全量可能中途叠加——
# 报告标记 suspect=true 供 perf-gate.sh 提示「结果不可信」，不参与收紧）
SUSPECT=false
if [ "$QUICK_MODE" != "1" ]; then
    LOAD1_AFTER=$(cut -d' ' -f1 /proc/loadavg 2>/dev/null | cut -d. -f1 || echo 0)
    if [ -n "$LOAD1_AFTER" ] && [ "$LOAD1_AFTER" -gt "$BUSY_THRESHOLD" ]; then
        SUSPECT=true
        echo "[WARN] 测量期间系统负载升高（loadavg=$LOAD1_AFTER），报告标记 suspect（可能被另一条流的 WPT/测试叠加污染）" | tee -a "$REPORT_TXT"
    fi
fi

# ---------- 组装报告 ----------
MICROBENCHES_ARR=$(jq -s -c . "$MICROBENCHES_JSONL")
jq -n \
    --argjson schema_version 1 \
    --argjson git_dirty "$GIT_DIRTY" \
    --arg generated_at "$DATE" \
    --arg git_sha "$GIT_SHA" \
    --argjson config_hash "\"$CONFIG_HASH\"" \
    --arg platform_class "$PLATFORM_CLASS" \
    --arg os "$OS_NAME" \
    --arg cpu_model "${CPU_MODEL:-unknown}" \
    --argjson cpu_cores "$(echo "${CPU_CORES:-0}" | grep -qE '^[0-9]+$' && echo "${CPU_CORES:-0}" || echo 0)" \
    --argjson bench_list "$(jq -R -s 'split("\n") | map(select(length>0))' <<<"$BENCH_LIST_SORTED")" \
    --argjson scenarios "$(jq -R -s 'split(",") | map(select(length>0))' <<<"$SCENARIOS_SORTED")" \
    --argjson iterations "$PAGE_ITERATIONS" \
    --argjson microbenches "$MICROBENCHES_ARR" \
    --argjson pages "$PAGES_JSON" \
    --argjson resource "$RESOURCE_JSON" \
    --argjson startup_ms "$STARTUP_MS" \
    --argjson suspect "$SUSPECT" \
    '{schema_version: $schema_version, kind: "bench-report", generated_at: $generated_at,
      git_sha: $git_sha, git_dirty: $git_dirty, suspect: $suspect,
      run_config: {config_hash: $config_hash, profile: "release", bench_list: $bench_list,
                   page_scenarios: $scenarios, viewport: [800, 600], iterations: $iterations},
      platform: {platform_class: $platform_class, cpu_model: $cpu_model, cpu_cores: $cpu_cores, os: $os},
      microbenches: $microbenches, pages: $pages.scenarios, resource: $resource, startup_ms: $startup_ms}' \
    > "$REPORT_JSON"

# ---------- 摘要 + 退出码 ----------
echo "" | tee -a "$REPORT_TXT"
echo "=== Summary ===" | tee -a "$REPORT_TXT"
echo "Passed: ${#PASSED[@]} / ${#BENCH_CRATES[@]} (microbenches)" | tee -a "$REPORT_TXT"
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed: ${FAILED[*]}" | tee -a "$REPORT_TXT"
fi
echo "Report saved to: $REPORT_JSON" | tee -a "$REPORT_TXT"

if [ ${#FAILED[@]} -gt 0 ]; then
    exit 1
fi
