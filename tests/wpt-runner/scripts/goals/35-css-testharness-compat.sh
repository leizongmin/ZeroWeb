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
  # M3 扩面首批（top15 分母目录过 M0 模板）：css-color（12,714 子测试 /
  # getComputedStyle 颜色序列化）+ css-align（5,311 / box alignment）+
  # css-sizing（6,258 / sizing 属性）。三目录均须 FORCE=1 首拉（精选仓已含
  # .html 会走深挖幂等快路径，css-variables M2 先例）。
  "css/css-color"
  "css/css-align"
  "css/css-sizing"
  # M3 扩面批次 2（top15 分母目录过 M0 模板）：css-typed-om（13,445 子测试 /
  # CSS Typed OM API 面）+ css-fonts（8,665 / @font-face 匹配）+ css-shapes
  # （7,527 / shape-outside）。typed-om 与 shapes 本地原为空目录（深挖全量
  # 首拉），fonts 精选仓已含 476 页（深挖幂等快路径会漏——首拉后如需补差
  # 按 blob hash 定向，FORCE=1 会连带 1,650 个字体资产全量重拉，不划算）。
  "css/css-typed-om"
  "css/css-fonts"
  "css/css-shapes"
  # M3 扩面批次 3（top15 分母目录过 M0 模板）：css-anchor-position（13,657
  # 子测试 / anchor positioning API 面）+ css-masking（6,399 / mask 与
  # clip-path 断言）+ css-backgrounds（6,223 / background/border 解析序列
  # 化）。anchor-position 与 masking 本地为空目录（深挖全量首拉）；
  # backgrounds 精选仓已含存量页（幂等快路径会漏差——补差口径同 fonts，
  # 按 pin 树清单 blob hash 定向，不 FORCE 全量重拉）。
  "css/css-anchor-position"
  "css/css-masking"
  "css/css-backgrounds"
  # M3 扩面批次 4：css-transforms（5,516 子测试 / transform 解析序列化）+
  # motion（4,959 / offset-* 运动路径——上游目录名 css/motion 无 css- 前缀，
  # runner 扫描根同名单点）。两目录均深挖全量首拉（本地原为空/精选仓子集
  # ——transforms 若已含顶层 .html 会走幂等快路径，补差口径同 fonts）。
  "css/css-transforms"
  "css/motion"
  # css 面共享 helper 目录（/css/support/parsing-testcommon.js 等——runner
  # script fetcher 按绝对路径解析到 wpt-data/css/support；selectors focus-visible
  # 解析族 32 子测试依赖。css-testharness-compat M0 试点实测补拉）。
  "css/support"
)

goals_fetch_all

# 根级 support 单文件定向补拉（不入 DIRS——上游根 resources/ 目录体量大，按需
# 点拉）：css-align/css-sizing 等目录 200+ 页 testharness 用 data-* 布局断言库
# check-layout-th.js（自包含，仅依赖 testharness.js）。M3 基线分簇实证缺它 →
# 页脚报 checkLayout undefined（css-sizing 29 子测直接报错 + 挂起超时簇）。
# testdriver*.js 不在此列：runner 装配层对 testdriver script 标签置空注入
# （testharness.rs 装配管道），本地无需落盘。
fetch_raw "resources/check-layout-th.js" \
  || echo "  ⚠ resources/check-layout-th.js 拉取失败（重跑脚本幂等续拉）" >&2

goals_inventory
goals_next_steps "${GOAL}"
