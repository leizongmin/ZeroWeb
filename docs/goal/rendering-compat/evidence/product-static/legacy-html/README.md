# Legacy Static Web smoke — DC-13 Tier 1（HTML 3.2/4 + CSS1/2）

> **R1650/R1651 建立**。用户 2026-06-26 优先级：HTML 3.2/4 + CSS1/2 老式静态页「可读、布局
> 不崩、核心语义可见」。本 fixture 集是 DC-13 Tier 1 的产品 smoke 验收面，**不替代** WPT reftest
> 或 DC-14 达标口径，作为短期修复优先级 + 回归门禁（trend-only）。

## 运行

```bash
# 确保 oracle 抓取用 chrome-for-testing 127（系统 chromium 150 在 WSL2 kernel 6.6 上 SIGTRAP）
bash scripts/install-oracle-chrome.sh
make product-smoke-legacy
```

- `make product-smoke-legacy` → `run-all.sh` → 逐 fixture 跑 `zero-wpt-runner product-smoke
  <fixture> --oracle <png> --struct-check`，报告 diff% + struct 状态。
- **trend-only（exit 0）**：diff% 是 font-wall baseline 数据（fontdue 行度量 vs chromium，
  R633 多会话 plateau），非 pass/fail；struct-check FAIL 是「待查清单」入口（真实结构性发现），
  不阻 CI。goal line 318-320 短期验收口径 = 「可读且结构不崩」。

## fixture 集（20 页）

`fixtures/*.html` — 每页隔离一个 legacy 特性：

| # | 覆盖 | 关键属性 |
|---|------|---------|
| 01 | body attrs | `bgcolor text link vlink alink` |
| 02 | table border | `<table border>` |
| 03 | cellpadding | `cellpadding` |
| 04 | cellspacing | `cellspacing` |
| 05 | tr bgcolor | `<tr bgcolor>` |
| 06 | td bgcolor+align | `<td bgcolor align>` |
| 07 | valign | `td valign=top/middle/bottom` |
| 08 | img align | `<img align=top/bottom>` |
| 09 | img size | `<img width height>` |
| 10 | font size | `<font size=1..7>` + 相对 `+2/-1` |
| 11 | font color/face | `<font color face>` |
| 12 | headings | `h1`-`h6` |
| 13 | lists | `ul/ol/li` |
| 14 | hr | `<hr>` + `size/width/align/noshade` |
| 15 | blockquote | `<blockquote>` |
| 16 | links | `<a>` + link/vlink 色 |
| 17 | center | `<center>` 元素（HTML4 块级） |
| 18 | nested table | 表套表 |
| 19 | testpage-minimal | 综合：导航条 + 表格 + font/b/i/a/img + copyright（goal line 319 代表页）|
| 20 | mixed-legacy | 综合布局：菜单 + 正文 + 嵌套表 + footer |
| 21 | dl/dt/dd | 定义列表 |
| 22 | ol attrs | `ol start type` + `li value type` |
| 23 | rowspan/colspan | 表格合并单元格 |
| 24 | img presentational | `<img border hspace vspace align>` |
| 25 | table width/height | `<table width height>` + `<td width>` |
| 26 | pre | `<pre>` 预格式化 |
| 27 | address | `<address>` + `<br>` |
| 28 | div align | `<div align>` |
| 29 | inline style | CSS1 `style=` 基础 |
| 30 | table sections | caption + colgroup + thead/tbody/tfoot |

## oracle

`oracle/*.png` — chrome-for-testing 127 截图（800×600）。重抓：

```bash
export PUPPETEER_EXECUTABLE_PATH="$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome"
for f in fixtures/*.html; do
  n=$(basename "$f" .html)
  node ../../../../../tests/wpt-runner/scripts/capture-legacy-oracle.mjs "$f" "oracle/$n.png"
done
```

## 当前基线（2026-07-18，R1653）

- **29/30 struct-check PASS**（avg diff 2.79%，font-wall baseline），30 fixtures 覆盖 HTML 3.2/4 + CSS1/2。
- **17-center** R1651 修复（`<center>` UA display:block，HTML4 块级）。
- **19-testpage-minimal** R1652 修 check 误报（check_text_concatenation 跳过 table-internal）。
- **30-table-sections** R1653 修复 `<caption>` 定位（caption-side:top 须让 rows 下移 caption 高度，
  否则 caption 与 thead overlap）：diff 12.95%→**4.45%**，struct FAIL→PASS。新增 `top_caption_extent`
  helper（position_cells + update_row_group_positions 一致偏移）。
- **27-address** struct FAIL（0.96%）：body/html h=0 未随单 block 子（address h=74）长高——root
  html/body 高度传播在「单 block 子 + body margin」场景的 niche bug，**视觉无影响**（fixture 无 body
  bg，address 内容正确渲染于 abs_y=8 h=74），待 dedicated 调查（html/body root sizing）。

## 新增 fixture

写入 `fixtures/`，对应 oracle PNG 写入 `oracle/`（命名一致）。run-all.sh 自动 glob `fixtures/*.html`。
