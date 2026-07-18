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
| 40 | phrase elements | b/i/u/em/strong/code/big/small/tt/kbd/samp/var/sub/sup/q/cite/dfn/abbr/acronym（R1666）|
| 41 | obsolete elements | `<dir>`/`<menu>`/`<marquee>` obsolete 块级/行内（R1666）|
| 42 | special inline | `<marquee>`/`<noembed>`/`<nobr>`/`<wbr>`/`<bgsound>`（R1667）|
| 43 | replaced elements | `<object>`/`<embed>`/`<applet>` + `<param>`（R1668）|
| 44 | image map | `<map>`/`<area>` 客户端图像映射（R1669；area→display:none）|
| 45 | progress / meter / keygen | 废弃/替换类表单控件（R1669；keygen 固有尺寸，progress/meter forward Bug C）|
| 46 | frameset / frame | frameset 帧模式探测（R1669；frame→display:none，帧网格渲染 unsupported）|
| 47 | datalist / misc | datalist/source/track/optgroup/output/bdi/bdo（R1675；datalist+source+track→display:none）|

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

### R1669 image-map / form 控件 / frameset 探测（area+frame→display:none，keygen 固有尺寸）

承接 R1668 forward「续 legacy fixture + LAYOUT_DUMP 复核 image map / form 控件 / frameset」。
新增 3 fixture（44-image-map / 45-progress-meter-keygen / 46-frameset）+ chrome-127 oracle。**LAYOUT_DUMP
深查（R1667 方法论——struct-check 只抓 overlap，display:none 类「不该出现的盒」须看 dump）抓到 3 类缺口**：

1. **`<area>` 误渲染**（fixture 44）：image map 的 `<area>` 渲成 6×24.6 inline 盒 ×3（@x=8/12/16 横排），
   致 `<map>` 容器报 collapsed container（h=0 < in-flow child h=25）。HTML 渲染规范 `area{display:none}`
   （area 仅定义 img 上可点击区，不渲染盒）。**fix**：`ua_default_display` None-arm + `area`。
2. **`<frame>` 误渲染**（fixture 46）：frameset 的 `<frame>` 渲成 6×24.6 盒 @**负 abs_y=-5.5**（frameset
   盒几何坏）。`<frame>` 是 nested browsing context（非普通 CSS 盒）；ZW 未实现 frameset 帧模式网格
   渲染。**fix**：None-arm + `frame`（避免断盒；frameset 帧网格本身是多 session 架构工作，forward）。
3. **`<keygen>` 塌缩**（fixture 45）：废弃 void 表单控件无固有尺寸 → 渲成 6×24.6 sliver → 包裹 `<p>`
   报 collapsed container h=0 < h=25 → struct FAIL。**fix**：InlineBlock UA list + `keygen` + `ua_decl_inputs`
   注入 bg/border/padding + width:90px height:24px（menulist 近似，≡ R1659 `<input>` / R1396 form-control
   谱系，最低优先级可被作者样式覆盖）。fixture 45 struct **FAIL→PASS**（keygen 现 96×30）。

**documented Bug C forward（本轮不修，progress/meter）**：`<progress>`(61.6×18)/`<meter>`(22.4×18) 现
按 fallback 文本宽渲染（应 inline-block 替换控件，chromium progress 10em×1em≈160×16 / meter 类似）。
progress/meter **含 fallback 文本子节点**（"60% done"/"30%"），chromium 替换元素渲染时**隐藏 fallback**
（仅不支持时显示）；ZW 若只加 InlineBlock+固有尺寸而不抑制 fallback，文本会溢出控件盒反而更差。
故 progress/meter 须**三步同修**（display:inline-block + 固有尺寸 + fallback-content 抑制），≡ R1668
object/embed/applet Bug B 谱系（替换元素 + fallback + sizing entanglement），多 session defer。
（progress/meter 不触发 struct FAIL——有 fallback 文本故 p 有高度；本轮 fixture 45 struct PASS。）

**A/B**：CSS2 oracle bit-identical **net-0**（6283 案，oracle-pass 4458/71.6% ↔ R1668 baseline 4458/71.6%
零变化——area/frame/keygen 在 WPT reftest 极罕见，≡ R1659 input / R1666-R1668 谱系）。load-bearing
单测 `test_hidden_elements_default_to_none` 钉死 None-arm 含 area/frame。

### R1675 `<datalist>`/`<source>`/`<track>` UA display:none（LAYOUT_DUMP+pixel 抓到第十三个真 legacy bug）·legacy fixture 47

