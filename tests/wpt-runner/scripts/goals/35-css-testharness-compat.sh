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
# runner 通道已落地（M0）：子命令 `testharness-css-values` / `testharness-selectors`
# （tests/wpt-runner/src/testharness_css_corpus.rs），运行面按内容规则筛减
# （`css_testharness_case_skipped`：manual / crash / 病态内存案名 / .https /
# helper 资产目录 / 非 testharness reftest 页 / iframe 依赖）——本脚本拉取面
# 与 runner 谓词两侧同源防漂移（html-semantics 先例）。全量跑走 Makefile 分片
# 通道 `make testharness-css-values` / `make testharness-selectors` /
# `make testharness-css-variables`（每案独立进程，run-css-testharness-corpus.sh）；
# 定向走 -one + FILTER。
# 内存治理（M1）已收口：单进程 RSS 无线性累积（归因实验否证立项前提，
# 8.8GB 实为病态案 4.9GB 抬升笼水位的平摊误读，见 goal evidence
# 2026-10-30-m1-rss-attribution.md）；病态案由 runner 谓词名级排除（M2 起
# `variable-exponential-blowup.html` 与 `*-crash.html` 同单点），css-variables
# 解禁进 CI coverage misc 分片（M2 / DC-3）。
# 注意 css/css-variables 本地数据源：fetch-wpt-data 精选仓（v1.10）只含 reftest
# 口径子集（harness 形态 59 页），上游 pin 全量 harness 面（250 文件）须本脚本
# GOAL_FETCH_DEEP 深挖补拉（FORCE=1 首次补拉——目录已含 .html 会走幂等快路径）。
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
  # M2 解禁（css-testharness-compat DC-3）：病态案 runner 谓词排除后
  # css-variables harness 面入账。数据双源——fetch-wpt-data 精选仓的 reftest
  # 子集 + 本脚本深挖的上游全量（首次须 FORCE=1 补拉缺失 harness 页）。
  "css/css-variables"
  # css 面共享 helper 目录（/css/support/parsing-testcommon.js 等——runner
  # script fetcher 按绝对路径解析到 wpt-data/css/support；selectors focus-visible
  # 解析族 32 子测试依赖。css-testharness-compat M0 试点实测补拉）。
  "css/support"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
