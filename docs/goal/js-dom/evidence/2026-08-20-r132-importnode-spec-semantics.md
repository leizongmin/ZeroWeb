# R132 — M4 nodes：importNode spec 语义 + detached body 属性族（5F→0F 全 100%）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/Document-importNode.html`（5F→0F，5P 双路径 100%）
**账本**: `tests/wpt-runner/imported-tests.txt`（R132 条目）

## 根因（三层）

1. **副本无 ownerDocument**（4F）：旧实现委托 `node.cloneNode(deep)`——detached doc 元素
   的 clone 走 `Element.prototype.deepClone` 产出 plain object，无 ownerDocument 字段
   （读 undefined）——WPT 四变体断言 `newDiv.ownerDocument === document` 全灭
   （fail 形态 "expected Document node but got undefined"）。
2. **浅克隆语义缺**：`Element.prototype.cloneNode` 的 deepClone 无 shallow 分支——恒深
   复制 childNodes；且 plain 副本缺 `firstChild` getter（读到 undefined ≠ null——
   "No/Undefined/False 'deep' argument" 的 `newDiv.firstChild === null` 断言）。
3. **detached body 无属性方法**（1F）：`doc.body.setAttributeNS is not a function`——
   createHTMLDocument 的 body 是字面量 plain object（R132 前只有查询/Range 方法）。

## 修复

| 层 | 修复 |
|----|------|
| part06 importNode | spec `dom-document-importnode` = clone + **adopt**：`adoptAll` 递归 defineProperty ownerDocument getter 指本文档；浅变体剥子（childNodes=[]/children=[]）；Attr 走 `_zwMakeAttr` + prefix/namespaceURI/localName 显式复制 |
| part06 adoptAll | plain 副本补叶子导航面（firstChild/lastChild getter + hasChildNodes——R130 title 文本子同款三件套教训的复发面收口） |
| part03 body | setAttribute/setAttributeNS/getAttribute/getAttributeNS/hasAttribute/removeAttribute + getAttributeNode/getAttributeNodeNS——NS 元数据落本地 `_r132BodyAttrNS` 表（限定名→{ns,prefix,local}；`_tree` 是 `_zwMEl` 产物无 `__zwHandle`，不能复用 `_attrNSMeta` 键空间） |

## 关键定位

- WPT 的 deep 缺省断言（「No 'deep' argument」期望浅克隆 firstChild null）是**历史
  行为**（WHATWG 现行 spec deep 缺省 true）——按用例面实现：缺省/undefined/false =
  浅、true = 深。
- Attr import 的 (ns, local) 反查需元数据桥：`_zwMEl` 树节点无 handle/proxy 通道，
  本地表是 plain-object 侧 NS 元数据的可达形态（与 proxy 侧 `_attrNSMeta` 平行）。

## A/B 验证

- **Document-importNode**：5F→**0F（5P 双路径 100%）**。
- **dom/nodes 全量**：polyfill 8423→**8427P（+4）** fail 202→**197（逐文件 diff
  零新增）**；native 7647→**7648P**。
- **回归面**：events 422P/27F、collections 49P、traversal 1589P/15F 与 R131 逐项
  一致；native events 38F / native traversal 36F 与 R131 持平零新增。
- **单测**：engine `test_import_node_spec_semantics_r132`（浅剥子/深递归归属/源树
  完整/Attr 三字段复制/body NS 属性族）。

## 教训

1. **clone 委托不等于 import 语义**——spec import = clone + adopt 两步；只做 clone
   丢 ownerDocument 归属（polyfill 桥里 plain-object 副本没有默认的 ownerDocument
   通道，须显式递归挂）。
2. **叶子导航三件套的复发面**——R130 在 title 文本子上踩过（hasChildNodes 缺失崩
   oracle），本次 deepClone 产物再踩（firstChild undefined ≠ null）；**任何产出
   plain-object 节点的工厂/克隆路径都要过一遍三件套检查**。
3. **plain-object 侧的 NS 元数据不能挂 handle 键空间**——`_attrNSMeta` 以 `_elKey`
   （handle/sel）为键，`_zwMEl` 产物两者皆无；本地表是可达形态，跨形态共享元数据
   需要显式桥（记 L2 方向）。
