# IndexedDB 真实化 — WPT 驱动的 IndexedDB 页面可用性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Done Criteria §3「Storage：IndexedDB 基础」+ Tier 2「IndexedDB」）

> **说明**
> 本文档是 ZeroWeb「IndexedDB 真实化」专项目标执行契约。目标是把页面 JS 的 IndexedDB 从
> in-memory 近似（part02.js:2564 注释自认「完整 in-memory」实现）迁移到 **zero-storage crate
> 的真实 IndexedDB 引擎**（~10k 行：事务/cursor/range query/object store 全 API 面），补上
> **跨会话持久化**，并以 WPT `IndexedDB` 真实用例通过率为验证标准。本文定义 Mission、边界、
> Done Criteria、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、
> evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：zero-web 父目标的自主推进面已收敛（security/net/storage
> deep-review 穷尽，活跃面剩用户/环境门控项）。存储方向是「Rust 资产已建成、页面不可用」的
> 典型——zero-storage 的 IndexedDB 子系统约 10k 行沉睡，页面侧只有 in-memory 近似。用户裁决
> 将存储方向拆为三个独立并行流：IndexedDB（本目标）、Cache API（storage-cache-api）、
> Service Worker（service-workers）。理由：① 上游 WPT `IndexedDB` 目录是最大的单目录之一
> （数百用例），独立验收面最厚；② Rust 底座已有（crates/storage/src/indexed_db/），主要工作是
> 接线与持久化，工程量可控；③ 与 js-dom 流零碰（不是 DOM node 对象，不碰 dom_bindings）。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **Rust 层**：`crates/storage/src/indexed_db/`（cursor.rs / types.rs / mod.rs + 5 个测试文件，
>   合计约 10k 行）——IdbKey、IdbKeyRange（only/lower_bound/upper_bound/bound + contains）、
>   IdbCursor（advance/continue_to/finish）、事务（commit/abort/mode/store_names）、
>   object store CRUD 全 API 面已实现并有单测。
> - **JS 页面层**：`part02.js:2564` `globalThis.indexedDB` 为 **in-memory 近似**（注释明言
>   「完整 in-memory」——为让 5 个 storage WPT smoke 用例不抛 `indexedDB is not defined`）。
>   与 Rust 引擎零接线：页面事务不进 storage crate，重启即失。
> - **持久化**：local_storage.rs 有持久化通道；indexed_db 子系统无落盘路径。
> - **WPT 面**：`tests/wpt-runner/wpt-data/` 无 IndexedDB 目录，无真实用例导入，无通过率基线。

---

## Mission

以 **WPT `IndexedDB` 真实用例通过率为验证标准**，把页面 `indexedDB` 全局（open/事务/
objectStore/索引/cursor/请求事件模型）接到 zero-storage 真实引擎并补持久化，对齐 Chromium
水平。分阶段里程碑校准执行预期（数字在首次导入后按实测校准）：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入 `IndexedDB` 目录范围内用例 + 通过率基线（当前无基线） |
| 中期 | **核心通路 60%+** | open/事务/store CRUD/cursor/getAll 走 Rust 引擎（替换 in-memory） |
| 长期 | **80%+** | 索引（index/openCursor/getAll）、请求事件模型（success/error/readyState）、持久化 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d / form-validation
——不允许手写 inline 用例替代或充数）。通过率统计的分母是上游 `IndexedDB` 目录中所有属于
范围内、不在 skip list 中的用例。

覆盖范围：

1. **工厂与连接** — `indexedDB.open`（版本升级/`onupgradeneeded`/`versionchange`/`blocked`）、
   `deleteDatabase`、`IDBFactory.cmp`
2. **事务模型** — `IDBTransaction`（readwrite/readonly/versionchange、abort/commit、
   `onabort`/`oncomplete`/`onerror`、事务排序与 auto-commit）
3. **Object Store** — createObjectStore/deleteObjectStore、put/add/get/delete/getAll/
   getAllKeys/count/clear、keyPath/autoIncrement、inline vs out-of-line keys
4. **索引** — createIndex/deleteIndex、`IDBIndex.get/getAll/getKey/getAllKeys/count/openCursor/
   openKeyCursor`、unique/multiEntry
5. **Cursor** — `openCursor`/`openKeyCursor`（range/direction/advance/continue/continuePrimaryKey）
6. **请求事件模型** — `IDBRequest`（result/error/source/readyState/transaction、success/error
   事件、事件顺序）
