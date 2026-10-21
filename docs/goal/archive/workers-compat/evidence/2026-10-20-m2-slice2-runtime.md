# M2 切片一：worker harness 通道 + importScripts 三相 + 全局最小面（2026-10-20）

## 范围与结果

本切片两块（对应 master 缺口 P3/P4 前半 + 通道解锁）：

1. **M2 harness 通道**（前 session 落在 working tree，本 session 验证 + 重构收口）：
   `.any.js` dedicated 变体 + `.worker.js` in-worker harness 形态接入
   `testharness-workers`；可执行面 161 案/371 子测试 → **175 案/543 子测试**。
2. **importScripts 三相语义 + worker 全局最小面**（engine shim + runner fetcher）。

| 基准 | 案 | 子测试 | 通过 | 率 |
|---|---|---|---|---|
| M1 窗口基线（2026-10-20-m1-baseline） | 161 | 371 | 169 | 45.6% |
| M2 通道基线（通道落地、语义切片前） | 175 | 497 | 232 | 46.7% |
| **M2 切片一后（本证据）** | 175 | 543 | **392** | **72.2%** |

JSON：`2026-10-20-m2-channel-baseline.json`（通道基线）、
`2026-10-20-m2-slice2-runtime.json`（切片后全量）。

## 变更明细

### runner 侧（tests/wpt-runner）

- **dedicated 变体外链化**：`.any.js` 作 worker 顶脚本改经
  `wpt_data_script_fetcher` 返回 ClassicWorkerHandler 同构包装源
  （`synthesize_any_worker_source`，正文内嵌防自引用成环 + `.sub` 模板全集替换），
  页面 `new Worker('/…case.any.js')`——worker 自真实合成 origin（wpt.test）服务。
  原 data: URL wrapper 形态 base 解析对根绝对 importScripts 必抛 SyntaxError，
  dedicated 通道 importScripts 族全灭 `tests:0`（catch.sub.any.js 等 7 案簇）。
- **script MIME 门**（fetcher）：派生序 wptserve 语义——① `<file>.headers`
  sidecar；② `workers/support/imported_script.py?mime=` 动态 handler（scripty
  成功回 `"\n"` 空体语义，不读 Python 源）；③ 扩展名 .js/.mjs。非 scripty →
  NetworkError（`Worker_script_mimetype`/`WorkerGlobalScope_importScripts_NosniffErr` 面）。
- **sidecar 目标带拉**（lib.sh）：`GOAL_PULL_HEADERS_TARGETS=1` 拉无扩展名脚本
  目标（`1`/`null`/`undefined`，wptserve foo+foo.headers MIME sidecar 语义）。

### engine shim 侧（js_dom_shim/part05.js）

- **importScripts 三相**（spec
  [worker-processing-model](https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-importscripts)）：
  ① 全参 URL 解析（失败 SyntaxError DOMException、不执行任何）；② 全参 fetch
  （失败 NetworkError、不执行——blob-url「前脚本 revoke」案钉全量先取）；③ 顺序
  执行（SyntaxError/运行时异常执行位透传中止后续）。双路径：
  - **构造期内联**（字面量/常量参数 + 语句位：源首/`;`/`}`/`{`/`)`/`:`）重写为
    direct eval 序列——sloppy direct eval var/function 声明落调用位 var-env，
    定义与 worker 脚本共享作用域（008/009 断言面）。实参解析注释感知（004 行注释）。
  - **运行时兜底**（变量参数）：Function 隔离执行，定义不泄漏——限制记账 P4。
  - 旧「顶部提升 + 正则剥行」实现删除（try/catch 内失败/顺序语义丢失）。
- **data:/blob: MIME 判定**（`_zwWorkerImportSource`）：data: 取 URL mediatype
  （缺省 text/plain 非 scripty）、blob: 取 Blob.type；同一 JavaScript MIME 全集
  （mimesniff）。`importscripts_mime(_local)` 72 子测试全绿。
- **DOMException 原生构造器**（R384 wrong-global 先例）：importScripts 相失败抛
  `new (globalThis.DOMException || DOMException)(…)`——词法 shim 构造器与用例
  经 wctx 镜像的原生构造器恒不同一（002 案 `assert_throws_dom` 面）。
- **worker 全局最小面**（P3）：`WorkerNavigator`（22 键 prototype accessor +
  只读赋值 TypeError）、`WorkerLocation`（href 等 8 键 + toString）、接口对象
  own-property 镜像表（Event/MessageChannel/FormData/Blob/FileReader/DOMException/
  类型化数组等 42 键）——`interface-objects/001.worker.js` own-property 面与
  `WorkerNavigator.any.js` 只读面。
- **worker error 事件升级 ErrorEvent**：blob 解析失败/脚本抛/fetch 失败三处派发
  `new ErrorEvent('error', { message, filename })`（spec error 事件即 ErrorEvent）。
- **fetch 失败 error 事件**：host 注册 fetcher 时外链 worker 取不到 → 异步
  ErrorEvent（R3080 未注册静默语义保留）。

### 引擎单测（js_dom_bridge_tests/part61.rs，新增 8 案）

003 data: 共享作用域顺序执行 / 004 中段语法错 + 注释实参表 / 002 非法 URL 原生
SyntaxError 且不执行 / fetch 失败 NetworkError 且不执行 / 006 运行时 throw 中止
后续 / 010 常量参数 undefined / navigator+location+接口 own property 整串断言 /
fetch 失败 error 事件 message。`cargo test -p zero-engine --lib test_worker_`
16/16（含既有 worker 案回归）。

## 残余失败簇（151F，后续切片清单）

| 簇 | F | 归属 |
|---|---|---|
| workers/interfaces 细粒度属性（WorkerNavigator 键值/WorkerLocation URL 解析） | 57 | M2 簇 2 后半 |
| workers/semantics（ErrorEvent filename/lineno 溯源、encodings） | 19 | M2 簇 5 |
| workers/constructors（构造器 WebIDL 分型/Symbol.toStringTag） | 17 | M2 簇 6 |
| Worker-multi-port / Worker-messageport（transfer/ports） | 10 | M3 簇 4 |
| worker-performance（performance.timing/navigation 须 undefined） | 4 | M2 簇 2 后半 |
| baseurl（`importScripts.py` 302 重定向 handler——runner 无 wptserve 动态重定向） | 3 | runner 基建记账 |
| data-url / parse-error-failure（parse error 溯源） | 4 | M2 簇 5 |
| 零星单案（constructor-proto/formdata/location/nested-importScripts-error） | 4 | 随簇收敛 |

report-error-*（6 案，workers/interfaces 簇内）需 worker 全局未捕获错误的
e.error/filename/lineno 精确上报（helper 断言 lineno=8）——归 ErrorEvent 溯源簇。

## 验证

- `cargo test -p zero-engine --lib test_worker_`：16/16。
- 定向：importScripts 族 27P2F→25P0F（002–012 全绿）；mime 族 36P36F→72P0F；
  catch.sub.any.js 0P→6P。
- 全量：`testharness-workers --json` 392/543 = 72.2%（见上表）。
- 跨通道回归：cache-storage / webmessaging spot-check（fetcher 共享面）——
  见 master 本轮记录。
