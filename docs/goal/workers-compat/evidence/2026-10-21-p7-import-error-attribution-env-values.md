# P7 后半 + 环境值面——importScripts 错误归因 + worker rAF/search（2026-10-21）

全量 **693/714 = 97.1%**（693P/12F/8T/1PF，178 案）。证据 JSON：
[2026-10-21-p7-import-error-attribution-env-values.json](2026-10-21-p7-import-error-attribution-env-values.json)
（runner `testharness-workers --json` 原样输出）。

## 数字对照

| 阶段 | Pass/Total | 率 | 备注 |
|---|---|---|---|
| M3 收口 | 678/714 | 95.0% | 上一切片台账（commit `3ca356bb8`） |
| P7 后半（会话内实测） | 691/714 | 96.8% | report-error-\* 6 案×2 子测试 + Worker-nested-importScripts-error 13F 全数翻绿 |
| 本切片收口 | 693/714 | 97.1% | +2P：worker rAF（数值严格递增时间戳）+ WorkerLocation.search 空 query |

Timeout 维持 8 不变；PF 1 为 structured-clone battery ReadableStream 先决条件面
（上游同形态，非失败）。

## 修改面

### 1. importScripts 错误归因（part05，P7 后半——report-error-\* 13F）

共享三相实现 `_zwImportRun(args, caller)`：resolve/fetch 失败错误打 `_zwCaller`
标记（filename/lineno 指向 importScripts **调用点**）；`new Function` 解析错误与
运行时错误打 `_zwScriptUrl`（指向**被导入脚本**，解析错误行 1、运行时错误取栈
坐标并按 `_zwImportBodyPrefix` 长度做列偏移修正）。三条路径均以
`_zwScriptUrl === undefined` 判内层优先——嵌套 importScripts 错误归因最内层脚本
（Worker-nested-importScripts-error 全 13 子测试面）。

- 构造期改写：内联路径 `_zwInlineImportScripts` 按调用点行列换算，非语句位/非
  字面量调用改写为 `__zwImportTracked(url, line, col, ...)`（不插换行，行号不
  偏移）；被导入源先递归改写再 eval 包装——跨层 helper 内 importScripts 归因
  （report-error-003 cross-origin 面）。
- 错误定位：`_zwErrorLocation(err, bodyOffset, colAdj)` 加列偏移参数；
  `_reportError` origin 优先级 `_zwCaller` > `_zwScriptUrl`+parse（行 1）>
  `_zwScriptUrl` 运行时（栈坐标+列修正）> 原路径，`_zwReportToOwner` 增加
  filename 传参。

### 2. worker 作用域 rAF（part05）

页面 part01 stub 为同步派发 + 原始 host 串时间戳（string、三次同值），不满足
`WorkerGlobalScope_requestAnimationFrame.worker.js`「三次嵌套回调时间戳数值型且
严格递增」面。wctx 增设专用 `requestAnimationFrame`/`cancelAnimationFrame`：
setTimeout(0) 逐回调派发（worker 无渲染帧，spec 近似「每回调一帧」）、
`performance.now()` 数值时间戳、max 下限保证跨回调严格递增（同毫秒粒度钟强制
步进 0.0001）、`_terminated`/`_closed` 门与错误走 `main._reportError`；
句柄表供 cancel 派发。`workers/worker-request-animation-frame.html`
（raf time relative to worker load）随数值化顺带翻绿。

### 3. URL search 空 query（js_dom_bridge `serialize_url`，根因修）

URL spec §search getter：query 为 null **或空串**均返空串，否则 `"?" + query`。
裸尾 `?` 被 url crate 解析为 `Some("")`，旧序列化产出 `"?"`（
`WorkerLocation_search_empty.htm` 失败根因）。修在共享序列化器——页面 URL 对象
与 WorkerLocation 同路径受益。`set_url_part` 的 search setter 空串置 None 语义
不变（setter 侧 url crate 规范）。

### 4. 引擎单测（part61）

P7 归因 4 案：importScripts 调用点归因（worker 脚本第 2 行）、解析错误归因被
导入脚本（30 字符截断消息）、嵌套内层优先、跨层 helper 运行时调用归因。
环境值面 2 案：worker rAF 三连数值递增、search 空 query。
`cargo test -p zero-engine --lib` 3339/3339。

## 残余 21（12F + 8T + 1PF，20 案）分簇

| 簇 | 案×子测试 | 缺口 |
|---|---|---|
| P8 基建/合成 origin | baseurl/new-Worker 1F、use-base-url 1F、blob-url 1F + baseurl/alpha 2T + location/redirect 1T；Worker-location.sub 1F（`:80`）、platform 1F、userAgentData 1F | runner 无 302 重定向语义 + 合成 origin 恒 https（上游按 http 交付）；值为 runner 环境差，shim 不可修 |
| URL 编码 | semantics/encodings/003 1F、worker-url-encoding 1F | 文档编码 → worker URL 解码管线（功能缺口） |
| terminate 深度 | Worker-terminate-forever 1F、Worker_terminate_event_queue 1T | terminate 中断执行语义（同 realm 近似固有边界） |
| 排除面记账（P6） | SharedWorker exposure 1F、ctor-\{1,null,undefined\} 3T、navigator/007 1T | SharedWorker 挂账；等待型页面（M1 基线即有） |
| 功能缺口 | resource timing 1F、nested_sync_xhr 1F | worker resource entry 采集；同步 XHR |
| battery 先决条件 | structured-clone dedicated 1PF | ReadableStream 非 transferable（上游同形态，非失败） |

## P2 契约门核验

本切片全部改动落在 engine 页面管线 shim 与 URL 序列化器（js_dom_bridge 纯函数）
，未触 zero-protocol / apps/renderer IPC 契约与 zero-page-runtime 协议面——
importScripts 装载表与错误归因在同 realm 近似内闭合，P2 契约协商门未触发。
