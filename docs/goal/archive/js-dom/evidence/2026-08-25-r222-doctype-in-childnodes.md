# R222 Evidence — iframe doc doctype 入 childNodes（R216 回退件重试成功）

**日期**: 2026-08-25
**切片**: M4——R222(a) insertNode 204F 重聚类 → doctype 累积根因 + R216 回退件在 fresh-doc 后重试
**改动面**: `part05.js`（_zwMakeIframeDoc 的 doctype 入 childNodes 首位）+ `part21.rs`（退化路径单测）

## 一、204F 聚类与 doctype 累积根因

| 簇 | 量 | 形态 |
|---|---|---|
| HRE | 69→73 | foreignDoc(31)/xmlDoc(30)/document(12) 作 node 的跨容器族 |
| null-nodeType | 66→22 | rows 25/26/29 各 22F |
| text-differ / assert_unreached | ~69 | 散布 |

R222-res 探针（doc.childNodes 逐轮 dump）：

- 轮 0 PRE `[HTML(0)]`——iframe doc 初始无 doctype 入树（R216 评估回退后
  getter-only）
- restoreIframe 清理循环清空 doc → 兜底 `createDocumentType` 触发 → 从轮 1 起
  `[dt, dt, HTML]` **双 doctype 稳定累积**——`[document,0,document,N]` 语义
  基准错位（rows 25/26 整行 22F）

## 二、修法

R216 回退件重试：`_zwMakeIframeDoc` 把 `_r209Dt` unshift 进 `doc.childNodes`
首位（`parentNode` 指回 doc）。R216 时净 -55 的扰动面（restoreIframe 清理节奏 +
referenceDoc 语义）已被 R221 fresh-doc 的 body/head 重绑吸收——初始
`[dt, docEl]` 使清理循环不清空 doc，兜底不触发，无双 doctype。

## 三、验证链（vs R221）

| 项 | R221 | R222 | Δ |
|---|---|---|---|
| Range-insertNode | 1637P | **1669P** | +32 |
| Range-surroundContents | 865P | **893P** | +28 |
| dom/nodes | 12661 | 12663 | +2 |
| dom/events | 577 | 579 | +2 |
| traversal / collections | 1595 / 49 | 1595 / 49 | 0 |
| extract / clone / delete / mutations | 103/156/68/1338 | 同 | 0 |

净 **≈ +64P**。

- **engine 单测**：**2368 全绿**（新增 `r222_iframe_doc_doctype_in_childnodes`
  ——unit 沙箱无 `__zw_fetch` 契约走 no-markup 退化路径的稳定性断言；
  doctype 入树断言由 WPT 25/26,x 族承载）。
- **fmt / clippy**：零警告；**make test** 1F = XOpenDisplayFailed 环境项。

## 四、R223 靶点

- **insertNode 剩 172F**：HRE 73（foreignDoc/xmlDoc/document 作 node 的跨容器
  族——host 不抛处 sim 期望 HRE 或反之）；null-nodeType 22（row 29：
  foreignDoc.childNodes 在 setup 后为空——**预存问题**，A/B 实证与 R222 无关，
  下一轮定位 createHTMLDocument 在 iframe realm 的返回形态）。
- **surround 剩 ~350F**（893P 基线重聚类）。

## 五、commit

bd4e83efb
