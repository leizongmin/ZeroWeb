# M4 切片 R81 — Document/Element 导航面 + createElementNS 校验 + textContent 语义（WPT 驱动）

**日期**: 2026-08-16
**Commit**: `322cf584`（rebase over 并行流 form-validation `6739cec8`）
**Evidence**: [../evidence/2026-08-16-r81-document-navigation-textcontent.json](../evidence/2026-08-16-r81-document-navigation-textcontent.json)
**接手背景**: 上一轮 429 中断留下的工作树遗留（part02-06 未提交 shim 改动，R81 主体已实现），本轮补单测、修复验证中暴露的回归、按 WPT 期望表对齐校验语义后 land。

## 驱动用例簇

- `dom/nodes/Document-createElement-namespace.html`（0P → 100%）
- `dom/nodes/Node-textContent.html`（~30P → 81P/0F 100%）
- `dom/nodes/Node-properties.html`（595P/163F → 726P/0F 100%）
- `dom/nodes/Document-createElement.html`（HTML-document 变体全通过）
- `dom/nodes/Document-createElementNS.html`（HTML-document 195/195）

## 核心修复

1. **createElement ns 文档类型派生**：contentType + `_docNS` 贯穿 createDocument/createHTMLDocument/new Document/DOMParser；XML doc createElement preserveCase。
2. **textContent 全语义**：融合 childNodes 拼接读（pending 子可见）；setter 替换全部子（registry 清 + `_zwNodeParent` 反链删 → 子 parentNode=null）+ 不解析 markup + 空值不注册空文本节点 + `_zwTextWritten` 写入值优先；undefined 与 null 同归空串（R3184 旧语义纠正）。
3. **ASCII-only 大小写**：`_zwAsciiUpper` + localName 内联 ASCII 小写——`'ı'`/`'K'`(U+212A) 不再被 Unicode 全量 toUpperCase/toLowerCase 错误转换。
4. **HTML createElement Name 校验**：`_zwIsValidHtmlElementName`（首字符 NameStartChar + 拒空白/`'>'`；`'}'`/`'<'`/`'￿'` 非首合法）；含冒号名不解析 prefix（localName = 全名 ASCII 小写）。
5. **createElementNS validate-and-extract 按 WPT 期望表**：`'f:o:o'` 有 ns 合法、`'0:a'` 合法 / `'a:0'` 非法（prefix 段从宽、localName 段首字符严格）、XMLNS ns 仅 xmlns 元素、xmlns prefix 仅 XMLNS ns。
6. **导航面大补**：doctype/fragment/CDATA/PI/comment/text 的 firstChild/lastChild/sibling/parentElement/length/wholeText；document.childNodes=[doctype, html] + Document 元数据族（URL/compatMode/characterSet/inputEncoding）+ 导航 null 族；detached doc 同款；docEl/headEl/body hasChildNodes；body appendChild 子 parentNode 重指 body 自身。
7. **TreeWalker sibling 结构序步进**：currentNode 被 whatToShow 滤掉（如 doctype 对 SHOW_ELEMENT）时 `_siblingByOrder`——从结构序向 dir 找同父节点逐个 check（ACCEPT/REJECT 剪枝/SKIP 跳非同级）。

## 验证

| 项 | 结果 |
|----|------|
| dom/nodes | 6294P → **6596P（+302 净）**（clean-HEAD 重建二进制 stash A/B） |
| dom/traversal | 1188P → **1195P（+7 净）**（过程 -21 回归同轮定位修复：body 缺 hasChildNodes + walker sibling 滤节点） |
| dom/events / collections | 189P / 48P 不变（零回归） |
| 单测 | part18 +3（R81 族）；engine v8 **2166** / quickjs **1424** / wpt-runner **172** 全绿 |
| fmt / clippy | 无 diff / v8+quickjs 双矩阵零警告 |
| pre-commit-guard | PASS |

## 剩余

- Document-createElement(NS) 的 XML/XHTML document 变体 = iframe contentDocument 深结构（既存未解决问题 #13，html-compat 域）
- NodeIterator `whatToShow=0xFFFFFFFF` 期望 4294967295 得 -1（`whatToShow|0` 符号位——WPT 152 subtest，下轮候选）
