# R207 Evidence — iframe 工厂可变面 + docEl 根修正 + textContent replace-all（M4）

**日期**: 2026-08-24
**切片**: M4——R206 通道暴露的 iframe 子文档层四件；restoreIframe 首轮端到端打通（`window.testRange` 产出、`ux:null`）；全量 **49534P/5005F → 49543P/5004F（净 +9P，零新增失败行）**
**改动面**: `part05.js`（`_zwIframeCreateElement` append 族 + textContent accessor；`_zwMakeIframeDoc` 根优先 `<html>` + docEl 可变面 + 新节点形态 hasChildNodes/isEqualNode）+ `part21.rs`（单测）

## 一、四件修复

| # | 修复 | 根因 |
|---|------|------|
| ① | `_zwIframeCreateElement` 补 **append/prepend/replaceChildren** | impl 路径（_zwMEl）有、iframe 工厂只有 appendChild——common.js `paras[5].append('9012')` 直接 TypeError |
| ② | iframe 工厂元素 **textContent accessor**（replace-all 语义） | 旧 plain 字段：赋值不建 Text 子 → firstChild 恒 null → `eval('paras[0].firstChild')` 喂 null 给 `ownerDocument(null).nodeType` |
| ③ | `_zwMakeIframeDoc` 根元素**优先 `<html>`**（组序归一） | 通用配对 regex 首命中文档首个「开-闭对」= `<title>…</title>` → documentElement.tagName=TITLE |
| ④ | iframe docEl **可变面**（appendChild/removeChild/append/hasChildNodes/firstChild/lastChild + HTMLHtmlElement.prototype） | restoreIframe 的 `refDoc.documentElement.cloneNode(true)` 链需要 |

## 二、两轮回归（全量逐行 diff 抓回）

- 首版新 Text 子/docEl 缺 `hasChildNodes` → cloneContents walk `node.hasChildNodes is not a function`（8 条新失败行）
- 修后剩 `isEqualNode` 缺失（cursor position 断言）→ 补上后全绿落地

## 三、验证链

- 探针：restoreIframe 分步 instrument——r0 全通（`tr:object, ux:null`）；r1+ 精确定位为
  **setupRangeTests 的 `document.querySelector('#test')` 服务于 stale markup parse 树**
  （live 组装树已前进，查询返回 parentNode=null 的元素 → removeChild 崩）——R208 的精确靶点
- 全量 polyfill **49543P/5004F** / native **逐行相同**；Range-cloneContents range-0 族 +
  insertNode cursor 行转绿；engine 2346 单测全绿；fmt/clippy 干净；make test 除
  XOpenDisplayFailed 环境项全绿

## 四、commit

`5140516e2`
