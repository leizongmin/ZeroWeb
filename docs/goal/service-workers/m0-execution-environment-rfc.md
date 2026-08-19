# RFC：Service Worker 独立执行环境与宿主所有权

**版本**：v1.1
**日期**：2026-08-19
**状态**：已批准（2026-08-19，方案 C）
**关联目标**：[../service-workers.md](../service-workers.md)
**WPT 证据**：[evidence/2026-08-19-m0-wpt-executable-surface.md](evidence/2026-08-19-m0-wpt-executable-surface.md)

---

## 0. 执行摘要

- **目标**：确定 SW 脚本在哪执行、谁拥有注册和控制状态、事件如何跨线程/进程派发。
- **推荐**：选择“复用并抽取 Web Worker 线程基建”的方案 C。每个运行中的 SW 使用独立
  JS 引擎线程；浏览器侧 `ServiceWorkerManager` 拥有 origin/scope 注册、版本、client
  control 和 fetch 路由；SW runtime 只执行脚本和事件。
- **不选择**：页面 JS 线程内增加 V8 context；从零复制一套独立线程 runtime。
- **M1 首交付**：真实抓取脚本、独立 global scope、install/activate + `waitUntil()`，
  以 WPT `activation-after-registration.https.html` 驱动。
- **M2 门禁**：等 js-dom S6 fetch 改造和 storage-cache-api M1 land 后，才把 browser
  fetch proxy 接入 `FetchEvent.respondWith()`。
- **批准记录**：用户于 2026-08-19 明确回复“批准方案 C RFC”，M0 启动门禁解除。

## 1. 决策边界

### 1.1 在范围内

- SW 独立脚本执行模型及 V8/QuickJS 双后端边界。
- 注册、版本、生命周期、client control 的 owner。
- 页面、browser、renderer、embedded WebView 间的命令与事件边界。
- install/activate/fetch/message 的异步完成模型。
- M1-M3 的模块职责、提交顺序、验证与回滚切点。

### 1.2 不在范围内

- Cache API 自身语义，归 `storage-cache-api` goal。
- Push、Notification、Background Sync。
- 多窗口/多 iframe 完整 client 枚举。
- SW 进程级隔离；本 RFC 选择线程隔离作为当前产品 envelope。
- 在 M0 阶段修改任何 Rust/JS/测试 runner 源码。

### 1.3 需求来源

| 类型 | 来源 |
|------|------|
| Mission / Done Criteria | `docs/goal/service-workers.md` |
| 当前实现事实 | storage/webview/engine/browser/renderer/script-sandbox 源码 |
| 规范行为 | Service Workers CRD + HTML Worker 模型 |
| 验收输入 | 上游 WPT `service-workers` |
| 并行约束 | `docs/rally/run-rules.md`、js-dom/storage-cache-api master |

## 2. 现状与关键纠偏

### 2.1 已有底座

1. `zero-storage` 有 `ServiceWorkerRegistry`、状态枚举、scope 匹配和每注册 CacheStorage。
2. WebView `fetch_url()` 在手工激活 registry 后可对主文档做静态 cache-first 拦截。
3. browser/renderer 均有独立页面 JS 线程、异步 resolver、FetchBridge 和 timer bridge。
4. `zero-script-sandbox::WorkerRuntime` 已有独立 OS 线程、独立 V8/QuickJS context、超时
   看门狗、强制中断和 bounded join。
5. browser process 已拥有多进程网络代理和持久 IndexedDB，具备成为 SW owner 的结构位置。

### 2.2 不能沿用的近似

- 页面 `navigator.serviceWorker` 维护私有 JS 数组，与 Rust registry 完全断开。
- `setTimeout(0)` 同时推进 installed/activated，未等待 worker Promise。
- `WorkerRuntime` 只接受 `Execute/PostMessage/Terminate` 字符串命令，只输出字符串消息；
  它不能直接表示 ExtendableEvent、FetchEvent、`waitUntil()` 或 `respondWith()`。
- Dedicated Worker 的生产 shim 仍是同 sandbox `new Function` 影子执行，不能作为 SW 隔离
  环境复用。
- WebView 现有 fetch 拦截只是“激活 registry 后查 cache”，不是执行 SW fetch handler。

### 2.3 所有权不变式

