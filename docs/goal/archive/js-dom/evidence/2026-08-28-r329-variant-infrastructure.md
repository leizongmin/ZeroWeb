# R329 Evidence — WPT variant 基建最小支持 + document IDL on* 全族（handler-count ?document + Range-in-shadow 4 subtest 转绿）

**日期**: 2026-08-28
**切片**: M4——R329(a) 备档集巡检：Range-in-shadow 2F + handler-count ?document 1F 同根因两件
**改动面**: `tests/wpt-runner/src/testharness.rs`（`case_variants` 解析 + run_dom_cases 逐变体跑）+ `crates/engine/src/js_dom_shim/part06.js`（`_defineDocOnHandler` 列表扩 GlobalEventHandlers 全族）

## 一、根因（两件不同域、同轮发现）

### 1. variant 基建缺失（runner 层，R289/R298 时代备档项）

`Range-in-shadow-after-the-shadow-removed.html` 声明 `<meta name="variant"
content="?mode=closed">` + `?mode=open`——上游以同一文件 + 不同 query 组成参数矩阵。
runner 只跑基础 URL（无 query），`URLSearchParams(location.search).get("mode")` 返
null → `attachShadow({mode: null})` 落 TypeError，2F。探针（zz-r329，已清理）实证
`search=`（空）、`mode=null`。同域 `handler-count.html`（?document/?window/?element）
因 `|| 'window'` 默认值碰巧在无 query 时跑 window 变体而部分通过。

**修**：`case_variants(source)` 解析 `<meta name="variant" content="?...">` 列表（单双
引号/无引号三形态；大小写不敏感）。`run_dom_cases` 对带 variant 的用例**逐 query 跑**
（基础 URL 跳过——上游 harness 对无参基础页不注册测试，跑只是空转 Timeout 伪败），
case 名带 query 区分（`Range-in-shadow-after-the-shadow-removed.html?mode=open`）。
query 经 `https://wpt.test/<path>?<query>` 进 `prepare_document_state`/`page_url` →
`__zw_get_page_url` → shim `location.search`；子资源抓取安全（image fetcher / script
fetcher 均已剥 query）。

### 2. document IDL on* 缺 GlobalEventHandlers 全族（shim 层）

`handler-count.html?document` 第二 subtest：`document.onclick = fn` 后点击计数 0。
`_defineDocOnHandler`（R2938/R2939）只列 4 个 Document 专有事件 + DOMContentLoaded
——`document.onclick` 赋值落 plain 属性，冒泡派发的 document 虚站（tgt='doc'）不触。
spec DOM §interface Document 继承 GlobalEventHandlers——click/dblclick/mouse*/key*/
input 等 IDL handler 属性合法。探针（zz-r329b/d，已清理）：setter 后 `typeof
document.onclick === 'function'` 但 `dispatchEvent` tally=0（plain 属性无 dispatch 面）。

**修**：`_defineDocOnHandler` 列表扩至与 window 级 R143 同源的 GlobalEventHandlers +
DocumentAndElementEventHandlers 全族（click/mouse*/pointer*/key*/input/change/submit/
drag*/clipboard*/animation*/transition*/scroll 等）+ 原 4 个专有事件保留。setter 经
`document.addEventListener` 注册（tgt='doc' 槽位）→ 冒泡虚站可触。

## 二、A/B

| 项 | R328 基线 | R329 | Δ |
|---|---|---|---|
| Range-in-shadow-after-the-shadow-removed | 2F（mode=null TypeError） | **4P/4 全 Pass**（?open + ?closed 各 2） | -2F |
| handler-count ?document | 1F（onclick tally 0） | **2P/2 全 Pass** | -1F |
| handler-count ?window / ?element | 2P/2P（基线经默认 window 已过） | 2P/2P + **新增显式变体 4P** | +4P 可见面 |
| **全量 dom sweep**（TIME_LIMIT=2400） | 54142P/55F/23T（含探针残留 17F） | **54150P/53F/23T**（含探针残留 17F） | **+8P/-2F，Fail set 恰 -2（仅 in-shadow 两件消失）零新增** |
| engine --lib（v8） | 2468 | 2468 | 持平（part06 改动由 WPT 资产锁定） |
| engine --lib（quickjs） | 1466 | 1466 | 零回归 |
| wpt-runner 全部单测 | 191 | 191 | 零回归 |
| clippy（wpt-runner v8 + quickjs、engine） | 干净 | 干净 | — |
| fmt | — | 无 diff | — |

## 三、教训

1. **「部分通过」会掩盖基建缺口**：handler-count 因 `|| 'window'` 默认值在无 query 时
   碰巧能跑 window 变体，让 variant 缺口少暴露 8 轮——`?document`/`?element` 两个
   变体从未被 runner 执行过。基线巡检要看「用例声明了什么」而不是「跑了什么」。
2. **variant 用例的基础 URL 是伪用例**：上游 harness 对依赖 query 的用例在无 query 时
   不注册任何 test——runner 逐变体跑时须跳过基础 URL，否则 0-subtest 空转落 Timeout
   伪败（本次设计先跳过；后续若上游出现「基础 URL 也有断言」的混合形态再评估）。
3. **两件同表象（attachShadow TypeError vs onclick 不触发）不同域**：一件在 runner 基建
   （query 缺失），一件在 shim 语义面（handler 列表不全）——探针把两个失败都归到
   `mode=null` 链路会误修；分步探针（search 是否为空 → doc onclick 是否注册 → 派发
   tally）按域拆开后各自一行修复。
