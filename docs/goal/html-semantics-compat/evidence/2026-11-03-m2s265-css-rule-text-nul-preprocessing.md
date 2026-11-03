# M2 s265 — shim CSSOM 规则文本输入预处理：U+0000 → U+FFFD（charset-bom 残 2F 收口）

日期：2026-11-03。切片号 s265（承 s264 下一步候选 #1）。1 刀 + 1 钉
（engine 车道），charset 家族全族 14/14 收口。

## 勘根

1. **s264 残 2F 位对位归因**（charset-bom UTF-16BE/LE 臂）：expected
   `���d�i�v� �`（11 单元）vs got
   `��\0d\0i\0v\0 \0`（11 单元）——每处 U+0000 恰对应 expected 的
   U+FFFD，位对位无一例外。规范链：CSS/text 数据模块响应体**恒
   UTF-8 解码**（Encoding Standard；0x00 是合法 UTF-8 → 解码臂保留
   U+0000，s264 已证）→ FFFD 化发生在 **CSS 解析入口预处理**
   （CSS Syntax §3.3 preprocessing the input stream；tokenizer 各
   消费步与 `\0` 转义路径同样折算 FFFD，全路径一致）。
2. **实现面定位**：rust 侧 css-parser tokenizer 已有该预处理
   （tokenizer.rs:280 CSS Syntax §3.3 注）；缺口仅在 JS shim——
   `_ruleFromText`（part06.js:2041）按首 `{` 切分、selectorText
   原文透传。三个调用点（insertRule 2214 / adopted 表 2372 /
   replaceSync 2389）共享该单点——**一处修复全覆盖**。
3. **顺序无涉证明**：NUL→FFFD 替换不增删 `{`/`}` 字符（二者均非
   NUL），replaceSync 先按 `}` 切分再逐段进 `_ruleFromText` 与先
   预处理再切分等价。
4. **红态可行性**：行为钉可自包含（replaceSync 注入 FFFD 翼 `\0`
   交错 selector，corpus 原形），HEAD 可实证 NUL 残留。

## 施工（1 文件 1 刀 + 1 钉）

1. **`_ruleFromText` 入口预处理**（part06.js）：入参文本
   `.replace(/\0/g, '�')` 后再 trim；注释附 CSS Syntax §3.3
   链接、解码臂/解析臂职责分界与顺序无涉说明。
2. **钉** `test_p56_css_rule_text_nul_preprocessed_to_fffd_s265`
   （part56.rs）：replaceSync 臂（corpus 原形位对位断言：
   selectorText == 上游判据串 + 无残留 `\0` + 规则数 1）+
   insertRule 臂（`\0a\0b\0c\0` → `FFFDaFFFDbFFFDcFFFD`）。

## 验证

- **红态实证**（HEAD + 钉，test-guard）：`false|false|1`——selector
  不匹配 + NUL 残留 + 规则数恰 1，与勘根逐位吻合。
- **绿态**（test-guard）：
  - `cargo test -p zero-engine --lib` **3721 passed + 5 failed**
    ——5 红为 R-t2qq adopted-stylesheets 系列既有（s261 记档同集，
    尾分号 Blink 形过期期望；stash 法归因确认与 s265 无涉，
    本流不入侵）；3720 基线 + 1 新钉。
  - fmt/clippy：`cargo fmt --all --check` clean；clippy
    zero-engine `--all-targets` 零告警。
  - **combined shim node --check OK**——part06.js 为拼接片
    （18 part 序），单文件 check 无意义（片段首尾不闭合）；按
    generate_js_dom_shim 拼接序合成后校验。
- **solo 复跑**（make testharness-html-semantics
  FILTER=css-module/charset-bom，test-guard 包裹）：
  **3/3 全 Pass**（s264 残 2F 全翻绿；日志 evidence/logs/
  s265-charset-bom-after-nul-fix.log）——charset 家族 5 页
  **14/14** 收口。
- **全树 A/B 免跑判据**：corpus 全树 NUL 字节扫描（python
  os.walk，2026-11-03）——12 载体分类：4 二进制图像（不入 CSS
  解析）/ 2 bom css（= 本切 face 本体）/ 2 bom json + 2 bom js
  （JSON.parse / JS eval 路径，无规则文本解析）/ 2 UTF-16 编码
  html（无 `<style>`、style 属性、insertRule、replaceSync——
  NUL 在 HTML 编码测试路径）→ **零其他 face**，引 s258 先例
  免跑（face 唯一 + solo 行为实证 + 套件绿态兜底）。

## 挂账更新

- shim CSSOM 规则文本 NUL 预处理销案（本切收口）。
- 新记录（零 corpus face，不扩面）：声明块直读路径未预处——
  `CSSStyleDeclaration.cssText` setter 与 style 属性解析直入
  `_parseDeclarations`（不经 `_ruleFromText`），NUL 原样保留；
  corpus 全树扫描零 face，按需随语料扩展补。
- 承前不动：R-t2qq 5 红（归该流自查）；css-module relative-urls /
  redirect-relative-urls 3F（CSS url() 基面，渲染流车道交接）；
  repeated-import ×3（serve-custom-response.py 端点缺口）；
  referrer-policies.sub（referrer-checker.py + `{{domains}}`）；
  NotRun 微面（json `;boundary` ×3、css `;params` ×2）。

## 下一步候选

1. script-element-css-src module MIME 检查（zero-net
   script_mime.rs 助手在册，本流；css-module 6F 余项）。
2. css-module relative-urls / redirect-relative-urls CSS url() 基
   面（渲染流车道交接注记）。
3. repeated-import ×3 serve-custom-response.py 端点补齐（runner
   车道）。