1. 注册和 active/waiting/installing 版本状态必须有单一 owner。
2. 多进程产品路径的 owner 必须在 browser process：网络请求在此汇聚，renderer 可重启，
   SW 仍需控制后续导航。
3. embedded WebView 没有 browser process，因此在实例内持有同一 manager 的 in-process
   adapter；算法与状态模型不能另写一份。
4. JS 引擎对象不跨线程；跨边界只传纯值 wire/typed enum。
5. fetch 未被 `respondWith()` 接管或 worker 失败时必须明确 pass-through，不得悬挂请求。

## 3. 候选方案

| 维度 | A：页面线程独立 context | B：全新 SW 线程 | C：抽取 Worker 线程核 |
|------|-------------------------|-----------------|----------------------|
| 脚本全局隔离 | 有 | 有 | 有 |
| 调度线程隔离 | 无 | 有 | 有 |
| 复用超时/终止安全 | 部分 | 无，需重写 | 完整 |
| V8/QuickJS 对称 | 需新增 | 需新增 | 已有基础 |
| 与 browser owner 配合 | 中 | 中 | 好 |
| 初始代码量 | 低 | 高 | 中 |
| 长期维护 | 差 | 差 | 好 |
| 规范方向 | 不接受 | 可接受 | 推荐 |

### 3.1 方案 A：同线程独立 V8 context

优点是首个 demo 代码最少。但页面脚本死循环或长任务会阻塞 SW；SW fetch handler 又可能在
页面 fetch Promise 的同一命令循环中形成重入/饥饿。现有 `Sandbox` 只暴露单 persistent
context，宿主回调注册表还是线程局部；增加多 context 会把 context identity、回调路由和
microtask checkpoint 全塞入页面 worker。

**裁决**：拒绝。它只能提供对象隔离，不能提供执行调度隔离，也与“页面尚未存在时处理导航”
的 SW 模型冲突。

### 3.2 方案 B：从零实现 SW 专用线程

语义上可行，但会复制 `WorkerRuntime` 已解决的 V8 初始化、堆限制、死循环中断、看门狗、
bounded join 和 QuickJS interrupt handler。两套安全机制很快会漂移。

**裁决**：拒绝。除非抽取线程核被实证不可行，否则不接受安全基础设施复制。

### 3.3 方案 C：抽取 Worker 线程核，增加 SW typed runtime

把 `WorkerRuntime` 中与 Dedicated Worker 无关的部分抽成 crate-private/public
`ThreadedScriptRuntime` 核：线程创建、引擎 context、命令 loop、超时、终止。Dedicated
Worker 保留现有字符串消息 adapter；新增 `ServiceWorkerRuntime` 提供 typed command/event：

```text
Command:
  Evaluate { script, script_url }
  DispatchInstall { event_id }
  DispatchActivate { event_id }
  DispatchFetch { event_id, request }
  DispatchMessage { event_id, payload }
  ResolveHostPromise { promise_id, result }
  Shutdown

Event:
  Evaluated
  ExtendLifetime { event_id, promise_id }
  RespondWith { event_id, promise_id }
  EventSettled { event_id, outcome }
  HostRequest { promise_id, operation }
  ScriptError { phase, message }
  Closed
```

**裁决**：推荐。复用执行隔离和安全机制，同时让 SW 事件协议不被 Dedicated Worker 的
`postMessage(String)` 限制。

## 4. 目标架构

```text
page JS (renderer / tab worker / WebView)
        |
        | register/query/unregister + state notifications
        v
ServiceWorkerManager (browser owner; WebView uses in-process adapter)
  - ServiceWorkerRegistry / version slots
  - client -> controller mapping
  - script fetch + update comparison
  - lifecycle coordinator
  - fetch routing / pass-through
        |
        | typed commands/events
        v
ServiceWorkerRuntime (one live worker thread per active job)
  - independent V8/QuickJS context
  - ServiceWorkerGlobalScope bootstrap
  - event listener registry
  - ExtendableEvent waitUntil aggregation
  - FetchEvent respondWith single-assignment
        |
        | HostRequest
        v
host services: network / CacheStorage / IndexedDB / clients
```

该图是作者综合。规范要求独立 worker 与事件生命周期；具体模块分层按 ZeroWeb 现有
browser-hosted network、storage owner 和 WorkerRuntime 形态推导。

