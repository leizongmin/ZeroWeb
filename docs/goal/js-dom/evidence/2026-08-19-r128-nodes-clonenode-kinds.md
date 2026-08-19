# R128 — M4 nodes：Node-cloneNode 全节点形态族（24F→5F，主文件 14F→0F 100%，+26 净）

**日期**: 2026-08-19
**Driving WPT**: `dom/nodes/Node-cloneNode.html`（14F→0F，135P/135P 双路径 100%）+
`Node-cloneNode-document-with-doctype.html`（3F→0F）+ `Node-cloneNode-XMLDocument.html`（1F→0F）
**账本**: `tests/wpt-runner/imported-tests.txt`（R128 条目）

## 根因

`cloneNode` 只对元素 proxy 实现——其余节点形态（fragment/text/comment/PI/Attr/doctype/
Document）要么无 cloneNode（`is not a function`），要么落到元素克隆返 nodeType 1 的错形态。
instanceof 面全断（canvas/unknown/DocumentType/XMLDocument/Document）。

## 修复面

| 层 | 修复 |
|----|------|
| part03 `Node.prototype.cloneNode` 泛型 | 按 this 形态分派：own-property 委托（R126/127 教训第三次应用）→ text/comment 走 `_zwMText`/`_zwMComment` 工厂 → PI 重建（createProcessingInstruction）→ Attr 走 `_zwMakeAttr` 四字段复制（ns/prefix/localName/value，与源 value 独立）→ doctype 经 `implementation.createDocumentType` → fragment 建新 fragment + 递归 clone 子 → Document 按源 contentType 分派（XML→createDocument 保 'application/xml'，HTML→createHTMLDocument('') title 空）+ deep 复制 doctype/documentElement 子 → 兜底 Element.prototype deepClone |
| part03 原型链接 | `Attr.prototype → Node.prototype`（attr.cloneNode 经原型链可达）+ `DocumentType` 构造器全局占位 + `XMLDocument` 占位（prototype 链 Document）+ `_makeDetachedDocument` 产物 `setPrototypeOf(Document.prototype)` + constructor 惰性 getter（按 contentType 返 XMLDocument/Document） |
| part03/06 doctype 工厂 | dt 字面量构建**后** `setPrototypeOf(DocumentType.prototype)`（instanceof 面；对象内 IIFE 因 tdz 拿不到 dt 本体——首版失败根因） |
| part04 proxy cloneNode trap | ① text/comment handle（`_textHandles`/`_commentHandles`）→ createTextNode/createComment 重建（保 nodeType 3/8）② PI handle（`_piHandles`）→ createProcessingInstruction 重建 ③ fragment handle（`_fragmentHandles`）→ 走泛型 fragment 分支 ④ NS 源恢复（`_nsHandles` 元数据 → `__zw_create_element_ns` 重建 + 重注册——WPT `createElementNS('x','foo:div').cloneNode().nodeName === 'FOO:DIV'`） |
| part05 canvas | `_zwMakeCanvas` 产物 `setPrototypeOf(HTMLCanvasElement.prototype)`（original instanceof 面）+ attributes 视图 + cloneNode 方法（重建 + 复制 width/height/属性 + attributes 重建 + 原型挂接） |
| part05 getPrototypeOf | 无 iface 映射的未知 tag → `HTMLUnknownElement.prototype`（旧统一回落 HTMLElement——`createElement(zz-unknown) instanceof HTMLUnknownElement` 恒 false）+ `_zwUserProto` 用户原型优先 |
| part05 setPrototypeOf trap（新） | `Object.setPrototypeOf(el, proto)` 存 `_zwUserProto`（key 与 _proxyCache 同源）——旧默认 trap 落 target 且 getPrototypeOf 不读 target，用户原型被静默丢弃 |
| part02 DOMParser doc | `_zwParsedDoc.prototype.cloneNode`（经 createHTMLDocument('') 承载——自带 [doctype(html,,,), html] 与用例期望一致） |

## A/B 验证

- **Node-cloneNode.html**：14F→**0F（135P 双路径 100%）**；document-with-doctype 3F→0F；
  XMLDocument 1F→0F；external-stylesheet 1F→0F（R128 前意外修复）。
- **dom/nodes 全量**：7876→**7902P（+26 净）**，fail 310→**284（逐文件 diff 零新增）**；
  连带意外修复：Document-constructor 1F + append-on-Document 3F + prepend-on-Document 3F
  （Document.prototype 链接的涟漪收益）；native 6375→**6398P**。
- **回归面**：events 419/27F、traversal 1595/9F、collections 48/0F、MO 105/10F——与 R127
  基线逐项一致零回归。
- **单测**：engine `test_clone_node_all_node_kinds_r128`（12 断言组）。
- `make test` 全绿 66 套件（v8+quickjs 双矩阵）；fmt 无 diff；clippy 零警告。

## 剩余 5F（记深结构清单）

1. **Node-cloneNode-svg 4F**：探针实证**源头**缺陷非 clone 缺陷——`document.querySelector
   ('svg')` 的源元素本身 namespaceURI 读 XHTML ns（应为 SVG ns）、`xmlns:xlink` 属性名丢
   `xmlns:` 前缀、localName 混乱。属「解析 markup 的 NS 元数据面」（parser/svg 域深结构，
   源修后 clone 自然跟随——clone 分支已具备 NS 恢复能力）。
2. **Node-cloneNode-on-inactive-document-crash 1F**：`<iframe>` 无 src 的 contentDocument
   返 null（R115 设计「无嵌套浏览上下文」）→ remove 后 cloneNode 崩。spec 应有 about:blank
   子文档——iframe 域深结构（R117 frame 族）。

## 教训

1. **对象字面量内的 IIFE 拿不到本对象**（tdz）——`{ __init: (function(){ setPrototypeOf(dt) })() }`
   在 dt 赋值前执行，静默失败（catch 吞）。「构建后接线」必须移到字面量之后。
2. **own-property 委托判定第三次应用**（R126 removeChild → R127 replaceChild → R128
   cloneNode 泛型）——同族方法的同一 bug 模式每次新方法都要重防；已形成范式：泛型方法
   首行 own-property 检查 + 排除自身。
3. **原型链接的涟漪收益不可预测但真实**——`setPrototypeOf(doc, Document.prototype)` 为
   cloneNode instanceof 而做，连带修好 append/prepend-on-Document 7F（Document 原型方法
   经链可达）。原型链接类修复的回归面要跑全量子目录才能捕捉全部收益。
