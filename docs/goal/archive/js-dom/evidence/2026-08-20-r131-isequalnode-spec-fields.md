# R131 — M4 nodes：isEqualNode spec 逐类型字段比较（6F→0F 全 100%，+13 净含涟漪）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/Node-isEqualNode.html`（6F→0F，9P 双路径 100%）；
涟漪收益 `dom/nodes/Node-compareDocumentPosition.html` foreignDoc 域 6F→0F
**账本**: `tests/wpt-runner/imported-tests.txt`（R131 条目）

## 根因

旧实现（R2819 `_nodeSig`）用 outerHTML 序列化签名比对，三个语义洞：

1. **NS/prefix 不参与序列化**（3F）：`createElementNS("namespace", "prefix:localName")`
   与 `"namespace2"`/`"prefix2"` 变体的 outerHTML 相同 → different namespace/prefix
   误等。
2. **PI 的 target 不进签名**（1F）：PI 序列化含 target，但 handle 形态 PI 的 data
   读经 `_zw_get_text_handle`（丢 target 前缀）——different target 误等。
3. **doctype/document 无方法**（2F）：`doctype1.isEqualNode is not a function`——
   plain object 无 get trap，原型链接（DocumentType.prototype→Node.prototype）虽
   在但 Node.prototype 上没有 isEqualNode 泛型。

## 修复

| 层 | 修复 |
|----|------|
| part03 `_zwIsEqualNode` | spec dom-node-isequalnode 统一实现：nodeType 不同→false；Text/Comment/CDATA 比data、PI 比target+data；doctype 比name/publicId/systemId；元素比 ns+prefix+localName+属性集；Document/Fragment 仅子节点；子节点逐对递归 |
| part03 属性比较 | 三元组（ns/local/value）**序无关配对** + **属性 prefix 不参与**（WPT "attribute with different prefix" 期望 true——spec 属性等价只看 namespace+localName+value） |
| part03 `Node.prototype` | isEqualNode 泛型挂载（`_zwDefProtoMethod`——doctype/document/fragment 经 R128 原型链接链可达） |
| part04 get trap | isEqualNode 分支改委托 `_zwIsEqualNode`（旧 `_nodeSig` 签名废弃） |
| part03 合成节点 | docEl/head/body 补 `namespaceURI=XHTML + prefix=null` 标注（spec：HTML 文档的 html/head/body 均 XHTML ns）——"default HTML documents, created different ways" 断言（createDocument+createElement vs createHTMLDocument 两径结构对齐） |

## 涟漪收益

合成 docEl/head/body 的 ns 标注连带修复 **Node-compareDocumentPosition foreignDoc
域 6F**（foreignComment↔foreignDoctype、foreignPara1↔foreignPara2/foreignTextNode
等——其期望值计算经节点排序/位置键，ns 字段参与）。R128 教训「原型链接的涟漪收益
真实但不可预测」的又一次印证：**字段标注类修复的回归面要跑全量子目录捕捉**。

## A/B 验证

- **Node-isEqualNode**：6F→**0F（9P 双路径 100%）**。
- **dom/nodes 全量**：polyfill 8410→**8423P（+13）** fail 214→**202（逐文件 diff
  零新增）**；native 7627→**7647P（+20）**。
- **回归面**：events 422P/27F（fail 集与 R130 一致零新增）、collections 49P/0F、
  traversal 1589P/15F 逐项一致；native events fail 集 diff 零新增、native traversal
  36F 与 R130 持平。
- **单测**：engine `test_is_equal_node_spec_fields_r131`（8 断言组——ns/prefix/属性
  prefix 不参与/PI target/doctype 三字段/文档结构/fragment 递归）。

## 教训

1. **序列化签名的等价语义是近似**——outerHTML 对 NS/prefix/PI-target 不可分辨
   （HTML 序列化本就不携带 NS）；spec 逐字段比较（isEqualNode 各类型字段表）没有
   序列化捷径，须按节点类型分派。
2. **属性等价的字段面与元素不同**——元素 prefix 参与、属性 prefix 不参与
   （spec 步骤 5 的 attributes 比较是 namespace+localName+value 三元组）；
   两层「prefix 参与性」相反，读 spec 字段表即可避免想当然。
3. **合成节点的 ns 标注是结构正确性而非装饰**——docEl/head/body 补 XHTML ns 一并
   修好 compareDocumentPosition 6F：字段标注类修复的影响面在全量子目录回归中
   捕捉（涟漪收益不可预测）。
