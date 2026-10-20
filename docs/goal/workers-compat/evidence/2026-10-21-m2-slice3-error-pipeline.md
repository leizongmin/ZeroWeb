# M2 切片三——错误管线三相 + close 语义 + 全局面收口（2026-10-21）

全量 **513/563 = 91.1%**（513P/39F/11T，175 案）。证据 JSON：
[2026-10-21-m2-slice3-error-pipeline.json](2026-10-21-m2-slice3-error-pipeline.json)
（runner `testharness-workers --json` 原样输出）。

## 数字对照

| 阶段 | Pass/Total | 率 | 备注 |
|---|---|---|---|
| M2 切片二收口（commit `412e17273`） | 392/543 | 72.2% | 上一切片台账 |
| 本切片会话内锚定全量（`/tmp` 运行，部分修复已入） | 447/562 | 79.5% | 回归对照锚点 |
| 本切片收口 | 513/563 | 91.1% | 对锚点回归 **0**，改善 66 子测试 |

分母 543→563：dedicated 变体外链化后原 `tests:0` 案例产出真实子测试。
Timeout 25→11。

## 修改面

### 1. 错误管线三相（P7 前半）

- worker 全局 `onerror` 走 legacy 5 参签名（message/filename/lineno/colno/error），
  返回 `true` → 抑制向 page 的 error 上报（`_reportError` 三相：① worker 全局
  dispatchEvent、①' legacy 调用、② page Worker 对象派发）。
- 派发去重：① 相 dispatchEvent 期间将 `onerror` own 属性临时置空
  （`_zwSavedOnErr` 保存/恢复），generic dispatch 的 on* 调用不重复触发，
  legacy 调用为唯一调用点。
- 解析错误 → page 侧 plain `Event`（非 ErrorEvent，
  dedicated-worker-runtime-error-is-not-parse-error 面）；运行时错误 → ErrorEvent
  （`error` 值仅 worker 内可见，page 侧 null——Worker_ErrorEvent_error 双面）。
- `WorkerGlobalScope_ErrorEvent_{message,filename,lineno,colno}.htm` 四案参数型面
  翻绿；`Worker-simultaneous-errors` 并发错误互不吞。

### 2. close() 全语义（upstream closeWithPendingEvents / closeWithError 面）

- `close()` 后 worker→page `postMessage` 仍投递（出站只受 `_terminated` 门；
  WorkerGlobalScope-close「close 后消息送达」+ sending-messages 面）。
- close 后 page→worker 投递丢弃（入站 guard `_terminated || _closed`）。
- close 前已排程定时器经 `_timerIds` 注册表清空（宿主定时器已入队回调不可撤回 →
  显式 clearTimeout/clearInterval）；close 后新定时器不排程但仍返回自增序号
  （WindowTimers/005「close 后 setInterval 返回 1 而非 0」面，`_closedTimerSeq`）。
- 同任务内 throw 后 close() 错误仍上报（`_zwReportToOwner` queueMicrotask 只查
  `_terminated`——closeWithError 面）。

### 3. worker 全局面补齐（P3 后半收口）

- `onoffline`/`ononline` own 属性、`btoa`/`atob` 接口镜像
  （DedicatedWorkerGlobalScope-members 19 子测试全绿）。
- WorkerPerformance：`measure(name, start, end)` 解析 mark startTime；
  `timing`/`navigation` undefined；`clearResourceTimings`/
  `setResourceTimingBufferSize` 桩（resource timing 记账 P3 残余）。
- MessageEvent 构造器 `data` 默认 null + `Symbol.toStringTag`；
  WorkerLocation `toString` 掩码；Worker/WorkerLocation 原型 toStringTag。
- 顶层 `this === self`、字符串定时器回调内顶层函数可达（with(self) 编译）、
  `self = 1` 赋值静默忽略（getter-only accessor）。

### 4. 构造器 WebIDL 面

- 无参 → TypeError；URL 解析失败且存在真实 base → SyntaxError DOMException
  （Worker-constructor invalid-URL/no-args 面）；about:blank 兜底（R3080 嵌入面）
  保持静默回退——`test_worker_api_surface_r3080` 等 base-less 单测场景不受影响。
- `Worker.prototype[Symbol.toStringTag] = 'Worker'`（proto 可写面）。

### 5. runner 侧（tests/wpt-runner）

- dedicated 变体对外 URL 命名 `.any.worker.js`（upstream wptserve
  ClassicWorkerHandler 面——examples/general.any.js 断言 `self.location.pathname`），
  fetcher 磁盘仍读 `.any.js` 源（`strip_suffix(".any.worker.js")` 映射）。
- `//META:` 无空格前缀兼容（upstream serve.py `//\s*META:`——
  Worker-constructor-proto.any.js 实例）。

### 6. 引擎单测（part61）

11 案新增（probe 删除）：onerror 返回值签名/抑面、解析错误 plain Event、
字符串定时器顶层可达、顶层 this、self 赋值、close 后 postMessage、close 页面流、
platform 对象 structuredClone 拒绝、performance 面、MessageEvent 面、
WorkerLocation 掩码。`cargo test -p zero-engine --lib worker` 47/47。

## 残余 50 失败（39F + 11T，36 案）分簇

| 簇 | 案×子测试 | 缺口 |
|---|---|---|
| ErrorEvent 溯源（P7 后半） | report-error-\* 6×2 + Worker-nested-importScripts-error 1 = 13F | importScripts 调用点 filename/lineno 归因（需脚本装载表） |
| 基建 302/baseurl（P8） | baseurl/alpha 1F+2T、use-base-url 1F、dedicated-worker-from-blob-url 1F、location/redirect 1T | runner 无重定向语义；blob URL base 解析 |
| M3 transfer/ports（P5） | Worker-multi-port 7F、Worker-messageport 1F+2T、second-argument-null 1F、structured-clone-imagedata 1F、semantics/structured-clone/dedicated 1F、Worker-structure-message 1T、Worker-termination-with-port-messages 1T | MessageChannel/MessagePort 序列化 + transfer 面未实现 |
| 环境值面 | Worker-location.sub 1F（`:80` 端口形态）、WorkerLocation_search_empty 1F（`"?"` vs `""`）、WorkerNavigator_platform 1F（"Win32"）、userAgentData 1F、rAF 1F、worker-performance resource timing 1F、nested_worker_sync_xhr 1F | 值域对齐/依赖面（XHR、rAF dt 类型） |
| URL 编码 | semantics/encodings/003 1F、worker-url-encoding 1F | 文档编码 → worker URL 解码管线 |
| terminate 深度 | Worker-terminate-forever 1F、Worker_terminate_event_queue 1F | terminate 中断执行语义（同 realm 近似的固有边界） |
| 排除面记账 | exposure.any.js SharedWorker 1F、ctor-\{1,null,undefined\} 3T、navigator/007 1T | SharedWorker 挂账（P6）；ctor-\* 页面无完成信号等待型（M1 基线即有）；navigator/007 遍历超时（M1 基线即有） |

Timeout 11 清单：baseurl/alpha×2、location/redirect、ctor-×3、navigator/007、
Worker-messageport×2、Worker-structure-message、Worker-termination-with-port-messages。
11 项均为 M1 基线即有（Worker-messageport×2 与 Worker-termination-with-port-messages
成因在 ports 面，归 M3 工作，但 M1 基线即已 Timeout）。