7. **持久化** — 数据库文件落盘（per-origin 隔离），跨会话可读

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| JS↔Rust 接线 | part02.js `globalThis.indexedDB` 段 → host 命令 → zero-storage 引擎 | in-memory 近似整段替换；host 命令经 `__zw_*` 回调或 js-dom 流的桥约定 |
| 持久化 | indexed_db 引擎落盘（per-origin 路径，参照 local_storage 通道） | 跨会话 e2e 测试 |
| API 语义 | IDBFactory/IDBDatabase/IDBTransaction/IDBObjectStore/IDBIndex/IDBRequest/IDBCursor | 以 WPT 用例为准 |
| WPT 基础设施 | `IndexedDB` 用例导入、testharness 执行、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` 资产化机制 |
| 单元测试 | 每项修复带单测（storage crate 级 + engine bridge 级） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **Cache API / CacheStorage** — 兄弟目标 `storage-cache-api.md`
- **Service Worker**（SW 对 IDB 的使用）— 兄弟目标 `service-workers.md`
- **OPFS（navigator.storage.getDirectory）** — part02.js 已有虚拟 FS 近似；持久化面可顺带
  接 storage_manager，但不立项独立验收
- **SQL 类提案（WebSQL）** — 已被标准废弃
- **存储配额 UI / 持久化许可提示**（navigator.storage.persist 用户 UI）— 浏览器 shell 域

### 依赖约束

- **与 js-dom 流碰撞管理**：接线若需改 `js_dom_shim` 的 host 回调注册段（非 DOM node 对象段），
  开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对；该流活跃则
  先做零碰撞面（Rust 引擎补强、WPT 导入、storage crate 单测）。
- **事件循环依赖**：IDB 请求的异步 success/error 事件依赖事件循环投递（P1a 已有 setTimeout/
  microtask 真实化），无需新机制。

---

## 当前能力/缺口基线

**详见** [storage-indexeddb/master.md](storage-indexeddb/master.md)（运行时控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **Rust 引擎全 API 面**：crates/storage/src/indexed_db/（~10k 行）——key/range/cursor/
  事务/store CRUD 已实现且有单测（types_coverage 5 轮 + tests_basic/advanced/edge）
- ⚠️ **缺口 1 — 页面零接线**：JS `indexedDB` 为 in-memory 近似（part02.js），与 Rust 引擎零关联
- ⚠️ **缺口 2 — 无持久化**：indexed_db 无落盘路径（local_storage 有），重启即失
- ⚠️ **缺口 3 — WPT 覆盖为零**：上游 `IndexedDB` 目录（数百用例）未导入，无基线
- ⚠️ **缺口 4 — 事件模型近似**：IDBRequest 事件序/readyState/auto-commit 语义未对 spec

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT IndexedDB 用例导入与通过率基线

- [ ] 从上游 WPT 仓库 `IndexedDB` 目录导入范围内真实用例（skip list 记录排除项与理由）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/storage-indexeddb/evidence/`，历史可追溯

### DC-2: 页面走真实引擎

- [ ] `globalThis.indexedDB` 全 API（factory/database/transaction/store/index/cursor/request）
  经 host 命令进 zero-storage 引擎，in-memory 近似代码删除或萎缩为 fallback
- [ ] 事务排序/auto-commit/事件序与 spec 一致（WPT 为准）

### DC-3: 持久化

- [ ] per-origin 落盘，跨会话 e2e：写入 → 重建 engine → 读回一致
- [ ] 磁盘错误（满盘/权限）不 panic，走 request error 事件

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT IndexedDB 基线建立

**目标**：导入 `IndexedDB` 用例（可分子目录分批），跑通 testharness 执行，记录通过率基线。

**切片建议**：
1. 用例导入 + 分类通过率报告（零源码改动，纯资产；上游目录大，按子目录分批）
2. 失败聚类分析 → in-memory 近似已覆盖面 vs 缺失面清单
3. 首个轻量修复队列（in-memory 近似可直接修的语义缺口）

### M2 — JS↔Rust 接线（核心通路）

**目标**：open/事务/store CRUD/cursor 走 Rust 引擎；每步 kill-switch + A/B 零回归。

### M3 — 索引 + 请求事件模型 + 持久化

**目标**：index 全 API、IDBRequest 事件序、落盘与跨会话 e2e。

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

1. **自主探索**当前 indexed_db 引擎与 shim 近似的差距（API 面/事件模型/持久化）
2. **自主导入** WPT IndexedDB 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（接线缺失？引擎语义？事件序？持久化？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（js_dom_shim host 回调注册段）前先 `git log` 核对；有活跃
   编辑则转零碰撞面（Rust 引擎、WPT 导入、storage 单测）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，
   当作当前任务的一部分修复。
2. **用例失败分析**：每个失败 case 必须分析根因（in-memory 近似残留？引擎缺 API？事件序？
   keyPath 解析？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/storage-indexeddb/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/storage-indexeddb/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/storage-indexeddb/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
