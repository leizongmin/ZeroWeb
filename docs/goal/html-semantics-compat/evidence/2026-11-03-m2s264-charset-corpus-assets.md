# M2 s264 — css-module/text-module charset 语料资产补拉轮：5 页 file-Timeout 解封

日期：2026-11-03。切片号 s264（承 s263 下一步候选 #1）。语料资产轮
（勘根 → 补拉 → 白名单挂账 → solo 验证），零引擎代码刀。

## 勘根

1. **5 页 file-Timeout 全为资产缺席**：css-module/charset.html、
   charset-2.html、charset-bom.html、text-module/charset.html、
   charset-2.html（s261-ab.log 基线）——页面 import 的目标文件不在
   wpt-data → fetch 永不完成 → async 永不收口。缺口根因 =
   `25-html-semantics-compat.sh` 白名单 ③b 旧注记「utf-8.css /
   utf-8.txt 仅经 serve-with-content-type.py?fn= 引用，runner 未实现
   该端点，落地时再补」——**端点已落地**（s115 端点形、s249/s250
   `__zw_mtype` 数据模块臂），注记失效未销。
2. **字节源三重甄别**（关键勘根反转）：上游 web-platform-tests/wpt
   master 已重编 charset fixture——gh api contents?ref=master 实测
   bom-utf-16be.css = UTF-8 BOM + 纯文本（真 UTF-16BE 字节不再在），
   与 corpus 判据不符（corpus charset-bom.html 断言 UTF-8 lossy 解码
   `���d…` mojibake，恰为真 UTF-16BE 字节的 UTF-8
   lossy 形）。corpus 资产权威源 = `lib.sh WPT_REV`
   315976933870b34d6ea30e3f6643403edae678ba（jsdelivr CDN pin 锁定，
   fetch_raw 同源）——**pin 版即 corpus 判据作者手笔字节形**。本环境
   jsdelivr/raw.githubusercontent 直连失败，走 gh api contents
   `?ref=<WPT_REV>` + `Accept: application/vnd.github.raw` 等价取回。
3. **BOM 刀反转销案**：前案预判 charset-bom 需引擎 BOM 嗅探解码刀；
   实测 corpus 判据（UTF-16 BOM 忽略 → UTF-8 解码，UTF-8 BOM 剥离）
   与 runner 既有 UTF-8 lossy 解码臂（testharness.rs 数据模块臂）+
   CSSOM shim 行为逐位吻合——**零引擎刀**。既有挂账「charset 参数 /
   BOM 嗅探未建模」按 spec「CSS/text 数据模块恒 UTF-8 解码、charset
   参数忽略、BOM 忽略」由本切实测销案。
4. **windows-1250.* pin 版字节形**：pin 版文件本身含 U+FFFD 序列
   （上游更早 vintage 已重编）；corpus 判据按该形写定——charset.html
   / charset-2.html 用 `assert_not_equals(content, "śćążź")`、
   text-module 用 `assert_false(startsWith("śćążź"))`（FFFD 形恒不
   等）。7 资产全量字节核验：bom-utf-16be `fe ff` + BE payload、
   bom-utf-16le `ff fe`、bom-utf-8 `ef bb bf`、utf-8.css/txt
   `śćążź`、windows-1250.* FFFD 侧翼 cp1250 文本。

## 施工（零 .rs；2 脚本文件挂账 + 7 语料资产本地补入）

1. **7 资产按 pin 版补入 wpt-data**：css-module/resources/{utf-8,
   windows-1250,bom-utf-8,bom-utf-16be,bom-utf-16le}.css +
   text-module/{utf-8.txt,windows-1250.txt}（wpt-data 为 gitignore
   本地数据，耐久件 = 白名单 + known-gaps）。
2. **`25-html-semantics-compat.sh` ③b**：7 条 `fetch_raw` 白名单
   （幂等快路径 skip-if-exists，本地字节即权威）；旧注记改写销案 +
   pin 版字节源警示（master 重编不符）+ 基准变更注记（5 页 Timeout→
   起跑，下轮全量 join GAINED 按语料变更归因非回归）。
3. **`scripts/update-wpt-data.sh` known-gaps**：7 文件条目（承
   s227/s249/s250 条目先例，标明升级新 ref 后
   `make fetch-wpt-html-semantics` 白名单幂等自动补齐）。

## 验证

- **solo 跑**（`make testharness-html-semantics FILTER=…`，test-guard
  包裹 + RAF_FRAME_DRIVEN=1；日志 evidence/logs/s264-charset-assets-
  {css,text}.log）：
  - `FILTER=css-module/charset`：charset.html **5 子测全 Pass** +
    charset-2.html **1 Pass** + charset-bom.html **1 Pass / 2 Fail**。
  - `FILTER=text-module/charset`：charset.html **5 子测全 Pass** +
    charset-2.html **1 Pass**。
  - 5 页 Timeout→起跑；**12 子测 Pass**，语料面预期全数兑现。
- **残 2F 归因**（charset-bom UTF-16BE/LE 臂）：expected
  `�…d�i…` vs got `��\0d\0i\0v\0 \0`——0x00
  字节在 UTF-8 解码合法保留 U+0000，而上游判据 FFFD 来自 **CSS
  Syntax §3.3 输入预处理（U+0000 → U+FFFD）**：shim CSSOM
  `_ruleFromText`（engine/src/js_dom_shim/part06.js:2041）按首 `{`
  切分、selectorText 原文透传，无预处理；rust 侧 css-parser
  tokenizer 已有该预处理（tokenizer.rs:280 CSS Syntax §3.3 注）——
  缺口仅在 JS shim 解析路径。engine 面 = 本流车道，**下一切片
  （s265）1 刀收口候选**，本切不施工（资产轮与引擎刀不混切）。
- 零引擎代码刀 → 无 A/B 义务；基准变更按白名单注记归因。

## 挂账更新

- 旧注记「端点落地时再补」销案（端点 s115/s249/s250 在案）。
- 「charset 参数 / BOM 嗅探未建模」挂账销案：语料实测判据 = 恒
  UTF-8 解码 + charset 参数忽略 + BOM 忽略，现有通道行为吻合
  （12 子测 Pass 直接证据）。
- 新挂账：shim CSSOM NUL→FFFD 预处理（s265 候选，engine 车道）。
- 承前不动：css-module redirect/relative-urls 3F（CSS url() 基面，
  渲染流车道交接注记）；repeated-import ×3（serve-custom-response.py
  端点缺口）；referrer-policies.sub（referrer-checker.py +
  `{{domains}}` 替换）；NotRun 微面（json `;boundary` ×3、css
  `;params` ×2）；script-element-css-src module MIME 检查（本流）。

## 下一步候选

1. s265：shim CSSOM CSS Syntax §3.3 NUL→FFFD 输入预处理（engine
   车道 1 刀 + shim 钉，charset-bom 残 2F 收口）。
2. script-element-css-src module MIME 检查（zero-net script_mime.rs
   助手在册，本流）。
3. scripting-1 module 面勘根分型（s248 遗留候选）。
