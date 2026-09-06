# Service Worker 真实化 — 运行时控制面板（master.md）

**入口文档**: [../service-workers.md](../service-workers.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-06（M2-44 streaming/cancel 专项——readable-stream 全案纳入 fetch baseline 85/85；settle 语义对齐 spec；fetch runner 30→31 case）

---

## 当前状态

**专项定位**：存储方向三拆之三。把 `navigator.serviceWorker` 从注册表状态机近似
（R3318）深化为真实 SW 执行环境 + fetch 拦截。用户已于 2026-08-19 明确批准方案 C，
M0 启动门禁解除；M1 core WPT 已收敛，M2 已完成 fetch runtime/manager/IPC foundation、
browser-process 页面 fetch 路由和 Service Worker `caches.match()` / `caches.open()` /
`Cache.put()` / `Cache.matchAll()` / `Cache.keys()` 桥接，并已透传 `ignoreSearch` /
`ignoreMethod` 查询选项；worker-global `fetch()` 已通过 browser-owned network bridge 接入，
SW runtime `Cache.add()` / `Cache.addAll()` 可用同一 fetch→put 链路写入 active registration
`CacheStorage`，`Cache.delete()` 与 `CacheStorage.delete()/has()/keys()` 也已接入同一
typed host bridge，并复用 `zero-storage` 的请求头快照、Vary/`ignoreVary` 匹配语义；
M3 worker-global registration 上游 WPT core baseline 已扩展到 65 case / 249 subtest /
249 Pass，覆盖 `registration.scope`、worker global `registration.{installing,waiting,active}`
slot 投影、registration/worker `EventTarget`、`updatefound` 与 `statechange` 可观察顺序，
message handler 抛错后的 worker-global `ErrorEvent` 字段与 source 回传，以及 page message
无法在 worker 反序列化时的 worker-global `messageerror` 事件，并覆盖受控 iframe
load 时 `controller` 投影与 iframe realm registration worker identity，并覆盖 `getRegistration()` 同源拒绝、fragment 忽略匹配和 controlled iframe 注销后查询；
并覆盖 top-level/iframe registration `waiting` slot 投影、未激活 iframe controller
nullability 与 waiting/active SameObject identity、iframe detach 后 controller 清空，以及
install listener 同步抛错的 worker-global `error` 报告且不失败 install、`waitUntil()`
rejection 仍失败 install 的 lifecycle error 边界，以及 activate listener 同步抛错的
worker-global `error` 报告且不失败 activate，并覆盖 `ExtendableEvent.waitUntil()`
等待全部 lifetime promises settle、首个 rejection 诊断和 activate rejection 不阻断激活，并覆盖
async `waitUntil()` 在 task/microtask 与 `respondWith()` lifetime extension 下的合法窗口；
并覆盖 iframe reload 后新 document 重新获得 active controller 以及 iframe realm
`ServiceWorker` identity；
首个
M2 fetch/interception 上游 WPT
`request-end-to-end.https.html`、`fetch-event-async-respond-with.https.html`、
`fetch-event-network-error.https.html`、`fetch-event-respond-with-argument.https.html`、
`iso-latin1-header.https.html`、`fetch-event-add-async.https.html`、
`fetch-event-within-sw.https.html`、`fetch-event-respond-with-custom-response.https.html` 与
`fetch-event-respond-with-stops-propagation.https.html`、`uncontrolled-page.https.html`、
`claim-fetch.https.html`、`claim-not-using-registration.https.html`、
`claim-using-registration.https.html`、`unregister-controller.https.html` 与
`fetch-event-throws-after-respond-with.https.html`、`fetch-on-the-right-interface.https.any.js`、
`historical.https.any.js`、`fetch-event-handled.https.html`、
`fetch-event-after-navigation-within-page.https.html` 与
`intercepted-referrer.https.html`、`controller-with-no-fetch-event-handler.https.html`、
`fetch-with-body.https.html`、`fetch-error.https.html`、`invalid-header.https.html` 与
`invalid-blobtype.https.html`、`fetch-event-respond-with-readable-stream-chunk.https.html` 与
`ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html` 与
`fetch-event-respond-with-body-loaded-in-chunk.https.html`、
`fetch-event-respond-with-response-body-with-invalid-chunk.https.html` 已形成独立 runner 与
30 case / 75 subtest / 75 Pass 确定性
baseline；SW CacheStorage
serviceworker wrapper 已
扩展到 `cache-storage`、`cache-storage-keys`、`cache-delete`、`cache-keys`、`cache-matchAll`、
`cache-storage-match`、`cache-match`、`cache-put`、`cache-add`、`cache-abort`、
`cache-keys-attributes-for-service-worker`、`credentials` 与 top-level
`cache-storage.https.any.js`、`cache-add.https.any.js`、`cache-delete.https.any.js`、
`cache-keys.https.any.js`、`cache-match.https.any.js`、`cache-matchAll.https.any.js`、
`cache-put.https.any.js`、`cache-abort.https.any.js`、`cache-storage-buckets.https.any.js`、
`cache-storage-keys.https.any.js` 与 `cache-storage-match.https.any.js`
Service Worker global variants、`cache-keys-attributes-for-service-worker` /
`credentials` 25 case / 318 subtest / 318 Pass 确定性
baseline，覆盖 worker-global
`caches.open()`、`CacheStorage.has/delete/keys/match()`、opened `Cache` identity、
delete dooming、缺参 TypeError、`Cache.match/delete/keys/matchAll()`、query options、Vary
matching、worker `fetch()` response URL/type/blob/readback、no-cors opaque filtered response、
worker `Cache.put()` response body consumption/cacheability、`Response.redirect()`、
`URL.hostname` mutation、`Cache.addAll()` 原子失败、重复 request / response `Vary`
重复检测、AbortError rejection、DOMString code-unit cache name 保真，以及 browser-created
navigation `Request.isReloadNavigation` / `Request.isHistoryNavigation` 经 `Cache.put()` 到
`Cache.keys()` 的保真，并覆盖 iframe XHR credentialed request URL 经 worker fetch
interception、Cache key storage 和 worker-to-controlled-iframe `Client.postMessage()` 的
端到端保真。broader fetch/cache 基线仍待后续切片，M3 控制语义继续推进。兄弟目标
`storage-cache-api` 已完成 WebView/in-process 页面 `caches.open()` + `Cache.put()/match()` /
`Cache.matchAll()` / `Cache.keys()` 与页面 `Cache.add()` / `Cache.addAll()` GET fetch→store
链路；共享 `zero-storage::Cache::put()` 已拒绝非 GET、非 HTTP(S)、206、`Vary: *` 与
允许 `Response.type == "error"` 作为 CacheStorage 条目写入读回，并已接入上游 CacheStorage
window 面 WPT baseline（23 case / 293 subtest / 293 Pass / 0 Fail），其中
delete-dooming 生命周期、DOMString code-unit name wire、Vary/`ignoreVary`、`Cache.matchAll()`、
`CacheStorage.match()`、cached `Response.type`/`Response.url` 读回保真、`Cache.put()`
body consumption、opaque 内部 206 / `Vary: *` 可缓存、`Response.redirect()` 与 Blob/FormData
response body、`Cache.addAll()` undefined entry 拒绝与 Vary-aware duplicate 判定等共享语义
以及 Window/Dedicated Worker/nested Dedicated Worker 共享同一 CacheStorage owner 的 WPT 路径已落地。
后续 sibling baseline 已扩展到 38 case / 448 subtest / 448 Pass / 0 Fail，继续覆盖
filtered response 类型矩阵、sandboxed iframe CacheStorage 安全边界，以及 top-level
credentialed request URL cache key 往返。该 sibling 的 page/WebView `StorageManager` owner 现已支持
per-origin CacheStorage 持久化和跨 WebView 重建读回，Browser normal profile 使用 sibling
CacheStorage 目录且 private profile 保持内存。SW active registration 的 registration-local
`CacheStorage` 现已纳入 `ServiceWorkerPersistentRegistration` snapshot/restore；normal profile
在 `caches.open()` / `Cache.put()` 成功后标记持久化 dirty 并写回 Service Worker persistence
JSON，private profile 继续只保留内存态。
该 sibling 与当前 SW runtime 链路共用 Cache API 语义，但 SW fetch/cache 专属 WPT baseline
和 opaque/basic/cors 等剩余 filtered response 生成/可缓存性矩阵仍归后续切片。

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
  fail-closed + testharness 账本已落；SW runner 已随 M1 实现（D1 方案 C 2026-08-19
  获批，core WPT 162/162 Pass——勘误 2026-08-28 巡检，原文「待 RFC 批准」已过时）
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
- ✅ Worker Global/import 裁决：13 case / 53 subtest 当前分为 3 core、4 runtime defer、
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
  48 core / 35 defer / 169 gated / 42 skip，可从原始 evidence 确定性重建；
  48 个 core 与 runner 导入账本、二十九批 case asset 及 blob SHA 精确对应
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
- ✅ M3-35：`active.https.html` 纳入 core baseline；`registration.active` 在 activating
  阶段可见，既有 iframe 初始 `controller` 保持 null，同窗口 active getter 对同一 worker
  返回同一 `ServiceWorker` 对象；core WPT 35/158
- ✅ M3-36：`skip-waiting-without-using-registration.https.html` 纳入 core baseline；
  未受控 iframe 上 `skipWaiting()` 不会隐式改变该 iframe 的 `controller`；core WPT
  36/160
- ✅ M3-37：`skip-waiting-using-registration.https.html` 纳入 core baseline；受控 iframe
  在 replacement 激活期间观测到 `controllerchange`，事件期 `controller.state` 保持
  `activating`，真实 worker-testharness 结果通道完成；core WPT 37/162
- ✅ M3-38：`registration-events.https.html` 与 `registration-end-to-end.https.html` 已通过
  单 case 验证并补齐 7 asset fail-closed manifest；`registration-updateviacache.https.html`
  长跑挂起已收敛，25/25 Pass
- ✅ M3-40：`registration-events.https.html` 与 `registration-end-to-end.https.html` 纳入
  core baseline；真实 install/activate 事件回传与完整生命周期状态序列均通过，core WPT
  39/164
- ✅ M3-41：`ServiceWorkerGlobalScope/isSecureContext.https.html` 纳入 core baseline；
  SW global 在 `WorkerGlobalScope.prototype` 暴露 `isSecureContext === true`，真实
  worker-testharness 结果通道完成；core WPT 40/166
- ✅ M3-42：`install-event-type.https.html` 纳入 core baseline；`InstallEvent`
  继承 `ExtendableEvent`，install 事件暴露 `type === "install"`、`bubbles === false`
  与 `cancelable === false`；core WPT 41/167
- ✅ M3-43：`ServiceWorkerGlobalScope/close.https.html` 纳入 core baseline；确认
  SW global 不暴露 `close()`；core WPT 42/169
- ✅ M3-44：`interface-requirements-sw.https.html` 纳入 core baseline；`FetchEvent`
  constructor 按 WebIDL required dictionary member 语义拒绝缺失/非法 `request`，并确认
  worker global 不暴露 `XMLHttpRequest` / `URL.createObjectURL`；core WPT 43/173
- ✅ M3-45：`historical.https.any.js` 纳入 core baseline；确认历史接口
  `FetchEvent.prototype.targetClientId` 不暴露；core WPT 44/175
- ✅ M3-46：`no-dynamic-import.any.js` 纳入 core baseline；classic Service Worker
  global 中 `import(url)` 返回 rejected promise，维持动态 import 禁用语义；core WPT
  45/176
- ✅ M3-47：`no-dynamic-import-in-module.any.js` 纳入 core baseline；`serviceworker-module`
  runner 以 module 类型注册 worker，classic 与 module worker 动态 `import(url)` 均返回
  rejected `TypeError` promise；core WPT 46/183
- ✅ M3-48：`global-serviceworker.https.any.js` 纳入 core baseline；worker global
  暴露只读 `self.serviceWorker`，install/activate 事件期 `registration.installing` /
  `registration.active` 指向当前 worker，启动期 `serviceWorker.postMessage()` 自消息
  `event.source === serviceWorker`；core WPT 47/188
- ✅ M3-49：`immutable-prototype-serviceworker.https.html` 纳入 core baseline；worker
  global prototype chain 对 `Object.setPrototypeOf()` / `Reflect.setPrototypeOf()` 的
  不可变语义已固定；core WPT 48/189
- ✅ M3-50：`ServiceWorkerGlobalScope/unregister.https.html` 纳入 core baseline；
  worker-global `registration.unregister()` 经 typed host bridge 完成，evaluation/install/
  activate 中注销进入 `redundant` 并移除 future matching，active controlling worker 注销后
  保留既有 controller 且不控制新 client；core WPT 49/193
- ✅ M3-51：`ServiceWorkerGlobalScope/message-event-ports.https.html` 纳入 core baseline；
  worker-side `MessageEvent.ports` getter 在同一事件对象上重复读取返回同一 ports array
  identity；core WPT 50/194
- ✅ M3-52：`ServiceWorkerGlobalScope/extendable-message-event.https.html` 纳入 core
  baseline；page/nested client → worker、worker loopback、active ↔ waiting worker
  `ExtendableMessageEvent` source/ports/origin 语义通过；core WPT 51/198
- ✅ M3-54：`ServiceWorkerGlobalScope/registration-attribute.https.html` 纳入 core
  baseline；worker global `registration.scope`、lifecycle slot 投影、registration/worker
  `EventTarget`、`updatefound` 与 `statechange` 顺序通过；core WPT 52/200
- ✅ M3-55：`ServiceWorkerGlobalScope/service-worker-error-event.https.html` 纳入 core
  baseline；page-to-worker message listener 抛错后，worker-global `error` listener
  可观察 `ErrorEvent.error/message/filename/lineno/colno` 并通过原 `WindowClient`
  source 回传；core WPT 53/201
- ✅ M3-56：`ServiceWorkerGlobalScope/error-message-event.https.html` 纳入 core
  baseline；page 侧 `canvas.captureStream()` synthetic track 进入 Service Worker
  `postMessage()` transfer list 时触发 worker-global `messageerror`，并通过原
  `WindowClient` source 回传；core WPT 54/202
- ✅ M3-57：`controller-on-load.https.html` 纳入 core baseline；新加载受控 iframe
  在 load 时暴露 `navigator.serviceWorker.controller`，且 iframe
  `getRegistration().active` 与同 window `controller` 保持 identity；core WPT
  55/203
- ✅ M3-58：`getregistration.https.html` 纳入 core baseline；`getRegistration()`
  按 document URL 查找 registration，跨源 document URL reject `SecurityError`，
  fragment 不影响 scope 匹配，controlled iframe 注销后查询返回 `undefined`；
  core WPT 56/209
- ✅ M3-59：`registration-iframe.https.html` 纳入 core baseline；iframe
  `register()` 按 iframe document URL 解析 scriptURL/scope，并在 resolved
  registration 上立即暴露 iframe realm `installing` worker；core WPT 57/212
- ✅ M3-60：`installing.https.html` 纳入 core baseline；top-level registration
  在安装期暴露 `installing` worker，且 `getRegistration(scope)` 返回同一 underlying
  worker 的 SameObject 投影；core WPT 58/214
- ✅ M3-61：`waiting.https.html` 纳入 core baseline；top-level/iframe registration
  在安装完成后暴露同一 underlying `waiting` worker，并确认未激活 iframe controller
  仍为 null、replacement registration 的 waiting/active SameObject 投影一致；core WPT
  59/216
- ✅ M3-62：`controller-on-disconnect.https.html` 纳入 core baseline；受控 iframe
  移除后，detached iframe 的 `navigator.serviceWorker.controller` 清空；core WPT
  60/217
- ✅ M3-63：`oninstall-script-error.https.html` 纳入 core baseline；install listener
  同步抛错经 worker-global `error` 报告但不让 install 失败，`event.waitUntil()`
  rejection 仍作为 install 失败信号；core WPT 61/223
- ✅ M3-64：`onactivate-script-error.https.html` 纳入 core baseline；activate listener
  同步抛错经 worker-global `error` 报告但不让 activate 失败；core WPT 62/228
- ✅ M3-65：`extendable-event-waituntil.https.html` 纳入 core baseline；install/activate
  `ExtendableEvent.waitUntil()` 等待全部 lifetime promises settle，install rejection
  仍失败 install，activate rejection 不阻断进入 activated；core WPT 63/234
- ✅ M3-66：`extendable-event-async-waituntil.https.html` 纳入 core baseline；
  per-event lifetime state 与 microtask checkpoint 收敛 async `waitUntil()` 合法窗口，
  `respondWith()` lifetime extension 边界通过；core WPT 64/248
- ✅ M3-67：`controller-on-reload.https.html` 纳入 core baseline；iframe reload
  会注销旧 nested window client，新 document 重新 observe 后获得 active controller，
  iframe realm `ServiceWorker` identity 保持正确；core WPT 65/249
- ✅ M3-68：CacheStorage `cache-put` "getReader() after Cache.put" 两案修复
  （`.any.js` + `serviceworker/` wrapper）。根因：M2 readable-stream body 暴露
  （50245f856）加入 worker `Response.body` getter 后，`response.body` 不再是
  undefined，`getReader()` 二次调用成功暴露 worker runtime `ReadableStream`
  无锁语义；09-02 CacheStorage baseline 记录时 getter 尚未合入，属潜伏红项。
  修复：① `ReadableStream.getReader()` 按 streams spec 加锁（locked 抛
  TypeError / `releaseLock()` 真解锁 / 只读 `locked`）；② `Cache.put()` 成功后
  按 SW spec disturb body（getReader 消费 + 置 `bodyUsed`）；③ `Response`
  构造器补 `_bodyNull` 标记，null-body response 不置 `bodyUsed`。新增回归测试
  `cache_put_disturbs_response_body_but_not_null_body`（V8/QuickJS 双引擎全绿）；
  cache-storage runner 316→318 全绿。
- ✅ M3-69：`ServiceWorkerGlobalScope/extendable-message-event.https.html` wave
  manifest 闭包修复——manifest 漏登记 `extendable-message-event-worker.js` 与
  `extendable-message-event-loopback-worker.js`（04067ce pin，4→6 asset），
  本地资产缺失导致该 case 3 subtest 红；补登记后 core WPT 249/249 全绿恢复。
  同轮恢复本地 47 个 wave manifest 全部缺失 asset（双 revision 共享文件按
  各 runner 依赖对齐 04067ce pin；注意 cache-storage wave manifest 内部混用
  24197a1/04067ce pin，`testharness-service-workers-cache-storage` 入口经其
  依赖的 fetch target 恢复为该 wave 的 pin 组合）。
- ✅ M3-53：`ServiceWorkerGlobalScope/postmessage.https.html` 纳入 fetch/message
  baseline；worker self-loopback、active → waiting worker transferred `MessagePort`
  转发以及最终 page port 回信语义通过；fetch/message WPT 28/73
- ✅ M2-42：`fetch-event-respond-with-body-loaded-in-chunk.https.html` 纳入 fetch/message
  baseline；worker-side network `fetch(...).body` 经
  `respondWith(new Response(body))` 转发到受控 iframe；fetch/message WPT 29/74
- ✅ M2-43：`fetch-event-respond-with-response-body-with-invalid-chunk.https.html` 纳入
  fetch/message baseline；非 `Uint8Array` stream chunk 通过 page-side
  `response.body` reader 以 TypeError reject；fetch/message WPT 30/75
- ✅ M2-44：streaming/cancel 专项落地——`fetch-event-respond-with-readable-stream.https.html`
  全 10 subtest 纳入 fetch/message baseline（31 case / 85 subtest / 85 Pass 双跑
  deterministic）。① respondWith 的 stream body 按 event_id 登记，新 typed
  `CancelFetchBody` 命令经 manager/host 到 runtime 调 stream.cancel（underlying
  source cancel 可观察，页面 body cancel 与 settle 后 signal abort 同桥幂等）；
  in-process 路径经内部 `X-Zero-Sw-Fetch-Id` 头 + `__zw_sw_fetch_body_cancel`
  回调桥；browser IPC host 为默认 no-op（生产 body 流式化前无反传面，显式注记）；
  ② worker `ReadableStream` 补 `cancel()` 方法（此前缺失）；③ respondWith 序列化
  加 pump watchdog（连续 64 泵无 done 交付已累积字节；不用 setTimeout——runtime
  timerTasks FIFO 无延迟序会插队，曾破坏 fetch-error 的 error-before-abandon 序）；
  ④ settle 语义修正：respondWith 已调时 FetchSettled 不再等 `waitUntil()`
  生命周期（spec：respondWith promise fulfill 即回应 fetch；否则 observe-cancel
  的 waitUntil 与 query fetch 互锁）。新增单测
  `cancel_fetch_body_invokes_stream_source_cancel`。
- ✅ M2-1：Service Worker `FetchEvent` runtime foundation、manager longest-scope dispatch
  与 renderer/browser IPC command/event 已接通；`respondWith(new Response(...))`、未调用
  `respondWith` pass-through、重复 `respondWith` failure、跨 origin/out-of-scope pass-through
  均有定向测试；生产页面 `FetchRequest` 路由与 Cache API 集成仍未接入
- ✅ M2-2：production `ProcessTabBackend::handle_fetch_request()` 接入 browser-owned
  `ServiceWorkerManager::dispatch_fetch()`；已提交 document authority + same-origin + longest-scope
  active worker 命中时先派发 SW `fetch` 事件，`respondWith(Response)` 转原 `FetchResponse`
  返回 renderer，未响应/无匹配/派发失败/内部 DNS prefetch/stream image/非 UTF-8 body 保持原网络
  fallback；browser 重写内部 `X-Zero-Final-URL` / `X-Zero-Resource-Type` 元数据避免 worker 伪造
- ✅ M2-3：Service Worker runtime 暴露 `globalThis.caches.match(input)`；通过 typed
  `CacheStorageRequested` / `CompleteCacheStorage` host bridge 查询 browser-owned active registration
  `CacheStorage`，命中时物化为 `Response` 并可直接用于
  `event.respondWith(caches.match(event.request))`
- ✅ M2-4：Service Worker runtime 暴露 `caches.open(name)` 与 `Cache.put(request, response)`；
  runtime/renderer/browser/manager/protocol 均改为 typed CacheStorage operation，
  `Cache.put()` 写入 browser-owned active registration `CacheStorage` 后可被同一 worker
  `Cache.match()` 读回；完整 WPT fetch/cache baseline 仍待接入
- ✅ M2-5：Service Worker runtime 暴露 `Cache.matchAll(input?)` 与 `Cache.keys(input?)`；
  runtime/renderer/browser/manager/protocol 延续 typed CacheStorage operation，结果数组有上限并逐项
  校验，optional request filter 保留 method-sensitive 匹配；完整 WPT fetch/cache baseline 仍待接入
- ✅ M2-6：Service Worker runtime 的 `Cache.match()`、`Cache.matchAll()`、`Cache.keys()` 与
  `CacheStorage.match()` 透传 `ignoreSearch`/`ignoreMethod`/`ignoreVary` 查询选项；browser-owned
  registration `CacheStorage` 已应用 `ignoreSearch`/`ignoreMethod`，`ignoreVary` 等待 sibling
  storage-cache-api 的 Vary 语义补齐；完整 WPT fetch/cache baseline 仍待接入
- ✅ M2-7：Service Worker runtime 暴露 worker-global `fetch()`；runtime 通过 typed
  `FetchRequested` / `CompleteFetch` host bridge 发起 browser-owned ordinary fetch，renderer/browser
  IPC、browser fetch proxy、WebView in-process fetch handler 均已接线；`Cache.add()` /
  `Cache.addAll()` 基于该 fetch→put 链路写入 active registration `CacheStorage`；共享
  `zero-storage::Cache::put()` 已拒绝非 GET、非 HTTP(S)、206、`Vary: *` 与
  允许 `Response.type == "error"` 作为 CacheStorage 条目写入读回；runtime/IPC/manager/
  browser/WebView 显式透传 `response_type`，且 FetchEvent response settlement 仍保持
  200..599 限制。
  broader fetch/cache baseline 仍待接入
- ✅ M2-8：首个 Service Worker fetch/interception WPT baseline 接入：
  `service-workers/service-worker/request-end-to-end.https.html` 固定 revision
  `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`，`testharness-service-workers-fetch`
  独立 runner 双跑 1 case / 1 subtest / 1 Pass / 0 Fail / 0 Timeout / deterministic true。
  该用例覆盖真实 worker `onfetch`、受控 iframe navigation FetchEvent、`FetchEvent.request`
  的 absolute URL / `GET` / referrer / `navigate` / `include` / `manual` 投影、
  immutable request headers 与 `new Request(event.request)`；同时新增 WebView in-process
  回归测试固定 iframe plain-text response body 与 iframe `src` absolute getter 行为。
  CacheStorage 方向的更宽 fetch/cache WPT baseline 仍待后续切片。
- ✅ M2-9：Service Worker runtime 暴露 `Cache.delete()` 与 `CacheStorage.delete()` /
  `CacheStorage.has()` / `CacheStorage.keys()`；runtime/renderer/browser/manager/protocol
  全链路新增 typed delete/listing 操作，entry 删除和命名 cache 删除均复用 registration-local
  `zero-storage::CacheStorage`，成功 delete mutation 会触发 normal profile SW persistence
  dirtying。SW cache-storage WPT 扩面仍待后续切片。
- ✅ M2-10：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-async-respond-with.https.html` 与
  `service-workers/service-worker/fetch-event-respond-with-argument.https.html`；worker global
  timer task queue、FetchEvent dispatch microtask checkpoint 与 page-side client-message polling
  已固定 `respondWith()` timing boundary：dispatch microtask 内调用可接受，后续 task 调用抛
  `InvalidStateError` 并走网络 fallback；`respondWith()` argument matrix 已覆盖
  `Response`、`Promise<Response>` 与非 Response 值转 network error。WebView iframe nested
  client 观测桥、iframe `contentWindow.fetch()` / `XMLHttpRequest` 的 iframe URL 相对解析与
  client id/referrer 透传、manager 按受控 client registration 优先派发 fetch 均有定向回归；
  fetch runner 当前 4 case / 7 subtest / 7 Pass / 0 Fail / 0 Timeout / deterministic true。
- ✅ M2-11：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-network-error.https.html`；FetchEvent
  `preventDefault()` 且未调用 `respondWith()` 现在产生 network error，`Response.text()`
  会标记 `bodyUsed`，已消费 body 的 Response 交给 `respondWith()` 时失败，worker-global
  `fetch('other.html')` 的 Response 仍可在未消费 body 时直接透传；fetch-wave 资产清单扩展到
  15 asset，runner 当前 4 case / 7 subtest / 7 Pass / 0 Fail / 0 Timeout / deterministic true。
- ✅ M2-12：Service Worker runtime 修正 fetch handler 在已成功调用 `respondWith()` 后同步
  抛错的结算语义；已提交的 response promise 继续决定 FetchEvent 结果，只有完全未调用
  `respondWith()` 的同步异常才立即失败。完整 WPT 纳入见 M2-30。
- ✅ M2-13：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/iso-latin1-header.https.html`；iframe `contentWindow`
  现在具备独立 message listener/dispatch 面，`postMessage()` 支持 transferred
  `MessagePort`，`MessageEvent.ports` 保真，且父窗口发往 iframe 时按目标窗口 origin
  校验并报告发送方 origin。该用例覆盖 worker `respondWith()` 合成响应中的
  ISO-8859-1 header value 通过受控 iframe `XMLHttpRequest` 路径返回；fetch-wave
  资产清单扩展到 18 asset，runner 当前 5 case / 8 subtest / 8 Pass / 0 Fail /
  0 Timeout / deterministic true。
- ✅ M2-14：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-add-async.https.html`；Service Worker runtime
  在无 lifecycle/fetch pending 时也会推进 worker-global timer task，修正直接
  `importScripts('/resources/testharness.js')` 的 worker 内部 `step_timeout(..., 0)` 结果通道
  长期 pending 的问题。该用例覆盖在后续 task 中添加 `fetch` listener 不抛异常；fetch-wave
  资产清单扩展到 20 asset，runner 当前 6 case / 10 subtest / 10 Pass / 0 Fail /
  0 Timeout / deterministic true。
- ✅ storage-cache-api 侧支撑：WebView/in-process 页面 `CacheStorage` 初始桥接已可通过共享
  `StorageManager` 执行 `caches.open/has/delete/keys/match` 与 `Cache.put/match/delete`；
  origin 由宿主页面 URL 推导，保持与 IndexedDB 相同单一 storage owner。该进展不等同于 SW
  runtime 写入面完成。
- ✅ storage-cache-api 侧支撑：CacheStorage host match/matchAll 已用 Cache 专属 `__zwcr:`
  wire payload 读回 cached `Response.type`，页面 `Response.clone()` 保留 response type；
  该进展固定共享 Cache API 元数据链路，但不等同于 SW fetch/cache WPT baseline 完成。
- ✅ storage-cache-api 侧支撑：page/WebView `StorageManager` owner 已支持 per-origin
  CacheStorage 持久化、跨 WebView 重建读回和磁盘 I/O 错误 Promise reject；该进展是 SW
  cache 模式底座；registration-local CacheStorage 持久化见下一条，SW cache-storage WPT
  扩面仍待后续。
- ✅ M3 registration-local CacheStorage persistence：active registration 的 `CacheStorage`
  通过 `CacheStorageSnapshot` 随 `ServiceWorkerPersistentRegistration` 落盘/恢复；normal
  profile 的 SW cache mutation 会触发现有 persistence writer，private profile 继续内存化。
- ✅ M2-15：Service Worker CacheStorage WPT baseline 扩面：
  `service-workers/cache-storage/serviceworker/{cache-storage,cache-storage-keys,cache-delete,cache-keys,cache-matchAll,cache-storage-match}.https.html`
  固定 revision `24197a11e8c5bd29a5cb7bdf18135a82be8a8546`，
  `testharness-service-workers-cache-storage` 独立 runner 双跑 6 case / 68 subtest /
  68 Pass / 0 Fail / 0 Timeout / deterministic true。该批用例在真实 Service Worker
  global 中运行上游 `script-tests/cache-storage*.js`、`cache-delete.js`、`cache-keys.js`、
  `cache-matchAll.js` 与 `cache-storage-match.js`，覆盖 `caches.open()`、
  `CacheStorage.has/delete/keys/match()`、opened `Cache` identity、delete dooming、
  empty cache name、缺参 TypeError、`Cache.delete/keys/matchAll()`、query options、
  Vary matching、worker `Cache.add()` 与 unpaired surrogate cache name 的 DOMString
  code-unit 保真。
- ✅ M2-16：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/cache-match.https.html`；资产清单固定到
  22 asset，runner 双跑 7 case / 94 subtest / 94 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片补齐 worker runtime 的最小 Blob/FileReader、cached
  `Response.url` 往返、response guard 隐藏 `Set-Cookie` 与内部 `X-Zero-*` 元数据、
  `Response.blob()/arrayBuffer()` 以及 cross-origin `fetch(..., {mode:'no-cors'})`
  的 opaque filtered response 投影；driving WPT 已记入 `imported-testharness.txt`。
- ✅ M2-17：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/cache-put.https.html`；资产清单固定到
  25 asset，runner 双跑 8 case / 121 subtest / 121 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片补齐 worker runtime 的 `Request.bodyUsed` 初值、
  `Response.redirect()`、Blob response body 序列化，以及 `URL.hostname` mutation 后
  `new Request(url, {mode: 'no-cors'})` 经 worker `fetch()` 生成 opaque filtered response
  的路径；driving WPT 已记入 `imported-testharness.txt`。
- ✅ M2-18：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/cache-add.https.html`；资产清单固定到
  27 asset，runner 双跑 9 case / 144 subtest / 144 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片补齐 worker runtime `Cache.addAll()` 的批量原子写入前
  duplicate 检查：同一 request key + headers 立即 reject `InvalidStateError`，fetch 后再按
  response `Vary` 对批次内 entry 做双向匹配，匹配冲突时不发出任何 `Cache.put()`；同时将
  worker-global `Request.credentials` 透传到 host fetch，使本地 WPT `vary.py` fixture 可区分
  `same-origin` cookie override 与 `omit` 路径；driving WPT 已记入
  `imported-testharness.txt`。
- ✅ M2-19：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/cache-abort.https.html`；资产清单固定到
  30 asset，runner 双跑 10 case / 154 subtest / 154 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片补齐 SW runtime `AbortController` / `AbortSignal`、
  `Request.signal` 和 worker `fetch()` abort 传播，使 aborted request 能让
  `Cache.put()` / `Cache.add()` / `Cache.addAll()` reject `AbortError`；同时让测试页在
  probe 间主动轮询 Service Worker client-message task，避免 abort fixture 的 timer message
  滞留。
- ✅ M2-20：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/cache-keys-attributes-for-service-worker.https.html`；
  资产清单固定到 32 asset，runner 双跑 11 case / 156 subtest / 156 Pass / 0 Fail /
  0 Timeout / deterministic true。该切片在 browser-created iframe navigation request 上
  透传 `isReloadNavigation` / `isHistoryNavigation`，并让这些 Request 属性经
  `Cache.put(event.request)` 到 `Cache.keys()` 保真；同时补齐 iframe-local
  `location.reload()` 与 `history.go(-1)` 的最小 WPT 路径。
- ✅ M2-21：Service Worker CacheStorage WPT baseline 扩展到
  `service-workers/cache-storage/serviceworker/credentials.https.html`；资产清单固定到
  35 asset，runner 双跑 12 case / 157 subtest / 157 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片让受控 iframe 的 `navigator.serviceWorker` 使用 iframe
  client id/container 轮询 worker `Client.postMessage()`，并补齐 iframe `XMLHttpRequest`
  的 readyState 常量与 `open()` username/password URL 注入，使 credentialed request URL
  经 worker fetch interception、`Cache.put()`、`Cache.match()` / `Cache.matchAll()` /
  `CacheStorage.match()` 和 `Cache.keys()` 保真。
- ✅ M2-40：Service Worker CacheStorage WPT baseline 扩展到顶层
  `service-workers/cache-storage/cache-keys-attributes-for-service-worker.https.html`；资产清单固定到
  36 asset，runner 双跑 13 case / 159 subtest / 159 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片将原 gated 顶层页纳入 CacheStorage wave，覆盖 browser-created
  navigation request 的 `isReloadNavigation` / `isHistoryNavigation` 经
  `Cache.put(event.request)` 到 `Cache.keys()` 保真，和现有 `serviceworker/` wrapper
  共同守住相对路径差异。
- ✅ M2-41：Service Worker CacheStorage WPT baseline 扩展到顶层
  `service-workers/cache-storage/credentials.https.html`；资产清单固定到 37 asset，
  runner 双跑 14 case / 160 subtest / 160 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片将原 gated 顶层页纳入 CacheStorage wave，覆盖
  credentialed request URL 经 iframe XHR、worker fetch interception、Cache key storage、
  `Cache.match()` / `Cache.matchAll()` / `CacheStorage.match()` 与 worker-to-client
  `postMessage()` 的端到端保真，和现有 `serviceworker/` wrapper 共同守住相对路径差异。
- ✅ M2-42：Service Worker CacheStorage WPT baseline 扩展到 `.any.js` 派生
  `service-workers/cache-storage/cache-storage.https.any.js` 的 Service Worker global variant；
  runner 补齐 WPT `// META: script=` support 注入，资产清单固定到 38 asset，runner 双跑
  15 case / 171 subtest / 171 Pass / 0 Fail / 0 Timeout / deterministic true。该切片覆盖
  `CacheStorage.open()`、delete dooming、空 cache name、缺参 TypeError、has/delete 命中与
  DOMString cache name 保真在真实 Service Worker `.any.js` harness 下直接运行，不再只依赖
  `serviceworker/` HTML wrapper。
- ✅ M2-43：Service Worker CacheStorage WPT baseline 扩展到 8 个额外 top-level `.any.js`
  Service Worker global variant：`cache-add`、`cache-delete`、`cache-keys`、`cache-match`、
  `cache-matchAll`、`cache-put`、`cache-storage-keys` 与 `cache-storage-match`；
  资产清单固定到 46 asset，runner 双跑 23 case / 305 subtest / 305 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片把 CacheStorage/Cache 核心语义在
  Service Worker `.any.js` harness 下直接运行，包括 add/addAll fetch→put、put body/cacheability、
  match/matchAll/delete/keys query options、Vary matching、Blob/FormData/redirect response
  round-trip、CacheStorage.keys/match 和 DOMString cache names。`cache-abort.https.any.js`
  top-level variant 已在 M2-45 纳入 core。
- ✅ M2-44：Service Worker CacheStorage WPT baseline 纳入
  `service-workers/cache-storage/cache-storage-buckets.https.any.js` 的 Service Worker
  global variant；SW runtime 补齐 `WorkerNavigator.storageBuckets` 最小面，bucket-local
  `CacheStorage` 通过内部 cache-name 前缀隔离，同步保留 bucket 删除后的 `UnknownError`
  拒绝语义；classic `importScripts()` 现在会把 imported helper 的顶层 function 声明投影到
  `globalThis`，使 WPT META support helper 在后续 case 中可见。资产清单固定到 48 asset，
  runner 双跑 24 case / 308 subtest / 308 Pass / 0 Fail / 0 Timeout / deterministic true。
- ✅ M2-45：Service Worker CacheStorage WPT baseline 纳入
  `service-workers/cache-storage/cache-abort.https.any.js` 的 Service Worker global variant；
  WPT runner 将既有 `cache-abort` 动态 fetch/stash fixture 注入扩展到 `.any.js`
  worker 包装路径，使 headers-received abort 场景拿到等价 JSON stash 响应。资产清单固定到
  52 asset，runner 双跑 25 case / 318 subtest / 318 Pass / 0 Fail / 0 Timeout /
  deterministic true。
- ✅ M2-22：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-within-sw.https.html`；fetch-wave 资产清单
  扩展到 23 asset，runner 双跑 7 case / 12 subtest / 12 Pass / 0 Fail /
  0 Timeout / deterministic true。该切片覆盖受控 iframe 的 `contentWindow.fetch()` 与
  `contentWindow.caches.open().add()` 都经 SW fetch 事件拦截，同时 worker-global
  `fetch()` / `Cache.add()` 不被同一 SW 自身拦截；iframe `contentWindow.caches`
  现在暴露 CacheStorage，并让 `Cache.add()` 的相对 URL 与 client/referrer 使用 iframe
  文档上下文。
- ✅ M2-23：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-respond-with-custom-response.https.html`；
  fetch-wave 资产清单扩展到 25 asset，runner 双跑 8 case / 23 subtest / 23 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片覆盖 worker `respondWith(new Response(...))`
  合成字符串、Blob、ArrayBuffer、ArrayBufferView、FormData 与 URLSearchParams body，
  并验证这些 response 同时可用于受控 iframe subresource fetch 与 navigation；worker
  bootstrap 补齐最小 TextEncoder/TextDecoder/FormData，页面 Response body shim 补齐
  ArrayBufferView 与 multipart FormData 读回。
- ✅ M2-24：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-respond-with-stops-propagation.https.html`；
  fetch-wave 资产清单扩展到 27 asset，runner 双跑 9 case / 24 subtest / 24 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片覆盖 `FetchEvent.respondWith()` 按
  Service Worker 规范触发 `stopImmediatePropagation()`，并补齐受控 iframe
  `navigator.serviceWorker.controller.postMessage({port}, [port])` 对象内 MessagePort
  transfer 与回信 polling 路径。
- ✅ M2-25：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/uncontrolled-page.https.html`；fetch-wave 资产清单扩展到
  31 asset，runner 双跑 10 case / 25 subtest / 25 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 Service Worker scope 边界：页面位于 registration scope
  外时保持 uncontrolled，其 `XMLHttpRequest` 请求直接走网络，不触发该 worker 的 fetch handler。
- ✅ M2-26：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/claim-fetch.https.html`；fetch-wave 资产清单扩展到
  34 asset，runner 双跑 11 case / 26 subtest / 26 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定现有 iframe client 在 message-time `clients.claim()` 前保持
  uncontrolled，claim 后 controller 投影更新，并让后续 iframe `fetch()` 进入 claiming
  active worker 的 fetch handler。
- ✅ M2-27：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/claim-not-using-registration.https.html`；fetch-wave 资产清单扩展到
  37 asset，runner 双跑 12 case / 28 subtest / 28 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 `clients.claim()` 只控制当前 longest-matching registration
  对应的 client，不会抢占已有更长匹配 registration 的 client。
- ✅ M2-28：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/claim-using-registration.https.html`；fetch-wave 资产清单保持
  38 asset，runner 双跑 13 case / 30 subtest / 30 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 `clients.claim()` 替换由其他 registration 控制的 client，
  同时拒绝同 registration waiting worker 发起 claim。
- ✅ M2-29：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/unregister-controller.https.html`；fetch-wave 资产清单扩展到
  41 asset，runner 双跑 14 case / 33 subtest / 33 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 `registration.unregister()` 只阻止后续 matching/control，
  不清除既有受控 iframe 的 incumbent controller，且该 client 丢弃后再停止旧 active worker。
- ✅ M2-30：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`；
  fetch-wave 资产清单扩展到 43 asset，runner 当前 15 case / 34 subtest / 34 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片让受控 iframe navigation fetch 可异步等待
  worker `respondWith()` promise，同时允许页面 MessagePort ACK 在 pending fetch 期间推进，
  固定 fetch handler 已提交 response 后同步 throw 不覆盖结果的端到端 iframe 语义。
- ✅ M2-31：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/ServiceWorkerGlobalScope/fetch-on-the-right-interface.https.any.js`
  与 `service-workers/service-worker/historical.https.any.js`；fetch-wave 资产清单扩展到
  46 asset，runner 双跑 17 case / 38 subtest / 38 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 `fetch` 暴露在 `WorkerGlobalScope.prototype` 而非
  `ServiceWorkerGlobalScope` instance own property，并确认历史
  `FetchEvent.prototype.targetClientId` 未暴露。
- ✅ M2-32：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-handled.https.html`；fetch-wave 资产清单扩展到
  48 asset，runner 双跑 18 case / 46 subtest / 46 Pass / 0 Fail / 0 Timeout /
  deterministic true。该切片固定 `FetchEvent.handled` 在 pass-through 和成功
  `respondWith()` 时 resolve，在 canceled pass-through、invalid response 和 rejected
  `respondWith()` promise 时 reject，并经 MessagePort 回传给受控页面。
- ✅ M2-33：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-after-navigation-within-page.https.html`；
  fetch-wave 资产清单扩展到 50 asset，runner 双跑 19 case / 48 subtest / 48 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片固定受控 iframe 经同文档 hash
  navigation 与 `history.pushState()` 后仍保留 fetch interception，并让 iframe
  `contentWindow.history.pushState()` / `replaceState()` 更新后续相对 fetch 的文档基准 URL。
- ✅ M2-34：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/intercepted-referrer.https.html`；fetch-wave
  资产清单扩展到 52 asset，runner 双跑 20 case / 49 subtest / 49 Pass /
  0 Fail / 0 Timeout / deterministic true。该切片固定 Service Worker 合成
  navigation response 生成的 iframe 子文档 `document.referrer` 保留父页面 URL，并让
  `ParentNode.append()` 动态插入 iframe 时启动子浏览上下文导航。
- ✅ M2-35：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/controller-with-no-fetch-event-handler.https.html`；
  fetch-wave 资产清单扩展到 55 asset，runner 双跑 21 case / 54 subtest /
  54 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定受控 client 在
  active Service Worker 没有 fetch event handler 时仍走普通 Fetch CORS/no-cors
  处理：跨源 no-cors 投影为 opaque，跨源 CORS 缺少 ACAO 时 reject `TypeError`，
  带 `Access-Control-Allow-Origin: *` 时生成 cors filtered response。
- ✅ M2-36：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-with-body.https.html`；fetch-wave
  资产清单扩展到 58 asset，runner 双跑 22 case / 55 subtest /
  55 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定受控
  client 的 `fetch(new Request(..., {method: "POST", body}))` 经 worker
  `respondWith(fetch(event.request))` 转发时保留 method/body，动态 WPT fixture
  能按请求体有无返回 200/400。
- ✅ M2-37：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/invalid-header.https.html`；fetch-wave
  资产清单扩展到 61 asset，runner 双跑 23 case / 56 subtest /
  56 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `Headers.append()` 与 Service Worker fetch response Rust/IPC 边界拒绝非法
  header name/value，含 NUL/CR/LF 的 synthetic response header 会产生 network error。
- ✅ M2-38：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/invalid-blobtype.https.html`；fetch-wave
  资产清单扩展到 64 asset，runner 双跑 24 case / 57 subtest /
  57 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `respondWith(new Response(new Blob(...)))` 遇到含 NUL 的 invalid Blob MIME type
  时仍允许 fetch 成功，但不得把该无效 MIME type 合成为 `Content-Type` response header。
- ✅ M3-38：Service Worker message/global WPT baseline 扩展到
  `service-workers/service-worker/ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html`；
  fetch-wave 资产清单扩展到 66 asset，runner 双跑 25 case / 67 subtest /
  67 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `ExtendableMessageEvent` 构造器默认值、initializer 转换、`ServiceWorker` /
  `MessagePort` source 接受，以及非法 source/ports 的 `TypeError` 边界。
- ✅ M2-39：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-error.https.html`；fetch-wave
  资产清单扩展到 68 asset，runner 双跑 26 case / 70 subtest /
  70 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `respondWith(new Response(stream))` 的 body stream 先产生进展、后续 error 时，
  页面 `fetch()` promise 先 resolve，随后的 `response.text()` body 消费再 reject。
- 🚧 M2-40：`fetch-event-respond-with-readable-stream.https.html` 预推进修复：
  Service Worker runtime 的最小 `ReadableStream` 构造器现在执行
  `underlyingSource.start(controller)`，并保持 typed-array chunk 边界校验；WebView
  regression 已固定受控 iframe `contentWindow.fetch('?stream')` 可读回 `PASS`；worker-global
  `fetch('pass.txt')` 返回的 `Response.body` 现在可作为 `new Response(response.body)` 的
  `ReadableStream<Uint8Array>` body 转发。单 case 探测在补齐未登记的本地 `resources/pass.txt`
  后显示前 6 个 subtest（subresource/main resource、delayed enqueue、fetch stream）
  已通过；剩余页面 cancel/abort 反传仍需要跨 runtime 边界的真实 streaming/cancel
  语义，且完整 case 资产闭包尚未纳入 fetch-wave manifest，故该 WPT 尚未纳入常驻 runner。
- ✅ M2-41：Service Worker fetch/interception WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-respond-with-readable-stream-chunk.https.html`；
  fetch-wave 资产清单扩展到 70 asset，runner 双跑 27 case / 71 subtest /
  71 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `respondWith(new Response(stream))` 的 pull-source `ReadableStream` 可将空 chunk 与
  非空 `Uint8Array` chunk 串接后经受控 iframe `response.text()` 读回；相邻的
  network `fetch(...).body` 转发、invalid chunk reader rejection 与 partial stream
  incremental reader 用例仍留在 streaming/cancel 专项。
- ✅ M3-53：Service Worker fetch/message WPT baseline 扩展到
  `service-workers/service-worker/ServiceWorkerGlobalScope/postmessage.https.html`；
  fetch-wave 资产清单扩展到 74 asset，runner 双跑 28 case / 73 subtest /
  73 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `ServiceWorker.postMessage()` 中 transferred `MessagePort` 的 worker self-loopback、
  active → waiting worker 转发和最终 page port 回信路径。
- ✅ M2-42：Service Worker fetch/message WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-respond-with-body-loaded-in-chunk.https.html`；
  fetch-wave 资产清单扩展到 77 asset，runner 双跑 29 case / 74 subtest /
  74 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定 worker-side
  network `fetch(...).body` 已加载的 chunk body 可通过
  `respondWith(new Response(body))` 转发给受控 iframe；WPT runner 为
  `fetch/api/resources/trickle.py` 提供确定性本地 fixture，并对缺失的
  Service Worker scope HTML 返回 404 document response。
- ✅ M2-43：Service Worker fetch/message WPT baseline 扩展到
  `service-workers/service-worker/fetch-event-respond-with-response-body-with-invalid-chunk.https.html`；
  fetch-wave 资产清单扩展到 80 asset，runner 双跑 30 case / 75 subtest /
  75 Pass / 0 Fail / 0 Timeout / deterministic true。该切片固定
  `respondWith(new Response(stream))` 遇到非 `Uint8Array` body chunk 时，
  页面 `fetch()` 仍 resolve 为 `Response`，后续
  `response.body.getReader().read()` 以 TypeError reject。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | SW 执行环境架构与独立 runtime | ✅ production browser owner + renderer discovery 真链路 |
| S2 | scriptURL 不下载执行 | ✅ production navigator 经 browser fetch/evaluate |
| S3 | fetch 拦截为零 | 🚧 M2-2 production 页面 fetch respondWith/pass-through 已接入；M2-3/4/5/6 `caches.match()`、`caches.open()`、`Cache.put()`、`Cache.matchAll()`、`Cache.keys()` 与 `ignoreSearch`/`ignoreMethod` 桥接已接入；M2-7 worker-global `fetch()`、SW runtime `Cache.add/addAll` 与 CacheStorage `Response.type` 保真已接入；M2-9 `Cache.delete()` 与 `CacheStorage.delete/has/keys` 已接入；registration-local CacheStorage 持久化已接入；`Response.error()` 可作为 CacheStorage 条目保存/读回，但 FetchEvent 响应结算仍拒绝 status 0；SW fetch/message WPT baseline 已扩展到 request projection + async fetch listener registration + respondWith timing/value validation + synthetic Latin-1 response header over iframe XHR + invalid response header network error + stream body error during body consumption + invalid Blob MIME type not promoted to `Content-Type` + controlled-window `Cache.add()` interception + worker-internal fetch/cache non-self-interception + synthetic custom Response body matrix + `FetchEvent.handled` resolve/reject + same-document iframe navigation interception + intercepted navigation `document.referrer` preservation + controlled client no-fetch-handler CORS/no-cors fallback + controlled client POST body forwarding + `respondWith()` stopImmediatePropagation + throw-after-respondWith iframe navigation + uncontrolled page scope bypass + message-time `clients.claim()` iframe control + claim longest-match boundary + unregister incumbent-controller retention + worker-global fetch prototype placement + historical FetchEvent targetClientId absence + `ExtendableMessageEvent` constructor semantics + ReadableStream pull-source chunk serialization + loaded network body forwarding + non-`Uint8Array` stream chunk body error transfer + ReadableStream start-source full-case（stream/delayed/fetch-stream/cancel/abort 可观察，M2-44）+ page cancel/abort 跨 runtime 反传，31/85 Pass；SW ReadableStream start-source body 已在 runtime/WebView 产品路径固定，partial/incremental stream delivery 仍为 headless 整体下发近似（cancel 反传已覆盖），真分块流式 IPC 通道留后续；SW CacheStorage baseline 扩展到 25/318 Pass，并覆盖 cached `Response.url`、Blob/FileReader、Cache.put cacheability、Cache.addAll duplicate/Vary atomicity、AbortError rejection、no-cors opaque readback、navigation request attributes、credentialed request URL cache keys、bucket-scoped CacheStorage 与 `.any.js` Service Worker global META support；broader fetch/cache/message 基线未完成 |
| S4 | 事件为 setTimeout 模拟 | ✅ manager transition log 为状态源；timer 只执行页面 task 投影 |
| S5 | WPT 覆盖为零 | ✅ core 65/65 case、249/249 Pass、0 Fail/Timeout/Unsupported |

## CI 守护记录（2026-08-22）

**预存失败（不强行修复，记录待后续 SW 轮次）**：webview 集成测试
`service_worker_runtime::update_permissions_follow_calling_worker_state_during_installation`
（M3-25 update permissions + MessagePort）在本地环境约 80% 概率超时失败（20s/60s
deadline 均超时），CI（812a9338，2026-08-21 19:00）通过。诊断矩阵显示卡点：
second（replacement）worker 只处理 awaitInstallEvent（messageSequence=1），callUpdate
消息不被处理，页面侧 nextMessage 永等 → 死锁。

- **根因**（架构时序脆弱性）：worker 侧 `registration.update()` 经 `__zwRequestUpdate`
  同步阻塞等 host 响应；host 的 `manager.poll()` 由页面轮询链
  （`__zw_sw_client_messages` 桥）间接驱动，而页面轮询链在
  `_messageSequence >= _messagePollTarget` 时停止。worker 线程处理延迟时轮询链过早
  停止 → `UpdateRequested` 滞留 host 队列 → worker 永久阻塞 → 死锁。
- **归因**：git bisect 无法归因到特定提交（失败点落在纯文档提交上，同代码多次运行
  PASS/FAIL 交替）——非本次变更引入的确定性回归，是竞态放大（本地环境调度特性
  比 CI 更易触发）。
- **处理**：修复需架构级改动（worker 请求的 host 响应不应依赖页面轮询链驱动），
  超出 CI 守护自主修复范围；不强行修复，等待 SW 专项轮次评估。

**flake 收口（2026-09-06）**：core runner 的
`skip-waiting-using-registration.https.html` 间歇红项（~1/3："Controller state
should be activating expected activating but got activated"）已根因修复（82fcb379f）。
探针实证：创建即受控的 iframe 文档，worker1 的初始 controller 通知被 wire 路径
误判为 change 派发 controllerchange（setTimeout 任务），与测试体
`oncontrollerchange` 赋值竞态——落在赋值后即抢答 saw_controllerchanged 必红。
spec：controller 自创建即定的赋值不是 change。修复：realm 创建期记录
`creationControllerId` 基线，dispatch 决策以 `notifiedId || creationId` 判 change，
初始受控赋值仅记账不派发；claim 场景（创建时无 controller）行为不变。单案 18 连跑
全绿，core 连续三跑 249/249；reftest 100% 无回归。webview 集成测试
`update_permissions_follow_calling_worker_state_during_installation` 本地超时
（上条）仍为待架构级处理项。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 批准方案 C：抽取 Worker 线程核 + SW typed runtime + browser manager owner | ✅ 2026-08-19 用户明确批准 |

## 下一步计划

1. **M2 fetch/cache WPT 扩面**：在首个 fetch/interception baseline、SW CacheStorage
   serviceworker 首片与持久化之后，继续挑选当前可执行的 Service Worker fetch/cache 上游用例，
   扩展 pass-rate evidence；streaming/cancel 专项已于 M2-44 落地（M2-44 见实测基线），
   剩余 broader fetch/cache 上游用例继续按 disposition contract 逐案评估
2. **M3 clients follow-up**：popup/auxiliary 真实 browsing context 创建后接入 browser owner

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 选型 RFC（门控） | ✅ 方案 C 已批准 |
| M1 — 脚本真实执行 + 生命周期真事件 | ✅ current core WPT 234/234 Pass |
| M2 — fetch 拦截 + Cache 集成 | 🚧 M2-2 production fetch respondWith/pass-through 完成；M2-3/4/5/6 `caches.match()`、`caches.open()`、`Cache.put()`、`Cache.matchAll()`、`Cache.keys()`、`ignoreSearch`/`ignoreMethod` 桥接完成；M2-7 worker-global `fetch()`、`Cache.add/addAll`、CacheStorage `Response.type` 保真与 registration-local CacheStorage 持久化完成；`Response.error()` 可作为 CacheStorage 条目保存/读回，FetchEvent 响应结算仍拒绝 status 0；SW fetch/message WPT baseline 已扩展到 request projection + respondWith timing/value validation + synthetic Latin-1 response header over iframe XHR + async fetch listener registration + stream body error during body consumption + invalid Blob MIME type not promoted to `Content-Type` + controlled-window `Cache.add()` interception + worker-internal fetch/cache non-self-interception + synthetic custom Response body matrix + `FetchEvent.handled` resolve/reject + same-document iframe navigation interception + intercepted navigation `document.referrer` preservation + controlled client no-fetch-handler CORS/no-cors fallback + controlled client POST body forwarding + `respondWith()` stopImmediatePropagation + throw-after-respondWith iframe navigation + uncontrolled page scope bypass + message-time `clients.claim()` iframe control + claim longest-match boundary + unregister incumbent-controller retention + worker-global fetch prototype placement + historical FetchEvent targetClientId absence + `ExtendableMessageEvent` constructor semantics + ReadableStream pull-source chunk serialization + loaded network body forwarding + non-`Uint8Array` stream chunk body error transfer + ReadableStream start-source full-case 与 page cancel/abort 反传（M2-44，settle 语义对齐 spec：respondWith fulfill 即回应 fetch），31/85 Pass；SW ReadableStream start-source body 已固定；SW CacheStorage baseline 25/318 Pass；broader fetch/cache/message 基线继续 |
| M3 — 控制语义 + 消息 + 收尾 | 🚧 classic startup graph + 控制/消息/update/persistence 完成 |

## 验证基线

- 测试基线：storage crate 既有单测全绿（立项时点）；clippy 零警告
- WPT service-workers 面：当前 core runner 65 case / 249 subtest 全绿，fetch/message runner
  31 case / 85 subtest 全绿，CacheStorage runner 25 case / 318 subtest 全绿；
  上游完整分母 294 个 testharness 源 / 331 URL，
  正文覆盖 294/294；分层与依赖信号见
  [M0 WPT evidence](evidence/2026-08-19-m0-wpt-executable-surface.md)，逐文件机器清单见
  [WPT case inventory](evidence/2026-08-19-m0-wpt-case-inventory.tsv)，候选 8/3/1 裁决见
  [M1 candidate closure](evidence/2026-08-19-m0-m1-candidate-resource-closure.md)，Tier A
  + active asset corpus 9 case / 30 subtest / 19 asset 见
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
  core lane、runner 导入账本与多批 case asset 的双向闭包
- Tier A 资产恢复：`make fetch-wpt-service-workers-tier-a`；默认使用独立
  `wpt-data/.service-workers-tier-a-root`，当前环境 19/19 blob SHA 验证通过
- Tier A 资产审计：`make audit-wpt-service-workers-tier-a`（无网络、只读）；
  `make test-wpt-service-workers-tier-a-assets` 覆盖缺失/篡改/修复回归
- Next-wave 资产恢复/审计：`make fetch-wpt-service-workers-next-wave` /
  `make audit-wpt-service-workers-next-wave`；与 Tier A 复用独立数据根，当前 7/7 通过；
  `make test-wpt-service-workers-next-wave-assets` 固化篡改/修复回归
- Static-wave 资产恢复/审计：`make fetch-wpt-service-workers-static-wave` /
  `make audit-wpt-service-workers-static-wave`；4 assets / 5 subtest；
  `make test-wpt-service-workers-static-wave-assets` 固化篡改/修复回归
- Worker secure-context wave 资产恢复/审计：`make fetch-wpt-service-workers-worker-secure-context-wave` /
  `make audit-wpt-service-workers-worker-secure-context-wave`；5 assets / 2 subtest；
  `make test-wpt-service-workers-worker-secure-context-wave-assets` 固化篡改/修复回归
- InstallEvent type wave 资产恢复/审计：`make fetch-wpt-service-workers-install-event-type-wave` /
  `make audit-wpt-service-workers-install-event-type-wave`；7 assets / 1 subtest；
  `make test-wpt-service-workers-install-event-type-wave-assets` 固化篡改/修复回归
- Worker close wave 资产恢复/审计：`make fetch-wpt-service-workers-worker-close-wave` /
  `make audit-wpt-service-workers-worker-close-wave`；6 assets / 2 subtest；
  `make test-wpt-service-workers-worker-close-wave-assets` 固化篡改/修复回归
- Worker interface wave 资产恢复/审计：`make fetch-wpt-service-workers-worker-interface-wave` /
  `make audit-wpt-service-workers-worker-interface-wave`；5 assets / 4 subtest；
  `make test-wpt-service-workers-worker-interface-wave-assets` 固化篡改/修复回归
- FetchEvent historical wave 资产恢复/审计：`make fetch-wpt-service-workers-fetch-event-historical-wave` /
  `make audit-wpt-service-workers-fetch-event-historical-wave`；3 assets / 2 subtest；
  `make test-wpt-service-workers-fetch-event-historical-wave-assets` 固化篡改/修复回归
- Update-wave 资产恢复/审计：`make fetch-wpt-service-workers-update-wave` /
  `make audit-wpt-service-workers-update-wave`；5 assets / 1 subtest；
  `make test-wpt-service-workers-update-wave-assets` 固化篡改/修复回归
- Import-response-wave 资产恢复/审计：5 assets / 24 subtest；
  `make test-wpt-service-workers-import-response-wave-assets` 固化篡改/修复回归
- Import-dynamic-wave 资产恢复/审计：11 assets / 5 subtest；
  `make test-wpt-service-workers-import-dynamic-wave-assets` 固化篡改/修复回归
- Import-event-wave 资产恢复/审计：3 assets / 5 subtest；
  `make test-wpt-service-workers-import-event-wave-assets` 固化篡改/修复回归
- Fetch-wave 资产恢复/审计：70 assets / 71 subtest；
  `make test-wpt-service-workers-fetch-wave-assets` 固化篡改/修复回归
- CacheStorage serviceworker-wave 资产恢复/审计：52 assets / 318 subtest；
  `make test-wpt-service-workers-cache-storage-wave-assets` 固化篡改/修复回归
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
- M2-3 `caches.match()`：runtime/manager/protocol/browser/renderer typed bridge 与
  `respondWith(caches.match(event.request))` 命中路径见
  [M2 cache match](evidence/2026-08-22-m2-cache-match.md)
- M2-4 CacheStorage 写入：Service Worker runtime/IPC/manager/browser owner 的
  `caches.open()` + `Cache.put()` + `Cache.match()` 写入读取链见
  [M2 CacheStorage write bridge](evidence/2026-08-22-m2-cache-storage-write.md)
- M2-5 Cache list operations：Service Worker runtime/IPC/manager/browser owner 的
  `Cache.matchAll()` + `Cache.keys()` 数组结果链见
  [M2 Service Worker Cache.matchAll and Cache.keys](evidence/2026-08-22-m2-cache-matchall-keys.md)
- M2-6 CacheQueryOptions：Service Worker runtime/IPC/manager/browser owner 的
  `ignoreSearch`/`ignoreMethod` 查询选项接线见
  [M2 Service Worker CacheQueryOptions](evidence/2026-08-22-m2-cache-query-options.md)
- M2-7 worker fetch + Cache.add/addAll：worker-global `fetch()`、typed host bridge、
  browser/WebView 网络接线与 SW runtime add/addAll 链路见
  [M2 Service Worker worker fetch and cache add](evidence/2026-08-22-m2-worker-fetch-cache-add.md)
- M2 Cache response type guard：`Response.error()`、runtime/IPC response type 透传、
  CacheStorage 与 FetchEvent 分离校验见
  [M2 Service Worker Cache Response Type Guard](evidence/2026-08-22-m2-cache-response-type-error.md)
- M2 fetch/interception WPT baseline：`request-end-to-end.https.html` +
  `fetch-event-add-async.https.html` +
  `fetch-event-async-respond-with.https.html` +
  `fetch-event-network-error.https.html` +
  `fetch-event-respond-with-argument.https.html` +
  `iso-latin1-header.https.html` +
  `fetch-event-within-sw.https.html` +
  `fetch-event-respond-with-custom-response.https.html` +
  `fetch-event-handled.https.html` +
  `fetch-event-after-navigation-within-page.https.html` +
  `fetch-event-respond-with-stops-propagation.https.html` +
  `uncontrolled-page.https.html` +
  `claim-fetch.https.html` +
  `claim-not-using-registration.https.html` +
  `claim-using-registration.https.html` +
  `unregister-controller.https.html` +
  `fetch-event-throws-after-respond-with.https.html` +
  `fetch-on-the-right-interface.https.any.js` +
  `historical.https.any.js` +
  `intercepted-referrer.https.html` +
  `controller-with-no-fetch-event-handler.https.html` +
  `fetch-with-body.https.html` +
  `invalid-header.https.html` +
  `invalid-blobtype.https.html` +
  `fetch-event-respond-with-readable-stream-chunk.https.html` +
  `ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html` +
  `ServiceWorkerGlobalScope/postmessage.https.html` +
  `fetch-event-respond-with-body-loaded-in-chunk.https.html` 独立 runner、
  资产清单与 29/74
  deterministic baseline 见
  [Service Worker Fetch WPT Baseline](evidence/2026-09-02-m2-fetch-body-loaded-in-chunk-baseline.md)
- M2-22 fetch custom-response 定向验证：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_respond_with_serializes_buffer_source_and_form_data_response -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_response_body_used_redirect_and_blob_formdata_cache_put_support -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- make test-wpt-service-workers-fetch-wave-assets`：25 assets / regression PASS
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner service_worker_fetch -- --nocapture`：2 passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-custom-response-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-custom-response-baseline.md`：8 cases / 23 subtests / 23 Pass，double-run deterministic
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo fmt --all -- --check`：passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- cargo clippy --workspace --all-targets -- -D warnings`：passed
  - `CARGO_BUILD_JOBS=1 ./target/test-guard --per-proc-mem 4 --total-mem 20 --time-limit 1800 -- make test`：passed
- M2-23 fetch stops-propagation baseline：
  [Service Worker Fetch WPT Baseline](evidence/2026-08-23-m2-fetch-stops-propagation-baseline.md)
- M2-23 fetch stops-propagation 定向验证：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_respond_with_stops_later_listeners -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-engine test_iframe_service_worker_controller_post_message_transfers_object_port -- --nocapture`：1 passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-service-workers-fetch FILTER=fetch-event-respond-with-stops-propagation.https.html`：1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- make test-wpt-service-workers-fetch-wave-assets`：27 assets / regression PASS
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-wpt-runner service_worker_fetch -- --nocapture`：2 passed
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-stops-propagation-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-stops-propagation-baseline.md`：9 cases / 24 subtests / 24 Pass，double-run deterministic
- M2-25 fetch uncontrolled-page baseline：
  - 新增 WPT：`service-workers/service-worker/uncontrolled-page.https.html`
  - 新增 support：`service-workers/service-worker/resources/fail-on-fetch-worker.js`、`worker-testharness.js`、`simple.txt`
  - `make testharness-service-workers-fetch FILTER=uncontrolled-page.https.html`：1 Pass
  - `make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：10 cases / 25 subtests / 25 Pass，double-run deterministic
- M2-26 fetch claim-fetch baseline：
  - 新增 WPT：`service-workers/service-worker/claim-fetch.https.html`
  - 新增 support：`service-workers/service-worker/resources/claim-worker.js`、`blank.html`
  - `WPT_SOURCE=$HOME/github/others/wpt make testharness-service-workers-fetch FILTER=claim-fetch.https.html`：1 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：11 cases / 26 subtests / 26 Pass，double-run deterministic
- M2-27 fetch claim registration-boundary baseline：
  - 新增 WPT：`service-workers/service-worker/claim-not-using-registration.https.html`
  - 新增 support：`service-workers/service-worker/resources/empty.js`、`empty-worker.js`
  - `WPT_SOURCE=$HOME/github/others/wpt make testharness-service-workers-fetch FILTER=claim-not-using-registration.https.html`：2 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：12 cases / 28 subtests / 28 Pass，double-run deterministic
- M2-28 fetch claim active-state baseline：
  - 新增 WPT：`service-workers/service-worker/claim-using-registration.https.html`
  - 复用 support：`service-workers/service-worker/resources/claim-worker.js`、`empty.js`、`blank.html`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-service-workers-fetch FILTER=claim-using-registration.https.html`：2 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：13 cases / 30 subtests / 30 Pass，double-run deterministic
- M2-29 fetch unregister-controller baseline：
  - 新增 WPT：`service-workers/service-worker/unregister-controller.https.html`
  - 新增 support：`service-workers/service-worker/resources/unregister-controller-page.html`、`simple-intercept-worker.js`
  - 复用 support：`service-workers/service-worker/resources/simple.txt`
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make testharness-service-workers-fetch FILTER=unregister-controller.https.html`：3 Pass
  - `WPT_SOURCE=$HOME/github/others/wpt ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-23-m2-fetch-baseline.md`：14 cases / 33 subtests / 33 Pass，double-run deterministic
- M2-30 fetch throw-after-respondWith iframe baseline：
  - 新增 WPT：`service-workers/service-worker/fetch-event-throws-after-respond-with.https.html`
  - 新增 support：`service-workers/service-worker/resources/respond-then-throw-worker.js`
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-webview controlled_iframe_fetch_waits_for_message_port_backed_response -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- cargo run -p zero-wpt-runner -- testharness-service-workers-fetch --wpt-data "$HOME/github/others/wpt" fetch-event-throws-after-respond-with.https.html`：1 Pass
  - 完整 fetch baseline：15 cases / 34 subtests / 34 Pass
- M2 fetch respondWith-after-throw runtime guard：已提交 response promise 不被后续同步 throw
  覆盖，见
  [SW fetch throw after respondWith guard](evidence/2026-08-22-m2-fetch-throw-after-respond-with.md)
- storage-cache-api shared Cache response type readback：CacheStorage 专属 `__zwcr:` wire、
  page `Response.type` / `clone().type` 保真与 host type validation 见
  [M2 Cache Response Type Readback](../storage-cache-api/evidence/2026-08-22-m2-cache-response-type-readback.md)
- storage-cache-api CacheStorage window WPT 扩面：37 case / 439 subtest 全绿，并校正
  `Response.error()` 可作为 CacheStorage 条目保存/读回、FetchEvent 响应结算仍拒绝 status 0
  的共享边界，以及 `Cache.match()` 对 `Response.url`、fetched MIME、cross-host fixture 和
  opaque response Vary 匹配、`Cache.put()` body consumption、opaque 内部 206 / `Vary: *`、
  `Response.redirect()`、Blob/FormData response body、body-less request consumption、
  `Cache.addAll()` undefined entry 拒绝、Vary-aware duplicate 判定和 Window/Dedicated
  Worker/nested Dedicated Worker 共享 CacheStorage owner、filtered response 类型矩阵、
  sandboxed iframe CacheStorage 安全边界与 top-level credentialed request URL cache key
  的页面侧语义，见
  [M2 CacheStorage Window WPT Expansion](../storage-cache-api/evidence/2026-08-22-m2-cache-window-expansion.md)
  、[M2 Cache.add WPT Expansion](../storage-cache-api/evidence/2026-08-22-m2-cache-add-wpt-expansion.md)
  、[M2 CacheStorage Worker Sharing WPT Expansion](../storage-cache-api/evidence/2026-08-22-m2-cache-worker-sharing-wpt-expansion.md)
  和 [M2 CacheStorage Nested Worker WPT Expansion](../storage-cache-api/evidence/2026-08-22-m2-cache-nested-worker-wpt-expansion.md)
- M2-37 fetch invalid-header baseline：
  [Service Worker Fetch WPT Baseline](evidence/2026-08-31-m2-fetch-invalid-header-baseline.md)
- M2-37 fetch invalid-header 定向验证：
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-protocol service_worker_fetch_response_rejects_invalid_header_fields`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-script-sandbox fetch_event_rejects_invalid_response_header_value --no-default-features --features quickjs`：1 passed
  - `WPT_ASSET_MANIFEST=$PWD/docs/goal/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv WPT_EXPECTED_ASSET_COUNT=61 WPT_CORPUS_LABEL="Service Worker fetch wave" ./tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only`：61 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-wpt-runner service_worker_fetch_manifest_has_request_end_to_end_case`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-wpt-runner service_worker_fetch_runner_reports_every_case_when_harness_is_missing`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo run -p zero-wpt-runner -- testharness-service-workers-fetch invalid-header --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`：1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-31-m2-fetch-invalid-header-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-31-m2-fetch-invalid-header-baseline.md`：23 cases / 56 subtests / 56 Pass，double-run deterministic
- M2-38 fetch invalid-blobtype baseline：
  [Service Worker Fetch WPT Baseline](evidence/2026-08-31-m2-fetch-invalid-blobtype-baseline.md)
  - `WPT_ASSET_MANIFEST=$PWD/docs/goal/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv WPT_EXPECTED_ASSET_COUNT=64 WPT_CORPUS_LABEL="Service Worker fetch wave" ./tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only`：64 assets matched pinned manifest
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-wpt-runner service_worker_fetch_manifest_has_request_end_to_end_case`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-wpt-runner service_worker_fetch_runner_reports_every_case_when_harness_is_missing`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo run -p zero-wpt-runner -- testharness-service-workers-fetch invalid-blobtype --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`：1 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-31-m2-fetch-invalid-blobtype-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-31-m2-fetch-invalid-blobtype-baseline.md`：24 cases / 57 subtests / 57 Pass，double-run deterministic
- M3-38 ExtendableMessageEvent constructor baseline：
  [Service Worker Fetch WPT Baseline](evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo test -p zero-script-sandbox extendable_message_event --no-default-features --features quickjs -- --nocapture`：2 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- env BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo run -p zero-wpt-runner -- testharness-service-workers-fetch extendable-message-event-constructor --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json`：10 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 420 -- make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md`：25 cases / 67 subtests / 67 Pass，double-run deterministic
- M2-39 fetch stream body error baseline：
  [Service Worker Fetch WPT Baseline](evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox fetch_event_readable_stream_body_error_is_serialized -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- ./target/release/zero-wpt-runner testharness-service-workers-fetch --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root fetch-error.https.html --json`：3 Pass
  - `make audit-wpt-service-workers-fetch-wave`：68 assets matched pinned manifest
  - `make baseline-wpt-service-workers-fetch OUTPUT=docs/goal/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-08-31-m3-extendable-message-event-constructor-baseline.md`：26 cases / 70 subtests / 70 Pass，double-run deterministic
- M2-41 fetch readable-stream chunk baseline：
  [ReadableStream chunk WPT evidence](evidence/2026-09-02-m2-fetch-readable-stream-chunk.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers-fetch --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root fetch-event-respond-with-readable-stream-chunk.https.html --json`：1 Pass
  - `make audit-wpt-service-workers-fetch-wave`：70 assets matched pinned manifest
  - `make baseline-wpt-service-workers-fetch`：27 cases / 71 subtests / 71 Pass，double-run deterministic
- M3-53 postMessage worker ports baseline：
  [PostMessage worker ports evidence](evidence/2026-09-02-m3-postmessage-worker-ports.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers-fetch --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root ServiceWorkerGlobalScope/postmessage.https.html --json`：2 Pass
  - `make audit-wpt-service-workers-fetch-wave`：74 assets matched pinned manifest
  - `make baseline-wpt-service-workers-fetch`：28 cases / 73 subtests / 73 Pass，double-run deterministic
- M3-54 registration attribute baseline：
  [Registration attribute evidence](evidence/2026-09-02-m3-registration-attribute.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 180 -- cargo run -p zero-wpt-runner -- testharness-service-workers registration-attribute --wpt-data /tmp/zw-wpt-cache-storage --json`：2 Pass
  - `make audit-wpt-service-workers-registration-attribute-wave`：7 assets matched pinned manifest
  - `make testharness-service-workers-core FILTER=ServiceWorkerGlobalScope/registration-attribute TIME_LIMIT=300`：2 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-02-m3-registration-attribute-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-02-m3-registration-attribute-baseline.md TIME_LIMIT=1200`：52 cases / 200 subtests / 200 Pass，double-run deterministic
- M3-58 getRegistration baseline：
  [getRegistration evidence](evidence/2026-09-03-m3-getregistration.md)
  - `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview get_registration_rejects_cross_origin_document_url -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root getregistration --json`：6 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-getregistration-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-getregistration-baseline.md TIME_LIMIT=1200`：56 cases / 209 subtests / 209 Pass，double-run deterministic
- M3-59 registration iframe baseline：
  [registration iframe evidence](evidence/2026-09-03-m3-registration-iframe.md)
  - `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-webview iframe_register_resolves_with_installing_worker -- --nocapture`：1 passed
  - `make testharness-service-workers-core FILTER=registration-iframe TIME_LIMIT=300`：3 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-registration-iframe-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-registration-iframe-baseline.md TIME_LIMIT=1200`：57 cases / 212 subtests / 212 Pass，double-run deterministic
- M3-60 installing baseline：
  [installing evidence](evidence/2026-09-03-m3-installing.md)
  - `make testharness-service-workers-core FILTER=installing TIME_LIMIT=300`：2 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-installing-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-installing-baseline.md TIME_LIMIT=1200`：58 cases / 214 subtests / 214 Pass，double-run deterministic
- M3-61 waiting baseline：
  [waiting evidence](evidence/2026-09-03-m3-waiting.md)
  - `make testharness-service-workers-core FILTER=service-worker/waiting.https.html TIME_LIMIT=300`：2 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-waiting-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-waiting-baseline.md TIME_LIMIT=1200`：59 cases / 216 subtests / 216 Pass，double-run deterministic
- M3-62 controller-on-disconnect baseline：
  [controller-on-disconnect evidence](evidence/2026-09-03-m3-controller-on-disconnect.md)
  - `make testharness-service-workers-core FILTER=service-worker/controller-on-disconnect.https.html TIME_LIMIT=300`：1 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-baseline.md TIME_LIMIT=1200`：60 cases / 217 subtests / 217 Pass，deterministic
- M3-63 oninstall script error baseline：
  [oninstall-script-error evidence](evidence/2026-09-03-m3-oninstall-script-error.md)
  - `make testharness-service-workers-core FILTER=service-worker/oninstall-script-error.https.html TIME_LIMIT=300`：6 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-oninstall-script-error-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-oninstall-script-error-baseline.md TIME_LIMIT=1200`：61 cases / 223 subtests / 223 Pass，deterministic
- M3-64 onactivate script error baseline：
  [onactivate-script-error evidence](evidence/2026-09-03-m3-onactivate-script-error.md)
  - `make testharness-service-workers-core FILTER=service-worker/onactivate-script-error.https.html TIME_LIMIT=300`：5 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-onactivate-script-error-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-onactivate-script-error-baseline.md TIME_LIMIT=1200`：62 cases / 228 subtests / 228 Pass，deterministic
- M3-65 ExtendableEvent waitUntil baseline：
  [ExtendableEvent waitUntil evidence](evidence/2026-09-03-m3-extendable-event-waituntil.md)
  - `make testharness-service-workers-core FILTER=service-worker/extendable-event-waituntil.https.html TIME_LIMIT=300`：6 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-baseline.md TIME_LIMIT=1200`：63 cases / 234 subtests / 234 Pass，deterministic
- M3-66 ExtendableEvent async waitUntil baseline：
  [ExtendableEvent async waitUntil evidence](evidence/2026-09-03-m3-extendable-event-async-waituntil.md)
  - `make test-wpt-service-workers-extendable-event-async-waituntil-wave-assets`：2 assets / regression PASS
  - `make testharness-service-workers-core FILTER=service-worker/extendable-event-waituntil.https.html TIME_LIMIT=300`：6 Pass
  - `make testharness-service-workers-core FILTER=service-worker/extendable-event-async-waituntil.https.html TIME_LIMIT=300`：14 Pass
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-baseline.json --summary docs/goal/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-baseline.md`：64 cases / 248 subtests / 248 Pass，deterministic
- M3-67 controller-on-reload baseline：
  [controller-on-reload evidence](evidence/2026-09-03-m3-controller-on-reload.md)
  - `BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-webview iframe_reload_observes_active_service_worker_controller -- --nocapture`：1 passed
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root controller-on-reload.https.html --json`：1 Pass
  - `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-09-03-m3-controller-on-reload-baseline.json SUMMARY=docs/goal/service-workers/evidence/2026-09-03-m3-controller-on-reload-baseline.md TIME_LIMIT=1200`：65 cases / 249 subtests / 249 Pass，deterministic
- M3-55 worker error event baseline：
  [Worker ErrorEvent evidence](evidence/2026-09-03-m3-worker-error-event.md)
  - `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox service_worker::tests::page_message_error_listener_observes_thrown_error_event -- --nocapture`：1 passed
  - `make audit-wpt-service-workers-next-wave`：9 assets matched pinned manifest
  - `make testharness-service-workers-core FILTER=service-worker-error-event TIME_LIMIT=300`：1 Pass
- M3 registration-local CacheStorage persistence：active registration `CacheStorage` snapshot/
  restore、normal profile mutation dirtying 与 owner 重建读回见
  [M3 Service Worker CacheStorage Persistence](evidence/2026-08-22-m3-registration-cache-storage-persistence.md)
- M2 Service Worker CacheStorage WPT baseline：12 个 serviceworker CacheStorage
  wrapper 加 2 个顶层 CacheStorage 页面和 11 个 `.any.js` Service Worker global variants，
  资产清单与 25/318 deterministic baseline 见
  [Service Worker CacheStorage WPT Baseline](evidence/2026-08-23-m2-cache-storage-serviceworker-baseline.md)
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
- M3-35 active controller-state：`registration.active` activating 可见性、未控制 iframe
  初始 controller null 与同窗口 active object identity 见
  [M1 core WPT baseline](evidence/2026-08-19-m1-wpt-core-baseline.md)
- M3-36 skipWaiting uncontrolled-client：未受控 iframe 上 `skipWaiting()` 不改变该
  iframe controller，见
  [M1 core WPT baseline](evidence/2026-08-19-m1-wpt-core-baseline.md)
- M3-37 skipWaiting controlled-client：受控 iframe 在 replacement 激活期间观测
  `controllerchange`，事件期 controller snapshot 保持 `activating`，并完成
  worker-testharness 结果通道，见
  [M3 skipWaiting controlled-client](evidence/2026-08-24-m3-skip-waiting-controlled.md)
- M3-48 Service Worker global self identity：worker global `self.serviceWorker` 只读投影、
  install/activate 事件期 registration slot 与启动期自消息见
  [M3 global serviceworker](evidence/2026-09-02-m3-global-serviceworker.md)
- M3-49 Service Worker immutable prototype：worker global prototype chain 的
  `Object.setPrototypeOf()` / `Reflect.setPrototypeOf()` 不可变语义见
  [M3 immutable prototype](evidence/2026-09-02-m3-immutable-prototype.md)
- M3-51 Service Worker message event ports：worker-side `MessageEvent.ports` getter
  identity 语义见
  [M3 message event ports](evidence/2026-09-02-m3-message-event-ports.md)
- M3-52 Service Worker ExtendableMessageEvent：page/nested client source 投影与
  active/waiting worker 互发消息见
  [M3 ExtendableMessageEvent](evidence/2026-09-02-m3-extendable-message-event.md)
- M2-1 fetch runtime foundation：runtime `FetchEvent`/`Request`/`Response` MVP、
  manager longest-scope dispatch、IPC command/event 与定向验证见
  [M2 fetch runtime foundation](evidence/2026-08-21-m2-fetch-runtime-foundation.md)
- M2-9 worker Cache delete/listing：`Cache.delete()` 与 `CacheStorage.delete()/has()/keys()`
  typed bridge、renderer/browser IPC 和 manager-owned registration store 见
  [M2 worker Cache delete/listing](evidence/2026-08-22-m2-worker-cache-delete-listing.md)

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
| 2026-08-23 | M3-35 active controller-state | `active.https.html` 纳入 core runner；registration.active / iframe controller null / active object identity；core 35/158 |
| 2026-08-24 | M3-36 skipWaiting uncontrolled-client | `skip-waiting-without-using-registration.https.html` 纳入 core runner；未受控 iframe controller 保持 null；core 36/160 |
| 2026-08-24 | M3-37 skipWaiting controlled-client | `skip-waiting-using-registration.https.html` 纳入 core runner；受控 iframe replacement controllerchange / activating snapshot / worker-testharness result channel；core 37/162 |
| 2026-09-01 | M3-38 registration lifecycle WPT | `registration-events.https.html` 与 `registration-end-to-end.https.html` 单 case 通过；message lifecycle asset manifest 7 asset fail-closed；完整 core runner 仍被 `registration-updateviacache.https.html` 挂起阻挡 |
| 2026-09-02 | M3-39 updateViaCache hang closure | `registration-updateviacache.https.html` 25/25 Pass；补齐 `empty-worker.js` asset manifest；注销后 runtime live-count 与页面 registration poll 停止回归 |
| 2026-09-02 | M3-40 registration lifecycle core promotion | `registration-events.https.html` 与 `registration-end-to-end.https.html` 纳入 core runner；`make baseline-wpt-service-workers-core` 双跑 39/164 deterministic Pass |
| 2026-09-02 | M3-41 worker secure context core promotion | `ServiceWorkerGlobalScope/isSecureContext.https.html` 纳入 core runner；`WorkerGlobalScope.prototype.isSecureContext === true`；`make baseline-wpt-service-workers-core` 双跑 40/166 deterministic Pass |
| 2026-09-02 | M3-42 install event type core promotion | `install-event-type.https.html` 纳入 core runner；install event `InstallEvent`/`ExtendableEvent` identity 与基础 event flags；`make baseline-wpt-service-workers-core` 双跑 41/167 deterministic Pass |
| 2026-09-02 | M3-43 worker close absence core promotion | `ServiceWorkerGlobalScope/close.https.html` 纳入 core runner；SW global 不暴露 `close()`；`make baseline-wpt-service-workers-core` 双跑 42/169 deterministic Pass |
| 2026-09-02 | M3-44 worker interface requirements core promotion | `interface-requirements-sw.https.html` 纳入 core runner；`FetchEventInit.request` required member 校验与 worker global negative surface；`make baseline-wpt-service-workers-core` 双跑 43/173 deterministic Pass |
| 2026-09-02 | M3-45 FetchEvent historical interface core promotion | `historical.https.any.js` 纳入 core runner；确认 `FetchEvent.prototype.targetClientId` 不暴露；`make baseline-wpt-service-workers-core` 双跑 44/175 deterministic Pass |
| 2026-09-02 | M3-46 classic no-dynamic-import core promotion | `no-dynamic-import.any.js` 纳入 core runner；classic Service Worker global 动态 import rejected promise 语义；`make baseline-wpt-service-workers-core` 双跑 45/176 deterministic Pass |
| 2026-09-02 | M3-47 module no-dynamic-import core promotion | `no-dynamic-import-in-module.any.js` 纳入 core runner；`serviceworker-module` runner 与 module/classic 动态 import TypeError rejected promise 语义；`make baseline-wpt-service-workers-core` 双跑 46/183 deterministic Pass |
| 2026-09-02 | M3-48 global serviceworker core promotion | `global-serviceworker.https.any.js` 纳入 core runner；`self.serviceWorker` 只读 identity、install/activate registration slot 与启动期自消息通过；`make baseline-wpt-service-workers-core` 双跑 47/188 deterministic Pass |
| 2026-09-02 | M3-49 immutable prototype core promotion | `immutable-prototype-serviceworker.https.html` 纳入 core runner；worker global prototype chain 不可变语义通过；`make baseline-wpt-service-workers-core` 双跑 48/189 deterministic Pass |
| 2026-09-02 | M3-50 worker-global unregister core promotion | `ServiceWorkerGlobalScope/unregister.https.html` 纳入 core runner；worker-global `registration.unregister()` 在 evaluation/install/activate/message 场景通过；`make baseline-wpt-service-workers-core` 双跑 49/193 deterministic Pass |
| 2026-09-02 | M3-51 message event ports core promotion | `ServiceWorkerGlobalScope/message-event-ports.https.html` 纳入 core runner；worker-side `MessageEvent.ports` getter identity 语义通过；`make baseline-wpt-service-workers-core` 双跑 50/194 deterministic Pass |
| 2026-09-02 | M3-52 extendable message event core promotion | `ServiceWorkerGlobalScope/extendable-message-event.https.html` 纳入 core runner；`ExtendableMessageEvent` page/nested client source、worker loopback 与 active/waiting worker message 语义通过；单 case 4/4 Pass |
| 2026-09-02 | M3-53 postMessage worker ports fetch/message promotion | `ServiceWorkerGlobalScope/postmessage.https.html` 纳入 fetch/message runner；worker self-loopback 与 active/waiting transferred `MessagePort` 回信通过；`make baseline-wpt-service-workers-fetch` 双跑 28/73 deterministic Pass |
| 2026-09-02 | M3-54 registration attribute core promotion | `ServiceWorkerGlobalScope/registration-attribute.https.html` 纳入 core runner；worker global registration scope/slot/EventTarget 与 lifecycle event ordering 通过；`make baseline-wpt-service-workers-core` 双跑 52/200 deterministic Pass |
| 2026-09-03 | M3-55 worker error event core promotion | `ServiceWorkerGlobalScope/service-worker-error-event.https.html` 纳入 core runner；message handler 抛错后 worker-global `ErrorEvent` 字段和 `WindowClient` source 回传通过；单 case 1/1 Pass |
| 2026-09-03 | M3-56 messageerror event core promotion | `ServiceWorkerGlobalScope/error-message-event.https.html` 纳入 core runner；page `canvas.captureStream()` track transfer 触发 worker-global `messageerror` 并保留 `WindowClient` source；`make baseline-wpt-service-workers-core` 双跑 54/202 deterministic Pass |
| 2026-09-03 | M3-57 controller on load core promotion | `controller-on-load.https.html` 纳入 core runner；iframe 默认 `getRegistration()` 按 iframe document URL 查询，iframe registration worker slots 使用 iframe realm wrapper，load 时 controller identity 通过；单 case 1/1 Pass |
| 2026-09-03 | M3-58 getRegistration core promotion | `getregistration.https.html` 纳入 core runner；跨源 document URL `SecurityError`、fragment 忽略匹配与 controlled iframe 注销后查询通过；单 case 6/6 Pass |
| 2026-09-03 | M3-59 registration iframe core promotion | `registration-iframe.https.html` 纳入 core runner；iframe `register()` 使用 iframe document URL 解析 scriptURL/scope，resolved registration 立即暴露 iframe realm `installing` worker；单 case 3/3 Pass |
| 2026-09-03 | M3-60 installing core promotion | `installing.https.html` 纳入 core runner；top-level registration installing slot visibility 与 `getRegistration(scope)` SameObject identity 通过；单 case 2/2 Pass |
| 2026-09-03 | M3-61 waiting core promotion | `waiting.https.html` 纳入 core runner；top-level/iframe registration waiting slot visibility、未激活 iframe controller nullability 与 waiting/active SameObject identity 通过；单 case 2/2 Pass |
| 2026-09-03 | M3-62 controller disconnect core promotion | `controller-on-disconnect.https.html` 纳入 core runner；受控 iframe detach 后 controller 清空；单 case 1/1 Pass |
| 2026-09-03 | M3-63 oninstall script error core promotion | `oninstall-script-error.https.html` 纳入 core runner；install listener 同步抛错触发 worker-global `error` 但 install 成功，`waitUntil()` rejection 仍失败；单 case 6/6 Pass |
| 2026-09-03 | M3-64 onactivate script error core promotion | `onactivate-script-error.https.html` 纳入 core runner；activate listener 同步抛错触发 worker-global `error` 但 activate 成功；单 case 5/5 Pass |
| 2026-09-03 | M3-65 ExtendableEvent waitUntil core promotion | `extendable-event-waituntil.https.html` 纳入 core runner；生命周期 waitUntil 等待全部 promises settle，activate waitUntil rejection 不阻断 activated；单 case 6/6 Pass |
| 2026-09-03 | M3-66 ExtendableEvent async waitUntil core promotion | `extendable-event-async-waituntil.https.html` 纳入 core runner；async waitUntil task/microtask eligibility 与 respondWith lifetime extension 边界通过；`make baseline-wpt-service-workers-core` 双跑 64/248 deterministic Pass |
| 2026-09-03 | M3-67 controller on reload core promotion | `controller-on-reload.https.html` 纳入 core runner；iframe reload 释放旧 nested window client，新 document 获得 active controller 且 iframe realm ServiceWorker identity 正确；`make baseline-wpt-service-workers-core` 双跑 65/249 deterministic Pass |
| 2026-09-06 | M3-68 cache-put getReader disturb 修复 | worker `ReadableStream` 加锁 + `Cache.put()` disturb body + `_bodyNull` 标记；回归测试双引擎通过；cache-storage runner 316→318 全绿（潜伏红项归因：50245f856 body getter 加入晚于 09-02 baseline，未复跑该 runner） |
| 2026-09-06 | M3-69 extendable-message-event wave manifest 闭包 | 补登记 2 个漏登记 worker fixture（4→6 asset）；core runner 244+3 红→249/249 全绿；本地全 wave asset 修复与三 runner 复测完成（core 249 / fetch 75 / cache-storage 318 全 Pass） |
| 2026-09-06 | M2-44 streaming/cancel 专项 | `fetch-event-respond-with-readable-stream.https.html` 全 10 subtest 纳入 fetch/message runner；cancel/abort 反传链 page shim → webview 回调 → manager → runtime `CancelFetchBody` → stream cancel 回调；settle 语义对齐 spec（respondWith fulfill 即回应）；fetch runner 30/75→31/85 双跑 deterministic；新增 `cancel_fetch_body_invokes_stream_source_cancel` 单测 |
| 2026-09-02 | M2 fetch body-loaded-in-chunk baseline | `fetch-event-respond-with-body-loaded-in-chunk.https.html` 纳入 fetch/message runner；worker-side loaded chunk body 经 `respondWith(new Response(body))` 转发到受控 iframe；`make baseline-wpt-service-workers-fetch` 双跑 29/74 deterministic Pass |
| 2026-09-02 | M2 fetch invalid stream chunk baseline | `fetch-event-respond-with-response-body-with-invalid-chunk.https.html` 纳入 fetch/message runner；非 `Uint8Array` stream chunk 通过 page-side `response.body` reader 以 TypeError reject；`make baseline-wpt-service-workers-fetch` 双跑 30/75 deterministic Pass |
| 2026-09-02 | M2 fetch readable-stream fetch-body prework | SW runtime `Response.body` getter 暴露 host fetch body 的 `ReadableStream<Uint8Array>`；`worker_global_fetch_exposes_response_body_stream` 固定 `fetch('./pass.txt').then(r => new Response(r.body))` 转发 `PASS\n`；临时补本地 `resources/pass.txt` probe 显示完整 readable-stream WPT 前 6 subtest 通过，剩余 cancel/abort 反传未纳入 baseline |
| 2026-08-22 | M3 registration CacheStorage persistence | SW active registration-local CacheStorage snapshot/restore；normal profile persistence dirtying |
| 2026-08-22 | M2 worker Cache delete/listing | SW runtime `Cache.delete()` 与 `CacheStorage.delete/has/keys` 贯穿 runtime/renderer/browser/manager/protocol |
| 2026-08-22 | storage-cache-api M3 persistence support | page/WebView owner CacheStorage per-origin 落盘 |
| 2026-08-22 | M2 fetch network-error baseline | `fetch-event-network-error.https.html` 纳入 fetch runner；4 case / 7 subtest 全绿 |
| 2026-08-24 | M2 fetch throw-after-respondWith iframe baseline | `fetch-event-throws-after-respond-with.https.html` 纳入 fetch runner；iframe navigation fetch 异步等待 MessagePort-backed response；15 case / 34 subtest 全绿 |
| 2026-08-22 | M2 fetch ISO Latin-1 header baseline | `iso-latin1-header.https.html` 纳入 fetch runner；iframe `postMessage()` / `MessageEvent.ports` 补齐；5 case / 8 subtest 全绿 |
| 2026-08-22 | M2 fetch async listener baseline | `fetch-event-add-async.https.html` 纳入 fetch runner；worker idle timer task pump；6 case / 10 subtest 全绿 |
| 2026-08-23 | M2 fetch within-SW baseline | `fetch-event-within-sw.https.html` 纳入 fetch runner；iframe `contentWindow.caches` 与 Cache.add iframe fetch context 补齐；7 case / 12 subtest 全绿 |
| 2026-08-23 | M2 fetch custom-response baseline | `fetch-event-respond-with-custom-response.https.html` 纳入 fetch runner；TextEncoder/TextDecoder/FormData 与 multipart formData 读回补齐；8 case / 23 subtest 全绿 |
| 2026-08-23 | M2 fetch uncontrolled-page baseline | `uncontrolled-page.https.html` 纳入 fetch runner；scope 外 uncontrolled 页面 XHR 不触发 fetch handler；10 case / 25 subtest 全绿 |
| 2026-08-23 | M2 fetch claim-fetch baseline | `claim-fetch.https.html` 纳入 fetch runner；message-time `clients.claim()` 控制既有 iframe client；11 case / 26 subtest 全绿 |
| 2026-08-23 | M2 fetch claim registration-boundary baseline | `claim-not-using-registration.https.html` 纳入 fetch runner；`clients.claim()` 遵守 longest-matching registration 边界；12 case / 28 subtest 全绿 |
| 2026-08-23 | M2 fetch unregister-controller baseline | `unregister-controller.https.html` 纳入 fetch runner；unregister 保留既有 iframe controller 并阻止后续 control；14 case / 33 subtest 全绿 |
| 2026-08-31 | M2 fetch global-scope baseline | `fetch-on-the-right-interface.https.any.js` 与 `historical.https.any.js` 纳入 fetch runner；`fetch` 位于 `WorkerGlobalScope.prototype`，历史 `FetchEvent.targetClientId` 不暴露；17 case / 38 subtest 全绿 |
| 2026-08-31 | M2 fetch handled baseline | `fetch-event-handled.https.html` 纳入 fetch runner；`FetchEvent.handled` 对 pass-through / respondWith 成功 resolve，对 canceled / invalid / rejected 路径 reject；18 case / 46 subtest 全绿 |
| 2026-08-31 | M2 fetch same-document navigation baseline | `fetch-event-after-navigation-within-page.https.html` 纳入 fetch runner；受控 iframe 同文档 hash / `history.pushState()` 后继续 fetch interception；19 case / 48 subtest 全绿 |
| 2026-08-31 | M2 fetch intercepted-referrer baseline | `intercepted-referrer.https.html` 纳入 fetch runner；SW 合成 iframe navigation response 保留父页面 `document.referrer`；20 case / 49 subtest 全绿 |
| 2026-08-31 | M2 fetch no-handler controller baseline | `controller-with-no-fetch-event-handler.https.html` 纳入 fetch runner；有 controller 但无 fetch handler 时跨源 fetch 保持 CORS/no-cors 处理；21 case / 54 subtest 全绿 |
| 2026-08-31 | M2 fetch request-body forwarding baseline | `fetch-with-body.https.html` 纳入 fetch runner；`respondWith(fetch(event.request))` 保留受控 client 的 POST method/body；22 case / 55 subtest 全绿 |
| 2026-08-31 | M2 fetch invalid-header baseline | `invalid-header.https.html` 纳入 fetch runner；`Headers.append()` 与 Rust/IPC response 校验拒绝非法 header name/value，NUL response header 转 network error；23 case / 56 subtest 全绿 |
| 2026-08-31 | M2 fetch invalid-blobtype baseline | `invalid-blobtype.https.html` 纳入 fetch runner；invalid Blob MIME type 不生成 `Content-Type` response header；24 case / 57 subtest 全绿 |
| 2026-08-31 | M3 message constructor baseline | `ServiceWorkerGlobalScope/extendable-message-event-constructor.https.html` 纳入 fetch/message runner；`ExtendableMessageEvent` 默认值、init 转换、source/ports TypeError 边界；25 case / 67 subtest 全绿 |
| 2026-09-02 | M2 fetch stream body error baseline | `fetch-error.https.html` 纳入 fetch runner；`respondWith(new Response(stream))` 后续 body error 在页面 `response.text()` 阶段 reject；26 case / 70 subtest 全绿 |
| 2026-09-02 | M2 fetch readable-stream start-source prework | SW runtime `ReadableStream` 构造器执行 `underlyingSource.start(controller)`；受控 iframe `contentWindow.fetch('?stream')` 可读回 Service Worker `Response(ReadableStream)` body；`fetch-event-respond-with-readable-stream.https.html` 前 4 subtest 通过，完整 WPT 仍因 stream forwarding/cancel 反传暂不纳入 runner |
| 2026-09-02 | M2 fetch readable-stream chunk baseline | `fetch-event-respond-with-readable-stream-chunk.https.html` 纳入 fetch runner；pull-source empty/non-empty `Uint8Array` chunk 经受控 iframe `response.text()` 串接读回；fetch/message runner 27/71 Pass |
| 2026-09-02 | M2 CacheStorage `.any.js` batch baseline | 8 个额外 top-level CacheStorage `.any.js` Service Worker global variants 纳入 CacheStorage runner；23 case / 305 subtest 全绿 |
| 2026-09-02 | M2 CacheStorage buckets Service Worker baseline | `cache-storage-buckets.https.any.js` Service Worker variant 纳入 CacheStorage runner；24 case / 308 subtest 全绿 |
| 2026-09-02 | M2 CacheStorage abort Service Worker baseline | `cache-abort.https.any.js` Service Worker variant 纳入 CacheStorage runner；WPT runner 对 `.any.js` worker 包装注入 cache-abort 动态 fetch/stash fixture；25 case / 318 subtest 全绿 |
| 2026-08-21 | M2-1 fetch runtime foundation | `FetchEvent`/`Request`/`Response` MVP；manager longest-scope dispatch；browser/renderer IPC command/event；生产页面 fetch/Cache 集成仍待后续 |
| 2026-08-19 | 三方案对比 | 拒绝同线程 context（无调度隔离）；拒绝从零线程（复制安全基建）；推荐抽取 Worker 线程核 |
| 2026-08-19 | owner | production browser process 单一 owner；WebView 只做同算法 in-process adapter |
| 2026-08-19 | 首个 driving WPT | `activation-after-registration.https.html` |
