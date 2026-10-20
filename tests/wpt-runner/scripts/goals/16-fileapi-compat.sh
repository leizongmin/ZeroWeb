#!/usr/bin/env bash
# Goal: fileapi-compat（编号 16——推进顺序见同目录 README.md）
# Blob/File/FileReader/objectURL 面（net-api M2 打过 Blob.type 底，从未 goal 化度量）
# M1 资产预置：fetch FileAPI 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/archive/fileapi-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/16-fileapi-compat.sh（FORCE=1 强制重拉）
#
# **拉取面（2026-10-11 M1 rally 轮定稿）**：lib.sh fetch_dir_html 递归一层覆盖
# FileAPI 全部子目录（blob/ file/ FileReader/ reading-data-section/ url/ BlobURL/
# filelist-section/ support/）；`FileAPI/url/resources` 两层深显式追加（multi-global
# helper：create/revoke-helper.js）。`.any.js` window 变体拉取（GOAL_PULL_ANY_JS=1）。
#
# **显式不拉（记账，重入条件见括号）**：
# - `*.worker.js`（lib.sh 默认排除——dedicated worker harness；FileReaderSync/
#   idlharness/Blob-in-worker/Worker-read-file-constructor 四案，worker 面归
#   workers-compat 管道）
# - `*.py` 服务端点（lib.sh 模式外——file/resources/echo-content-escaped.py；依赖它
#   的 send-file-form*.html 表单提交回显面 runner 侧 skip 规则排除，无 wptserve）
# - `.txt/.png/.zip` 数据资产（lib.sh 模式外——upload.zip/upload.txt/file_test1.txt/
#   blue-100x100.png；消费者全是 manual/worker/filelist 真实选档面，可执行子集外）
# - idlharness.*（依赖 /resources/WebIDLParser.js build 期生成资产，window 可执行
#   面之外——clipboard idlharness 先例）；`*.https.html`（BlobURL 分区面 +
#   historical，存储分区/HTTPS 前提 runner 无）；`*-manual.html`（真实用户交互）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="fileapi-compat"
# 两层深的 resources（lib.sh 只递归一层）显式追加；"FileAPI" 递归覆盖其余子目录。
DIRS=( "FileAPI" "FileAPI/url/resources" "FileAPI/BlobURL/resources" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
