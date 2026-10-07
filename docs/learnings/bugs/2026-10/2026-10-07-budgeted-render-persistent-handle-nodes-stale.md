---
date: 2026-10-07
modules: engine,webview,renderer
---

# 预算渲染路径不清 persistent_handle_nodes 致跨文档 handle 身份漂移

## 问题描述

slice37 s30-multimatch 集成面间歇 FAIL：判定 div 的 textContent 读成空/`n/a`。CDP 诊断（同 URL 反复导航的合成 probe 页）实证：第 3 次起迭代 `document.getElementById` 返回包装的 `__zwHandle` 与本迭代 `createElement` 产出的 handle 不一致（如 vHandle=`__n2` vs divHandle=`__n4`），且 `selector→handle` 反查表逐迭代累积旧条目（`__n0..__n10` 同 selector 共存），`__zw_handle_for_selector` 值扫描命中任意一个。

## 根因分析

handle→NodeId 持久表（`Pipeline::persistent_handle_nodes`）的清零点只挂在 `render_html`/`render_html_animated` 的全量重建尾部（slotmap 换代语义，R100）。而 renderer 生产导航的渲染入口是**预算渲染路径**（`AsyncPageLoad::tick` → `advance_budgeted_render` → `apply_render_result`），换代时只装 `cached_doc`，从不清该表。链条：

1. 文档 A 的 apply 回填 `persistent_handle_nodes`（结构性 apply 的 rescue/backfill 语义）；
2. 导航到文档 B：`prepare_document_state` 清 webview 的 `selector_handle_map` 等，但**不清 pipeline 的 persistent_handle_nodes**；预算渲染不经过 `render_html`，无清零点；
3. 文档 B 首个 apply：`apply_dom_mutations_full` 把 pn 条目**预植**进本批 ephemeral map，并经返回的 handle_selectors **返出**；
4. `page_scripts.rs` 把 handle_selectors merge 进 worker 反查表（`reset_document_state` 刚清过也被重新污染）；webview 侧同表同理。

旧 NodeId 在新文档 slotmap 空间里还可能被复用，加重身份错乱。同一 URL 重导航时 `SetDomSnapshot` 的 `url_changed` 条件清表也不触发（worker 侧 `reset_document_state` 无条件清，但挡不住步骤 3-4 的再注入）。

## 解决方案

文档换代边界补清零：`Pipeline::clear_persistent_handle_nodes()`（新 pub 方法，镜像 `clear_detached_sel_nodes` 惯用法），在 `WebView::prepare_document_state` 调用（该函数是 run_staged_load / navigate_with / AsyncPageLoad 共同的换代入口）。回归钉：`webview::tests::prepare_document_state_clears_persistent_handle_nodes_s37`，走「prepare → 预算渲染 → apply」真实生产序列，删行即红（hs 夹带上一文档 handle）。

## 如何避免

- 新增「文档域状态」时，换代清零点要覆盖**全部**换代路径：`load_html`（全量渲染自清）、`prepare_document_state`（预算渲染路径无全量渲染）、worker 侧 `reset_document_state`。只在渲染尾部清，等于把不变式押在「换代必走全量渲染」上——预算渲染/增量渲染路径出现即破。
- 跨 apply 持久的表（persistent_nodes、反查表、detached stash）都属此类；本仓 `clear_detached_sel_nodes` 曾在 `set_cached_content` 补挂，同型教训。
