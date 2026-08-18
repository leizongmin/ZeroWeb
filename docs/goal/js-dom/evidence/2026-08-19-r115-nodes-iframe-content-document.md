# R115 evidence — nodes：静态 iframe contentDocument（createElement/createElementNS 大簇解锁）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**基线**: R114（nodes 6658P/1539F；events 419P/32F；`make test` 18,228P）

## 聚类分析（nodes 1539F 按文件）

最大簇 = **iframe contentDocument**（`<iframe src="/common/dummy.xml|.xhtml">`）：Document-createElementNS 390F + case 154F + Document-createElement 98F + attributes ~40F + getElementsByTagNameNS 14F + DOMImplementation 11F ≈ **750F 单一根因**（master.md 未解决问题 #13 的「最小子集」——不做完整跨 realm iframe，只做静态 src 子文档）。

## 实现（五件）

1. **fetch-dom-subset.sh 补 `common/dummy.xml` + `common/dummy.xhtml`**（上游 pinned rev；缺文件 → contentDocument 恒 null）。
2. **iframe 元素 `contentDocument`/`contentWindow` getter**（part04，IFRAME tag 专属）：src 经 host `__zw_fetch` **同步契约**（webview.rs headless 路径直接返 wire——与 app 层异步版互斥判空串）加载，消除 async fetch 与 window load 的竞态；缓存 per-iframe key。wire 解析（`__zwfr:` + `\x1f` 四段，body 末字段原样）。
3. **`_zwMakeIframeDoc`**（part05）：复用 `_makeDetachedDocument`（查询/mutation/Range 全可用）+ `documentElement`（markup 根元素提取，textContent 供用例 load 断言）+ `defaultView`（→ contentWindow，`__r115SetWin` 回填槽）。XML doc：contentType 'application/xml' + `_docNS` null；XHTML：'application/xhtml+xml' + HTMLNS。
4. **`_zwIframeCreateElement` + `doc.createElement/createElementNS/createTextNode`**（part05）：
   - createElement：**大小写转换仅 HTML 文档**（XML/XHTML 是 XML 解析——localName/tagName 保持原样，WPT 期望表）；WebIDL DOMString 转换（undefined → 'undefined'）；非法名抛 InvalidCharacterError（`_zwIsValidHtmlElementName` Name production）；元素挂 Element.prototype 链（instanceof win.Element ✓——win 构造器转发主 realm）。
   - createElementNS：validate-and-extract 复用主 document 语义（空前缀/空 local/非 NameStartChar/空白'>' → InvalidCharacterError；prefix 无 ns / xml·xmlns 保留绑定 / 无 prefix 的 localName 'xmlns' 非 XMLNS ns → NamespaceError；**带 prefix 的 'test:xmlns' 合法**、'xmlns:foo' 在 XMLNS ns 内合法——WPT 期望表逐条对齐）；保大小写 + prefix/localName/namespaceURI 正确 + nodeValue null。
5. **`_zwMakeIframeWin`**：最小 window 面（document + Element/Node/HTMLElement/Text/Comment/DOMException 等构造器转发主 window）。

## A/B 结果（WPT testharness）

| 项 | R114 基线 | R115 | 净 |
|---|---|---|---|
| Document-createElement.html | 49P/98F | **147P/0F（100%）** | +98 |
| Document-createElementNS | 206P/390F | **596P/0F（100%）** | +390 |
| dom/nodes 全量 | 6658P/1539F | **7146P/1051F** | **+488 净** |
| dom/events | 419P/32F | 419P/32F | 0 |
| dom/collections | 48P/1F | 48P/1F | 0 |
| dom/traversal | 1595P/10F | 1595P/10F | 0 |

## 单测（part20.rs +1）

- `test_iframe_content_document_r115`：contentDocument 加载（documentElement.textContent）+ defaultView 回指 + XML createElement 保大小写/ns null/instanceof + createElementNS prefix/localName/ns + NamespaceError（prefix 无 ns / 无 prefix xmlns）+ InvalidCharacterError（空名）。

## 验证

- `make test` **18,235 passed / 0 failed**（exit 0；中途 product-version `embedded_version_uses_short_date_format` 一次失败 = 构建日期跨午夜 flake——clean-HEAD 同样失败、touch 重建后消失，非本切片回归）
- `cargo fmt --all -- --check` 无 diff；workspace clippy 零警告
- engine js_dom_bridge 600 单测全绿（含 R115 +1）

## 教训

1. **最大簇先聚类再动手**：nodes 1539F 里 ~750F 挂在同一根因（iframe 静态子文档）——按文件聚类后的单一杠杆远高于逐 API 修。
2. **异步加载与 load 时序竞态**：iframe src 首版用 fetch Promise——window load 先于 fetch 完成时 contentDocument 恒 null。headless 宿主的 `__zw_fetch` 本就是**同步契约**（直接返 wire）——先查既有回调的契约形态再选路径。
3. **大小写语义按文档类型**：createElement 的 ASCII lower/upper 是 **HTML 文档专属**（spec）；XML/XHTML（XML 解析）localName/tagName 保持原样——XHTML 不是「HTML 的一种」而是「XML 解析的文档」。
4. **保留绑定规则的边界**：xmlns 的 prefix/localName 保留规则各有豁免（'xmlns:foo' 在 XMLNS ns 内合法、'test:xmlns' 带 prefix 合法）——WPT 期望表逐条对齐，不能凭记忆写全量拒绝。
5. **heredoc 写 JS 的 artifact**：python heredoc 生成的注释 `#+` 前缀会使整个 shim CompileError（SyntaxError）——生成后 node --check 一次再跑。
