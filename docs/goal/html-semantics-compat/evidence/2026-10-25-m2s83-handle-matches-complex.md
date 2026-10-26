# M2-s83：handle 形元素 matches complex 读面（R5283）

日期：2026-10-25 ｜ 切片：s83 ｜ 提交：R5283 ｜ 前序：[s82](2026-10-25-m2s82-handle-closest.md)

## 1. 分诊（s82 遗留面重估）

s82 账本将 matches complex stale 面记遗留，注「需 Rust 视图文档 matches 回调或 query_all 回退，非平凡」。本轮 probe 重估**推翻非平凡判定**：

- 边界实证：gap **非同 turn 窗口而是永久**——post-drain（setTimeout+rAF 排水后）`matches('#anc span')` 仍 false。qS 返回 handle 形 proxy（`__zwHandle` 真、`__zwSelector` null），`_zwQueryWrapIdentity` 倒查命中并 `_r100Remember` 登记锚，但 t6 锚臂 host `__zw_matches` 读**原始 dom_html 快照**（callbacks.rs `html.lock()`，selector_match.rs `element_matches_test_selector` parse dom_html），apply 后锚不可解析——host 侧 '0' 短路 return，任何后置补偿都到不了。
- s82 已落双形态父链解析（`_zwClosestParentOfPx`）+ `_matchCompoundOf` + `_parseSelectorListOf`/`_parseComplexOf`——JS 侧判定材料齐备，无需 Rust 回调。

## 2. 修复

part05 `_zwHandleMatchesComplex`（三态契约）+ part04 t6 臂重排（R134 → **s83 JS 走** → t6 锚臂 → regex → false）：

- 列表逐 complex 判定（spec 列表任一命中即 true）；compound 带 pseudo/unsupported 或组合器非 ` `/`>` → 该 complex **不可判**（`_matchCompoundOf` 不求值 pseudos，混入会 over-match）。
- 右 compound 对自匹配，沿父链右→左：descendant 找任意祖先、child 限父；48 跳全程共享封顶（R125 口径）；nodeType≠1 停（不越 shadow 边界，s82 同款）。
- 三态：true（可判且命中）/ false（全可判全未命中）/ null（有不可判且无命中——落回锚臂 host 全语义，**pseudo 面不劣化**）。
- JS 走必须先于锚臂：锚臂 '0' 无条件 return（t6 实站语义不动），后置补偿不可达——此为 probe 定位的关键序。

## 3. 验证

- 契约测试 `handle_element_matches_complex_s83`（part90，18 断言）：(a) 单 tag/tag+attr/descendant/child/列表三面/双级链 7 面；(b) s81 改名 interplay（lw 现值命中/旧 id stale）；(c) pseudo/`~`/`+` 保守 false；(d) detached 三面；(e) 右 compound 否决两面；(f) 混合可判/不可判列表 false。
- probe 同 turn + post-drain 全 Pass（临时已删）。
- engine lib：3523P/0F（test-guard 守卫跑）。
- 域（FILTER=embedded-content）：976P/252F/30T 持平，REG 0。
- pin（FILTER=interactive-elements）：439P/44F/0T bit-exact。
- fmt clean；clippy 仅 s78 既有 concat! 警告（不在本 diff）。

## 4. 遗留

- 兄弟组合器（`+`/`~`）保守 false：`_zwNodeParent` 反链无前兄弟指针（previousElementSibling 走 host 面），语料 0 需求，不扩。
- 锚臂 host `__zw_matches` 对 handle 元素锚不可解析的底层（runner 流 webview merge 发布面）未动——JS 走已覆盖可判面，host 侧修复收益仅剩 pseudo complex 面（无已知消费方）。
