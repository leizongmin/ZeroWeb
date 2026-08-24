# R214 Evidence — iframe documentElement 结构修复两件（ownerDocument + TITLE 根因）

**日期**: 2026-08-24
**切片**: M4——R214(a) insertNode 残余聚类的 138F 大簇根因（`ownerDocument(docEl).createRange` undefined）+ 连带结构层修复
**改动面**: `part05.js`（docEl ownerDocument 两形态 + 无 `<html>` fallback 门控）+ `part21.rs`（回归单测）

## 一、根因（聚类 + 探针链）

1. **`Cannot read properties of undefined (reading 'createRange')` 138F**：
   common.js `rangeFromEndpoints` 经 `ownerDocument(docEl).createRange()` 建域内
   Range——iframe doc 的 documentElement（mEl 解析产物与 R177 合成 html 两形态）
   都缺 `ownerDocument` 字段。
2. **结构层（连带暴露）**：`Range-test-iframe.html` 无显式 `<html>` 标签——R207
   的 fallback 落**通用配对 regex**，首个「开-闭对」= `<title>…</title>` 使
   docEl=TITLE（`refDoc.documentElement.cloneNode` 链跟着 TITLE 化——restoreIframe
   的 referenceDoc 重建结构性错误）。真浏览器对无显式 html 的 HTML 文档**合成
   `<html>` 根**。修：HTML/XHTML kind 落 R177 合成 html 路径（mEl=null），
   XML kind 保持通用 regex（真 XML 根语义）。

## 二、验证链

- **全量（polyfill）**：R213 基线 51243P/3795F/21T → **51302P/3737F/20T
  （净 +59P）**——F2P 66（surround +36 / clone +12 / extract +12 / delete +6）/
  P2F 8（delete 形态重分布——docEl 修复使 setup 路径变化，R215 复查）
- **单文件**：insertNode 628P 计数持平（createRange 簇修后暴露下一层
  Maximum call stack——restoreIframe 累积循环，R215 靶点）；surround 829P 保持
- **全量（native 对照）**：**51301P/3737F/21T**——flips 仅 1 既存 flaky
- **engine 单测**：2353 全绿（新增 `test_iframe_docelement_structure_r214`
  ——docEl tagName/ownerDocument/clone 链/docEl-rooted range 建立四断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R214 行）

## 三、R215 靶点

1. insertNode 的 restoreIframe **累积循环**（Maximum call stack——测试对 23×40
   subtest 逐轮 restoreIframe 重建 referenceDoc；探针 2 轮正常，~290 轮累积溢出
   ——疑 R191 adoptAll 递归或 clone 链的累积引用）
2. deleteContents 8 个 P2F 形态复查（range 18/24/27/40/41/46——元素容器形态）
3. insertNode HRE 336F 簇（ensurePreInsertionValidity 缺失）

## 四、commit

（落盘时待填）