### 4.1 模块职责

| 模块 | 变更方向 | 职责 |
|------|----------|------|
| `zero-storage::service_worker` | 收窄/扩展数据模型 | 注册元数据、版本状态、scope；不执行 JS |
| `zero-script-sandbox` | 抽取线程核，新增 SW runtime | 引擎线程、global、事件执行、安全终止 |
| `zero-page-runtime` | 新增 manager/host 契约 | 生命周期协调、typed host operation、in-process adapter |
| `zero-engine` | 新增 SW bridge + shim 萎缩 | 页面对象投影、Promise resolve、状态事件 |
| `zero-protocol` | 新增 SW IPC | renderer 页面 API 与 browser manager 通信 |
| `zero-browser` | production owner | manager、脚本/网络、持久 storage、fetch 路由 |
| `zero-renderer` | IPC client | 注册请求、状态通知、controller 投影 |
| `zero-webview` | embedded adapter | 实例内 manager，共享相同核心算法 |
| `zero-wpt-runner` | SW fixture host | pinned case、资源映射、清理和结果收集 |

### 4.2 生命周期状态

保留公开状态名，但状态推进只能由事件结果触发：

```text
register
  -> fetch/evaluate script failed ------------------------> redundant + reject
  -> installing -- install waitUntil rejected -----------> redundant + reject
  -> installed(waiting)
  -> activating -- activate waitUntil rejected ----------> redundant
  -> activated
  -> update/unregister/replacement -----------------------> redundant
```

M1 单客户端简化：没有旧 active worker 时 installed 立即进入 activating；有旧 active 时进入
waiting，只有无受控 client 或 `skipWaiting()` 才替换。多客户端枚举不在范围，但不能把
waiting 永久删除，否则后续无法兼容更新语义。

## 5. 关键协议

### 5.1 注册

1. 页面 bridge 规范化 script URL/scope，先执行 secure-context、same-origin 和 scheme 校验。
2. manager 以 `(storage partition, origin, normalized scope)` 为 registration key。
3. host 抓脚本；HTTP/解码/语法失败使注册 Promise reject，不能留下 active 近似。
4. runtime 先安装 SW global bootstrap，再执行脚本；顶层同步异常失败。
5. 脚本求值成功后派 install；所有 `waitUntil()` Promise fulfilled 才进入 installed。
6. 无旧 controller 时派 activate；完成后投影 `active`、`ready` 和 controller change。

M1 的 `register()` Promise 在新 installing worker 可观察时 resolve，不等待最终 activate；
WPT helper 随后通过 `statechange` 等待目标状态。

### 5.2 ExtendableEvent

- 每个事件有唯一 `event_id` 和 pending Promise 集。
- `waitUntil(p)` 只允许在事件 dispatch 活跃窗口调用；多次调用并入同一集合。
- handler 返回后关闭新增窗口，但等待已登记 Promise settle。
- 任一 install Promise reject => 安装失败；activate reject => 激活失败并保留旧 active。
- host 设置硬超时；超时按 reject 处理并终止该 event，不得永久占用 worker。

### 5.3 FetchEvent

- manager 只向 controller 匹配且 scope 内的请求派 fetch。
- `respondWith(p)` 仅首次调用有效，且必须在 dispatch 活跃窗口内调用。
- 未调用 `respondWith()` => `PassThrough`。
- Promise fulfilled `Response` => manager 返回该响应。
- Promise rejected/非 Response => fetch network error；worker 线程崩溃按明确策略返回 error，
  不静默伪装成缓存命中。
- SW 内部 `fetch()` 必须带 bypass 标记，避免再次被同一 SW 拦截形成递归。

M2 的插点在 browser `TabFetchProxy::enqueue()`：主文档和受控 client 子资源都汇聚在这里。
WebView 的 `fetch_url()` 使用同一 manager adapter。直到 js-dom S6 完成前，不修改现有
FetchBridge；否则两个目标会同时重写同一 fetch bridge。

### 5.4 Client control

- 首次注册并激活不追溯控制当前加载中的页面；下一次 in-scope navigation 获得 controller。
- `clients.claim()` 在单页面 envelope 内可把当前同 origin/in-scope client 标为受控。
- controller mapping 使用稳定 client id，不把 renderer 内 JS 对象身份存入 manager。
- renderer 重启后由 browser owner 根据导航重新投影 controller。

