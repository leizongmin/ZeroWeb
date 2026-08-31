# R328 Evidence — template 内容 inert 化：页面脚本提取跳过 `<template>` 内联脚本（spec 正确性切片）

**日期**: 2026-08-28
**切片**: M4——备档集巡检（remove-next-sibling-during-replace-with 1F 根因修复）
**改动面**: `crates/engine/src/pipeline/extract.rs`（提取层跳过 template 子树脚本 + `is_inside_template` helper + 1 单测）+ `crates/dom/src/document/mod.rs`（`is_template_element` public accessor）

## 一、根因

`extract_page_scripts_indexed` 经 `zero_dom::Document::get_elements_by_tag_name("script")`
全树 DFS 提取页面脚本，**不跳过 `<template>` 子树**（`collect_by_tag_name` 无 template
过滤；html5ever TreeSink 的 `get_template_contents` 暂返 template 元素自身，脚本节点
留在文档树内）→ template 内联脚本被当页面脚本在加载期执行。

spec HTML §the-template-element：template 内容是 **inert DocumentFragment**——
内容文档中「scripts are not executed」（只有克隆/升级进活动文档后才按脚本语义处理）。
Driving 用例 `remove-next-sibling-during-replace-with` 的 template
`<script>document.querySelector('b').remove();</script>` 在测试体前抢跑，测试体
`container.querySelector('script')` 的查询视图与抢跑 mutation 叠加 miss →
"Cannot read properties of null (reading 'remove')"。

## 二、修复

- **`is_inside_template(doc, id)`**：沿 `parent_node` 上行判定祖先是否 template 元素；
- **提取循环内 `continue`**：template 内联/外链脚本均不入执行序列；**序号计数不受
  影响**（`this_idx` 在过滤前递增）——与 shim `getElementsByTagName('script')` 的
  全文档序一一对应（`document.currentScript` 对齐面不回归）；
- **`Document::is_template_element`**（public）：从私有 `is_query_opaque` 拆出的
  语义化 accessor（R145 的查询 opaque 判定复用同一 `local_name == "template"`）。

## 三、A/B

| 项 | R327 基线 | R328 | Δ |
|---|---|---|---|
| 全量 dom sweep（`make testharness-dom TIME_LIMIT=2400`） | 54143P/56F/22T | **54142P/55F/23T** | Fail set 恒等（Timeout ±1 为并发噪声带：`query-target-in-load-event.html` 单跑 Timeout 在 clean main 同样复现——预存，非本切片引入）；探针残留文件（R222-probe/zz-r54×3/zz-probe-r157b/zz-r180）17F 为 gitignored wpt-data 遗留，非产品 fail |
| extract 单测 | 2467 | **2468** | +1（`extract_page_scripts_skips_template_contents_r328`：template 内脚本不执行 + 顶层脚本保留 + 序号含 template script 递增）|
| zero-engine / zero-dom --lib | 2467 / 853 | **2468 / 853** | 全绿 |
| clippy（engine/dom/webview 三 crate --all-targets） | — | 干净 | — |
| fmt | — | 无 diff | — |
| template 消费方套件（Node-cloneNode 145P / svg-template-querySelector 3P / DocumentFragment-getElementById 5P / ChildNode-after·before 45P×2 / Element-remove 7P） | 全绿 | 全绿 | 零回归 |

## 四、残余（本切片不追，记档）

`remove-next-sibling-during-replace-with` 仍 1F——**下一层缺口是克隆产物 script 的
插入期执行**（spec：template 内容克隆进活动文档时 script 须按新脚本语义执行），
与 R321 确证的「handle 展开子对 host 快照查询不可见」（identity 双源）同域，归
L2 深水区；探针（模板无 script 版）实证 `kids=DIV,B`（克隆 span 未入融合视图）+
`span=null`——同 R321 facts 锁定的已知限制。

## 五、教训

1. **解析层语义缺口会在桥接层伪装成查询 bug**：抢跑脚本的副作用（remove mutation）
   让失败形态像「querySelector miss」，根因却在提取层多执行了一个不该执行的脚本——
   备档集巡检先核对 spec 生命周期语义（何时该执行）再做桥接面归因。
2. **序号对齐是提取层过滤的隐性契约**：`extract_page_scripts_indexed` 的 index 与
   shim `getElementsByTagName('script')` 一一对应（currentScript 对齐），任何过滤
   都必须在计数递增之后、只影响入队不影响编号。
