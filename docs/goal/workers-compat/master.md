# Worker 并发兼容 — 运行时控制面板（master.md）

**入口文档**: [../workers-compat.md](../workers-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-21（P7 后半 + 环境值面收口——importScripts 错误归因 + worker rAF/search 97.1%）

---

## 当前状态

**P7 后半 + 环境值面完成（2026-10-21）**：importScripts 错误归因（共享三相实现
——调用点/被导入脚本/嵌套内层优先，report-error-\* 13F 翻绿）+ worker 作用域
rAF（setTimeout 逐回调派发 + 数值严格递增时间戳）+ URL search 空 query 根因修
（共享序列化器，裸尾 `?` 返空串）。全量 **693/714 = 97.1%**
（M3 收口 95.0% → +15P，含 P7 会话内 691 锚点）。证据：
[evidence/2026-10-21-p7-import-error-attribution-env-values.md](evidence/2026-10-21-p7-import-error-attribution-env-values.md)
（含残余 12F+8T+1PF 六簇清单）。引擎单测 part61 累计 275+6 案，
`cargo test -p zero-engine --lib` 3339/3339。

**M3 完成（2026-10-21）**：transfer/ports + structured-clone 语义（serialize-with-
transfer 同一转移体、validate-then-consume 三相、MessagePort 改挂随迁、视图面
DataView/resizable/OOB/SAB、包装对象/Error/FileList/ImageData/OffscreenCanvas）+
postMessage 第二参联合形态 + 投递载体 microtask→task + ImageBitmap/MessagePort
接口类化。**678/714 = 95.0%**（M2 切片三 91.1% → +165P，Timeout 11→8）。
证据：[evidence/2026-10-21-m3-transfer-ports-structured-clone.md](evidence/2026-10-21-m3-transfer-ports-structured-clone.md)
（分母 563→714 因 battery 支撑资产补齐）。
提交 `3ca356bb8`。

**M2 切片三完成（2026-10-21）**：错误管线三相（worker 全局 onerror legacy 5 参
签名 + 返回 true 抑制 + 派发去重 + 解析错误 plain Event/运行时错误 ErrorEvent
分型）+ close() 全语义（出站消息仍投递/入站丢弃/定时器清空与排程禁止/同任务
错误仍上报）+ worker 全局面收口（onoffline/ononline、btoa/atob、WorkerPerformance
measure、MessageEvent data 默认值、WorkerLocation 掩码、顶层 this===self）+
构造器 WebIDL（无参 TypeError、真实 base 解析失败 SyntaxError）+ dedicated 变体
对外命名 `.any.worker.js` + `//META:` 无空格前缀。全量 **513/563 = 91.1%**
（上一切片 392/543 = 72.2% → +121P；对会话内锚点回归 0，Timeout 25→11）。
证据：[evidence/2026-10-21-m2-slice3-error-pipeline.md](evidence/2026-10-21-m2-slice3-error-pipeline.md)
（含残余 39F+11T 六簇清单）。引擎单测 part61 新增 11 案，`cargo test -p
zero-engine --lib worker` 47/47。

**M2 切片二（2026-10-20）**：M2 harness 通道（any.js dedicated 变体外链化 +
.worker.js in-worker 形态，可执行面 175 案）+ importScripts 三相语义
（内联/运行时双路径 + data:/blob: MIME + 原生 DOMException）+ worker 全局最小面
（WorkerNavigator/WorkerLocation/接口 own-property 镜像）+ error 事件 ErrorEvent 化。
收口 **392/543 = 72.2%**。
证据：[evidence/2026-10-20-m2-slice2-runtime.md](evidence/2026-10-20-m2-slice2-runtime.md)。

**M1（2026-10-20）**：workers corpus（526 文件，pin `3159769338`）导入 +
`testharness-workers` 通道 + 可执行子集甄别 + 基线 **169/371 = 45.6%**。证据：
[evidence/2026-10-20-m1-baseline.md](evidence/2026-10-20-m1-baseline.md)。

**架构现状修正（对入口文档 2026-09-12 基线事实）**：「Worker 构造器桩」描述过时——
页面管线 shim（`js_dom_shim/part05.js`，R3091/R34xx 演进）已有**同 realm worker
运行时近似**：data:/blob:/外链三源脚本执行、DedicatedWorkerGlobalScope/
WorkerGlobalScope 原型链、structuredClone + queueMicrotask 双向 postMessage、
importScripts 内联预取、WorkerLocation、嵌套 Worker、onerror 微任务派发。
`dom_bridge.rs` 的 echo 桩属 execute_script 旧路径 polyfill，非页面管线。
M2 按「不另起炉灶」约束在此实现上补语义（M1 失败簇即工作清单）；独立 isolate/
线程隔离按 module worker 同款评估记账，不预先承诺。

**与兄弟 goal 的边界**：
- zero-page-runtime — worker 实例归属/消息通道**先对契约再动协议**（§9）
- script-sandbox / wasm-sandbox — 隔离组织范式复用，其对外宿主桥接边界不碰
- event-loop-spec（已归档）— 微任务机制先例复用，遗留问题记账回流
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | workers/dedicated-workers corpus 导入 + 基线 | ✅ M1 + M2 通道扩展（175 案/563 子测试，含 any.js dedicated 变体 + .worker.js 形态） |
| P2 | 与 zero-page-runtime 的 worker 契约协商（实例归属/消息通道） | ⏳ M3 前置（transfer/ports 若触协议面先对契约） |
| P3 | worker 全局作用域最小面（self/navigator/location/onerror） | ✅ 大面完成 + 值域收口（rAF 数值递增、search 空 query 根因修）；残余记账：Worker-location.sub `:80`、platform、userAgentData（P8 合成 origin 簇）、resource timing/nested_sync_xhr（功能缺口） |
| P4 | classic 脚本执行 + importScripts；module worker 评估 | ✅ 完成（三相语义 + MIME 全集 + 原生 DOMException + 调用点/被导入脚本/嵌套内层错误归因）；module 面目录级排除记账不变 |
| P5 | postMessage/structuredClone 往返 + transferable（ArrayBuffer transfer） | ✅ M3 完成（同一转移体 + validate-then-consume + 视图面全集；残余仅 ReadableStream 1PF 先决条件面） |
| P6 | SharedWorker / nested worker 挂账定稿 | ⏳ M4（SharedWorker exposure 1F + ctor-\* 3T + navigator/007 1T 等待型；定稿挂账） |
| P7 | ErrorEvent 溯源簇（filename/lineno 精确上报） | ✅ 完成（前半 legacy 5 参 + 抑制面；后半 importScripts 调用点/被导入脚本/嵌套内层优先归因，13F 翻绿） |
| P8 | runner 基建：wptserve 动态 handler 302 重定向（baseurl 族） | ⏳ 记账（baseurl 3F+3T + location/redirect 1T + Worker-location.sub/platform/userAgentData 3F——302 语义 + 合成 origin 恒 https 两根因；URL 编码 2F 同属文档编码管线缺口） |

残余失败簇最新清单（6 簇）见
[evidence/2026-10-21-p7-import-error-attribution-env-values.md](evidence/2026-10-21-p7-import-error-attribution-env-values.md)
「残余 21」节。

## 已完成切片

- **P7 后半 + 环境值面（2026-10-21）**：importScripts 错误归因共享三相实现
  （`_zwCaller` 调用点 / `_zwScriptUrl` 被导入脚本 / 嵌套内层优先；构造期调用点
  改写 `__zwImportTracked` 不偏移行号 + 递归改写跨层 helper）+ worker 作用域
  rAF（setTimeout 逐回调 + 数值严格递增 + cancel 句柄表）+ URL search 空 query
  根因修（serialize_url 裸尾 `?` 返空串）。97.1%（+15P）。引擎单测 part61 +6 案。
- **M3（2026-10-21）**：transfer/ports + structured-clone 语义（serialize-with-
  transfer 同一转移体 + validate-then-consume 三相 + MessagePort 改挂随迁 +
  视图面 DataView/resizable/OOB/SAB + 包装对象/Error/FileList/ImageData/
  OffscreenCanvas + postMessage 第二参联合 + 投递 task 化 + 接口类化）。
  95.0%（+165P，Timeout 11→8）。引擎单测 part61 9 案。提交 `3ca356bb8`。
- **M2 切片三（2026-10-21）**：错误管线三相（legacy 5 参 onerror + 返回 true
  抑制 + dispatch 置空去重 + 解析/运行时错误分型）+ close() 全语义（出站投递/
  入站丢弃/定时器清空+排程禁止+序号自增/同任务错误上报）+ 全局面（onoffline/
  ononline、btoa/atob、performance.measure、MessageEvent data 默认、Location
  掩码、顶层 this===self、self 赋值静默）+ 构造器 WebIDL（无参 TypeError/
  真实 base SyntaxError）+ `.any.worker.js` 对外命名 + `//META:` 无空格前缀。
  91.1%（+121P，Timeout 25→11）。引擎单测 part61 新增 11 案。
- **M2 切片二（2026-10-20）**：harness 通道（dedicated 变体经 fetcher 返回
  ClassicWorkerHandler 同构包装——data: URL wrapper 的 base 解析缺陷废弃）+ script
  MIME 门（sidecar/imported_script.py/扩展名三序）+ importScripts 三相（内联
  direct eval 同 var-env + 运行时兜底，注释感知实参解析）+ data:/blob: MIME 判定 +
  原生 DOMException（R384 先例）+ WorkerNavigator/WorkerLocation/接口 own-property
  镜像 + error 事件 ErrorEvent 化。72.2%（+160P）。引擎单测 part61 8 案。
- **M1（2026-10-20）**：`70-workers-compat.sh` 深挖 + any.js opt-in（GOAL_FETCH_DEEP=1，
  语料三层嵌套 shallow 拉不全）；`testharness_workers.rs` 通道（全递归扫描 +
  workers_case_skipped 规则，与 runner 实跑集合逐一取证一致）；Makefile
  `fetch-wpt-workers` / `testharness-workers`；基线 45.6% + suites CSV 回填。
  lib.sh 深挖扩展模式补 `.htm`/`.headers`（共享面一行级改动，与 shallow 同表——
  渲染流不触此文件）。

## 下一步计划

1. **M4 收口评估**：对照入口文档 DC-1~4 逐项判定——DC-4 需最终 `make test`/
   `make reftest` 零回归确认 + clippy/fmt；残余 12F+8T+1PF 定稿（P6
   SharedWorker/nested 挂账 + P8 runner 基建/合成 origin + URL 编码/terminate
   深度/同步 XHR 功能缺口，均不阻 DC 即记 license）。
2. **可选推进**（不阻 M4）：P8 runner 302 重定向语义（baseurl 族 3F+3T）；文档
   编码 → worker URL 解码管线（URL 编码 2F）；worker resource timing 采集。
3. 门禁：每切片 `make test` 全绿 + clippy -D warnings + fmt；v8/quickjs 双矩阵在
   M2 动 shim 后评估。

**待用户决策清单**：（空）