承接 R1674 forward「pivot 回 legacy/UA-display vein」。本轮加 [`47-datalist-and-misc.html`](./fixtures/47-datalist-and-misc.html)
（datalist/source/track/optgroup/output/bdi/bdo）+ chrome-127 oracle。★ **LAYOUT_DUMP + pixel 采样抓 bug**：
① `<datalist>` 的 option 文本当 inline 渲染（pixel 采样 y=118 有暗像素 = option 文本 "apple/banana/cherry"
可见），chromium `datalist{display:none}` 完全移除（pixel 白）；② `<source>`/`<track>` 渲成 6×24.6 断盒
（media 子元素应无盒——source 提供 src / track 提供文本轨道），致 `<video>` collapsed-container h=0<25 +
source/track sibling overlap → **fixture 47 struct FAIL**。★ **fix**（[`lib.rs`](../../../../../crates/style-system/src/lib.rs)
None-arm）：加 `datalist`+`source`+`track` + 扩 `test_hidden_elements_default_to_none`（≡ R1667-R1669 noembed/
bgsound/param/basefont/area/frame 谱系）。fixture 47 LAYOUT_DUMP 复核 datalist+options 与 source/track 全消失，
struct **FAIL→PASS**。diff 6.93% 持平（source/track 透明无像素差；datalist 文本移除 ≈ 内容上移抵消；残余 =
font-wall + optgroup/output/bdi/bdo 文本）。★ **未修 optgroup**：fixture 47 standalone optgroup 仍渲染（w=17.6
+ options 堆叠），但 standalone optgroup 非标准（真实页 optgroup 总在 select 内）——select 内 optgroup/option
渲染属 select widget 域（= R1670/R1671 forward 的 replaced 子节点抑制架构缺口，非 display:none）。

**门禁**：fmt clean / clippy --workspace --all-targets -D warnings clean / make test 全 workspace 0 failed /
product-smoke welcome struct PASS 字节一致（零回归——welcome 无 area/frame/keygen）/ legacy smoke
46 fixture（44 struct PASS，2 known struct FAIL = 27-address + 37-form 不变；avg excl. 46-probe ≈2.99%）。

### R1670 `<progress>`/`<meter>` inline-block 固有尺寸（sizing 半，≡ R1659 input 谱系）

承接 R1669 forward ①「progress/meter 三步同修」。**研究（select/option 机制）关键发现**：ZW **无「replaced
元素抑制子节点 layout」机制**——`is_replaced`（engine.rs:1327）仅含 img/video/iframe/embed/object/svg/canvas
且只影响 sizing，不抑制子节点；`<select>` 的 `<option>` 同样当 inline 文本渲染（**latent gap，非 progress/meter
独有**）。故 progress/meter 的「fallback-content 抑制」须 tree.rs 加跨元素子节点抑制机制（影响 select/
object/embed/applet/progress/meter），多 session 架构工作，**本轮 defer**。

**本轮落地 sizing 半**（≡ R1659 input sizing → R1660 input value-paint 的两轮拆分之前半）：
`<progress>`/`<meter>` → InlineBlock UA list + `ua_decl_inputs` 注入 track 外观（border 1px + bg #d5d5d5 灰
track 近似）+ UA 固有尺寸。**chrome-127 oracle 实测**：progress x[8,167]=160px（value-fill 60%=96px ✓）、
meter x[8,87]=80px（value-fill 30%=24px ✓）→ progress **160×16**（chromium 10em×1em）、meter **80×16**
（5em），最低优先级 specificity(0,0,0) 可被作者样式覆盖。fixture 45 progress **61.6×18→162×18**（160+2 border）、
meter **22.4×18→82×18**（80+2 border），struct PASS。

**diff 4.47%→5.05%（+0.58pp，trend-only）**：ZW 现 track 盒 + fallback 文本（"60% done"）vs chromium track +
绿色 value bar。+0.58pp = fallback 文本 + 缺 value-bar 绘制（≡ R1660 input +0.08pp font-wall 噪声判例——
「核心语义可见」验收口径优先于 font-wall pixel 噪声：progress/meter 现**正确固有尺寸的可见 track 控件**，
非薄 sliver）。残余 diff 待 **paint 半**（R1671，≡ R1660）：paint_progress/meter_value 绘 value bar/gauge
（model `paint_input_value`）。

**forward（架构）**：① value-bar/gauge 绘制（paint 半，R1671）；② **replaced 元素子节点抑制机制**（tree.rs，
跨 select/object/embed/applet/progress/meter——ZW 当前无任何元素抑制子节点，select/option latent gap）；
③ object/embed/applet width/height attr sizing（R1668 Bug B，低 ROI）。

### R1671 `<progress>`/`<meter>` value 填充条绘制（paint 半，≡ R1660 paint_input_value；sizing+paint 两轮收尾）

承接 R1670 forward ①「value-bar/gauge 绘制（paint 半）」。**新增** [`controls.rs`](../../../../../crates/engine/src/paint/painter/controls.rs)
（`text.rs` 已 2438 行超 2000 行限，独立成文件 per CLAUDE.md §5）含 `paint_progress_meter_value`。
progress 按 value/max 比例绘填充条，meter 按 value/(max-min) 比例 + HTML §4.10.16 三区域算法（green/
yellow/red）着色。**chrome-127 oracle 实测颜色**：progress value `#0075FF`（accent 蓝，(0,117,255)）+
track `#EFEFEF`；meter green `(16,124,16)`（value=0.3 在 [low=0.2,high=0.8]，optimum=0.5 同段 → green）。
R1670 track bg `#d5d5d5`→`#efefef`（chrome 实测校正）。

