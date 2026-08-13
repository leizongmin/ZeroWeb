# R15 — implementation.createDocumentType + detached doc implementation（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R15
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): implementation.createDocumentType + detached doc implementation）

## 背景

DOMImplementation-createDocumentType.html 81 失败。根因：① 主 document 的 `implementation.createDocumentType` 是 `return null` stub（79 subtest "Cannot read null.name"）② detached doc（createHTMLDocument）无 implementation（用例 doTest(doc,...) 经 doc.implementation.createDocumentType）。

## 改动

### 1. createDocumentType 返 DocumentType 对象（part06 主 + part03 detached doc）

spec `dom-domimplementation-createdocumenttype`：建 DocumentType 节点（nodeType 10）。spec 不校验（publicId/systemId 任意串；qualifiedName 校验在 createDocument 而非此处）。返 name=nodeName=qualifiedName、publicId、systemId、nodeType 10、ownerDocument、nodeValue=null、textContent=null（Node 接口标准）。主 document 的 ownerDocument=globalThis.document；detached doc 的 ownerDocument=doc（spec：doctype.ownerDocument === 创建它的 document）。

### 2. detached doc 加 implementation（part03 _makeDetachedDocument）

detached doc 此前无 implementation（用例 doc.implementation.createDocumentType 崩）。加 implementation 块（hasFeature + createDocumentType，ownerDocument 指 detached doc）。

### 3. 顺带修并行 canvas 流 2 个 clippy 错误（main 既有红灯）

- crates/canvas/src/context/types.rs:199 `.or_else(|| Some(...))` → `.or(Some(...))`（clippy unnecessary_lazy_evaluations）。
- crates/engine/src/js_dom_bridge/canvas.rs:873 setWordSpacing 嵌套 if 合并（clippy collapsible_if）。
均为 canvas 流 R34xx 引入的 main 既有 clippy 红灯，机械修正无逻辑变化（不修则 CI `-D warnings` 红）。

### 4. 单测（part07）

`test_create_document_type_r15`：主 document createDocumentType（name/nodeName/publicId/systemId/nodeType/ownerDocument/nodeValue）、空参、detached doc createDocumentType（ownerDocument===detached doc）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R14 | R15 | Δ |
|------|----|----|---|
| polyfill | 50.33% | **52.11%** | +1.78pp |
| native | 50.07% | **51.84%** | +1.77pp |

双路径对等差 0.27pp。**createDocumentType 用例**：1P/81F → **80P/2F**（+79 pass）。完整 JSON 快照入 evidence。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告（含修 canvas 流 2 个 clippy 红灯）。

## 下一步

- classlist 剩 60F / createEvent 剩 15F + event target null / createElementNS 大小写。
- iframe.contentDocument（深结构 html-compat 域，待评估）。
