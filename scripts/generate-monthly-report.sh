#!/usr/bin/env bash
# 月度工程报告生成脚本（调研 P6/C2，对照 Ladybird「This Month in Ladybird」）
#
# 从 git 历史 + WPT 趋势数据自动生成月度报告，落盘 docs/monthly/YYYY-MM.md。
# 报告构成：
#   1. WPT 趋势（读 evidence/wpt-trends/trend.csv 当月记录——绝对数口径）
#   2. 当月提交统计（git log 按 feat/fix/docs/chore/其它 分组）
#   3. 里程碑列表（当月提交标题，按日期倒序）
#   4. 决策与亮点（占位，供人工补充）
#
# 用法：
#   bash scripts/generate-monthly-report.sh            # 上月（默认）
#   bash scripts/generate-monthly-report.sh 2026-07    # 指定月份
#   bash scripts/generate-monthly-report.sh --dry-run  # 只打印将生成的路径

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${REPO_ROOT}/docs/monthly"
TREND_CSV="${REPO_ROOT}/docs/goal/rendering-compat/evidence/wpt-trends/trend.csv"

MONTH="${1:-}"
DRY_RUN=false
if [[ "$MONTH" == "--dry-run" ]]; then
  DRY_RUN=true
  MONTH="${2:-}"
fi
if [[ -z "$MONTH" ]]; then
  MONTH=$(date -d "$(date +%Y-%m-01) -1 month" +%Y-%m 2>/dev/null || date +%Y-%m)
fi

OUT_FILE="${OUT_DIR}/${MONTH}.md"

if [[ "$DRY_RUN" == "true" ]]; then
  echo "将生成: ${OUT_FILE}"
  exit 0
fi

mkdir -p "$OUT_DIR"

# ── 1. WPT 趋势（当月 trend.csv 记录）──
trend_block="（当月无 trend.csv 记录——先运行 make reftest-trend）"
if [[ -f "$TREND_CSV" ]]; then
  month_records=$(grep "^${MONTH}-" "$TREND_CSV" | grep -v "^#" || true)
  if [[ -n "$month_records" ]]; then
    trend_block="| 日期 | 模式 | 套件 | total | passed | rate | 备注 |"
    trend_block+=$'\n|---|---|---|---|---|---|---|'
    while IFS= read -r line; do
      IFS=',' read -r date mode ref total passed rate extra sha note <<< "$line"
      trend_block+=$'\n'"| ${date} | ${mode} | ${ref} | ${total} | ${passed} | ${rate}% | ${note} |"
    done <<< "$month_records"
  fi
fi

# ── 2. 当月提交统计 ──
month_start="${MONTH}-01"
month_end=$(date -d "${month_start} +1 month" +%Y-%m-01 2>/dev/null || echo "${MONTH}-31")
total_commits=$(git -C "$REPO_ROOT" log --since="${month_start}" --until="${month_end}" --oneline | wc -l)
feat_count=$(git -C "$REPO_ROOT" log --since="${month_start}" --until="${month_end}" --grep="^feat" --oneline | wc -l)
fix_count=$(git -C "$REPO_ROOT" log --since="${month_start}" --until="${month_end}" --grep="^fix" --oneline | wc -l)
docs_count=$(git -C "$REPO_ROOT" log --since="${month_start}" --until="${month_end}" --grep="^docs" --oneline | wc -l)
chore_count=$(git -C "$REPO_ROOT" log --since="${month_start}" --until="${month_end}" --grep="^chore" --oneline | wc -l)
other_count=$((total_commits - feat_count - fix_count - docs_count - chore_count))

# ── 3. 里程碑列表（提交标题，倒序）──
milestones=$(git -C "$REPO_ROOT" log -n 40 --since="${month_start}" --until="${month_end}" --pretty=format:"- %ad %s" --date=format:"%m-%d")

# ── 写文件 ──
cat > "$OUT_FILE" <<EOF
# ZeroWeb 月度工程报告 — ${MONTH}

> 生成时间：$(date +%F)（scripts/generate-monthly-report.sh 自动生成 + 人工补充）

## 1. WPT 趋势（绝对数口径）

${trend_block}

## 2. 提交统计

- 总提交：${total_commits}
- feat：${feat_count} ｜ fix：${fix_count} ｜ docs：${docs_count} ｜ chore：${chore_count} ｜ 其它：${other_count}

## 3. 里程碑（提交标题）

${milestones}

## 4. 决策与亮点（人工补充）

- （记录本月用户拍板事项、架构决策、关键技术修复，如 A1 分阶段里程碑 / D1 图像解码独立进程）
EOF

echo "已生成: ${OUT_FILE}"
