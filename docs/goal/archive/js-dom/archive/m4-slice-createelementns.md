# M4 — createElementNS HTML 大写 tagName + validate-and-extract + Node 常量（R80）

**日期**: 2026-08-16
**Commit**: `f8e75c28`
**前置**: R79（contains/compareDocumentPosition `3214d883`）
**证据**: [evidence/2026-08-16-r80-createelementns.json](../evidence/2026-08-16-r80-createelementns.json)

## 背景

R79 后 nodes 剩余最大簇 Document-createElementNS **0P/596F**（两引擎共有）。

## 根因（五重）

1. HTML 文档 + HTML ns 的 createElementNS 元素 tagName 须 ASCII 大写（spec create-element-ns），旧保留原值。
2. validate-and-extract 校验全缺失（InvalidCharacterError/NamespaceError 不抛）。
3. 非 HTML ns 元素被 `instanceof HTMLElement=true`（getPrototypeOf 恒走 HTML iface 映射）。
4. Node 接口常量（`element.ELEMENT_NODE`）不可见——proxy get trap 未知属性恒 undefined；且 **Reflect.get(target) 不可用**（target {} 真实原型是 Object.prototype，与 getPrototypeOf trap 声明的链是两回事）。
5. Element.nodeValue 返 undefined（spec 恒 null）。

## 实现

- `_nsHandles[handle]` 增 `htmlUpper`（HTML ns）；tagName/nodeName 按标记大写；prefix/localName 从原值解析（spec：大小写转换只作用于 qualified name——`'html:span'` → prefix 'html' / localName 'span' / tagName 'HTML:SPAN'）。
- validate-and-extract（part06 createElementNS 前置）：空前缀段（':foo'）/空 localName 段（'foo:'）/非 Name 字符 → InvalidCharacterError；空 ns 带 prefix / 段内二冒号（'f:o:o'）/ xml 前缀非 XML ns / xmlns 前缀 / xmlns localName 非 XMLNS ns → NamespaceError。
- getPrototypeOf（part05）：非 HTML ns 的 isNs 元素 → Element.prototype；HTML ns 按精确大小写 localName 查 iface（'SPAN' 无映射 → HTMLUnknownElement.prototype）。
- Node 常量（nodeType 1-12 + DOCUMENT_POSITION_* 六个）挂 Node.prototype + Node 构造器；part04 get trap 的 SCREAMING_SNAKE 属性沿 getPrototypeOf 链手工查找（≤8 层 guard）。
- Element.nodeValue = null 分支（part04）。

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| Document-createElementNS | 0P/596F | **187P/409F**（全部 HTML-document 变体通过） |
| nodes 目录 | 5568P | **6294P（+726）** |

- 剩余 409F 全为 iframe contentDocument 深结构簇（XML/XHTML document 变体——既存未解决问题 #13，html-compat 域）。
- quickjs = quickjs-native = 187P；v8-native 0P 为**既存分歧**（clean HEAD 重建二进制同 0P——native 路径对 document 方法的覆盖方式不同，非本切片引入，default-on 前对齐项）。
- collections/events/traversal 零回归。

## 验证

- 单测 +2（part18 R80 族：大写/校验六 throw/常量/nodeValue）。
- engine v8 2163 / quickjs 1424 全绿；wpt-runner 171；integration **767P/0F**（html_compat default_actions 已被并行流在 main 修复）；fmt/clippy 干净；pre-commit-guard PASS。

## 教训

Proxy get trap 的原型链回落不能 `Reflect.get(target)`——target 的真实原型与 getPrototypeOf trap 声明的原型是两回事，沿链手工查找才是 trap 一致的读法。
