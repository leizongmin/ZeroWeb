# Service Worker 真实化 — 运行时控制面板（master.md）

**入口文档**: [../service-workers.md](../service-workers.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-21（M2-1 fetch runtime/manager/IPC foundation 完成）

---

## 当前状态

**专项定位**：存储方向三拆之三。把 `navigator.serviceWorker` 从注册表状态机近似
（R3318）深化为真实 SW 执行环境 + fetch 拦截。用户已于 2026-08-19 明确批准方案 C，
M0 启动门禁解除；M1 core WPT 已收敛，M2 已完成 fetch runtime/manager/IPC foundation，
生产页面 fetch 管线与 Cache API 端到端接入仍待后续切片，M3 控制语义继续推进。

**M0 推荐决策**：抽取 `zero-script-sandbox::WorkerRuntime` 的独立线程/引擎/看门狗核心，
新增 typed `ServiceWorkerRuntime`；production 由 browser process 的
`ServiceWorkerManager` 单一拥有注册、控制与 fetch 路由，embedded WebView 使用同一 manager
算法的 in-process adapter。详见 [M0 执行环境 RFC](m0-execution-environment-rfc.md)。

**与兄弟 goal 的边界**：
- [storage-indexeddb](../archive/storage-indexeddb.md)（已归档）/ storage-cache-api —
  IDB 与 Cache API 自身语义归其管；本目标只消费
  `indexedDB`/`caches` 接口做 SW 模式集成验收
- js-dom — fetch 拦截的生产页面 `FetchRequest` 插入点**等其 fetch 改造（L2/S6）land 后再开**；
  runtime/manager/IPC foundation 可独立推进；生命周期段碰 part02.js R3318 段前先 `git log`
  核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ 注册 API 面：R3318（part02.js:2496）——register/getRegistration/getRegistrations/
  ready/unregister + scope 派生 + oncontrollerchange + installing/waiting/active 经
  setTimeout(0) 逐态推进
- ✅ Rust 状态机：`crates/storage/src/service_worker.rs`（818 行）——
  ServiceWorkerRegistry register/unregister/state/scope 匹配 + 单测
- ✅ WebView 静态拦截底座：手工激活 registry 后，`fetch_url()` 可先查该注册的
  CacheStorage；**这不是 SW fetch 事件执行**，页面注册 shim 也未接入
- ⚠️ register 的 scriptURL **不被下载执行**——SW 事件处理器无从注册
- ⚠️ 页面 fetch 事件拦截为零；install/activate 为 setTimeout 模拟非真事件
- ⚠️ WPT `service-workers` 未导入；当前标准入口真实可执行文件数为 0。固定 revision
  完整 manifest 为 294 个 testharness 源 / 331 个 URL；初筛 12 个 M1 候选经资源闭包
  校准为 8 个静态首批 / 3 个高阶事件案 / 1 个动态 server 阻塞案
- ✅ Tier A 资产化：8 case / 28 subtest / 18 asset 已固定，fetch target + blob-SHA
  fail-closed + testharness 账本已落；SW runner 仍待 RFC 批准后实现
- ✅ 第二批资产化：14 个 M1/no-signal review 中 3 case / 4 subtest 已固定并入共享独立
  corpus；5 advanced defer / 5 dynamic-server gated / 1 update defer，剩余逻辑 review 138
- ✅ iframe 裁决：11 case / 41 subtest 分为 single-iframe/worker/controller defer、
  fetch gated、multi-client skip；剩余逻辑 review 127
- ✅ message-channel 裁决：7 case / 18 subtest 分为 lifecycle result-channel defer、
  controller/fetch/update gate、cross-origin client skip；剩余逻辑 review 120
- ✅ M1 剩余裁决：25 case / 99 subtest 分为 19 dynamic/server gated、6 support-envelope
  skip；M1 review 57/57 已收口，全量剩余逻辑 review 95
- ✅ Static Routing 裁决：本批 11 case / 70 subtest 全部 skip；连同前批 1 案，
  family 12/12 已裁决，全量剩余逻辑 review 84
- ✅ Worker Global/import 裁决：13 case / 53 subtest 当前分为 2 static core、5 runtime defer、
  4 server gated、1 M2 defer、1 worker-client skip；全量剩余逻辑 review 71
- ✅ static-wave 资产化：`serviceworkerobject-scripturl` + `import-scripts-data-url`
  2 case / 5 subtest / 4 assets 已固定并记入 testharness 账本
- ✅ IDL harness 裁决：4 generated URL / 787 subtest（175 window + 155 dedicated +
  155 shared + 302 serviceworker）；全量剩余逻辑 review 70
- ✅ Navigation/redirect 裁决：15 source / 16 URL / 224 subtest 分为 2 defer /
  10 gated / 3 skip；全量剩余逻辑 review 55
- ✅ Request/response/timing 裁决：17 source / 83 subtest 分为 7 defer /
  9 gated / 1 skip；全量剩余逻辑 review 38
- ✅ Final remaining 裁决：38 source / 270 subtest 分为 14 defer /
  8 gated / 16 skip；初始 review 152/152，逻辑剩余 0
- ✅ Runner disposition contract：294 source / 331 URL 唯一映射为
  34 core / 49 defer / 169 gated / 42 skip，可从原始 evidence 确定性重建；
  34 个 core 与 runner 导入账本、二十批 case asset 及 blob SHA 精确对应
- ✅ M0 registry 契约补强：新增 4 项 Rust 单测，固定候选版本不提前替换 active、
  非法激活不扰动 active、注销旧 redundant 不删除新映射、跨 origin 替换隔离
- ✅ M1 WorkerRuntime readiness：V8 20/20、QuickJS 3/3，WebView 双后端各 17/17；
  三种 feature clippy 通过，抽取边界与 QuickJS timeout/evaluate handshake 缺口已固定
- ✅ SW 执行环境 RFC 方案 C 已获用户明确批准
- ✅ M1-1：共享 threaded core + 双引擎 typed `ServiceWorkerRuntime` evaluate 骨架；
  V8/QuickJS 各 7/7，Dedicated Worker 与 WebView 双后端基线保持全绿
- ✅ M1-2：scope-keyed `ServiceWorkerManager` + installing/waiting/active version slots；
  双引擎 10 项 conformance、page-runtime 三矩阵各 56/56
- ✅ M1-3a：双引擎 ServiceWorkerGlobalScope lifecycle bootstrap，真实 install/activate
  listener dispatch + `waitUntil()` outcome；runtime 10/10、manager forwarding 11/11
- ✅ M1-3b：manager 自动消费 lifecycle outcome；WebView 同源安全校验、真实 script fetch
  与 in-process manager adapter，双后端端到端各 4/4
- ✅ M1-3c：页面 register/snapshot/unregister callbacks 接 manager，R3318 生命周期模拟删除；
  双引擎页面 API 6/6，首次 controller 保持 null
- ✅ M1-4a：renderer↔browser typed Service Worker request/response/snapshot/error contract；
  纯值、无 script source，协议全套 298/298
- ✅ M1-4b：browser process 单一 manager owner、committed-navigation authority、normal/private
  profile 隔离与 browser-owned async script fetch；response 保持原 `IpcMessage.id`
- ✅ M1-4c：renderer Service Worker response router + JS worker host callbacks；
  fresh browser/renderer register→active→unregister 全链通过，normal owner 跨 renderer 存活
- ✅ M1-4d：browser-backed `getRegistration()` / `getRegistrations()`；
  新 renderer 无需旧 registration ID 即可恢复稳定 JS 投影
- ✅ M1-5b：manager transition log + renderer cursor；ServiceWorker/Registration EventTarget；
  updatefound/statechange 与 slots 逐 task 投影，lifecycle 5 个红项及 interface brand 2 项转绿
- ✅ M1-5c：共享 registration URL validator + WebIDL scope conversion + typed rejection；
  scope/scriptURL/DOMException 最后 6 个红项转绿
- ✅ M1-5：12 case / 36 subtest core WPT 稳定 baseline 为
  36 Pass / 0 Fail / 0 Timeout / 0 Unsupported
- ✅ M3-1：worker global `skipWaiting()` 经 typed lifecycle settlement 进入 manager；
  replacement install 成功后无需宿主命令即可激活，旧 active 随后 redundant，registration
  identity 保持稳定
- ✅ M3-2：每 Document controller 按 active scope 初始化；首次注册不反向控制当前页面；
  已受控页面的 `skipWaiting()` replacement 切换 controller 并按 task 派发 `controllerchange`
- ✅ M3-3：activate `clients.claim()` 经 typed settlement 和 browser committed authority，
  控制当前 matching Document 并按 task 派发 `controllerchange`
- ✅ M3-4：`ServiceWorker.postMessage()` 经 browser authorization 和 typed runtime command
  派发 worker `MessageEvent`；JSON-compatible structured payload，handler failure 不改变 lifecycle
- ✅ M3-5：worker `Client.postMessage()` 经 per-Document client log 和 renderer cursor，
  向 container 派发 `MessageEvent`；导航换代隔离旧队列，browser/WebView 双路径一致
- ✅ M3-6：`ServiceWorkerRegistration.update()` 经 browser-owned fetch 与 top-level script
  byte comparison；相同脚本 no-op，变化脚本创建 replacement 并派发 `updatefound`
- ✅ M3-7：production normal profile 原子持久化 active registration 与 script source；
  browser restart 重建 runtime/controller，不重放 install/activate；private profile 不落盘
- ✅ M3-8：`updateViaCache` 作为 typed registration metadata 贯穿页面、IPC、manager 与
  persistence；browser-owned top-level update fetch 按 `imports`/`all`/`none` 选择 cache mode
- ✅ M3-9：classic `importScripts()` 经 typed blocking bridge 和 browser-owned batch fetch
  在同一 global 顺序执行；startup graph 持久化并参与完整 update byte comparison
- ✅ M3-10：classic import 跨源 no-cors 与动态 MIME response policy；结构化 WebView fixture、
  NetworkError DOMException、remote worker 65-message result channel；core WPT 16/62
- ✅ M3-11：version-local script resource map、有状态 redirect/stash/update fixture、
  WorkerLocation/URLSearchParams 与 registration-key unregister；core WPT 18/67
- ✅ M3-12：event-time worker fetch context 与 script resource map updated flag；
  install 可 fetch、activate/message 仅 replay，late import 返回 NetworkError；core WPT 19/72
- ✅ M3-13：registration script type 贯穿 JS/IPC/manager/storage/persistence/renderer；
  module loader 接入前显式 fail closed，不再静默按 classic 执行
- ✅ M3-14：renderer runtime 递归加载并编译 static module graph；browser-owned fetch
  按真实 importer 解析 canonical URL，完整 graph 持久化并参与 update bytecheck；
  module scope WPT 3/3，core WPT 20/75
- ✅ M3-15：状态化 bytecheck fixture 覆盖 classic/module 的 main/imported bytes
  unchanged/changed 4×2 矩阵；`update-bytecheck.https.html` 8/8，core WPT 21/83
- ✅ M3-16：static module 跨源依赖由 browser/WebView response adapter 执行 CORS
  校验，classic 维持 no-cors；cross-origin bytecheck 8/8，core WPT 22/91
- ✅ M3-17：module graph 支持 named/star/namespace re-export；依赖按 importer
  canonical URL 递归抓取并进入 persistence/update bytecheck，V8/QuickJS 回归通过
- ✅ M3-18：module link 阶段校验 named/default export；registration 的 network、
  parse、runtime、instantiation、TLA 错误均 fail closed；core WPT 23/101
- ✅ M3-19：重复 register 比较 URL/完整 graph/type；classic↔module 切换、unchanged
  registration 与跨类型求值失败语义收敛；core WPT 24/108
- ✅ M3-20：main/import script request mode、`Service-Worker` header 与 no-cache
  ETag revalidation 收敛；core WPT 26/110
- ✅ M3-21：`updateViaCache` main/import cache 矩阵、重复 register no-op、
  跨 iframe registration 投影和失败回滚收敛；core WPT 27/135
- ✅ M3-22：dynamic main/import response、404 failure rollback、移除失效 import、
  cross-origin import update 与无受控 client 的 replacement activation 收敛；
  core WPT 29/142
- ✅ M3-23：main script MIME/redirect/syntax validation、install failure、pending
  uninstall 与 shrinking script update 收敛；core WPT 30/149
- ✅ M3-24：同 registration key 的并发 update job 复用单一 fetch/runtime/installing
  candidate，burst 后 update 继续可用；core WPT 31/150
- ✅ M3-25a：client 在首次 worker installing 期间调用 `registration.update()` 复用当前
  candidate 并成功，不重复 fetch/runtime 或派发 `updatefound`；core WPT 31/150
- ✅ M3-25：lifecycle wait 可穿插 message dispatch；MessagePort endpoint 经
  page/IPC/manager/runtime 双向 transfer；worker global `registration.update()` 按 calling
  worker 状态拒绝 installing、允许 active 合并 replacement；core WPT 32/153
- ✅ M3-26：worker global 在无受控 client 时单次及并发调用 `skipWaiting()` 均 resolve
  `undefined`；真实 worker-testharness 结果通道通过；core WPT 33/155
- ✅ M3-27：browser-owned committed client registry 经 typed host-thread query 投影
  `clients.matchAll({includeUncontrolled:true})`；worker evaluation 主动 `Client.postMessage()`
  按目标 client ID 路由；core WPT 34/156
- ✅ M3-28：worker global `clients.get(id)` 经 browser-owned client registry 返回同 origin
  `Client` 或 `undefined`；协议/browser owner/renderer/WebView 双路径接线，完整上游
  `clients-get.https.html` 仍因 resultingClientId/fetch 子项留在 M2 门控
- ✅ M3-29：`clients.matchAll()` 对 window client 按 spec 使用 focus-first/recent-focus
  ordering；browser active tab 每轮投影到 browser-owned client registry，未聚焦窗口保持创建顺序
- ✅ M3-30：manager/browser owner 增加 window client `frameType` 显式投影入口，IPC
  `ServiceWorkerClientInfoWire` 补齐 `auxiliary` 合法枚举；当前生产路径仍默认 top-level，
  同 tab 多 browsing-context client 生命周期留待下一切片
- ✅ M3-31：browser owner 的 tab→client registry 从单 client 扩展为一 tab 多 window
  client；同 tab top-level + nested client 可同时进入 `clients.matchAll()`，tab 断开时整组
  client 与消息队列清理，tab focus 投影继续优先 top-level/auxiliary client
- ✅ M3-32：production navigation commit 主动把当前 Document 观测为 top-level
  Service Worker window client；导航 replacement 在 start 阶段移除旧 client，commit 后以
  新 navigation epoch 生成稳定 client id，不再依赖页面先调用 `navigator.serviceWorker.*`
- ✅ M3-33：browser owner 暴露 window client 创建/销毁生命周期入口；`auxiliary`/`nested`
  client 可按显式 `frameType` 登记，单 client 销毁只清理对应 registry 记录和消息队列，
  不误删同 tab 的 top-level/popup client
- ✅ M3-34：renderer iframe `contentDocument` / `contentWindow` 物化经 typed IPC
  观察为 browser-owned `nested` window client；iframe 删除、替换和清空子树路径注销已登记
  iframe client，client id 由 browser 归一到 committed top-level Document 命名空间下
- ✅ M2-1：Service Worker `FetchEvent` runtime foundation、manager longest-scope dispatch
  与 renderer/browser IPC command/event 已接通；`respondWith(new Response(...))`、未调用
  `respondWith` pass-through、重复 `respondWith` failure、跨 origin/out-of-scope pass-through
  均有定向测试；生产页面 `FetchRequest` 路由与 Cache API 集成仍未接入

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | SW 执行环境架构与独立 runtime | ✅ production browser owner + renderer discovery 真链路 |
| S2 | scriptURL 不下载执行 | ✅ production navigator 经 browser fetch/evaluate |
| S3 | fetch 拦截为零 | 🚧 M2-1 runtime/manager/IPC foundation；生产页面 fetch 与 Cache API 未接入 |
| S4 | 事件为 setTimeout 模拟 | ✅ manager transition log 为状态源；timer 只执行页面 task 投影 |
| S5 | WPT 覆盖为零 | ✅ core 34/34 case、156/156 Pass、0 Fail/Timeout/Unsupported |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 批准方案 C：抽取 Worker 线程核 + SW typed runtime + browser manager owner | ✅ 2026-08-19 用户明确批准 |

## 下一步计划

1. **M2 production fetch pipeline**：js-dom S6 land 后，把页面 `FetchRequest` 路由接入
   `ServiceWorkerManager::dispatch_fetch()`，实现 `respondWith` 响应回填与 pass-through 网络回退
2. **M2 Cache API 集成**：storage-cache-api M1 land 后，让 SW runtime 的 fetch handler 可消费
   `caches.match()` 端到端模式
3. **M3 clients follow-up**：popup/auxiliary 真实 browsing context 创建后接入 browser owner

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 选型 RFC（门控） | ✅ 方案 C 已批准 |
| M1 — 脚本真实执行 + 生命周期真事件 | ✅ current core WPT 156/156 Pass |
| M2 — fetch 拦截 + Cache 集成 | 🚧 M2-1 foundation 完成；生产 fetch/Cache 集成待接入 |
| M3 — 控制语义 + 消息 + 收尾 | 🚧 classic startup graph + 控制/消息/update/persistence 完成 |

## 验证基线

- 测试基线：storage crate 既有单测全绿（立项时点）；clippy 零警告
- WPT service-workers 面：当前标准入口可执行 0 文件；上游完整分母 294 个 testharness
  源 / 331 URL，正文覆盖 294/294；分层与依赖信号见
  [M0 WPT evidence](evidence/2026-08-19-m0-wpt-executable-surface.md)，逐文件机器清单见
  [WPT case inventory](evidence/2026-08-19-m0-wpt-case-inventory.tsv)，候选 8/3/1 裁决见
  [M1 candidate closure](evidence/2026-08-19-m0-m1-candidate-resource-closure.md)，静态首批
  8 case / 28 subtest / 18 asset 见
  [Tier A baseline contract](evidence/2026-08-19-m1-tier-a-baseline-contract.md)
- 第二批生命周期面：14 case / 78 subtest 裁决及 next-wave 3 case / 4 subtest 见
  [M1 next-wave review](evidence/2026-08-19-m1-next-wave-review.md)
- Iframe 生命周期面：11 case / 41 subtest 的 defer/gated/skip 裁决见
  [M1 iframe review](evidence/2026-08-19-m1-iframe-review.md)
- Message-channel 生命周期面：7 case / 18 subtest 的 result-channel/controller/update 裁决见
  [M1 message review](evidence/2026-08-19-m1-message-review.md)
- M1 剩余面：25 case / 99 subtest 与 40 个关键资源闭包见
  [M1 final review](evidence/2026-08-19-m1-final-review.md)
- Static Routing 面：11 case / 70 subtest 的 out-of-scope 裁决见
  [Static Routing review](evidence/2026-08-19-static-routing-review.md)
- Worker Global/import 面：13 case / 53 subtest 的 core/defer/gated/skip 裁决见
  [Worker Global/import review](evidence/2026-08-19-worker-global-import-review.md)
- IDL harness 面：4 generated URL / 787 subtest 的逐项分母见
  [IDL harness review](evidence/2026-08-19-idlharness-review.md)
- Navigation/redirect 面：15 source / 16 URL / 224 subtest 的分层裁决见
  [Navigation review](evidence/2026-08-19-navigation-review.md)
- Request/response/timing 面：17 source / 83 subtest 的分层裁决见
  [Request/response review](evidence/2026-08-19-request-response-review.md)
- Review 总账：最后 38 source / 270 subtest 及初始 review 152/152 收口见
  [Review closure](evidence/2026-08-19-review-closure.md)
- Runner disposition：294 source / 331 URL 的唯一执行 lane 见
  [WPT disposition contract](evidence/2026-08-19-wpt-disposition.tsv)；
  `make audit-wpt-service-workers-disposition` 从原始账本重建并逐字节校验，同时检查
  core lane、runner 导入账本与七批 case asset 的双向闭包
- Tier A 资产恢复：`make fetch-wpt-service-workers-tier-a`；默认使用独立
  `wpt-data/.service-workers-tier-a-root`，当前环境 18/18 blob SHA 验证通过
- Tier A 资产审计：`make audit-wpt-service-workers-tier-a`（无网络、只读）；
  `make test-wpt-service-workers-tier-a-assets` 覆盖缺失/篡改/修复回归
- Next-wave 资产恢复/审计：`make fetch-wpt-service-workers-next-wave` /
  `make audit-wpt-service-workers-next-wave`；与 Tier A 复用独立数据根，当前 7/7 通过；
  `make test-wpt-service-workers-next-wave-assets` 固化篡改/修复回归
- Static-wave 资产恢复/审计：`make fetch-wpt-service-workers-static-wave` /
  `make audit-wpt-service-workers-static-wave`；4 assets / 5 subtest；
  `make test-wpt-service-workers-static-wave-assets` 固化篡改/修复回归
- Update-wave 资产恢复/审计：`make fetch-wpt-service-workers-update-wave` /
  `make audit-wpt-service-workers-update-wave`；5 assets / 1 subtest；
  `make test-wpt-service-workers-update-wave-assets` 固化篡改/修复回归
- Import-response-wave 资产恢复/审计：5 assets / 24 subtest；
  `make test-wpt-service-workers-import-response-wave-assets` 固化篡改/修复回归
- Import-dynamic-wave 资产恢复/审计：11 assets / 5 subtest；
  `make test-wpt-service-workers-import-dynamic-wave-assets` 固化篡改/修复回归
- Import-event-wave 资产恢复/审计：3 assets / 5 subtest；
  `make test-wpt-service-workers-import-event-wave-assets` 固化篡改/修复回归
- Dynamic-import-update-wave 资产恢复/审计：17 assets / 7 subtest；
  `make test-wpt-service-workers-dynamic-import-update-wave-assets` 固化篡改/修复回归
- Update-failure-wave 资产恢复/审计：12 assets / 7 subtest；
  `make test-wpt-service-workers-update-failure-wave-assets` 固化篡改/修复回归
- Multiple-update-wave 资产恢复/审计：5 assets / 1 subtest；
  `make test-wpt-service-workers-multiple-update-wave-assets` 固化篡改/修复回归
- Update-not-allowed-wave 资产恢复/审计：6 assets / 3 subtest；
  `make test-wpt-service-workers-update-not-allowed-wave-assets` 固化篡改/修复回归
- Skip-waiting-no-client-wave 资产恢复/审计：6 assets / 2 subtest；
  `make test-wpt-service-workers-skip-waiting-no-client-wave-assets` 固化篡改/修复回归
- Clients-matchAll-evaluation-wave 资产恢复/审计：5 assets / 1 subtest；
  `make test-wpt-service-workers-clients-matchall-evaluation-wave-assets` 固化篡改/修复回归
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` +
  `make test`（V8/QuickJS/GPU capability）全过
- Registry 契约测试：`cargo test -p zero-storage service_worker::tests`，40/40 通过；
  `cargo clippy -p zero-storage --all-targets -- -D warnings` 通过
- WorkerRuntime 抽取前基线：双引擎 crate/WebView 测试、调用点、feature union 与禁止偷换见
  [M1 WorkerRuntime readiness](evidence/2026-08-19-m1-worker-runtime-readiness.md)
- M1-1 实现证据：共享线程核、typed SW evaluate、资源上限、双引擎与 WebView 回归见
  [M1 threaded runtime](evidence/2026-08-19-m1-threaded-runtime.md)
- M1-2 manager 证据：scope version slot、失败保留旧 active、容量/输入门禁与双引擎矩阵见
  [M1 manager lifecycle](evidence/2026-08-19-m1-manager-lifecycle.md)
- M1-3a lifecycle runtime：真实 listener dispatch、`waitUntil()` outcome 与双引擎差异修复见
  [M1 lifecycle runtime](evidence/2026-08-19-m1-lifecycle-runtime.md)
- M1-3b WebView host bridge：manager 自动推进、同源安全校验、真实 script fetch 与双引擎
  端到端见 [M1 WebView host bridge](evidence/2026-08-19-m1-webview-host-bridge.md)
- M1-3c 页面 bridge：宿主 callbacks、真实 state snapshot、R3318 模拟删除与页面 API 双引擎
  验证见 [M1 page bridge](evidence/2026-08-19-m1-page-bridge.md)
- M1-4a IPC contract：纯值 request/response/snapshot/error、追加判别值与 round-trip 门禁见
  [M1 IPC contract](evidence/2026-08-19-m1-ipc-contract.md)
- M1-4b browser owner：committed URL authority、profile 隔离、异步 script fetch/evaluate、
  typed response correlation 见 [M1 browser owner](evidence/2026-08-19-m1-browser-owner.md)
- M1-4c renderer bridge：独立 response router、JS callbacks、fresh peer production register 全链见
  [M1 renderer bridge](evidence/2026-08-19-m1-renderer-bridge.md)
- M1-4d registration discovery：scope representative、browser-backed query、跨 renderer JS identity
  恢复见 [M1 registration discovery](evidence/2026-08-19-m1-registration-discovery.md)
- M1-5 core WPT：固定 12-case runner、两轮确定性 baseline 与 13 个红项分组见
  [M1 core WPT baseline](evidence/2026-08-19-m1-wpt-core-baseline.md)
- M1-5b lifecycle task：manager transition log、IPC cursor、EventTarget/slot task 与 30/36
  baseline 见 [M1 lifecycle task projection](evidence/2026-08-19-m1-lifecycle-task-projection.md)
- M1-5c registration URL：共享 validator、WebIDL conversion、typed rejection 与 36/36
  baseline 见 [M1 registration URL contract](evidence/2026-08-19-m1-registration-url-contract.md)
- M3-1 `skipWaiting()`：typed worker signal、replacement activation、版本 identity 与全量门禁见
  [M3 skipWaiting activation](evidence/2026-08-20-m3-skip-waiting.md)
- M3-2 controller：每 Document active scope、replacement controllerchange、双引擎及生产链见
  [M3 Document controller](evidence/2026-08-20-m3-controller.md)
- M3-3 `clients.claim()`：typed claim signal、committed controller IPC、当前 Document
  controllerchange 与全量门禁见 [M3 clients.claim](evidence/2026-08-20-m3-clients-claim.md)
- M3-4 page→worker message：structured payload、typed command、browser authorization 与失败隔离见
  [M3 page-to-worker message](evidence/2026-08-20-m3-page-to-worker-message.md)
- M3-5 worker→page message：Client source、per-Document cursor log、container MessageEvent 与资源上限见
  [M3 worker-to-page message](evidence/2026-08-20-m3-worker-to-page-message.md)
- M3-6 update job：browser-owned refetch、byte comparison、changed/no-op/error projection 与 13/37 WPT 见
  [M3 update job](evidence/2026-08-20-m3-update-job.md)
- M3-7 persistence：single-writer snapshot、runtime restart、controller restore、private 隔离与损坏恢复见
  [M3 registration persistence](evidence/2026-08-20-m3-registration-persistence.md)
- M3-8 `updateViaCache`：typed metadata、browser HTTP cache mode、持久化迁移与双引擎投影见
  [M3 updateViaCache](evidence/2026-08-20-m3-update-via-cache.md)
- M3-9 `importScripts()`：typed blocking bridge、browser batch fetch、graph update comparison、
  persistence 与 14/38 WPT 见 [M3 importScripts graph](evidence/2026-08-20-m3-import-scripts-graph.md)
- M3-10 import response policy：classic no-cors、动态 MIME、NetworkError DOMException、
  remote worker result channel 与 16/62 WPT 见
  [M3 import response policy](evidence/2026-08-20-m3-import-response-policy.md)
- M3-11 dynamic import semantics：version-local resource map、redirect/stash/update、
  WorkerLocation/URLSearchParams、registration-key unregister 与 18/67 WPT 见
  [M3 dynamic import semantics](evidence/2026-08-20-m3-import-dynamic.md)
- M3-12 event-time import context：persistent worker fetch context、updated flag、
  install fetch、activate/message replay 与 19/72 WPT 见
  [M3 event-time import context](evidence/2026-08-20-m3-import-event-context.md)
- M3-13 module type contract：registration type typed wire、storage/persistence 迁移与
  classic fallback 拒绝见 [M3 module type contract](evidence/2026-08-20-m3-module-type-contract.md)
- M3-14 static module graph：canonical referrer 递归加载、browser-owned fetch、
  persistence/update bytecheck 与 20/75 WPT 见
  [M3 static module graph](evidence/2026-08-20-m3-module-static-graph.md)
- M3-15 module update bytecheck：classic/module main/imported 4×2 更新矩阵与
  21/83 WPT 见 [M3 module bytecheck](evidence/2026-08-20-m3-module-bytecheck.md)
- M3-16 module CORS：跨源 module ACAO 校验、classic no-cors 隔离与 22/91 WPT 见
  [M3 module CORS](evidence/2026-08-20-m3-module-cors.md)
- M3-17 module re-export：named/star/namespace graph extraction 与 canonical transform 见
  [M3 module re-exports](evidence/2026-08-20-m3-module-reexports.md)
- M3-18 module registration errors：link-time export validation、错误分类与 23/101 WPT 见
  [M3 module registration](evidence/2026-08-20-m3-module-registration.md)
- M3-19 module type update：重复 registration graph/type 比较与 24/108 WPT 见
  [M3 module type update](evidence/2026-08-20-m3-module-type-update.md)
- M3-20 module request metadata：main/import mode、Service-Worker header、ETag
  revalidation 与 26/110 WPT 见
  [M3 module request metadata](evidence/2026-08-20-m3-module-request-metadata.md)
- M3-21 updateViaCache matrix：main/import cache、策略切换、iframe 投影与 27/135 WPT 见
  [M3 updateViaCache matrix](evidence/2026-08-20-m3-update-via-cache-matrix.md)
- M3-22 dynamic import update：主脚本切换、import 404 回滚、失效 import 移除、
  无受控 client 激活与 29/142 WPT 见
  [M3 dynamic import update](evidence/2026-08-21-m3-dynamic-import-update.md)
- M3-23 update failure matrix：main script MIME/redirect/syntax、install throw、
  pending uninstall、shrinking update 与 30/149 WPT 见
  [M3 update failure matrix](evidence/2026-08-21-m3-update-failure.md)
- M3-24 multiple update：并发 job coalescing、单一 candidate/runtime 与 31/150 WPT 见
  [M3 multiple update](evidence/2026-08-21-m3-multiple-update.md)
- M3-25 update permissions + MessagePort：worker update 状态矩阵、browser-owned fetch、
  双向 port transfer 与 32/153 WPT 见
  [M3 update permissions](evidence/2026-08-21-m3-update-not-allowed.md)
- M3-26 skipWaiting without client：worker-testharness 结果通道、并发 Promise 语义与
  33/155 WPT 见
  [M3 skipWaiting no client](evidence/2026-08-21-m3-skip-waiting-no-client.md)
- M3-27 clients.matchAll evaluation：browser-owned client registry、typed query、
  主动消息路由与 34/156 WPT 见
  [M3 clients.matchAll evaluation](evidence/2026-08-21-m3-clients-matchall-evaluation.md)
- M3-28 clients.get：typed 单 client 查询、同 origin 过滤、unknown `undefined` 与
  browser/WebView 双路径见 [M3 clients.get](evidence/2026-08-21-m3-clients-get.md)
- M3-29 clients focus order：window client 最近 focus 排序、browser active tab 投影与
  manager/browser owner 定向测试见
  [M3 clients focus order](evidence/2026-08-21-m3-clients-focus-order.md)
- M3-30 client frameType：`top-level`/`auxiliary`/`nested` 投影、invalid frameType
  fail-closed 与 browser→renderer IPC 透传见
  [M3 client frameType](evidence/2026-08-21-m3-client-frametype.md)
- M3-31 same-tab clients：browser owner 一 tab 多 client 索引、同 tab nested 枚举、
  disconnect 成组清理与 focus target 选择见
  [M3 same-tab clients](evidence/2026-08-21-m3-same-tab-clients.md)
- M3-32 committed top-level client：production navigation commit 登记 top-level client、
  replacement start 清旧 epoch client 见
  [M3 committed top-level client](evidence/2026-08-21-m3-committed-top-level-client.md)
- M3-33 window client lifecycle：browser owner 显式 `auxiliary`/`nested` 创建与单 client
  销毁入口、消息队列清理见
  [M3 window client lifecycle](evidence/2026-08-21-m3-window-client-lifecycle.md)
- M3-34 renderer iframe client lifecycle：iframe 物化/销毁经 renderer IPC 接入 browser-owned
  nested window client registry 见
  [M3 renderer iframe client lifecycle](evidence/2026-08-21-m3-renderer-iframe-client-lifecycle.md)
- M2-1 fetch runtime foundation：runtime `FetchEvent`/`Request`/`Response` MVP、
  manager longest-scope dispatch、IPC command/event 与定向验证见
  [M2 fetch runtime foundation](evidence/2026-08-21-m2-fetch-runtime-foundation.md)

## M0 证据与决策记录

| 日期 | 事项 | 结果 |
|------|------|------|
| 2026-08-19 | WPT 可执行面 | 当前 0；M1 纳入单页面生命周期，M2 纳入单客户端 fetch，重依赖用例逐案 skip |
| 2026-08-19 | WPT 完整分母 | manifest：801 源文件；294 testharness 源生成 331 URL；正文 294/294，核心 228/276 命中直接重依赖信号 |
| 2026-08-19 | WPT 逐文件清单 | 294 唯一路径 / 331 URL / 12 candidate / 130 gated / 152 review，294/294 blob SHA 匹配，inventory SHA-256 `8905f3de41dd53432758461b64cf68a59ebcdecd970f3d0add724957e709a3e7` |
| 2026-08-19 | M1 候选资源闭包 | 12/12 已审计，39/39 对象 blob SHA 匹配；8 Tier A keep-first / 3 Tier B defer / 1 Tier C dynamic-server |
| 2026-08-19 | Tier A 验收合约 | 8 case / 28 subtest / 18 asset（235,111 bytes）/ 5 驱动阶段；assets SHA-256 `c9b8089dc425873e3249d0e834176139c054f3e33845ba6c4080521f23fa6bc0` |
| 2026-08-19 | Tier A 资产化 | 独立 WPT root + raw/jsDelivr 双源 + WPT_SOURCE；本地/网络 18/18、幂等/续传/篡改修复、非法路径 fail-closed、clippy、make test 全通过 |
| 2026-08-19 | Tier A 可重复审计 | verify-only 18/18；缺失/篡改 fail closed；audit + shell regression Make target 已落 |
| 2026-08-19 | M1 第二批裁决 | 14 case / 78 subtest：3 core（4 subtest）/ 5 advanced / 5 dynamic-server / 1 update；next-wave 7 assets |
| 2026-08-19 | M1 第二批资产化 | 3 case 记入 testharness 账本；共享根 7/7，幂等/篡改修复/非法 count/Tier A 回归通过 |
| 2026-08-19 | M1 iframe 裁决 | 11 case / 41 subtest：3 single-iframe / 1 worker / 3 controller defer，1 fetch gated，3 multi-client skip |
| 2026-08-19 | M1 message-channel 裁决 | 7 case / 18 subtest：2 lifecycle defer，2 controller defer，2 update/fetch gated，1 cross-origin skip |
| 2026-08-19 | M1 review 收口 | 最后 25 case / 99 subtest：19 gated / 6 skip；M1 review 57/57，全量逻辑 review 剩余 95 |
| 2026-08-19 | Static Routing 裁决 | 本批 11 case / 70 subtest 全部 skip；family 12/12，全量逻辑 review 剩余 84 |
| 2026-08-19 | Worker Global/import 裁决 | 13 case / 53 subtest：1 core / 6 runtime defer / 4 server gated / 1 M2 defer / 1 skip；剩余 71 |
| 2026-08-19 | Static-wave 资产化 | scriptURL 1 case / 4 subtest / 2 assets；fetch/audit/regression targets 与账本已落 |
| 2026-08-19 | IDL harness 裁决 | 4 generated URL / 787 subtest：window 175、dedicated/shared 各 155、serviceworker 302；剩余 70 |
| 2026-08-19 | Navigation/redirect 裁决 | 15 source / 16 URL / 224 subtest：2 defer / 10 gated / 3 skip；剩余 55 |
| 2026-08-19 | Request/response/timing 裁决 | 17 source / 83 subtest：7 defer / 9 gated / 1 skip；剩余 38 |
| 2026-08-19 | WPT review 收口 | 最后 38 source / 270 subtest：14 defer / 8 gated / 16 skip；初始 review 152/152，剩余 0 |
| 2026-08-19 | WPT runner disposition | 294 source / 331 URL：12 core / 51 defer / 189 gated / 42 skip；机器 contract 可确定性重建 |
| 2026-08-19 | Core runner 供应链 | 12 core = 12 imported testharness = 8+3+1 case asset；revision 与 blob SHA 双向一致 |
| 2026-08-19 | Registry 契约测试 | 4 项替换/失败/隔离中间态不变量；Service Worker 模块 40/40，zero-storage clippy 通过 |
| 2026-08-19 | M1 WorkerRuntime readiness | V8 20/20、QuickJS 3/3、WebView 双后端各 17/17；确认 QuickJS timeout 与 evaluate handshake 缺口 |
| 2026-08-19 | RFC 决策 | 用户明确批准方案 C；browser manager owner、WebView adapter 与 M1 实施顺序生效 |
| 2026-08-19 | M1-1 typed runtime | 抽取共享线程核；新增双引擎 typed SW evaluate/shutdown，资源封顶与错误分类；全矩阵通过 |
| 2026-08-19 | M1-2 manager | scope-keyed 三版本 slot + runtime owner；失败保持旧 active；容量/输入 fail closed；三矩阵各 56/56 |
| 2026-08-19 | M1-3a lifecycle runtime | 双引擎 install/activate + waitUntil typed outcome；runtime 10/10、manager 11/11 |
| 2026-08-19 | M1-3b WebView host bridge | manager 自动推进；WebView 同源真实 fetch/install/activate；双后端 E2E 各 4/4 |
| 2026-08-19 | M1-3c page bridge | navigator register/snapshot/unregister 接 manager；删除 timer 状态模拟；双后端 6/6 |
| 2026-08-19 | M1-4a IPC contract | register/snapshot/unregister/activate-waiting typed wire；无 script source；protocol 298/298 |
| 2026-08-19 | M1-4b browser owner | normal/private manager 单一 owner；committed URL authority；browser-owned async script fetch；browser 370 tests |
| 2026-08-19 | M1-4c renderer bridge | request ID router + register/snapshot/unregister callbacks；fresh peer production E2E；owner 跨 renderer 存活 |
| 2026-08-19 | M1-4d registration discovery | getRegistration(s) typed IPC；active-first scope representative；fresh renderer 无旧 ID 恢复 registration |
| 2026-08-19 | M1-5 core WPT baseline | 12/12 case；36/36 subtest 有结果；23 Pass / 12 Fail / 1 Timeout / 0 Unsupported；两轮稳定 |
| 2026-08-19 | M1-5b lifecycle task | transition log + cursor；EventTarget/updatefound/statechange/slot task；30 Pass / 6 Fail / 0 Timeout；两轮稳定 |
| 2026-08-19 | M1-5c registration URL | shared validator + WebIDL scope + typed rejection；36 Pass / 0 Fail / 0 Timeout；两轮稳定 |
| 2026-08-20 | M3-1 skipWaiting | typed lifecycle signal；replacement 自动激活；旧 active redundant；core WPT 36/36 与全量门禁通过 |
| 2026-08-20 | M3-2 controller | 新 Document active scope；首次注册保持 uncontrolled；replacement controllerchange；双引擎与生产链通过 |
| 2026-08-20 | M3-3 clients.claim | activate typed claim；committed Document controller；双引擎/fresh renderer/全量门禁通过 |
| 2026-08-20 | M3-4 page-to-worker message | ServiceWorker.postMessage；typed runtime MessageEvent；handler failure 隔离；全量门禁通过 |
| 2026-08-20 | M3-5 worker-to-page message | Client.postMessage；per-Document immutable log；container MessageEvent；双引擎/生产链通过 |
| 2026-08-20 | M3-6 update job | registration.update；top-level byte comparison；changed/no-op/error；disposition 13 core / 50 defer；WPT 13/37 |
| 2026-08-20 | M3-7 registration persistence | normal active snapshot；runtime/controller restart；no lifecycle replay；private/损坏隔离 |
| 2026-08-20 | M3-8 updateViaCache | typed registration policy；top-level browser cache mode；persistence migration；双引擎/生产链通过 |
| 2026-08-20 | M3-9 importScripts graph | browser-owned batch fetch；ordered same-global execution；graph bytecheck/persistence；WPT 14/38 |
| 2026-08-20 | M3-10 import response policy | classic no-cors；动态 MIME；65-message worker result channel；WPT 16/62 |
| 2026-08-20 | M3-11 dynamic import semantics | resource map；redirect/stash/update；WorkerLocation；key unregister；WPT 18/67 |
| 2026-08-20 | M3-12 event-time import context | persistent context；updated flag；install fetch；message late-import rejection；WPT 19/72 |
| 2026-08-20 | M3-13 module type contract | type 贯穿 JS/IPC/storage/persistence/renderer；module loader 前 fail closed |
| 2026-08-20 | M3-14 static module graph | canonical referrer 递归 fetch/compile/execute；module WPT 3/3；core 20/75 |
| 2026-08-20 | M3-15 module update bytecheck | classic/module main/imported 4×2 矩阵 8/8；core 21/83 |
| 2026-08-20 | M3-16 module CORS | cross-origin module ACAO fail closed；bytecheck 8/8；core 22/91 |
| 2026-08-20 | M3-17 module re-export | named/star/namespace re-export；canonical recursive graph |
| 2026-08-20 | M3-18 module registration errors | network/parse/runtime/instantiation/TLA 10/10；core 23/101 |
| 2026-08-20 | M3-19 module type update | classic/module 切换与 unchanged registration 7/7；core 24/108 |
| 2026-08-20 | M3-20 module request metadata | request mode/no-cache headers 2/2；core 26/110 |
| 2026-08-20 | M3-21 updateViaCache matrix | cache policy/iframe/rollback 25/25；core 27/135 |
| 2026-08-21 | M3-22 dynamic import update | import 404/移除/cross-origin update 7/7；core 29/142 |
| 2026-08-21 | M3-23 update failure matrix | MIME/redirect/syntax/install/uninstall/shrink 7/7；core 30/149 |
| 2026-08-21 | M3-24 multiple update | 10 路 burst 共享 update candidate；core 31/150 |
| 2026-08-21 | M3-25 update permissions + MessagePort | worker update 权限矩阵；双向 port transfer；core 32/153 |
| 2026-08-21 | M3-26 skipWaiting no client | 单次/8 路并发均 resolve undefined；core 33/155 |
| 2026-08-21 | M3-27 clients.matchAll evaluation | 同源 uncontrolled client 顶层枚举与主动消息；core 34/156 |
| 2026-08-21 | M3-28 clients.get | 同源 client id 查询与 unknown/cross-origin 隔离；上游完整文件仍因 resultingClientId/fetch 门控 |
| 2026-08-21 | M3-29 clients focus order | `clients.matchAll()` 按最近 focus window 优先排序；active tab 投影到 SW client registry |
| 2026-08-21 | M3-30 client frameType | manager 显式 `frameType` 观测入口；IPC 允许 `auxiliary`；nested 透传定向测试 |
| 2026-08-21 | M3-31 same-tab clients | browser owner 保留同 tab 多个 client；disconnect tab 成组移除；focus 仍选择 top-level/auxiliary |
| 2026-08-21 | M3-32 committed top-level client | production navigation commit 登记 top-level SW client；replacement start 清旧 epoch client |
| 2026-08-21 | M3-33 window client lifecycle | browser owner 暴露 window client 创建/销毁入口；移除 nested 不影响同 tab top-level/auxiliary |
| 2026-08-21 | M3-34 renderer iframe lifecycle | iframe contentWindow 物化触发 nested client observe；删除/替换/清空子树触发 remove；browser 归一 child client id |
| 2026-08-21 | M2-1 fetch runtime foundation | `FetchEvent`/`Request`/`Response` MVP；manager longest-scope dispatch；browser/renderer IPC command/event；生产页面 fetch/Cache 集成仍待后续 |
| 2026-08-19 | 三方案对比 | 拒绝同线程 context（无调度隔离）；拒绝从零线程（复制安全基建）；推荐抽取 Worker 线程核 |
| 2026-08-19 | owner | production browser process 单一 owner；WebView 只做同算法 in-process adapter |
| 2026-08-19 | 首个 driving WPT | `activation-after-registration.https.html` |
