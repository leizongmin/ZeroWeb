# Worker 并发兼容 — 运行时控制面板（master.md）

**入口文档**: [../workers-compat.md](../workers-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-20（M1 收口——corpus 通道 + 基线 45.6%）

---

## 当前状态

**M1 完成（2026-10-20）**：workers corpus（526 文件，pin `3159769338`）导入 +
`testharness-workers` 通道 + 可执行子集甄别（161 案 / 371 子测试）+ 基线
**169/371 = 45.6%**（131 F + 71 T，59 案全绿）。证据：
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
| P1 | workers/dedicated-workers corpus 导入 + 基线 | ✅ M1（2026-10-20；pin 无 dedicated-workers 目录，workers/ 全量承载；21 案 .worker.js 挂 M2 harness 通道评估） |
| P2 | 与 zero-page-runtime 的 worker 契约协商（实例归属/消息通道） | ⏳ M2 前置（同 realm 实现不动进程/协议面，预期契约面收窄） |
| P3 | worker 全局作用域最小面（self/navigator/location/onerror） | ⏳ M2——WorkerNavigator 缺位、WorkerLocation in-worker 引用、FormData/indexedDB 缺位（基线簇 2） |
| P4 | classic 脚本执行 + importScripts；module worker 评估 | ⏳ M2——importScripts MIME 白名单/网络错误/跨源报告 ~34F（基线簇 3）；module 面目录级排除记账 |
| P5 | postMessage/structuredClone 往返 + transferable（ArrayBuffer transfer） | ⏳ M3——transfer 序列校验 + port 转移 + e.ports ~12F（基线簇 4） |
| P6 | SharedWorker / nested worker 挂账定稿 | ⏳ M4（SharedWorker 排除面 66 案已在基线甄别表记账） |

M2/M3 全部失败簇清单（8 簇，含 ErrorEvent 语义 / 构造器 WebIDL / terminate 结构性
限制）见基线 evidence「M2/M3 失败簇」节。

## 已完成切片

- **M1（2026-10-20）**：`70-workers-compat.sh` 深挖 + any.js opt-in（GOAL_FETCH_DEEP=1，
  语料三层嵌套 shallow 拉不全）；`testharness_workers.rs` 通道（全递归扫描 +
  workers_case_skipped 规则，与 runner 实跑集合逐一取证一致）；Makefile
  `fetch-wpt-workers` / `testharness-workers`；基线 45.6% + suites CSV 回填。
  lib.sh 深挖扩展模式补 `.htm`/`.headers`（共享面一行级改动，与 shallow 同表——
  渲染流不触此文件）。

## 下一步计划

1. **M2 前置**：zero-page-runtime 契约确认（worker 实例归属/消息通道是否触协议面；
   同 realm 近似预期不动 IPC 契约——确认后记 evidence）
2. **M2**：按基线簇收敛——① worker 全局最小面（navigator/location，簇 2）；
   ② importScripts MIME/网络错误语义（簇 3）；③ ErrorEvent 位置属性（簇 5）；
   ④ 构造器 WebIDL 分型（簇 6）；每簇带单测/集成测试
3. **M3**：transfer/ports + structured-clone 语义（簇 4）；terminate/事件循环簇
   （簇 7）评估记账
4. 门禁：每切片 `make test` 全绿 + clippy -D warnings + fmt；v8/quickjs 双矩阵在
   M2 动 shim 后评估

**待用户决策清单**：（空）
