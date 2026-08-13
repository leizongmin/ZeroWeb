# Author font fallback 不应共用 generic 性能门

日期：2026-08-13

相关模块：`engine::paint::text_shaping`、`render-foundation::font`

## 问题

`font-size-adjust-013` 的缺失 glyph 落到默认字体，而不是下一个显式 `@font-face` family。test/reference 同时缺少另一段 inline 内容，因此 self-source 仍为 approximate green，Chromium Oracle 则停在 `17.19%`。

## 根因

R3243 因 multi-face shaping 导致普通 CJK 页面 paint 性能回归，将 `ZW_SHAPED_FALLBACK` 改为 opt-in。paint 随后把所有有序 face 列表截断为一项，并关闭 per-face `font-size-adjust`，其中也包括完全由 author font 组成的短列表。generic/system fallback 与显式 CSS family fallback 的成本和必要语义不同，不应共用同一个性能门。

## 解决方案

任何包含 generic/system face 的列表继续走现有 single-face 路径。仅当所有 resolved face 都是显式 author face 时，保留多个 face 与 per-face adjustment。`ZW_AUTHOR_FONT_FALLBACK=0` 可恢复旧行为。

验证必须使用同一 HEAD 的 Oracle A/B，不能只看 self-source。最终 css-fonts 为 7 改善、0 回归、275 持平，总差异 `909.51→905.07pp`。
