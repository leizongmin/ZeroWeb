#!/usr/bin/env bash
# Goal: css-testharness-compat（编号 35——推进顺序见同目录 README.md）
# 解锁 css/* 目录 testharness 面（coverage-2026-10-27 对账：css topdir
# 195,173 子测试 / 89 目录，覆盖 0——与 css-variables 病态案无关，主体是
# testharness 形态语料未导入；本地 wpt-data/css/ 各目录只有 reftest 清单
# 文件，渲染账归 rendering-compat 不在本 coverage 口径）。
#
# 本脚本预置 M0 试点两目录：css/css-values（10,364 子测试 / 519 文件，值
# 与单位语义——贴近引擎 css 值解析强项）+ css/selectors（6,229 / 833，选择
# 器匹配 API 面）。试点产出「通道 → 基线 → 分簇」模板后再按 top15 顺序
# 扩面（里程碑见契约 docs/goal/css-testharness-compat.md）。
#
# runner 通道（子命令/skip 规则/Makefile target）属 M0 rally 轮落地内容；
# 长套件内存治理（css-variables 全语料单进程 >8.8GB 三跑被杀的根因）是
# M1 前置——治理落地前扩面套件进不了 CI coverage。
# 用法：bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh
# （FORCE=1 强制重拉；GITHUB_TOKEN 认证列目录）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="css-testharness-compat"
GOAL_FETCH_DEEP=1
DIRS=(
  "css/css-values"
  "css/selectors"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
