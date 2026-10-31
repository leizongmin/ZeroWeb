# M2-S170 scrollingElement compatMode 分派归位（s168c 账本新记承接）

**日期**: 2026-10-31
**刀位**: s168c 后候选③——`Document.scrollingElement` 恒返 documentElement，无视 quirks；HTML §3.1.1 要求 standards（CSS1Compat）→ documentElement、quirks（BackCompat）→ body element。

## 勘根

s151 已真值化 `document.compatMode` getter（宿主 `quirks_mode` 真值经 `__zw_get_compat_mode` 桥），但 shim 中 scrollingElement 旧实现是**规范链接缺失的简化形**：直接 `return documentElement`，未读 compatMode。两处偏差随之而来：

1. **quirks 文档分派错**：无 doctype（BackCompat）文档的 scrollingElement 应返 `body`（存在时），现返 documentElement——webview 侧无任何断言钉住该面（r3259 案连 doctype 都没写，fixture 恰好落 quirks 模式却按 standards 语义断言，双向掩盖）。
2. **测试假钉**：`test_document_scrolling_element_r3259` fixture 缺 `<!DOCTYPE html>`，实际跑在 quirks 模式断言 standards 结果——修复前该案对 compatMode 分派不敏感，无回归网价值。

规范前提：body getter 同处已有 frameset 分派（s151 前），scrollingElement 的 body 返回值与 `document.body` 同源（body 不存在时返 null，spec「body element is null → return null」）。

## 施工（2 文件）

- `engine/src/js_dom_shim/part06.js`：scrollingElement getter 改 compatMode 分派——`BackCompat → document.body || null`，否则 `documentElement`；补 HTML §3.1.1 规范链接 + s170 标记。
- `webview/src/tests/coverage.rs`：r3259 fixture 补 `<!DOCTYPE html>` 钉 standards 面；新增 `test_document_scrolling_element_quirks_s170`（无 doctype，断言 `compatMode === 'BackCompat'`、`scrollingElement` 标签 `BODY`、与 `document.body` 同一性、非 documentElement）。

## 验证

- webview lib **823P/1F**：双 scrolling 案翻绿（r3259 归位 + quirks 新案）；唯一 F = image_decoder 环境案（既有基线）。
- engine lib **3612P/11F**：失败集合与 s168c 基线逐案相同（fetch 族 9 + ua_display_li + style_sheet_from_handle_r113），shim 改动零回归。
- shim 拼接 `node --check` SYNTAX_OK。
- 使用方抽查：`make testharness-html-semantics FILTER=keyboard-scroll` / `FILTER=scrollable-overflow-fixed-positioned` 均 P=0/F=0/T=0——两文件位于 `wpt-data/css/css-overflow|css-contain/`，不在 testharness-html-semantics 语料域（html/semantics），属域覆盖事实非回归；standards 文档分派不变（doctype 前提），该面由 webview lib 双案 + engine lib 基线承载。
- fmt / guarded-clippy workspace 全绿。
- reftest/product-smoke 未跑：分派仅 DOM 查询面（shim 属性 getter），无渲染管线变更；quirks 案在 webview lib 常驻。
