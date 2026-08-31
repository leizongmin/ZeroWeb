# M4 R15 切片 — implementation.createDocumentType + detached doc implementation

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R14（createEvent alias + NotSupportedError）
**commit**: 见 `git log`（feat(js-dom): implementation.createDocumentType + detached doc implementation）

## 背景

DOMImplementation-createDocumentType.html 81 失败。根因：createDocumentType 返 null stub + detached doc 无 implementation。

## 改动（4 文件）

### 1. createDocumentType 返 DocumentType（part06 主 + part03 detached doc）

spec dom-domimplementation-createdocumenttype：DocumentType 节点（nodeType 10），不校验。返 name/nodeName=qualifiedName、publicId、systemId、nodeType 10、ownerDocument、nodeValue/textContent=null。主 document ownerDocument=globalThis.document；detached doc ownerDocument=doc。

### 2. detached doc 加 implementation（part03）

detached doc 此前无 implementation（用例 doc.implementation.createDocumentType 崩）。加 hasFeature + createDocumentType（ownerDocument 指 detached doc）。

### 3. 顺带修 canvas 流 2 个 clippy 红灯（main 既有）

crates/canvas/src/context/types.rs:199 `.or_else(|| Some)`→`.or(Some)`；crates/engine/src/js_dom_bridge/canvas.rs:873 setWordSpacing 嵌套 if 合并。canvas 流 R34xx 引入，机械修正无逻辑变化（不修 CI 红）。

### 4. 单测（part07）

test_create_document_type_r15。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R14 | R15 | Δ |
|------|----|----|---|
| polyfill | 50.33% | 52.11% | +1.78pp |
| native | 50.07% | 51.84% | +1.77pp |

createDocumentType 用例 1P/81F → 80P/2F（+79）。双路径对等差 0.27pp。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 下一步

classlist 剩 60F / createEvent 剩 15F + event target null / createElementNS 大小写 / iframe.contentDocument（深结构）。
