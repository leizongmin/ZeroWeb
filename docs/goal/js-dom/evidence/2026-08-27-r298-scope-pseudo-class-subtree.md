# R298 Evidence — querySelector :scope 子树作用域（scope 套件 2P/2F→4P/0F 100%）

**日期**: 2026-08-27
**切片**: M4——R298(a) selector 小簇续首件（scope 2F）
**改动面**: `js_dom_bridge/selector_match.rs`（`query_match_in_subtree_doc` / `query_all_in_subtree_doc` 的 `:scope` token 解析 + `resolve_subtree_scope` helper）+ `js_dom_bridge_tests/part24.rs`（+1 单测）

## 一、根因

WPT `ParentNode-querySelector-scope` 两失败：`div.querySelector(":scope > p")` 返
null（期望 p）、`div.querySelectorAll(":scope > span")` 误空。

- spec（selectors-4 §6.4 + dom spec querySelector）：元素子树查询的 `:scope` =
  **调用元素自身**（scoping root）；
- dom crate 对 `:scope` 的静态实现是「文档根元素 html」（`is_scope_element`——
  样式表语义），`query.rs:135` 注释早已记档「querySelector 的调用元素即 scope
  语义需把 scope NodeId 贯穿 matches 链，为 follow-up」；
- 结果：`:scope > p` 在 div 子树内被求值为 `html > p` → div 内恒无匹配。

R153 已在 **closest** 路径解决同源问题（`replace_scope_tokens`：独立 `:scope`
token 文本替换为调用元素唯一选择器）——但只接了 closest，子树查询两入口未消费。

## 二、修复

R153 模式复用（文本替换，零 dom crate 结构改动——scope NodeId 贯穿 matches
链的深结构方案仍可后续统一）：

- `resolve_subtree_scope(doc, root, sel)`：选择器含 `:scope` 时，`replace_scope_tokens`
  把独立 token 替换为 root（find_by_selector 解析的调用元素）的
  `unique_selector_for_node`；root 无唯一选择器时原样返回（兜底走文档根语义，
  不劣化）；
- `query_match_in_subtree_doc` / `query_all_in_subtree_doc` 两入口接线（querySelector
  与 querySelectorAll 对称）。

`div.querySelector(":scope > p")` → `div:nth-child(1) > p` 子树求值 → 命中直接
子 p；`:scope > span` → span 是孙层 → 恒空 ✓。

## 三、验证

| 套件 | 基线 | R298 | Δ |
|---|---|---|---|
| ParentNode-querySelector-scope | 2P/2F | **4P/0F（100%）** | +2P/-2F |
| ParentNode-querySelector 全族（All/case/content/escapes/scope/removed） | 2048P/7F | **2050P/5F** | +2/-2（恰 scope 两例；余 tree-order 4F + removed 1F 既存） |
| Element-closest（R153 :scope 消费方） | 29P/0F | 29P/0F | 持平 |
| Element-matches | 675P/0F | 675P/0F | 持平 |
| shadow 族（:scope 间接消费方） | 12P/2F | 12P/2F | 持平（stash A/B） |
| engine 单测 | 2435 | **2436**（part24 +1：`:scope > p` 命中直接子 p + `:scope > span` 孙层空 + `:scope > h1` + 无 :scope 回归） | +1 |
| make test | — | 1F = `window_surface_present_smoke`（XOpenDisplayFailed 环境项） | 持平 |
| fmt / clippy（-p zero-engine --all-targets -D warnings） | — | 干净 | — |

## 四、记档

- **深结构备档**：`replace_scope_tokens` 文本替换是 pragmatic 近似——对
  `:has(> :scope)` 嵌套形态 closest 已实证可用，但极端形态（`:is(:scope)` 参数
  内层）未覆盖；dom crate 的 scope NodeId 贯穿 matches 链（`query.rs:135`
  follow-up 注记）是统一正解，归 L2/选择器深结构域。
- **R299 剩余**：selector 小簇的 mixed-case 1F（属性选择器 `s`/`i` flag 的
  NS 感知大小写折叠）未动——首断言在 `[testAttr="alpha" s]`，css-parser
  `AttrCaseModifier` 已解析未消费、dom `ci` 已实现，缺 HTML 文档属性名折叠
  （viewBox→viewbox 命中 HTML 元素、SVG 区分）。
