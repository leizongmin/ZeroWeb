# Service Worker 真实化 — 生命周期与 fetch 拦截的页面可用性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active（方案 C RFC 已批准；M1 实施中）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 2「Service Worker（基础）」+ M12「Service Worker 基础（注册、fetch 事件拦截、缓存管理）」列项）

> **说明**
> 本文档是 ZeroWeb「Service Worker 真实化」专项目标执行契约。目标是把 `navigator.serviceWorker`
> 从注册表状态机近似（R3318：register 返 Promise + setTimeout 模拟 install→waiting→active
> 状态推进，**无真实 worker 执行环境、无 fetch 拦截、无 install/activate 事件派发**——
> part02.js:2369 注释自认）深化为真实 SW 生命周期与 fetch 拦截。本文定义 Mission、边界、
> Done Criteria、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、
> evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：与
> [已归档 IndexedDB 目标](archive/storage-indexeddb.md) 同批拆出（存储方向三拆之三）。
> 理由：① SW 是 PWA/离线能力的中枢——Top 真实网站 SW 注册脚本普遍存在，目前全靠 stub 糊弄；
> ② Rust 侧已有 ServiceWorkerRegistry 状态机（service_worker.rs 818 行）+ 页面侧注册 API 面
> （R3318），底座非零；③ 与 js-dom 流的碰撞面明确可控（fetch 拦截段等 js-dom S6 land 后再开，
> 见依赖约束）。**注意**：本目标是三拆中唯一需要架构决策的——「真实 worker 执行环境」
> （独立线程跑 SW 脚本）是深结构，须先 RFC 选型（复刻 P1b 先 RFC 后实施的做法）。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **Rust 层**：`crates/storage/src/service_worker.rs`（818 行 / 59 函数）——
>   ServiceWorkerRegistry 状态机（register/unregister/state 推进/scope 匹配）已实现并有单测。
> - **JS 页面层**：part02.js:2369 R3318——`navigator.serviceWorker` 注册 API 完整面
>   （register/getRegistration/getRegistrations/ready/unregister + oncontrollerchange +
>   installing/waiting/active 字段经 setTimeout(0) 逐态推进）。**但是**：无真实 worker 执行
>   （register 的 scriptURL **不被下载执行**）、无 fetch 事件拦截、无 install/activate/message
>   真事件。
> - **WPT 面**：`tests/wpt-runner/wpt-data/` 无 service-workers 目录，无基线。上游
>   `service-workers` 目录大量用例依赖真实 worker 环境 + iframe + 多客户端语义。

---

## Mission

把 Service Worker 从注册表近似深化为**真实 SW 执行环境**：register 的 scriptURL 被真实下载
执行（独立于页面的脚本上下文）、install/activate/fetch/message 事件真实派发、fetch 拦截走
Cache API（与兄弟目标 storage-cache-api 集成）。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| M0（门控） | **选型 RFC** | SW 执行环境架构（独立 V8 context / 独立线程 / 复用 Worker 基建）——**须用户批准** |
| 第一阶段 | **脚本真实执行** | register → scriptURL 下载 → 独立上下文执行 → install/activate 生命周期真事件 |
| 中期 | **fetch 拦截** | activate 后页面 fetch 经 SW 的 fetch 事件（respondWith//passThrough）+ Cache API 集成 |
| 长期 | **80%+（可校准）** | message/postMessage、controller 语义、claim/skipWaiting、update 语义 |

**关键约束**：验证以 WPT `service-workers` 目录中**当前环境可执行**的用例为准（大量上游用例
依赖多 iframe 客户端/https 服务环境，超出范围入 skip list 并注明理由）；SW 环境下的
cache-storage 用例归本目标（兄弟目标只收 window 面）。

覆盖范围：

1. **注册与生命周期** — register（scriptURL 真实下载执行）、installing→waiting→active
   真状态机（替换 setTimeout 模拟）、update/unregister、`registration.installing/waiting/active`
2. **SW 执行环境** — SW 脚本在独立上下文执行（`self`、`importScripts`、`addEventListener('fetch')`
   等事件注册）；headless 无 UI 的合理简化须记录
3. **fetch 拦截** — scope 内页面请求经 SW fetch 事件；`respondWith`（Response 构造）/
   不响应（passThrough 走网络）；与 Cache API 集成（`caches.match` 先行后网等模式）
4. **控制语义** — `navigator.serviceWorker.controller`、`oncontrollerchange`、
   `clients.claim()`、`skipWaiting()`
5. **消息** — `postMessage` 双向（页面↔SW）+ message 事件

执行方式：**门控推进** — M0 RFC 批准前不动源码（文档/调研/WPT 导入可自主）；批准后转轻量
修复优先。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 选型 RFC | SW 执行环境架构选型（M0，须用户批准） | 独立 V8 context vs 独立线程 vs 复用 Web Worker 基建（tab_js_worker 已有 Worker 线程先例） |
| 生命周期真实化 | service_worker.rs 状态机 + R3318 shim 段深化 | 真事件替换 setTimeout 模拟 |
| SW 执行环境 | scriptURL 下载（net crate）+ 独立上下文执行 | MVP 可从「同线程独立 context」起步（RFC 定） |
| fetch 拦截 | fetch 管线插入 SW 拦截层 | **等 js-dom S6 land 后开**（S6 改造 Fetch 桥段） |
| WPT 基础设施 | `service-workers` 用例导入（可执行面）、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **Push API / Notification** — 依赖推送服务端，Tier 3 远期
- **Background Sync** — 依赖浏览器调度，远期
- **多客户端语义**（多 iframe/window 的 SW client 枚举与逐 client 控制）— headless 单页面
  环境外，记 skip list
