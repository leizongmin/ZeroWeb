# R339 — DC-2 QuickJS feature 同页对齐（2026-08-28）

## 背景

DC-2（真实 SPA / Web Components 端到端）剩余项：「QuickJS feature 在 DC-7 达成后跑
同一验收页对齐」。评估发现对齐**无需等待 DC-7**——验收页走的是 polyfill 引擎中立路径。

## 评估与修改

- vue e2e（`tests/integration/tests/e2e_vue_library.rs`）驱动方式：
  `WebView::new(WebViewConfig) → run_page_scripts → execute_script_with_dom`——
  **引擎中立**（`run_page_scripts` 按 feature 选择沙箱后端）。
- 原门 `#[cfg(all(test, feature = "v8"))]` 为历史保守设置（无任何 v8 专属 API），
  放宽为 `#[cfg(all(test, any(feature = "v8", feature = "quickjs")))]`。

## 结果（--features quickjs）

- `e2e_vue_library`：**3/3 Pass**（vue_mount_lands / vue_reactive_and_event_lands /
  vue_reconciliation_lands，1.43s）——**零测试改动**
- `e2e_lit_library`：6/6 Pass（本就无 feature gate，已被 quickjs 763 全绿矩阵覆盖）
- `e2e_web_components`：8/8 Pass（同上）
- integration 全量：763P/0F/59 ignored（quickjs）
- v8 矩阵复跑：3/3 零影响；clippy 双矩阵干净；fmt 无 diff

## 结论

**DC-2 的 SPA（Vue）+ WC（lit/原生 CE）验收现于 v8/quickjs 双 feature 均验证通过。**
DC-7 的「双 feature 行为等价」验收面相应收窄为原生绑定域（dom_bindings quickjs 测试点
对齐），不再是验收页整体。

## 教训

历史 feature 门要核对是否真依赖：引擎中立路径上的 v8-only gate 是保守惯性，
阻止了验收面在第二引擎上的复用。
