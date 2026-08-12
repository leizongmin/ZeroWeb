# @font-face unicode-range 被忽略

- 日期：2026-08-12
- 相关模块：css-parser、render-foundation/font、engine、webview、browser、renderer、wpt-runner

## 问题描述

`size-adjust-02.html` 的 self-source reftest 近似通过，但 Chromium Oracle 差异为 5.45%。ZeroWeb 把 `unicode-range: U+41-5A` 的 Ahem face 用于整词，lowercase 也被绘成连续黑块；Chromium 只用 Ahem 绘制大写字母，其余字符回退到后续 family。

## 根因分析

Tokenizer 从不生成已有的 `Token::UnicodeRange`，`@font-face` AST 也未保存 descriptor。字体异步加载只传 family、weight、style、stretch 和 feature settings；FontLoader 解析字符 face 时只检查 cmap，不检查声明范围。同源 test/ref 都丢失该约束，因此会互相抵消。

## 解决方案

按 CSS Syntax 生成 unicode-range token，解析单值、区间和 wildcard，并将闭区间随 font-face metadata 传到所有宿主。FontLoader 为每个 face 保存范围，在 shaping、advance 和 legacy raster fallback 选 face 前统一检查。`ZW_FONT_UNICODE_RANGE=0` 可回退旧行为。

验证必须同时包含真实 face 分段截图和 Chromium Oracle。该修复使 css-fonts Oracle 5 案改善、0 回归，rounded 总差异减少 2.89pp；`size-adjust-02` 从 5.45% 降至 4.41%。
