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
| 31 | nested list | ul/ol 嵌套三层 |
| 32 | table align | `<table align=center/right>` |
| 33 | css1 float | `float:left/right` + `clear` 基础 |
| 34 | br + phrasing | `<br>` + b/i/u/strong/em/small/big/tt/sub/sup/code/samp/kbd/var |
| 35 | xmp/listing/plaintext | HTML4 obsolete raw-text 块级元素（≡ `<pre>`，内容字面渲染）|
| 36 | isindex | HTML3.2 obsolete `<isindex prompt>` |
| 37 | form controls | input(text/password/checkbox/radio/submit/reset)/button/select/textarea/fieldset/legend/label |
| 38 | noframes | `<noframes>` 帧不支持回退内容（display:none in frame-capable UAs）|
| 39 | menu | `<menu><li>` 菜单列表（HTML UA 块级）|

### R1657 `<noframes>` display:none 修复

fixture 38 struct PASS 但 diff 2.74% 异常（高于同类短文本 fixture）。**像素 y-band 实测定位**：
ZW 渲染 5 行文本（2 段 + noframes 回退 3 行，y=19/54/73/92/125）vs chromium 仅 2 段（y=10/44）——
ZW **误渲染 `<noframes>` 回退内容**（chromium 等 frame-capable UA 按 HTML 渲染规范隐藏）。
**fix**：`ua_default_display` display:none arm 加 `noframes`（与 `noscript` 同列——脚本启用时隐藏 noscript、
帧支持时隐藏 noframes，语义完全平行）。fixture 38 diff **2.74%→1.34%**（13170→6426 px）。
load-bearing 单测 `test_hidden_elements_default_to_none` 钉死 None-arm 含 noframes/noscript。

### R1658 `<pre>`/`<xmp>`/`<listing>`/`<plaintext>` UA white-space:pre（spec correctness）

R1656 forward finding：ZW `default_impl.rs` `white_space` 默认 `Normal` 全元素，故 `<pre>` 此前
折叠空白/换行（真 bug——pre-family 应按 HTML 渲染规范 `white-space: pre` 保真）。**fix**：
`ua_decl_inputs`（UA 默认样式机制，≡ h1-h6/p/hr）加 `pre|xmp|listing|plaintext` → `white-space:pre`。
fixture 26-pre **1.06%→1.00%**、35-xmp **3.94%→3.85%**（bbox diff 8,54,335,121 证白空格保真生效）。
**css-text 全 1644 oracle A/B net-0**（433/1644 = 26.3% ↔ 26.3%，零回归——test-guard --per-proc-mem 12
绕过默认 6GB 跨用例累积上限）。load-bearing 单测 `pre_family_gets_white_space_pre_from_ua`。
**monospace 字体未加**（font-wall 高方差，单独 A/B 切片）。

## oracle

`oracle/*.png` — chrome-for-testing 127 截图（800×600）。重抓：

```bash
export PUPPETEER_EXECUTABLE_PATH="$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome"
for f in fixtures/*.html; do
  n=$(basename "$f" .html)
  node ../../../../../tests/wpt-runner/scripts/capture-legacy-oracle.mjs "$f" "oracle/$n.png"
done
```

## 当前基线（2026-07-18，R1666）

- **39/41 struct-check PASS**（avg diff 2.83%，font-wall baseline），41 fixtures 覆盖 HTML 3.2/4 + CSS1/2。
- **R1666 chrome-127 oracle 捕获可用**（系统 chromium 150 SIGTRAP，但 chrome-for-testing 127 经
  `PUPPETEER_EXECUTABLE_PATH` 正常——重抓 oracle 用 `PUPPETEER_EXECUTABLE_PATH=$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome`）。
- **40-phrase-elements** R1666 新增（b/i/u/em/strong/code/big/small/tt/kbd/samp/var/sub/sup/q/cite/dfn/abbr/acronym），
  struct PASS（diff 5.63%，残余 = sub/sup vertical-align + code/tt monospace 字体墙）。
- **41-obsolete-elements** R1666 新增 + 修 `<dir>` UA display:block（≡ ul 块级列表；smoke 抓到第五个真 legacy bug：
  dir 误渲染 4×60 inline 盒 sibling-overlap → R1666 加 ua_default_display block 列表 → struct FAIL→PASS，diff 4.56%）。
- **37-form-controls** R1659 修复 `<input>` 固有尺寸（7.03%→4.25%）+ R1660 修复 `<input>` value 文本渲染
 （4.25%→4.33%，+0.08pp font-wall 噪声但语义正确——按钮/输入框标签与值现在可见 = 「核心语义可见」）。
  struct 仍 FAIL（残余 = R109 inline-`<label>` 含 inline-block `<input>` 被拆成 block 盒的已知 entanglement）。
