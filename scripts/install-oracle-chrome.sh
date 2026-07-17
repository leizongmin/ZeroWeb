#!/usr/bin/env bash
# 安装 oracle 截图用 chromium（rendering-compat reftest-oracle / legacy-html fixture oracle）。
#
# 背景：系统 /usr/bin/chromium 在本机 WSL2 kernel 6.6 上 SIGTRAP（exit 133，chromium 150
# 特有；--headless / --headless=new / --no-zygote --single-process 全崩，见 master.md R1641/R1649）。
# 解法：用 @puppeteer/browsers 下载 chrome-for-testing 稳定旧版（chrome 127 实测在本机
# WSL2 kernel 6.6 上正常渲染 + 截图，exit 0，puppeteer screenshot 像素正确）。
#
# 安装位置：$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome（home 跨 session 持久）。
# 截图脚本经 `PUPPETEER_EXECUTABLE_PATH` 环境变量读取该路径（capture-chromium-screenshots.mjs
# / capture-legacy-oracle.mjs / chromium-oracle-shot.mjs 等均支持）。
#
# 用法：
#   bash scripts/install-oracle-chrome.sh           # 下载 + 持久化
#   source scripts/install-oracle-chrome.sh         # 同时 export PUPPETEER_EXECUTABLE_PATH
# 依赖：node + @puppeteer/browsers（npx --yes 自动拉取）+ 网络（~/use-proxy 代理可设）。
set -euo pipefail

CHROME_VERSION="${ZW_ORACLE_CHROME_VERSION:-127.0.6533.119}"
INSTALL_DIR="$HOME/.cache/zw-oracle-chrome"
CHROME_BIN="$INSTALL_DIR/chrome-linux64/chrome"

if [ -x "$CHROME_BIN" ] && "$CHROME_BIN" --version >/dev/null 2>&1; then
  echo "[install-oracle-chrome] already installed: $("$CHROME_BIN" --version 2>/dev/null | head -1)"
else
  echo "[install-oracle-chrome] downloading chrome-for-testing $CHROME_VERSION → $INSTALL_DIR ..."
  # 代理（~/use-proxy 设的是无 scheme 的 host:port；npm/node 需要 http:// 前缀）
  if [ -n "${http_proxy:-}${HTTP_PROXY:-}" ]; then
    case "$http_proxy" in
      http://*|https://*) ;;
      *) export http_proxy="http://$http_proxy"; export https_proxy="http://$http_proxy" ;;
    esac
  fi
  TMP="$(mktemp -d)"
  npx --yes @puppeteer/browsers@latest install "chrome@$CHROME_VERSION" --path "$TMP" >/dev/null
  rm -rf "$INSTALL_DIR"
  mkdir -p "$INSTALL_DIR"
  mv "$TMP"/chrome/linux-*/chrome-linux64 "$INSTALL_DIR/"
  rm -rf "$TMP"
  echo "[install-oracle-chrome] installed: $("$CHROME_BIN" --version 2>/dev/null | head -1)"
fi

# 导出供截图脚本使用（source 时生效；直接运行时打印提示）
export PUPPETEER_EXECUTABLE_PATH="$CHROME_BIN"
if [ "${1:-}" != "--silent" ]; then
  echo "[install-oracle-chrome] export PUPPETEER_EXECUTABLE_PATH=$CHROME_BIN"
fi
