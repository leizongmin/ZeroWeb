# s10k70 刀58：:link 超链接收窄＋:target doc-node URL 直读＋relList MathML 摘除

- 日期：2026-11-28（UTC 22:45 发射）
- 基线：chain69（刀57 后残池 225F；bin 74b68e49b4b5＝8a620f6a8）
- 刀体：4bcb46c34（rebase 重放自 92bf8f19d，对端 98468bb67 docs-only 零码域交集）
- 字节绑定：bin 2ec3aca260c7＝4bcb46c34 树（fmt 后重建）；发射前置断言＋终 md5 恒等（批中零重建）
- 电池：chain70 pid 3027399，17.7min，EXIT=1 与 chain69 同形（san/dom/tt rc=2 常态计数、ref rc=0）

## 目标键（17F 候选 → 5F 实刀）

| # | 键 | 根因域 | 修复 |
|---|----|--------|------|
| 1 | ParentNode-querySelector-All :: Document.querySelector `#head :link, #head :visited`（got 2 expected 0） | zero-dom `is_any_link` 把 `<link>` 算进 `:link` | PseudoClass::Link 拆分＋`is_hyperlink`（a/area[href]） |
| 2 | ParentNode-querySelector-All :: Document.querySelectorAll 同上 | 同上 | 同上 |
| 3 | Element-matches :: `:target`（no refNodes） | shim R173 `_zwOwnerTree` 门对 doc-node root（nodeType 9）恒空 URL | doc-node 直读 `_zwFragmentUrl` 槽；元素维持成员门 |
| 4 | Element-webkitMatchesSelector :: `:target` 同上 | 同上 | 同上 |
| 5 | lists/DOMTokenList-coverage-for-attributes :: MathML a.relList should be undefined | part04 R374 SVG-a 臂误 OR MathML ns（注释与代码矛盾笔误） | MathML 摘除，仅 HTML a/area/link＋SVG a |

其余 12F（attributes 2＋Node-properties 2＋MO-document 2＋getElementById/removeChild/adoptNode/Document-URL/thcrash/MO-childList 各 1）本轮不展开，挂账归因续刀。

## 判定

- dom 腿：237F→232F（−5），cleared 集恰 5 目标键、零新增（comm 双向）；57648P（翻面页 ΔP 恰 +5＝2+1+1+1 镜像闭合，余 +38 页预算截断显影）
- san 5F／tt 32F 集恒等零漂移；ref rc=0 输出同形
- solo 四目标页全绿：ParentNode-querySelector-All 1976P/0F、Element-matches 675P/0F、Element-webkitMatchesSelector 669P/0F、DOMTokenList-coverage-for-attributes 175P/0F；Element-closest 29P/0F 零漂移
- cargo fmt --all -- --check 零 diff；lei-pre-commit-guard PASS（0 发现 0 提醒）

## 规范依据

- `:link`（CSS Selectors L4 §18）＝超链接元素：HTML `<link>` 是 external resource link 非超链接，仅归 `:any-link`（a/area/link[href] 并集）。WPT selectors.js:222/223 "not matching link elements with href attributes" 钉面。
- `:target`：target element 须在 document tree 内；iframe 子文档 URL 由 shim 槽 `_zwFragmentUrl` 承载（part01.js:3118 加载完成注入），权威判定在 zero-dom `Document::is_target_element`。
- relList coverage 表：HTML ns a/area/link＋SVG ns a（SVGAnchorElement）；MathML-a 无 relList。

## 残相归因（无语料钉面，不扩刀面）

probe2 三岔定位：zero-dom 本体正确（DOMParser doc qSA(:link)=1 ✓）、序列化保真（outerHTML ✓）。主文档域缺口两处：
1. shim 主文档快照漏斗（`__zw_matches`/`__zw_query_all` 读 dom_html 快照）不识 `:link`——probe2 P-d（主文档 in-tree qSA(a:link)=0）/P-e（in-tree matches(:link)=false）红。
2. trap-world handle-only（detached createElement）伪类保守 return false（part04 t6 尾既有保守面）——P-b（detached matches a:link）红。

selectors.js 全族跑在 iframe doc（zero-dom 域，已修），corpus 无主文档域 `:link` 钉面。探针文件 zz-k58-probe1/probe2 已删未入库。

## census

84,528 → 84,533（+5）。DC-3 校准线 ≥94,093，差 9,560。
