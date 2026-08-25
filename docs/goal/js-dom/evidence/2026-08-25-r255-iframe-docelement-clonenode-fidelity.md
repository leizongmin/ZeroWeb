# R255 Evidence — iframe docEl own cloneNode 保真 + body 内容标准化提取（16,x 诊断基建，WPT 净 0）

**日期**: 2026-08-25（本文件为 R256 轮补录——R255 会话在 commit 后因 API 限额中断，evidence 未落盘；内容以 commit 92d3e1fbb 为事实源重建）
**切片**: M4——16,x `[document.body,4,document.body,5]` startOffset 11F 诊断基建
**改动面**: `part05.js`（body 内容标准化提取 + docEl own cloneNode）+
`part23.rs`（r255_repro_reference_doc_clone_chain 单测）
**commit**: 92d3e1fbb

## 一、16,x 探针链定位的两层基建缺陷

1. **body 内容标准化提取**：无显式 `<html>…</html>` 对的 HTML 子文档，
   bodyInner = 全 markup（含 doctype/`<html>`/`<head>` 段），ensureTree 的
   `'<body>'+全文+'</body>'` 重解析让 host parser 对 body 上下文里的
   html/head 标签 foster/剥离——**body 视图丢 `<script>` 元素**（probe
   R255P 实证首次 onload body 7 子含 2 SCRIPT vs 旧视图只剩
   [DIV,#text]）。真浏览器 body 含 script 元素使 offset 4 合法；host 丢
   script 使 sim/host 的 startOffset 算术分歧（host 落 so=4，期望 2）。
   修：有 `<body …>` 开标签取其后全文（忽略未闭合——HTML 解析器同款）；
   无 `<body>` 标签去 doctype/`<html…>`/`<head>…</head>` 段取剩余。
2. **docEl own cloneNode(deep)**：运行时 docEl.childNodes 是工厂空数组
   （R220 教训：运行时把 head/body 链入 docEl.childNodes 曾 -158P——sim
   insertNode 的 referenceNode 解析路径改变而 host 未随动），
   `documentElement.cloneNode(true)`（WPT restoreIframe 的 referenceDoc
   克隆链）经 `_zwDeepCloneEl` 只产空壳，克隆文档丢 body 的 script 等
   元素。修：**只改克隆产物不改运行时结构**——克隆树按 spec 解析产物
   形态 `[head, body]` 递归克隆 doc.head/doc.body 两视图（浅克隆走通用
   工厂；覆盖 mEl 提取与 R177 合成两条 docEl 路径）。

## 二、验证（vs R254 基线）

| 项 | R254 | R255 | Δ |
|---|---|---|---|
| Range-surroundContents | 1810P/30F | 1810P/30F | 持平 |
| ranges 全量（Range-*） | 38676P | 38680P | set-diff 0/0 |
| engine 单测 | 2394 | 2395 | +1 回归单测全绿 |

## 三、16,x 本体遗留

16,x startOffset 11F 未随基建修复翻绿——真根因在 iframe contentDocument
的**异步 fetch-rebuild 时序**：harness 的 onload 链（actualIframe.onload →
expectedIframe.src → referenceDoc.appendChild(...)）与 ZeroWeb iframe doc
的同步 rebuild 分歧。深项延后（需评估 harness onload 语义同步化）。
