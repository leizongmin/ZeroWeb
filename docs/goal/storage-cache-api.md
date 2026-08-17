# Cache API 真实化 — WPT 驱动的 CacheStorage 页面可用性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 2「存储：IndexedDB + Cache API + OPFS」+ M12「Cache API」列项）

> **说明**
> 本文档是 ZeroWeb「Cache API 真实化」专项目标执行契约。目标是把页面 `caches`（CacheStorage）/
> `Cache` 全 API 从**零接线**（shim 中 grep 不到 `caches`——页面访问 `caches.open(...)` 直接
> ReferenceError）接到 zero-storage crate 的 `cache_api.rs`（976 行、67 函数）真实实现，补
> per-origin 持久化，并以 WPT `cache-storage` 真实用例通过率为验证标准。本文定义 Mission、
> 边界、Done Criteria、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常
> 进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：与 storage-indexeddb 同批拆出（存储方向三拆之二）。
> 理由：① **页面侧完全空白**（indexedDB 至少有 in-memory 近似，`caches` 连全局对象都没有），
> 是存储三件套里缺口最彻底的；② Rust 底座已有（cache_api.rs 976 行：Cache/CacheStorage/
> match/matchAll/add/addAll/put/delete/keys 全 API 面）；③ 上游 WPT `cache-storage` 目录
> 用例量厚（idle-binding/hit-web/basics 等数十文件）；④ 独立验收面（SW 无关的 Cache API
> 语义可先立——`caches` 在普通页面可用，不依赖 Service Worker 环境）。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **Rust 层**：`crates/storage/src/cache_api.rs`（976 行 / 67 函数）——CacheStorage
>   （open/has/delete/keys）、Cache（match/matchAll/add/addAll/put/delete/keys）、
>   CacheQueryOptions（ignoreSearch/ignoreMethod/ignoreVary）已实现并有单测。
> - **JS 页面层**：`js_dom_shim` part01-06.js **无任何 `caches`/`CacheStorage` 定义**——
>   页面 `caches.open()` 抛 ReferenceError，PWA 缓存脚本全挂。
> - **持久化**：cache_api.rs 为内存结构，无落盘路径。
> - **WPT 面**：`tests/wpt-runner/wpt-data/` 无 cache-storage 目录，无基线。

---

## Mission

以 **WPT `cache-storage` 真实用例通过率为验证标准**，把页面 `caches`/`Cache` 全 API 接到
zero-storage 真实实现并补持久化，对齐 Chromium 水平。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入 `cache-storage` 范围内用例 + 通过率基线（当前无基线） |
| 中期 | **核心通路 60%+** | caches.open/has/delete/keys + Cache.put/match/matchAll/delete/keys 走 Rust |
| 长期 | **80%+** | add/addAll（Request 构造 + fetch 集成）、CacheQueryOptions 全语义、Vary 头、持久化 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d /
form-validation——不允许手写 inline 用例替代或充数）。通过率统计的分母是上游
`cache-storage` 目录中所有属于范围内、不在 skip list 中的用例。

**注意**：上游 `cache-storage` 部分用例在 Service Worker 环境下跑（`sw` 子目录）——本目标
只收 **window 环境可执行**的用例；SW 环境依赖的子目录记入 skip list 并注明归兄弟目标
`service-workers.md` 处理，不充数也不误排除。

覆盖范围：

1. **CacheStorage** — `caches.open/has/delete/keys`（name 空间、open 幂等、delete 级联）
2. **Cache** — `put/match/matchAll/delete/keys`、`add/addAll`（Request 构造 + fetch 集成）、
   Response 类型限制（opaque 等可缓存性判定）
3. **CacheQueryOptions** — ignoreSearch/ignoreMethod/ignoreVary 全语义
4. **Request/Response 集成** — put 的 Request→Response 关联、URL 归一化（fragment 剥离）、
   Vary 头匹配