### 5.5 消息

页面到 SW、SW 到页面均使用结构化纯值 wire；M3 首期支持 JSON-compatible 值和
MessageEvent 基础字段。transferable/MessagePort 完整语义若 WPT 依赖则单独列 TBD，不在 M1
提前实现。

## 6. 功能需求与验收

### FR-001：独立脚本执行

**要求**：注册必须抓取并在独立引擎线程执行 script URL；页面全局与 SW 全局互不可见。
**优先级**：必须（M1）。

```text
场景: 注册静态 SW
  假设 wpt.test 下存在 empty-worker.js
  当页面调用 register 并等待 installing
  那么脚本在独立 runtime 求值，registration.installing 可见
  验证: WPT activation-after-registration.https.html

场景: 脚本抓取或求值失败
  假设 URL 返回失败或脚本顶层抛异常
  当页面调用 register
  那么 Promise reject，注册不进入 activated
  验证: manager/runtime 单测 + 对应上游 registration error case
```

### FR-002：真实 install/activate

**要求**：状态推进必须等待真实事件及其 `waitUntil()` Promise。
**优先级**：必须（M1）。

```text
场景: waitUntil fulfilled
  假设 install 和 activate handler 各登记一个 fulfilled Promise
  当 manager 派发生命周期事件
  那么状态按 installing -> installed -> activating -> activated 推进
  验证: WPT lifecycle case + manager 状态序列单测

场景: install waitUntil rejected
  假设 install handler 登记 rejected Promise
  当 Promise settle
  那么 worker 变 redundant，register/update 不产生 active worker
  验证: 上游 install-event rejection case + runtime 单测
```

### FR-003：fetch 路由

**要求**：受控请求必须按 handler 结果路由到 respondWith 响应或 pass-through，SW 内部
fetch 必须绕过当前 worker。
**优先级**：必须（M2，依赖门禁）。

```text
场景: respondWith Response
  假设 client 已受控且 URL 在 scope
  当 fetch handler 调用 respondWith(Promise.resolve(Response))
  那么 browser fetch proxy 返回该 Response
  验证: 上游单客户端 fetch-event case + IPC/WebView 集成测试

场景: handler 不接管
  假设 handler未调用 respondWith 或 URL 不在 scope
  当请求发生
  那么请求恰好一次进入正常网络路径
  验证: pass-through 计数集成测试
```

### FR-004：单一 owner 与双路径一致

**要求**：production browser path 和 embedded WebView path 必须消费同一 manager 算法。
**优先级**：必须。

```text
场景: 相同生命周期输入
  假设两条 host adapter 使用相同脚本和事件结果
  当完成注册和激活
  那么 registration state/controller 投影一致
  验证: shared conformance test suite 对两个 adapter 参数化运行

场景: renderer 退出
  假设 browser manager 已有 active registration
  当 renderer 关闭并为同 scope 创建新 client
  那么注册仍存在，新 client 可重新获得 controller
  验证: browser/renderer IPC 集成测试
```

## 7. 非功能与安全约束

| ID | 约束 | 优先级 | 验证 |
|----|------|--------|------|
| NFR-001 | 每个 runtime 必须有堆上限、脚本超时、强制中断和 bounded shutdown | 必须 | 复用 R3399 测试并加 SW 死循环 case |
| NFR-002 | page 输入的 URL/scope/headers/body 全按不可信输入处理 | 必须 | same-origin、secure context、长度/尺寸边界单测 |
| NFR-003 | manager 不持有 V8/QuickJS 对象，IPC 只传 serde 纯值 | 必须 | 编译边界 + protocol round-trip |
| NFR-004 | 每个 manager 最多同时运行 32 个 SW 线程；超过上限时回收 idle runtime 或排队，不丢注册 | 必须 | 33 个注册的并发压力测试 |
| NFR-005 | 日志不得包含脚本正文、缓存 body 或凭据 | 必须 | 代码审查 + 日志测试 |
| NFR-006 | V8 与 QuickJS 生命周期核心语义一致 | 必须 | 双 feature scoped test |

