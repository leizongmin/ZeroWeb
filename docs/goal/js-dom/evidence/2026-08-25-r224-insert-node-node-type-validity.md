# R224 Evidence — insertNode 的 node 自身类型合法性检查

**日期**: 2026-08-25
**切片**: M4——R224(a) insertNode 剩 148F 的 HRE ~70 跨容器族（foreignDoc/xmlDoc/document 作 node）
**改动面**: `part06.js`（Range.insertNode 的 `_r215Validate` 内新增第四查）+ `part21.rs`（单测 `r224_insert_node_rejects_document_type_node`）

## 一、根因

WPT Range-insertNode 的 71F 簇（foreignDoc 27 / xmlDoc 31 / document 13）形态为
「A HIERARCHY_REQUEST_ERR must be thrown」——**Document 节点（nt=9）作被插 node**
时 sim（common.js ensurePreInsertionValidity）抛 HRE 而 host 不抛。

spec `concept-node-ensure-pre-insertion-validity` 步骤「If node is not a
DocumentFragment, DocumentType, Element, or CharacterData node, throw
HierarchyRequestError」——host 的 `_r215Validate`（R215 引入）只实现了
parent 类型 / 环 / Text-入-Document / Doctype 位序四查，**缺 node 自身类型**
这一查，使 Document 型 node 插入静默成功。

## 二、修法

`_r215Validate` 内 parent 类型检查之后插入 node 类型白名单检查：

- 允许：DocumentFragment(11) / DocumentType(10) / Element(1) /
  Text(3) / CDATASection(4) / ProcessingInstruction(7) / Comment(8)
- 其余（含 Document(9) / Attr(2)）→ `HierarchyRequestError`
  （`'Nodes of type <nt> cannot be inserted.'`）

https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity

## 三、验证链（vs R223）

| 项 | R223 | R224 | Δ |
|---|---|---|---|
| Range-insertNode | 1693P | **1817P** | **+124** |
| dom/nodes | 12663P | 12662P | -1（±1 flake 内） |
| dom/events | 579P | 579P | 0 |
| dom/collections | 49P | 49P | 0 |
| dom/traversal | 1602P | 1602P | 0 |
| Range-surroundContents | 893P | 893P | 0 |
| Range-set / compareBoundaryPoints 等 | — | 同基线 | 0 |

Range-insertNode 文件级 F：148 → **23**（-125）。净 **≈ +124P**。

insertNode 剩 23F 形态（下轮靶点）：
- `25/26/29/31,x:16/18`（document/foreignDoc/xmlDoc 容器 × PI/comment node，
  resulting DOM + position 双断言）——doc 级子位序的深形态；
- `30,4`（foreignDoc.body 容器 insert foreignPara1）；
- `0/4/8/10/15,20`（docfrag 作 node 的 resulting range position）。

- **engine 单测**：**2377 全绿**（新增 `r224_insert_node_rejects_document_type_node`
  ——Document 型 node 抛 HRE + Element 型照常插入两断言）。
- **fmt / clippy**：零警告。

## 四、commit

555ba16cd
