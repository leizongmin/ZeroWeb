# OPFS 真实化 — WPT 驱动的 Origin Private File System 目标

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（Tier 2「存储：IndexedDB + Cache API + OPFS」最后一块）

> **说明**
> 本文档是 ZeroWeb「OPFS 真实化」专项目标执行契约。目标是把页面 `navigator.storage.getDirectory()`
> 返回的 OPFS 文件系统从 **JS shim 内存虚拟树**（进程内、不持久化）接到 zero-storage crate 的
> **真实实现 + per-origin 持久化**，以 WPT 文件系统真实用例通过率为验证标准。本文定义 Mission、
> 边界、Done Criteria、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、
> evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标 Tier 2 存储线拆出（存储三件套之三——IndexedDB
> 2026-08-19 收口、Cache API 2026-09-06 收口，仅剩 OPFS）。理由：① 两个兄弟 goal 已验证「Rust
> 底座 + shim 接线 + WPT 驱动」模式可复制，OPFS 是最后一块，收口即闭合父目标 Tier 2 存储全项；
> ② 页面侧 JS API 面已有内存近似版（`part02.js` navigator.storage 段），缺口清晰聚焦在 Rust 层
> 真实实现与持久化；③ 落盘可复用 storage_manager 既有「JSON + 临时文件 + rename + fsync」模式，
> 改动面独立，与 rendering-compat 渲染流域**零碰撞**。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **JS shim 层（已有内存近似）**：`crates/engine/src/js_dom_shim/part02.js` 行 2755–2972，
>   `navigator.storage.getDirectory()` → `FileSystemDirectoryHandle`；`getFileHandle`/
>   `getDirectoryHandle({create})`/`removeEntry({recursive})`/`keys`/`entries`/`values`/
>   `isSameEntry`；`getFile()`（返 Blob）、`createWritable({keepExistingData})` →
>   write/seek/truncate/close/abort；名称校验 `_zwFsValidName`。后端为进程内虚拟 FS 树
>   （`{kind:'dir',children}` / `{kind:'file',data:Uint8Array}`），**纯内存不持久化**。代码内
>   自述限制（行 2761–2768）：无持久化、无 `createSyncAccessHandle`、无 permission/move/transferable、
>   无 `showOpenFilePicker`。
> - **Rust 存储层（零实现）**：`crates/storage/src/` 下 grep `opfs|getDirectory|FileSystemHandle`
>   **零命中**——OPFS 在 zero-storage 无任何对应物。
> - **持久化模式可复用**：`crates/storage/src/storage_manager.rs` 行 21–60 `default_storage_dir()`
>   （`ZERO_STORAGE_DIR` env → 平台 data dir → `.zero-browser-storage/`）与
>   `StorageManager::with_persistence(root)`；`indexed_db/persistence.rs`（448 行）与
>   `cache_api/persistence.rs`（255 行）的「JSON + 临时文件 + `replace_file` + `sync_directory`
>   + 中断恢复扫描」既有模式。
> - **测试基建**：`crates/engine/src/js_dom_bridge_tests/part16.rs`（R3314/R3315/R3254 系列
>   bridge 测试）；WPT 上游 `fs/`（File System）用例目录**未导入**（`wpt-data/` 无 storage/fs
>   类别，IndexedDB/CacheStorage 各有 `scripts/fetch-*-subset.sh` 先例可照抄）。

---

## Mission

以 **WPT 文件系统（OPFS 面）真实用例通过率为验证标准**，把页面 OPFS 从 JS shim 内存虚拟树
升级为 zero-storage 真实实现 + per-origin 持久化，对齐 Chromium 水平。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入上游 OPFS 可执行用例 + 通过率基线（当前全内存版的表现即验收清单） |
| 中期 | **核心通路** | getDirectory/getFileHandle/createWritable/getFile 走 Rust 真实实现 |
| 长期 | **持久化 + 语义收口** | per-origin 落盘、跨会话 e2e、剩余语义（removeEntry 递归/流式写/sync access handle 评估） |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 indexeddb / cache-storage
先例——不允许手写 inline 用例替代或充数）。通过率统计的分母是上游范围内、不在 skip list 中的用例。
上游用例依赖 File System Access window picker 的部分（`showOpenFilePicker` 等）入 skip list
并注明属浏览器 shell 域，不充数也不误排除。

覆盖范围：

1. **FileSystemDirectoryHandle** — `getDirectoryHandle`/`getFileHandle`/`removeEntry`/
   `resolve`/`keys`/`entries`/`values`/`isSameEntry`
2. **FileSystemFileHandle** — `getFile`（Blob 视图）、`createWritable`（write/seek/truncate/
   close/abort、`keepExistingData`）