secure context、same-origin、Service-Worker-Allowed scope 上限是 M1 必须校验项，不因 headless
环境而省略。多进程路径继续由 browser network/security context 发请求，不让 renderer SW
线程绕过 browser 安全策略直接联网。

## 8. 实施交接

### 8.1 推荐顺序与提交切片

| 提交 | 允许修改 | 结果 | 验证 |
|------|----------|------|------|
| M1-1 | `script-sandbox` | 抽取 threaded core；Dedicated Worker 行为字节级不变；新增 SW typed runtime 骨架 | 双引擎 worker/runtime 单测 |
| M1-2 | `storage`、`page-runtime` | registration key/version slots + manager 生命周期协调 | manager 状态/失败/超时单测 |
| M1-3 | `engine`、`webview` | 页面 bridge 接 manager；删/萎缩 timer 模拟；in-process 真 install/activate | 首个 WPT + WebView 集成 |
| M1-4 | `protocol`、`browser`、`renderer` | browser owner + IPC 投影，生产路径与 WebView 一致 | IPC round-trip + multiprocess test |
| M1-5 | `wpt-runner`、scripts/账本 | pinned 层级 A/B runner 与基线报告 | `testharness-service-workers` |
| M2-1 | browser fetch proxy、WebView adapter | typed fetch event、respondWith/pass-through | 层级 C WPT |
| M2-2 | Cache host bridge | SW `caches.match` 集成 | 层级 D WPT |
| M3 | manager/runtime/bridge | claim、skipWaiting、message、update 收尾 | 层级 E WPT |

每个切片单独提交，前一切片全绿后再进入下一片。M1-1 不改变页面可见行为，是最小回滚点；
M1-3 首次改变 `navigator.serviceWorker` 行为，必须同提交带 driving WPT。

### 8.2 代码边界

**允许修改**：

- `crates/script-sandbox/**`
- `crates/storage/src/service_worker.rs`
- `crates/page-runtime/**`
- `crates/engine/src/js_dom_shim/part02.js` 及新增 SW bridge
- `crates/webview/**`
- `crates/protocol/**`
- `apps/browser/**`
- `apps/renderer/**`
- `tests/wpt-runner/**`
- `docs/goal/service-workers/**`

**禁止修改**：

- `crates/css-parser/**`、`style-system/**`、`layout-engine/**`、`render-foundation/**`：
  与 SW 无关且属于并行 rendering 域。
- `docs/goal/js-dom/**`、`docs/goal/storage-cache-api/**`：只读取依赖状态，不代改其他 goal 控制面。
- js-dom S6 未 land 前禁止修改 fetch bridge 主路径。

### 8.3 实现来源

| 能力 | 来源 | 承载位置 |
|------|------|----------|
| JS 线程、超时、强制终止 | 抽取现有 `WorkerRuntime` | `zero-script-sandbox` |
| 注册/scope/cache 元数据 | 演进现有 registry | `zero-storage` |
| 生命周期算法 | Service Workers 规范 + 仓内实现 | `zero-page-runtime` |
| 网络与安全策略 | 复用 browser fetch proxy / ResourceLoader | `zero-browser` / WebView adapter |
| CacheStorage | 复用 `zero-storage::cache_api`，页面接线等兄弟 goal | host operation |
| WPT 运行 | 复用现有 testharness core + pinned fetch script | `zero-wpt-runner` |

不新增第三方 crate。若实现中发现现有引擎 API 无法抽取而必须引入新 runtime 依赖，视为偏离
RFC，停止并重新审批。

## 9. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 抽取 WorkerRuntime 引发 Dedicated Worker 回归 | 高 | M1-1 保持 public adapter，先跑既有全部 worker 测试 |
| manager/browser/WebView 双 owner 漂移 | 高 | manager 算法单一，host 仅 adapter；参数化 conformance |
| fetch 重入/自拦截 | 高 | inner fetch 显式 bypass token，计数测试 |
| Promise 永不 settle | 高 | event deadline + runtime interrupt + manager 清理 |
| renderer/browser IPC 断连 | 中 | pending command 失败并回收，registration owner 不丢 |
| js-dom/storage-cache-api 并行冲突 | 高 | M2 严格等待门禁；共享文件开工前 `git log` |
| 每注册一线程造成资源放大 | 中 | 仅事件需要时启动，idle shutdown 延后到 M3；M1 先设 live 上限 |
| QuickJS 行为落后 | 中 | typed 协议引擎无关；每切片双 feature 测试 |