**调用时序关键**：`paint_progress_meter_value` 须在 `paint_text` **之后**调用——bar 覆盖 fallback 文本
（ZW 无 replaced 子节点抑制机制，fallback 仍 layout+paint；bar 后绘覆盖之，近似 chromium 不显示 fallback）。
indeterminate progress（无 value 属性）不绘条。

**diff 5.05%→4.76%（-0.29pp，paint 半把 R1670 +0.58pp 收回近半）**：ZW 现 #0075FF progress 填充条（60%=96px）+
green meter 填充条（30%=24px）+ #efefef track，颜色精确匹配 chrome。残余 +0.29pp（vs R1669 baseline 4.47%）=
track 边框/高度 UA-appearance 保真（ZW 162×18 含 border vs chrome 160×16 recessed track），精确 UA-appearance
rendering 是 forward。

**A/B**：CSS2 oracle bit-identical **net-0**（6283 案，oracle-pass 4458/71.6% ↔ R1668/R1669/R1670 baseline 零变化）。
**门禁全绿**：fmt / clippy --workspace --all-targets -D warnings clean / make test 0 failed / product-smoke welcome
struct PASS 字节一致 / legacy smoke 仅 45 变化（4.76%）。**sizing+paint 两轮收尾**（progress/meter 现完整渲染：
track + value 条，≡ input R1659 sizing + R1660 value-paint 谱系）。

**forward**：① **replaced 元素子节点抑制机制**（tree.rs 架构，跨 select/object/embed/applet/progress/meter——
彻底隐藏 fallback 而非 bar 覆盖）；② track UA-appearance 精确保真（recessed 边框/高度）；③ object/embed/applet
width/height attr sizing（R1668 Bug B）。

## oracle

`oracle/*.png` — chrome-for-testing 127 截图（800×600）。重抓：

```bash
export PUPPETEER_EXECUTABLE_PATH="$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome"
for f in fixtures/*.html; do
  n=$(basename "$f" .html)
  node ../../../../../tests/wpt-runner/scripts/capture-legacy-oracle.mjs "$f" "oracle/$n.png"
done
```

## 当前基线（2026-07-18，R1675）

- **45/47 struct-check PASS**（avg diff excl. 46-frameset probe ≈ 3.04%，font-wall baseline），47 fixtures 覆盖
  HTML 3.2/4 + CSS1/2。2 known struct FAIL = 27-address（body/html 高度传播，R1047/R109 高风险 defer）+
  37-form-controls（R109 inline-`<label>` 含 inline-block `<input>` entanglement）。
- **R1671 progress/meter value 填充条绘制**（paint 半，≡ R1660 paint_input_value；sizing+paint 两轮收尾）：
  新增 `controls.rs::paint_progress_meter_value`，progress #0075FF / meter 三区域 green-yellow-red，chrome-127
  oracle 实测颜色精确匹配；track bg #d5d5d5→#efefef 校正。fixture 45 diff 5.05%→4.76%。
- **R1670 progress/meter sizing 半**（≡ R1659 input 谱系）：progress/meter → InlineBlock UA + 固有尺寸
  （progress 160×16 / meter 80×16，chrome-127 oracle 实测）+ track 外观。
- **R1669 新增 3 fixture + 3 display 修复**（详见下 R1669 段）：`<area>`+`<frame>`→display:none +
  `<keygen>`→inline-block UA + 固有尺寸（R1659 input 谱系）。
- **46-frameset 是 probe**（非 font-wall）：diff ~100% 是 inherent——chrome 用默认 canvas bg `#DDDDDD` +
  帧边框填满视口（帧 src 缺失仍渲染帧网格），ZW 未实现 frameset 帧模式网格渲染 → 空白 frameset。
  该 diff 反映「frameset unsupported」架构缺口，**不计入 font-wall 趋势**（avg excl. probe 才是趋势口径）。
  本轮 `<frame>`→display:none 修的是「frame 渲成 6×24.6 断盒 @负 y」真 bug（LAYOUT_DUMP 抓到）；
  frameset 帧网格渲染本身是多 session 架构工作，forward。

- **R1666 chrome-127 oracle 捕获可用**（系统 chromium 150 SIGTRAP，但 chrome-for-testing 127 经
  `PUPPETEER_EXECUTABLE_PATH` 正常——重抓 oracle 用 `PUPPETEER_EXECUTABLE_PATH=$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome`）。
- **42-special-inline** R1667 新增 + 修 `<noembed>`+`<bgsound>` UA display:none（LAYOUT_DUMP 深查抓到：
  noembed/bgsound 误渲染应 display:none——noembed ≡ noframes/noscript R1657 谱系，bgsound IE obsolete head/void 元素；
  struct-check 未报因无 overlap，须 LAYOUT_DUMP 才暴露）。fixture 42 struct PASS（diff 4.47%）。
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