5. **持久化** — per-origin 落盘，跨会话可读

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| JS↔Rust 接线 | 新建 shim `caches` 段（无现存段可替换）→ host 命令 → cache_api.rs | host 回调命名遵循 `__zw_*` 约定 |
| Request/Response 集成 | add/addAll 需走 fetch 管线（P1a FetchBridge 已真实化） | 复用 fetch_bridge，不重建 |
| 持久化 | cache_api 落盘（per-origin 路径） | 跨会话 e2e 测试 |
| API 语义 | CacheStorage/Cache/CacheQueryOptions/CacheBatchOperation | 以 WPT 用例为准 |
| WPT 基础设施 | `cache-storage` 用例导入、testharness 执行、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` |
| 单元测试 | 每项修复带单测（storage crate 级 + engine bridge 级） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **Service Worker 环境的 cache 用例**（`cache-storage/sw` 类）— 兄弟目标 `service-workers.md`
- **IndexedDB** — 兄弟目标 `storage-indexeddb.md`
- **HTTP disk cache（net crate 的 disk_cache.rs）** — 这是浏览器内部 HTTP 缓存，与页面
  Cache API 是两个东西；不碰
- **Storage quota UI / `navigator.storage.estimate` 精确数值** — shell 域，仅 stub 反射可保留

### 依赖约束

- **与 js-dom 流碰撞管理**：新建 shim 段若需改 `js_dom_shim` host 回调注册段（非 DOM node
  对象段），开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对；
  该流活跃则先做零碰撞面（Rust 补强、WPT 导入、storage 单测）。
- **fetch 依赖**：add/addAll 走 P1a 已真实化的 FetchBridge；若 js-dom S6 改造 fetch 桥段，
  本流只消费接口不共建，冲突面小。

---

## 当前能力/缺口基线

**详见** [storage-cache-api/master.md](storage-cache-api/master.md)（运行时控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **Rust 层全 API 面**：cache_api.rs（976 行）——CacheStorage/Cache/CacheQueryOptions
  已实现并有单测
- ⚠️ **缺口 1 — 页面零接线**：shim 无 `caches` 全局，页面 ReferenceError
- ⚠️ **缺口 2 — 无持久化**：cache_api 为内存结构，重启即失
- ⚠️ **缺口 3 — WPT 覆盖为零**：上游 `cache-storage` 未导入，无基线
- ⚠️ **缺口 4 — Request/Response 集成缺失**：add/addAll 的 fetch 链路、Response 可缓存性
  判定未实现

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT cache-storage 用例导入与通过率基线

- [ ] 从上游 WPT 仓库 `cache-storage` 目录导入范围内真实用例（window 环境可执行面；
      SW 环境子目录入 skip list 并注明归 service-workers 目标）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/storage-cache-api/evidence/`，历史可追溯

### DC-2: 页面走真实引擎

- [ ] `caches`/`Cache` 全 API 经 host 命令进 zero-storage 实现
- [ ] CacheQueryOptions/Vary/URL 归一化语义与 spec 一致（WPT 为准）
- [ ] add/addAll 经真实 fetch 管线

### DC-3: 持久化

- [ ] per-origin 落盘，跨会话 e2e：缓存 → 重建 engine → match 命中
- [ ] 磁盘错误不 panic，Promise reject

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT cache-storage 基线建立

**目标**：导入 `cache-storage` 用例（window 面），跑通 testharness 执行，记录通过率基线。

**切片建议**：
1. 用例导入 + 分类通过率报告（零源码改动，纯资产；先导入后接线——`caches` 未接线前
   基线即「全 ReferenceError」，这正是验收清单）
2. shim `caches` 全局骨架（open/has/delete/keys 直通 Rust——首个可量化切片）
3. Cache 核心 API（put/match/matchAll/delete/keys）

### M2 — Cache 全 API + 查询语义

**目标**：CacheQueryOptions/Vary/URL 归一化/add/addAll；每步 kill-switch + A/B 零回归。

### M3 — 持久化 + 剩余语义收尾

**目标**：落盘与跨会话 e2e、Response 可缓存性判定、剩余用例修复。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前 cache_api.rs 能力面与 shim 空白的确切差距
2. **自主导入** WPT cache-storage 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（全局缺失？API 语义？fetch 集成？持久化？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（js_dom_shim host 回调注册段）前先 `git log` 核对；有活跃
   编辑则转零碰撞面（Rust 补强、WPT 导入、storage 单测）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，
   当作当前任务的一部分修复。
2. **用例失败分析**：每个失败 case 必须分析根因（API 缺失？匹配语义？Vary？fetch 链路？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/storage-cache-api/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/storage-cache-api/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/storage-cache-api/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