## 10. 回滚

- M1-1/M1-2 可按提交直接 revert，页面行为未改变。
- M1-3 若 WPT 或产品回归，回滚整个 bridge 切片，恢复 R3318 近似；不得保留一半 Rust
  manager + 一半 JS 私有数组。
- M1-4 IPC 失败时可回滚 production adapter，保留已验证的 in-process runtime，但
  `master.md` 必须明确 production 未完成，不能宣称 M1 达成。
- M2 以 fetch router 单一开关做开发期 A/B；合入目标是默认开启，不长期保留双实现。
- storage schema 变更必须向后兼容；M1 不迁移或删除既有用户持久数据。

## 11. Spec Lint

### 结构完整性

| 规则 | 裁决 | 依据 |
|------|------|------|
| 执行摘要 | Pass | §0 含目标、范围、推荐、首步、门禁 |
| 场景存在性 | Pass | FR-001~004 各有正常和异常场景 |
| 异常路径覆盖 | Pass | 每个 FR 各 1 正常 + 1 失败/恢复场景 |
| 测试绑定 | Pass | §6 每个场景均列 WPT/单测/集成测试 |
| TBD 清零 | Pass | 仅 transferable 为 M3 范围边界，不阻塞 M1 决策 |
| 实施交接 | Pass | §8 含模块、顺序、批次、来源和首步 |

### 语言与一致性

| 规则 | 裁决 | 依据 |
|------|------|------|
| 范围冲突 | Pass | §1 与 goal 排除项一致 |
| 约束冲突 | Pass | browser owner 与 WebView adapter 共享算法，不是双权威状态 |
| 方案漂移 | Pass | §3 推荐与 §4/§8 均为抽取 Worker 线程核 |
| 实现来源闭合 | Pass | §8.3 逐能力指定现有模块或仓内实现 |
| 依赖清单一致 | Pass | 明确不新增第三方 crate |
| 代码边界 | Pass | §8.2 同时列允许和禁止范围 |
| 外部事实保守化 | Pass | 规范/Chromium/WPT 有引用；ZeroWeb 结论来自源码 |
| 首步可执行性 | Pass | M1-1 抽取 threaded core，并绑定双引擎 worker 测试 |

**汇总**：14 Pass / 0 Warning / 0 Fail / 0 Skip
**门禁判定**：允许提交用户审批；未批准前禁止实施。

## 12. 用户决策

**D1：方案 C 及其 owner 模型已批准。**

- 批准内容：抽取 Worker 独立线程核；SW runtime 独立线程；browser manager 为 production
  单一 owner；WebView 使用同算法的 in-process adapter；按 §8 顺序实施。
- 不包含：立即开启 M2 fetch 改造、改变其他 goal、引入新依赖或扩大多客户端范围。

## 13. 参考资料

1. [Service Workers CRD](https://www.w3.org/TR/service-workers/)
2. [Chromium Service Worker architecture](https://github.com/chromium/chromium/blob/main/content/browser/service_worker/README.md)
3. [MDN: Using Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers)
4. [MDN: ServiceWorkerGlobalScope](https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerGlobalScope)
5. [WPT project](https://github.com/web-platform-tests/wpt)
6. [WPT activation case](https://github.com/web-platform-tests/wpt/blob/master/service-workers/service-worker/activation-after-registration.https.html)
7. [WPT SW helpers](https://github.com/web-platform-tests/wpt/blob/master/service-workers/service-worker/resources/test-helpers.sub.js)
8. `crates/script-sandbox/src/worker.rs`
9. `apps/browser/src/tab_js_worker.rs`
10. `apps/renderer/src/js_worker.rs`
11. `apps/browser/src/fetch_proxy.rs`
12. `crates/storage/src/service_worker.rs`
13. `crates/webview/src/webview.rs`
14. `tests/wpt-runner/src/testharness.rs`

## 14. 修订历史

| 版本 | 日期 | 内容 |
|------|------|------|
| v1.0 | 2026-08-19 | M0 初稿：三方案对比，推荐抽取 Worker 线程核 + browser manager owner |
| v1.1 | 2026-08-19 | 用户明确批准方案 C；解除 M0 源码实施门禁 |
