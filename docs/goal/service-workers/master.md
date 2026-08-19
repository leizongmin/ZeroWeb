# Service Worker 真实化 — 运行时控制面板（master.md）

**入口文档**: [../service-workers.md](../service-workers.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-08-19（M1-3c navigator.serviceWorker bridge 完成）

---

## 当前状态

**专项定位**：存储方向三拆之三。把 `navigator.serviceWorker` 从注册表状态机近似
（R3318）深化为真实 SW 执行环境 + fetch 拦截。用户已于 2026-08-19 明确批准方案 C，
M0 启动门禁解除；当前进入 M1 in-process host bridge。

**M0 推荐决策**：抽取 `zero-script-sandbox::WorkerRuntime` 的独立线程/引擎/看门狗核心，
新增 typed `ServiceWorkerRuntime`；production 由 browser process 的
`ServiceWorkerManager` 单一拥有注册、控制与 fetch 路由，embedded WebView 使用同一 manager
算法的 in-process adapter。详见 [M0 执行环境 RFC](m0-execution-environment-rfc.md)。

**与兄弟 goal 的边界**：
- [storage-indexeddb](../archive/storage-indexeddb.md)（已归档）/ storage-cache-api —
  IDB 与 Cache API 自身语义归其管；本目标只消费
  `indexedDB`/`caches` 接口做 SW 模式集成验收
- js-dom — fetch 拦截段**等其 fetch 改造（L2/S6）land 后再开**；生命周期段碰 part02.js
  R3318 段前先 `git log` 核对（run-rules §9）

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
- ✅ Worker Global/import 裁决：13 case / 53 subtest 分为 1 static core、6 runtime defer、
  4 server gated、1 M2 defer、1 worker-client skip；全量剩余逻辑 review 71
- ✅ static-wave 资产化：`serviceworkerobject-scripturl` 1 case / 4 subtest / 2 assets
  已固定并记入 testharness 账本
- ✅ IDL harness 裁决：4 generated URL / 787 subtest（175 window + 155 dedicated +
  155 shared + 302 serviceworker）；全量剩余逻辑 review 70
- ✅ Navigation/redirect 裁决：15 source / 16 URL / 224 subtest 分为 2 defer /
  10 gated / 3 skip；全量剩余逻辑 review 55
- ✅ Request/response/timing 裁决：17 source / 83 subtest 分为 7 defer /
  9 gated / 1 skip；全量剩余逻辑 review 38
- ✅ Final remaining 裁决：38 source / 270 subtest 分为 14 defer /
  8 gated / 16 skip；初始 review 152/152，逻辑剩余 0
- ✅ Runner disposition contract：294 source / 331 URL 唯一映射为
  12 core / 51 defer / 189 gated / 42 skip，可从原始 evidence 确定性重建；
  12 个 core 与 runner 导入账本、三批 case asset 及 blob SHA 精确对应
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

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | SW 执行环境架构与独立 runtime | 🔄 in-process runtime/manager/page bridge 已落；IPC 待接 |
| S2 | scriptURL 不下载执行 | 🔄 WebView navigator 真链路已落；browser IPC 待接 |
| S3 | fetch 拦截为零 | ⬜ M2（等 js-dom fetch 改造） |
| S4 | 事件为 setTimeout 模拟 | ✅ 生命周期状态仅来自 manager；timer 只轮询 snapshot |
| S5 | WPT 覆盖为零 | 🔄 12 case 已资产化；294-source contract 已落；runner/真实 red baseline 待 M1 bridge |

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 批准方案 C：抽取 Worker 线程核 + SW typed runtime + browser manager owner | ✅ 2026-08-19 用户明确批准 |

## 下一步计划

1. **M1-4**：browser owner + renderer IPC callbacks，跨 renderer 保持 registration
2. **M1-5**：SW WPT runner，建立 Tier A red/green baseline
3. **M2 继续门控**：js-dom S6 与 storage-cache-api M1 均 land 后才改 fetch 主路径

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 选型 RFC（门控） | ✅ 方案 C 已批准 |
| M1 — 脚本真实执行 + 生命周期真事件 | 🔄 M1-3c in-process 页面真链路完成 |
| M2 — fetch 拦截 + Cache 集成 | ⬜ 门控：js-dom fetch 改造 land |
| M3 — 控制语义 + 消息 + 收尾 | ⬜ |

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
  core lane、runner 导入账本与三批 case asset 的双向闭包
- Tier A 资产恢复：`make fetch-wpt-service-workers-tier-a`；默认使用独立
  `wpt-data/.service-workers-tier-a-root`，当前环境 18/18 blob SHA 验证通过
- Tier A 资产审计：`make audit-wpt-service-workers-tier-a`（无网络、只读）；
  `make test-wpt-service-workers-tier-a-assets` 覆盖缺失/篡改/修复回归
- Next-wave 资产恢复/审计：`make fetch-wpt-service-workers-next-wave` /
  `make audit-wpt-service-workers-next-wave`；与 Tier A 复用独立数据根，当前 7/7 通过；
  `make test-wpt-service-workers-next-wave-assets` 固化篡改/修复回归
- Static-wave 资产恢复/审计：`make fetch-wpt-service-workers-static-wave` /
  `make audit-wpt-service-workers-static-wave`；2 assets / 4 subtest；
  `make test-wpt-service-workers-static-wave-assets` 固化篡改/修复回归
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
| 2026-08-19 | 三方案对比 | 拒绝同线程 context（无调度隔离）；拒绝从零线程（复制安全基建）；推荐抽取 Worker 线程核 |
| 2026-08-19 | owner | production browser process 单一 owner；WebView 只做同算法 in-process adapter |
| 2026-08-19 | 首个 driving WPT | `activation-after-registration.https.html` |
