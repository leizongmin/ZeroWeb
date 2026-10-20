# Worker 并发兼容 — 运行时控制面板（master.md）

**入口文档**: [../workers-compat.md](../workers-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-20（M2 切片一收口——harness 通道 + importScripts 三相 + 全局最小面 72.2%）

---

## 当前状态

**M2 切片一完成（2026-10-20）**：M2 harness 通道（any.js dedicated 变体外链化 +
.worker.js in-worker 形态，可执行面 175 案/543 子测试）+ importScripts 三相语义
（内联/运行时双路径 + data:/blob: MIME + 原生 DOMException）+ worker 全局最小面
（WorkerNavigator/WorkerLocation/接口 own-property 镜像）+ error 事件 ErrorEvent 化。
全量 **392/543 = 72.2%**（M1 窗口基线 45.6%、M2 通道基线 46.7% → +160P）。
证据：[evidence/2026-10-20-m2-slice2-runtime.md](evidence/2026-10-20-m2-slice2-runtime.md)
（含残余 151F 七簇清单）。引擎单测 part61 8 案 + `test_worker_` 16/16。

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
| P1 | workers/dedicated-workers corpus 导入 + 基线 | ✅ M1 + M2 通道扩展（175 案/543 子测试，含 any.js dedicated 变体 + .worker.js 形态） |
| P2 | 与 zero-page-runtime 的 worker 契约协商（实例归属/消息通道） | ⏳ M3 前置（transfer/ports 若触协议面先对契约） |
| P3 | worker 全局作用域最小面（self/navigator/location/onerror） | ◐ M2 前半完成（构造器 own-property/只读 TypeError/error 事件 ErrorEvent 化）；后半：WorkerNavigator 键值精确面 + WorkerLocation URL 解析 57F、performance.timing/navigation undefined 4F |
| P4 | classic 脚本执行 + importScripts；module worker 评估 | ◐ M2 前半完成（三相语义 + MIME 全集 72P + 原生 DOMException）；后半：运行时路径定义泄漏记账（P4 限制）、module 面目录级排除记账不变 |
| P5 | postMessage/structuredClone 往返 + transferable（ArrayBuffer transfer） | ⏳ M3——Worker-multi-port 7F + Worker-messageport 3F + e.ports 面 |
| P6 | SharedWorker / nested worker 挂账定稿 | ⏳ M4（SharedWorker 排除面 66 案已在基线甄别表记账） |
| P7 | ErrorEvent 溯源簇（filename/lineno 精确上报，report-error-* 6 案断言 lineno=8） | ⏳ M2 后半（semantics 19F + parse-error 2F 同簇） |
| P8 | runner 基建：wptserve 动态 handler 302 重定向（baseurl importScripts.py 族 3F） | ⏳ 记账（文件型 runner 无重定向语义；不为一案簇引入 HTTP 栈） |

M2/M3 失败簇最新清单（7 簇）见
[evidence/2026-10-20-m2-slice2-runtime.md](evidence/2026-10-20-m2-slice2-runtime.md)
「残余失败簇」节。

## 已完成切片

- **M2 切片一（2026-10-20）**：harness 通道（dedicated 变体经 fetcher 返回
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

1. **M2 后半**：① ErrorEvent 溯源簇（P7——filename/lineno，含 report-error-*）；②
   interfaces 细粒度属性面（WorkerNavigator 键值/WorkerLocation URL 解析 57F）；③
   构造器 WebIDL 分型（Symbol.toStringTag 等 17F）；④ worker_performance undefined 面。
2. **M3**：transfer/ports + structured-clone 语义（P5）；动协议面前先对 zero-page-runtime
   契约（P2）；terminate/事件循环簇评估记账。
3. **M4**：DC 判定收口 + SharedWorker/nested/module 挂账定稿（P6）。
4. 门禁：每切片 `make test` 全绿 + clippy -D warnings + fmt；v8/quickjs 双矩阵在
   M2 动 shim 后评估。

**待用户决策清单**：（空）