3. **持久化** — per-origin 落盘（复用 storage_manager 根目录机制），跨会话可读
4. **`navigator.storage.estimate()`** — 从静态近似升级为真实用量统计（低优先级）
5. **`createSyncAccessHandle`** — Worker 环境专用，评估后决定做或记入 skip（做则限 Worker 面）

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| Rust 真实实现 | zero-storage 新增 `opfs` 模块（目录树/文件句柄/读写流） | 照 cache_api.rs 结构 |
| 持久化 | per-origin 落盘（JSON 或二进制，复用临时文件+rename+fsync 模式） | 跨会话 e2e 测试 |
| JS↔Rust 接线 | `part02.js` navigator.storage 段 → host 命令 → opfs 模块 | host 回调命名遵循 `__zw_*` 约定 |
| API 语义 | 句柄身份、名称校验、错误类型（NotFoundError/TypeError 等） | 以 WPT 用例为准 |
| WPT 基础设施 | `fs` OPFS 面用例导入、testharness 执行、通过率报告 | 复用 tests/wpt-runner + `make import-wpt`；新增 `scripts/fetch-fs-subset.sh` 照 indexeddb/cache-storage 先例 |
| 单元测试 | 每项修复带单测（storage crate 级 + engine bridge 级） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **File System Access（window picker 面）** — `showOpenFilePicker`/`showSaveFilePicker`/
  `showDirectoryPicker`：需要用户授权 UI，属 browser-shell 域，仅保留 shim 现状
- **Permission API 深化** — `query()`/`request()` 对 OPFS 句柄的权限语义，依赖授权 UI，排除
- **拖放/文件上传的 File 后端对接** — 与本目标只共享 Blob 类型，不共建
- **OPFS 非页面环境（扩展脚本沙箱）** — script-sandbox 域不碰

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `crates/storage` +
  `crates/engine/src/js_dom_shim/part02.js`（navigator.storage 段）+ WPT 导入资产 +
  本 goal 控制面。渲染流域（css-parser/style-system/layout-engine/render-foundation）
  **零重叠**；`part02.js` 属 engine 共享面——开工前 `git log --since="14 days ago" --
  crates/engine/src/js_dom_shim/` 核对渲染流活跃面，有活跃编辑则先做零碰撞面（storage
  单测、WPT 导入、opfs 模块 Rust 实现）。
- **与已归档兄弟 goal**：storage-indexeddb（2026-08-19 归档）、storage-cache-api
  （2026-09-06 归档）已完成，其持久化模式是本目标的参照实现（只读参照，不改其代码）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 从上游 WPT 仓库导入 OPFS 面（`fs` 目录 window 环境可执行子集）真实用例；依赖
      window picker 的子目录入 skip list 并注明归属
- [ ] 新增 `scripts/fetch-fs-subset.sh`（照 `fetch-cache-storage-window-subset.sh` 模式）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经常驻断言集并记入账本（`imported-testharness.txt`）
- [ ] 通过率报告持久化到 `docs/goal/storage-opfs/evidence/`，历史可追溯

### DC-2: 页面走真实引擎

- [ ] zero-storage 新增 opfs 模块：目录树操作、文件读写流全 API 面（对应上面覆盖范围 1-2）
- [ ] `part02.js` navigator.storage 段从内存虚拟树切换到 host 命令 → opfs 模块
      （kill-switch + A/B 零回归，内存版留作 no-storage 回退路径）
- [ ] 句柄身份/名称校验/错误类型语义与 spec 一致（WPT 为准）

### DC-3: 持久化

- [ ] per-origin 落盘，跨会话 e2e：写入 → 重建 engine → 读回一致（page/WebView owner）
- [ ] 磁盘错误不 panic，Promise reject（照 cache_api persistence 模式）
- [ ] 中断恢复（恢复临时文件/清理半写状态）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT 基线建立 + opfs 模块骨架

**目标**：导入 OPFS 面用例、记录内存版基线；zero-storage 建 opfs 模块骨架（目录树 + 文件句柄
数据结构与单测）。

**切片建议**：
1. `fetch-fs-subset.sh` + 用例导入 + 分类通过率报告（零源码改动，纯资产；先导入后接线）
2. opfs 模块：目录树数据结构 + 名称校验 + getDirectoryHandle/getFileHandle + 单测
3. 失败聚类 → 修复队列

### M2 — 读写流 + JS 接线

**目标**：createWritable 写流全语义（write/seek/truncate/close/abort）、getFile Blob 视图、
removeEntry 递归；`part02.js` 切换到 host 命令（kill-switch + A/B）。

### M3 — 持久化 + 收尾

**目标**：per-origin 落盘 + 跨会话 e2e、estimate 真实化、sync access handle 评估定论、
剩余用例修复 → DC 全满足判定。

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
`cargo build` + `make test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前 opfs 缺口的确切差距（内存 shim 行为 vs WPT 期望）
2. **自主导入** WPT OPFS 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（Rust 层缺失？接线？持久化？语义？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`make test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 `part02.js`（engine 共享面）前先 `git log` 核对渲染流域活跃；有活跃
   编辑则转零碰撞面（Rust opfs 模块、WPT 导入、storage 单测）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **用例失败分析**：每个失败 case 必须分析根因（API 缺失？语义？持久化？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/storage-opfs/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/storage-opfs/archive/`：只追加不修改。
- **证据区域** `docs/goal/storage-opfs/evidence/`：通过率报告、失败分析等验证证据，持续追加。
