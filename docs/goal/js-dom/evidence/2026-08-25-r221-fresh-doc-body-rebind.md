# R221 Evidence — fresh-doc：iframe doc 的 body/head 重绑（appendChild docEl 时）

**日期**: 2026-08-25
**切片**: M4——R220 实证的跨轮残留结构根因（restoreIference 克隆空壳 + document.head/body 平行树）的正面修复
**改动面**: `part03.js`（detached doc appendChild 的 R221 重绑段）+ `part21.rs`（回归单测）

## 一、修复机理

restoreIframe（WPT Range mega-case 的每轮重置）语义：
`doc.removeChild(首末子直到 doctype)` + `doc.appendChild(referenceDoc.docEl.cloneNode(true))`
+ `contentWindow.setupRangeTests()`。

R220 定位的平行树：appendChild 换入克隆 docEl 后，`doc.body`/`doc.head` 仍指
factory 时的原字面量——setup 建的 paras 落在字面量 body 树、rangeFromEndpoints
走的克隆 docEl 树，两树分裂。

**R221 修法**：iframe doc（`_zwMarkup` 印章）的 doc 级 appendChild 收到 HTML
元素时，`doc.body`/`doc.head` 经 defineProperty 重绑到克隆子树的 BODY/HEAD
子节点——setup 内容与 range 树归一。非 restoreIframe 路径（createHTMLDocument
等）克隆无 head/body 子时保持原值零扰动。

## 二、验证链（vs R220 基线）

| 文件 | R220 | R221 | Δ |
|---|---|---|---|
| Range-insertNode | 1094P | **1637P** | **+543** |
| Range-mutations | 0P | **1338P** | **+1338**（整族解锁） |
| Range-surroundContents | 854P | 865P | +11 |
| Range-deleteContents | 56P | 68P | +12 |
| Range-extractContents | 100P | 103P | +3 |
| Range-cloneContents | 153P | 156P | +3 |
| Range-attributes/adopt/collapse/cAC/cBP/detach/stringifier/StaticRange/intersectsNode/isPointInRange/comparePoint/set | — | 全同 | 0 |
| dom/nodes·events·traversal·collections | 12661/576/1595/49 | 12661/577/1595/49 | +1 |

**ranges 族净增 ≈ +1910P**。

- **kill-switch 复测**：fresh-doc 落地后重试 `_r219ProtoMethods=true`——surround
  865→837（-28），insertNode 不变 → 三方法原型兜底**仍保持关**（sim 的
  mySurroundContents 经其它缝隙泄漏，非本根因）。
- **engine 单测**：**2367 全绿**（新增 `r221_iframe_doc_body_rebind_on_docel_append`
  ——重绑 identity + in-tree 落位 + head 三断言）。
- **fmt / clippy**：零 diff / 零警告。
- **make test**：（见 master.md R221 行）

## 三、残余（R222 靶点）

- insertNode 剩 204F：rows 25/26/29（[document,0,document,1..2] /
  [foreignDoc,1,foreignComment,2] 跨容器族，22F×3）+ HRE 69 + null-nodeType 66。
- surround 剩 452F：R221 后重聚类。

## 四、commit

（land 时回填）