- **Cache API 自身语义** — 兄弟目标 `storage-cache-api.md`（本目标只消费 `caches` 接口）
- **IndexedDB** — 兄弟目标已完成，见 [归档入口](archive/storage-indexeddb.md)

### 依赖约束

- **启动门控（M0）**：SW 执行环境是深结构（新执行上下文/线程模型），按 run-rules rule 11
  （用户决策门禁），**RFC 须用户批准后才动源码**。M0 期间的调研、RFC 起草、WPT 可执行面
  分析可自主推进。
- **与 js-dom 流的时序依赖**：fetch 拦截层建在 fetch 管线上，而 js-dom M1（L2 polyfill-live
  合一）与 S6 都会改 fetch 桥段。**fetch 拦截里程碑等 js-dom 流 fetch 改造 land 后再开**；
  生命周期/执行环境里程碑不受此限。
- **与 storage-cache-api 的时序依赖**：SW 缓存模式（caches.match）依赖兄弟目标的 `caches`
  接线；其 M1 land 后本目标 fetch 拦截才有完整验收面。

---

## 当前能力/缺口基线

**详见** [service-workers/master.md](service-workers/master.md)（运行时控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **注册 API 面**：R3318——register/getRegistration/getRegistrations/ready/unregister +
  scope 派生 + oncontrollerchange（part02.js）
- ✅ **Rust 状态机**：service_worker.rs ServiceWorkerRegistry（register/unregister/state/
  scope 匹配）
- ⚠️ **缺口 1 — 脚本不执行**：register 的 scriptURL 不被下载执行——SW 事件处理器
  （addEventListener('fetch')）无从注册
- ⚠️ **缺口 2 — fetch 拦截为零**：scope 内请求完全绕过 SW
- ⚠️ **缺口 3 — 事件为模拟**：install/activate 经 setTimeout(0) 推进，非真事件
- ⚠️ **缺口 4 — WPT 覆盖为零**：上游 `service-workers` 未导入，无基线
- ⚠️ **缺口 5 — 架构未选型**：SW 执行环境（线程/上下文模型）无 RFC——M0 门控项

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 选型 RFC 已批准并落地

- [x] SW 执行环境 RFC 完成并经用户批准（2026-08-19，方案 C）
- [ ] 实现与 RFC 一致；偏离处记录原因

### DC-2: 真实生命周期与执行

- [ ] register → scriptURL 下载 → SW 上下文执行 → install/activate 真事件全链路
- [x] setTimeout 生命周期模拟已删除；页面 timer 仅逐 task 投影 manager transition log
- [ ] scope 匹配/controller/oncontrollerchange/skipWaiting/claim 语义与 spec 一致（WPT 为准）

### DC-3: fetch 拦截

- [ ] scope 内 fetch 经 SW fetch 事件；respondWith 响应 / passThrough 走网络
- [ ] 与 Cache API 集成（caches.match 模式可端到端跑通）

### DC-4: WPT 基线与通过率

- [ ] `service-workers` 可执行面用例导入（skip list 注明 https/多客户端依赖项）
- [ ] 建立分类通过率报告，持久化到 `docs/goal/service-workers/evidence/`
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 记入 `imported-tests.txt`

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + WPT 用例资产化

---

## 活跃里程碑

### M0 — 选型 RFC（已完成）

**目标**：SW 执行环境架构选型 RFC 起草并获批。

**切片建议**：
1. WPT `service-workers` 可执行面分析（哪些用例当前环境能跑——零源码改动）
2. 候选架构调研：独立 V8 context（同线程）/ 独立线程 / 复用 Web Worker 基建
   （tab_js_worker.rs 已有 Dedicated Worker 线程先例）——各自工程量/风险/与事件循环的集成面
3. RFC 起草 → 提交用户审批（**停止源码改动，记「待用户决策」**）

### M1 — 脚本真实执行 + 生命周期真事件（RFC 批准后）

**目标**：register 链路真实化（下载/执行/install/activate）。**当前活跃**。

### M2 — fetch 拦截 + Cache 集成（等 js-dom fetch 改造 land）

**目标**：fetch 事件/respondWith/passThrough/caches.match 模式端到端。

### M3 — 控制语义 + 消息 + 收尾

**目标**：controller/claim/skipWaiting/postMessage；WPT 通过率达标（阈值按 M0 基线校准）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用（M0 等用户审批不是 BLOCK——记待决策后转 WPT 导入等零碰撞面） |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前 SW 状态机与真实生命周期的差距（M0 期间）
2. **自主分析** WPT service-workers 可执行面（skip list 有据）
3. **自主起草**选型 RFC（候选对比 + 推荐 + 风险）
4. RFC 批准后：**自主实现/测试/验证**，每修 net≥0 即 land
5. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **门控纪律**：M0 RFC 未批不动源码；等待期间转零碰撞面（WPT 导入、Rust 状态机补强单测）。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：fetch 拦截段等 js-dom 流 fetch 改造 land；生命周期段碰 `js_dom_shim`
  part02.js R3318 段前先 `git log` 核对。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **用例失败分析**：每个失败 case 必须分析根因（执行环境？事件序？scope 匹配？拦截层？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/service-workers/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/service-workers/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/service-workers/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