- **17-center** R1651 修复（`<center>` UA display:block，HTML4 块级）。
- **19-testpage-minimal** R1652 修 check 误报（check_text_concatenation 跳过 table-internal）。
- **30-table-sections** R1653 修复 `<caption>` 定位（caption-side:top 须让 rows 下移 caption 高度，
  否则 caption 与 thead overlap）：diff 12.95%→**4.45%**，struct FAIL→PASS。新增 `top_caption_extent`
  helper（position_cells + update_row_group_positions 一致偏移）。
- **27-address** struct FAIL（0.96%）：body/html 高度传播 bug——**child-count 相关不一致**：
  单 content-driven block 子 → body h=0（reproB/D 实证）；2 子 → body h=首子高仅（reproC）；
  3+ 子 → body 正确长高（fixture 01 body h=154 + bgcolor 填满视口，pixel(400,550)=(255,255,238)
  正确）。根因 = auto-height body 的 content-driven 子经 IFC 长 high 后，**父高度回填不一致**
 （部分子的高增长未加回 body；显式 height 子不受影响——reproA `height:74px` body h=74 正确）。
  **视觉影响 minimal**：内容经子盒正确渲染；body bg/border 仅在「单/双 content-driven 子 + body 有 bg」
  时漏涂（罕见：多数实际页多子 → body 正确长高 + bg 填满）。**修复高风险**：parent-height propagation
  经 R1047（sibling-push）/ R109 BACKFILL 多轮 net-negative（margin-collapse 交互），scoped fix 须
  守已工作的多子案 + 不破 margin-collapse，待 dedicated 多 session。trend-only smoke 不阻（exit 0）。
- **35-xmp-listing-plaintext** R1656 修复（`<xmp>`/`<listing>`/`<plaintext>` UA display:block，
  HTML 渲染规范 raw-text 块级元素 ≡ `<pre>` 谱系）：struct FAIL→PASS。修复前 ZW 把这三元素当 inline
  （listing 盒仅 83px 宽）致与后续 block sibling overlap。**残余 3.94% diff** = font-wall + 一个独立
  pre-existing gap（ZW 未对 `<pre>`/`<xmp>`/`<listing>`/`<plaintext>` 应用 `white-space:pre` + monospace
  —— `default_impl.rs` white_space 默认 `Normal` 全元素，pre 本身亦然，fixture 26 pre 同谱系；
  该 gap 属 font-wall/white-space 范畴，独立多 session，非本轮 scoped 修复）。
- **37-form-controls** struct FAIL（7.03%→**4.25%** 经 R1659；残余 struct FAIL = R109，见下）：
  **R1659 已修** `<input>` 固有尺寸（void inline-block 无固有尺寸时 ZW 把 auto 宽当全容器宽 = 784×6）：
  按 `type` 注入 UA width/height——文本类按 `size`（默认 20）估宽 ~148px + 15px 内容高；
  checkbox/radio/color 固定 13px 方框；submit/reset/button 按 `value` 字符数估宽。select/textarea 已按
  内容（option/文本子节点）正确测宽，不加 width。**A/B**（CSS2 6226/css-flexbox 497/css-grid 49 三 dir，
  含全 corpus `<input>` 文件：CSS2 54 / grid 6 / flex 4）**bit-identical net-0**（UA lowest-priority，
  WPT reftest 罕用 bare input）。
  **R1660 已修** `<input>` value 文本渲染（paint 侧，form-control slice-2）：void `<input>` 无 DOM 文本子节点，
  value 属性此前不渲染。R1660 按 `type` 绘 value——submit/reset/button value（默认 "Submit"/"Reset"）水平居中；
  text 类 value 左对齐；password value 渲为 `•`；checkbox/radio/hidden/range 等不绘。**A/B** 三 dir
  （CSS2/css-flexbox/css-grid）**bit-identical net-0**；fixture 37 diff 4.25%→4.33%（+0.08pp font-wall glyph
  噪声，trend-only），但按钮/输入框标签与值现可见（「核心语义可见」验收口径优先于 font-wall pixel 噪声）。
  **残余 struct FAIL（5 issue，非 form-control 缺口）= R109 entanglement**：inline `<label>` 含 inline-block
  `<input>` 子被 ZW 拆成 block 盒 → 同父 label 垂直堆叠 overlap（3）+ `<p>` IFC 吸收 block 子文本
  concatenation（2）。此为 inline-box-model Phase-A 已知硬 vein（R125/R198/R205 谱系 net-negative），
  独立于 form-control 固有尺寸/value 渲染，需 IFC 统一解，非 scoped slice。trend-only smoke 不阻（exit 0）。

## 新增 fixture

写入 `fixtures/`，对应 oracle PNG 写入 `oracle/`（命名一致）。run-all.sh 自动 glob `fixtures/*.html`。
